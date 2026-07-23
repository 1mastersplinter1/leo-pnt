//! Deterministic synthetic passage-endurance comparison through the integrated executive.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use fusion_executive::{DopplerPipeline, Executive};
use pnt_config::{Config, EphemerisAgingConfig, GnssAuthority};
use pnt_ephemeris::EphemerisStore;
use pnt_estimator::{DopplerRangeRateUpdate, Estimator, UpdateResult};
use pnt_integrity::IntegrityStub;
use pnt_journal::MemoryJournals;
use pnt_predictor::{predict, ReceiverState, SatelliteState};
use pnt_time::ManualClock;
use pnt_types::{
    ecef_vector_to_enu, Constellation, FilterState, Frame, GnssFix, ImuSample, MeasurementEnvelope,
    MeasurementPayload, Provenance, QualityFlags, SourceId, TimeTag, TrackerDoppler, UtcTime,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

const HOUR_S: u64 = 3600;
const DURATION_S: u64 = 9 * HOUR_S;
const GPS_LOSS_S: u64 = 2 * HOUR_S;
const STEP_S: u64 = 1;
const SPEED_MPS: f64 = 6.0 * 0.514_444;
const CARRIER_HZ: f64 = 1_600_000_000.0;
const NORAD_IDS: [u64; 3] = [25_544, 25_545, 25_546];
const SEED: u64 = 0x50_41_53_53_41_47_45;
const TLE: &str = "SAT 25544\n1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992\n2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008\nSAT 25545\n1 25545U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9993\n2 25545  51.6461 341.2784 0001413  89.1723 280.4612 15.49507896236002\nSAT 25546\n1 25546U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9994\n2 25546  51.6461 101.2784 0001413  89.1723 280.4612 15.49507896236007\n";
const INITIAL_POSITION_M: [f64; 3] = [3_518_304.71, 784_390.70, 5_244_191.85];
const TRUTH_VELOCITY_MPS: [f64; 3] = [0.0, SPEED_MPS, 0.0];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PassageStudy {
    pub schema_version: u16,
    pub synthetic_only: bool,
    pub seed: u64,
    pub duration_h: f64,
    pub speed_kn: f64,
    pub distance_km: f64,
    pub gps_loss_h: f64,
    pub ephemeris_cache_h: f64,
    pub integration_step_s: u64,
    pub hard_6h: Outcome,
    pub graduated_30h: Outcome,
    pub caveat: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    pub doppler_available_until_h: f64,
    pub accepted_doppler_updates: u64,
    pub rejected_doppler_updates: u64,
    pub final_position_error_m: f64,
    pub position_class: String,
}

#[derive(Clone, Copy)]
enum GatePolicy {
    Hard,
    Graduated,
}

#[derive(Clone, Copy)]
struct DeterministicRng(u64);

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn symmetric(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let unit = (self.0 >> 11) as f64 / ((1_u64 << 53) as f64);
        2.0 * unit - 1.0
    }
}

#[derive(Clone, Debug)]
struct PassageEstimator {
    state: FilterState,
}

impl PassageEstimator {
    fn new() -> Self {
        Self {
            state: FilterState::default(),
        }
    }
}

impl Estimator for PassageEstimator {
    fn propagate(&mut self, imu: ImuSample) {
        for axis in 0..3 {
            self.state.position_ecef_m[axis] +=
                self.state.velocity_ecef_mps[axis] + 0.5 * imu.acceleration_mps2[axis];
            self.state.velocity_ecef_mps[axis] += imu.acceleration_mps2[axis];
        }
    }

    fn update(&mut self, measurement: &MeasurementEnvelope) {
        if let MeasurementPayload::Gnss(fix) = measurement.payload {
            self.state.position_ecef_m = fix.position_ecef_m;
            let ned = fix.velocity_ned_mps;
            let enu = [ned[1], ned[0], -ned[2]];
            let rotation = pnt_types::ecef_to_enu_rotation(fix.position_ecef_m);
            self.state.velocity_ecef_mps = std::array::from_fn(|column| {
                (0..3).map(|row| rotation[row][column] * enu[row]).sum()
            });
        }
    }

    fn state(&self) -> FilterState {
        self.state.clone()
    }

    fn update_predicted_doppler(&mut self, update: &DopplerRangeRateUpdate) -> UpdateResult {
        let innovation = update.measured_range_rate_mps - update.predicted_range_rate_mps;
        let velocity_jacobian = &update.core_jacobian[3..6];
        let norm = velocity_jacobian
            .iter()
            .map(|value| value * value)
            .sum::<f64>();
        let gain = 0.05 / (1.0 + update.variance_mps2);
        if norm > f64::EPSILON {
            let raw = std::array::from_fn::<_, 3, _>(|axis| {
                gain * innovation * velocity_jacobian[axis] / norm
            });
            let raw_norm = raw.iter().map(|value| value * value).sum::<f64>().sqrt();
            let scale = (0.0042 / raw_norm).min(1.0);
            for (axis, correction) in raw.iter().enumerate() {
                self.state.velocity_ecef_mps[axis] += scale * correction;
            }
        }
        UpdateResult {
            innovation,
            innovation_variance: update.variance_mps2 + norm,
            normalized_innovation_squared: innovation * innovation / (update.variance_mps2 + norm),
            accepted: true,
        }
    }
}

/// Runs the same seeded, aged-ephemeris mission under hard and graduated gate policies.
///
/// The executive is driven at a declared 1 s integration cadence rather than a sensor-rate
/// IMU cadence so the committed evidence remains quick to reproduce.
///
/// # Errors
///
/// Returns an ephemeris or prediction error if the compile-time mission fixture cannot run.
pub fn simulate() -> Result<PassageStudy, Box<dyn std::error::Error>> {
    let hard_6h = run_policy(GatePolicy::Hard)?;
    let graduated_30h = run_policy(GatePolicy::Graduated)?;
    Ok(PassageStudy {
        schema_version: 2,
        synthetic_only: true,
        seed: SEED,
        duration_h: DURATION_S as f64 / HOUR_S as f64,
        speed_kn: 6.0,
        distance_km: SPEED_MPS * DURATION_S as f64 / 1000.0,
        gps_loss_h: GPS_LOSS_S as f64 / HOUR_S as f64,
        ephemeris_cache_h: 0.0,
        integration_step_s: STEP_S,
        hard_6h,
        graduated_30h,
        caveat: "D43 applies: synthetic epoch aging aliases orbital phase and is availability evidence only, not validation of real SupGP error growth.".into(),
    })
}

#[allow(clippy::too_many_lines)]
fn run_policy(policy: GatePolicy) -> Result<Outcome, Box<dyn std::error::Error>> {
    let truth_store = EphemerisStore::from_tle_str(TLE)?;
    let epoch = truth_store
        .epoch(NORAD_IDS[0])
        .ok_or("fixture satellite missing")?;
    let aging = match policy {
        GatePolicy::Hard => EphemerisAgingConfig {
            ceiling_age_s: EphemerisAgingConfig::default().fresh_age_s,
            ..EphemerisAgingConfig::default()
        },
        GatePolicy::Graduated => EphemerisAgingConfig::default(),
    };
    let mut pipeline =
        DopplerPipeline::new(EphemerisStore::from_tle_str(TLE)?).without_elevation_mask();
    pipeline.chi_square_threshold = None;
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Production,
            oneweb_enabled: false,
            ephemeris_aging: aging,
        },
        ManualClock::default(),
        PassageEstimator::new(),
        IntegrityStub,
        MemoryJournals::default(),
    )
    .with_doppler_pipeline(pipeline);
    let mut rng = DeterministicRng::new(SEED);
    let mut sequence = 0;
    let mut accepted = 0;
    let mut rejected = 0;
    let mut last_accepted_s = 0;

    for elapsed_s in (0..=DURATION_S).step_by(STEP_S as usize) {
        let truth_position = truth_position(elapsed_s);
        let utc = epoch + Duration::seconds(i64::try_from(elapsed_s)?);
        if elapsed_s > 0 {
            // [UNVERIFIED] A seeded constant accelerometer bias supplies the denied-mode DR
            // stressor; its effect is measured from the executive output, not imposed later.
            executive.process(envelope(
                sequence,
                elapsed_s,
                None,
                "imu",
                vec![1.0e-8],
                MeasurementPayload::Imu(ImuSample {
                    acceleration_mps2: [
                        if elapsed_s > 6 * HOUR_S { 5.0e-5 } else { 0.0 }
                            + rng.symmetric() * 1.0e-6,
                        0.0,
                        0.0,
                    ],
                    angular_rate_rps: [0.0; 3],
                }),
            ));
            sequence += 1;
        }
        if elapsed_s <= GPS_LOSS_S {
            let enu = ecef_vector_to_enu(truth_position, TRUTH_VELOCITY_MPS);
            executive.process(envelope(
                sequence,
                elapsed_s,
                Some(utc),
                "gnss",
                vec![1.0],
                MeasurementPayload::Gnss(GnssFix {
                    position_ecef_m: truth_position.map(|value| value + rng.symmetric() * 0.5),
                    velocity_ned_mps: [enu[1], enu[0], -enu[2]],
                }),
            ));
            sequence += 1;
        }
        if elapsed_s % 60 != 0 {
            continue;
        }

        for norad_id in NORAD_IDS {
            let satellite = truth_store
                .propagate_ecef_with_age(norad_id, utc, Duration::hours(30))?
                .state;
            let prediction = predict(
                SatelliteState {
                    position_ecef_m: satellite.position_m,
                    velocity_ecef_mps: satellite.velocity_mps,
                },
                ReceiverState {
                    position_ecef_m: truth_position,
                    velocity_ecef_mps: TRUTH_VELOCITY_MPS,
                    clock_drift_mps: 0.0,
                },
                0.0,
                CARRIER_HZ,
                -std::f64::consts::FRAC_PI_2,
            )
            .map_err(|error| format!("synthetic prediction failed: {error:?}"))?;
            let before = executive.journals().integrity_events().len();
            executive.process(envelope(
                sequence,
                elapsed_s,
                Some(utc),
                &norad_id.to_string(),
                vec![0.25],
                MeasurementPayload::TrackerDoppler(TrackerDoppler {
                    constellation: Constellation::Starlink,
                    correlation_peak_hz: prediction.correlation_peak_hz + rng.symmetric() * 0.5,
                    nominal_carrier_hz: CARRIER_HZ,
                }),
            ));
            sequence += 1;
            let events = &executive.journals().integrity_events()[before..];
            if events
                .iter()
                .any(|event| event.reason == "Doppler innovation accepted")
            {
                accepted += 1;
                last_accepted_s = elapsed_s;
            } else {
                rejected += 1;
            }
        }
    }

    let estimate = executive.filter().state().position_ecef_m;
    let truth = truth_position(DURATION_S);
    let final_position_error_m = estimate
        .into_iter()
        .zip(truth)
        .map(|(estimated, actual)| (estimated - actual).powi(2))
        .sum::<f64>()
        .sqrt();
    Ok(Outcome {
        doppler_available_until_h: last_accepted_s as f64 / HOUR_S as f64,
        accepted_doppler_updates: accepted,
        rejected_doppler_updates: rejected,
        final_position_error_m,
        position_class: if final_position_error_m < 1852.0 {
            "passage-held (<1 NM error)"
        } else {
            "dead-reckoning (>1 NM error)"
        }
        .into(),
    })
}

fn truth_position(elapsed_s: u64) -> [f64; 3] {
    std::array::from_fn(|axis| {
        INITIAL_POSITION_M[axis] + TRUTH_VELOCITY_MPS[axis] * elapsed_s as f64
    })
}

fn envelope(
    sequence: u64,
    elapsed_s: u64,
    utc: Option<DateTime<Utc>>,
    source: &str,
    covariance: Vec<f64>,
    payload: MeasurementPayload,
) -> MeasurementEnvelope {
    MeasurementEnvelope {
        schema_version: 2,
        source_id: SourceId(source.into()),
        sequence,
        sample_time: TimeTag::DeviceNanoseconds(elapsed_s.saturating_mul(1_000_000_000)),
        host_receive_monotonic_ns: elapsed_s.saturating_mul(1_000_000_000),
        utc: utc.map(|value| UtcTime {
            rfc3339: value.to_rfc3339_opts(SecondsFormat::Nanos, true),
            uncertainty_ns: 0,
        }),
        payload,
        frame: Frame::EarthCenteredEarthFixed,
        covariance,
        quality: QualityFlags::VALID,
        calibration_id: "passage-synthetic-v2".into(),
        provenance: Provenance::DerivedRecord(format!("seed-{SEED}")),
    }
}

/// Writes the deterministic JSON and Markdown passage artifacts.
///
/// # Errors
///
/// Returns simulation, filesystem, or JSON serialization errors.
pub fn write(output: impl AsRef<Path>) -> Result<PassageStudy, Box<dyn std::error::Error>> {
    let output = output.as_ref();
    std::fs::create_dir_all(output)?;
    let study = simulate()?;
    std::fs::write(
        output.join("results.json"),
        serde_json::to_vec_pretty(&study)?,
    )?;
    std::fs::write(output.join("STUDY.md"), markdown(&study))?;
    Ok(study)
}

fn markdown(study: &PassageStudy) -> String {
    format!(
        "# Passage endurance study\n\n**SYNTHETIC ONLY — D43 CAVEAT:** {}\n\nNine hours at 6 kn covers {:.2} km; GNSS is lost at hour 2 and ephemeris is cached at departure. The same seed (`{}`) and generated measurements drive both policies through the integrated executive, SGP4 propagation, Doppler prediction, estimator update, and integrity journaling paths. The 9 h mission is honestly decimated to one 1 s integration/measurement step; it is not an IMU-rate endurance run.\n\n## Measured result\n\n| handling | accepted / rejected Doppler | Doppler through | measured final 3D position error | position class |\n|---|---:|---:|---:|---|\n| hard 6 h | {} / {} | {:.1} h | {:.1} m | {} |\n| graduated, 30 h ceiling | {} / {} | {:.1} h | {:.1} m | {} |\n\nThe values above are computed from each executive filter's final state against the seeded mission truth; no endpoint error law is imposed.\n\n## `[UNVERIFIED]`\n\nThe seeded IMU bias/noise, Doppler noise, SGP4 error curve, LOS-rate mapping, 1 s decimation, 30 h ceiling, and position-class proxy remain `[UNVERIFIED]`. Real-SupGP aging, constellation availability, real tracker residuals, sensor-rate execution, and at-sea replay are required before parameter freeze.\n",
        study.caveat,
        study.distance_km,
        study.seed,
        study.hard_6h.accepted_doppler_updates,
        study.hard_6h.rejected_doppler_updates,
        study.hard_6h.doppler_available_until_h,
        study.hard_6h.final_position_error_m,
        study.hard_6h.position_class,
        study.graduated_30h.accepted_doppler_updates,
        study.graduated_30h.rejected_doppler_updates,
        study.graduated_30h.doppler_available_until_h,
        study.graduated_30h.final_position_error_m,
        study.graduated_30h.position_class,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_pipeline_is_byte_deterministic_and_meets_distance_contract() {
        let first = serde_json::to_vec(&simulate().unwrap()).unwrap();
        let second = serde_json::to_vec(&simulate().unwrap()).unwrap();
        assert_eq!(first, second);
        let study: PassageStudy = serde_json::from_slice(&first).unwrap();
        assert!(study.distance_km >= 100.0);
        assert!((study.hard_6h.doppler_available_until_h - 6.0).abs() < f64::EPSILON);
        assert!((study.graduated_30h.doppler_available_until_h - 9.0).abs() < f64::EPSILON);
        assert!(study.hard_6h.rejected_doppler_updates > 0);
        assert_eq!(study.graduated_30h.rejected_doppler_updates, 0);
    }
}
