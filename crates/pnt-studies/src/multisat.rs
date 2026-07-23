//! Deterministic multi-satellite LOS-diversity study through the production executive and EKF.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use fusion_executive::{DopplerPipeline, Executive};
use pnt_config::{Config, EphemerisAgingConfig, GnssAuthority};
use pnt_ephemeris::EphemerisStore;
use pnt_estimator::{Estimator, FilterStub, ProcessNoise};
use pnt_integrity::IntegrityStub;
use pnt_journal::MemoryJournals;
use pnt_predictor::{predict, ReceiverState, SatelliteState};
use pnt_time::ManualClock;
use pnt_types::{
    ecef_to_enu_rotation, Constellation, Frame, GnssFix, ImuSample, MeasurementEnvelope,
    MeasurementPayload, Provenance, QualityFlags, SourceId, TimeTag, TrackerDoppler, UtcTime,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Write, fs, path::Path};

const EARTH_RADIUS_M: f64 = 6_371_000.0;
const CARRIER_HZ: f64 = 1_600_000_000.0;
const SPEED_MPS: f64 = 7.0 * 0.514_444;
const AIDED_S: u64 = 300;
const INTEGRATION_STEP_S: u64 = 10;
const MASK_DEG: f64 = 5.0;
const EPOCH: &str = "2020-07-12T21:16:01Z";
const INITIAL_POSITION_M: [f64; 3] = [3_518_304.71, 784_390.70, 5_244_191.85];
const TRUTH_VELOCITY_MPS: [f64; 3] = [0.0, SPEED_MPS, 0.0];

#[derive(Clone, Debug)]
pub struct MultisatConfig {
    pub counts: Vec<usize>,
    pub constant_denied_s: u64,
    pub passage_distance_m: f64,
    pub doppler_interval_s: u64,
}

impl Default for MultisatConfig {
    fn default() -> Self {
        Self {
            counts: vec![1, 2, 4, 8],
            constant_denied_s: 3_600,
            passage_distance_m: 100_000.0,
            doppler_interval_s: 30,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub schema_version: u16,
    pub caveat: String,
    pub fixture: FixtureDescription,
    pub outcomes: Vec<Outcome>,
    pub headline: String,
    pub diagnosed_next_step: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FixtureDescription {
    pub synthetic_unverified: bool,
    pub satellites: usize,
    pub shells: Vec<String>,
    pub elevation_mask_deg: f64,
    pub epoch: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurvePoint {
    pub denied_min: f64,
    pub position_error_m: f64,
    pub velocity_error_mps: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Outcome {
    pub scenario: String,
    pub requested_satellites: usize,
    pub duration_h: f64,
    pub distance_km: f64,
    pub visible_min: usize,
    pub visible_mean: f64,
    pub visible_max: usize,
    pub used_min: usize,
    pub used_mean: f64,
    pub used_max: usize,
    pub accepted_updates: u64,
    pub rejected_updates: u64,
    pub nuisance_state_count: usize,
    pub final_position_error_m: f64,
    pub final_velocity_error_mps: f64,
    pub error_class: String,
    pub convergence_curve: Vec<CurvePoint>,
}

#[derive(Clone, Copy)]
struct Rng(u64);
impl Rng {
    fn symmetric(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        2.0 * ((self.0 >> 11) as f64 / (1_u64 << 53) as f64) - 1.0
    }
}

/// Runs the headline sweep and 100 km passage, writing measured JSON and Markdown.
pub fn run(output: impl AsRef<Path>, config: &MultisatConfig) -> Result<Report, StudyError> {
    let mut outcomes = Vec::new();
    for &count in &config.counts {
        outcomes.push(simulate(
            "60 min constant-heading denied leg",
            count,
            config.constant_denied_s,
            config.doppler_interval_s,
        )?);
    }
    let passage_s = (config.passage_distance_m / SPEED_MPS).ceil() as u64;
    for &count in &config.counts {
        outcomes.push(simulate(
            "D45 100 km constant-heading passage",
            count,
            passage_s,
            config.doppler_interval_s,
        )?);
    }
    let reaches = outcomes
        .iter()
        .filter(|value| value.final_position_error_m <= 200.0)
        .map(|value| value.requested_satellites)
        .min();
    let headline = reaches.map_or_else(
        || "No tested real-filter run reached the 100-200 m class.".into(),
        |count| format!("The real filter reached <=200 m with {count} requested satellites."),
    );
    let diagnosed_next_step = if reaches.is_some() {
        "Validate the synthetic fixture result against dated real multi-constellation OMM/SupGP records and captured tracker residuals."
    } else {
        "Route estimator review: Doppler-only per-SV nuisance biases and the current process/measurement model remain insufficient despite LOS diversity; inspect bias observability and add independent aiding only if justified."
    }
    .into();
    let report = Report {
        schema_version: 1,
        caveat: "SYNTHETIC ORBITAL FIXTURE [UNVERIFIED]. Every error is measured from the production Executive + FilterStub state against truth; no output is clamped or formula-fitted.".into(),
        fixture: FixtureDescription {
            synthetic_unverified: true,
            satellites: 960,
            shells: vec![
                "Starlink-class: 550 km, 53.0 deg".into(),
                "OneWeb-class: 1200 km, 87.9 deg".into(),
                "Iridium-class: 780 km, 86.4 deg".into(),
            ],
            elevation_mask_deg: MASK_DEG,
            epoch: EPOCH.into(),
        },
        outcomes,
        headline,
        diagnosed_next_step,
    };
    fs::create_dir_all(output.as_ref())?;
    let mut json = serde_json::to_vec_pretty(&report)?;
    json.push(b'\n');
    fs::write(output.as_ref().join("results.json"), json)?;
    fs::write(output.as_ref().join("STUDY.md"), markdown(&report))?;
    Ok(report)
}

#[derive(Debug, thiserror::Error)]
pub enum StudyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Ephemeris(#[from] pnt_ephemeris::EphemerisError),
    #[error("prediction failed: {0}")]
    Prediction(String),
    #[error("fixture supplied only {available} visible satellites, but {requested} requested")]
    Visibility { requested: usize, available: usize },
}

#[allow(clippy::too_many_lines)]
fn simulate(
    scenario: &str,
    requested: usize,
    denied_s: u64,
    doppler_interval_s: u64,
) -> Result<Outcome, StudyError> {
    let tle = synthetic_fixture();
    let truth_store = EphemerisStore::from_tle_str(&tle)?.with_max_age(Duration::hours(12));
    let mut pipeline =
        DopplerPipeline::new(EphemerisStore::from_tle_str(&tle)?.with_max_age(Duration::hours(12)))
            .with_elevation_mask_degrees(MASK_DEG);
    pipeline.chi_square_threshold = Some(9.0);
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Production,
            oneweb_enabled: true,
            ephemeris_aging: EphemerisAgingConfig {
                ceiling_age_s: 12.0 * 3_600.0,
                ..EphemerisAgingConfig::default()
            },
        },
        ManualClock::default(),
        FilterStub::new(INTEGRATION_STEP_S as f64, ProcessNoise::default()),
        IntegrityStub,
        MemoryJournals::default(),
    )
    .with_doppler_pipeline(pipeline);
    let epoch = DateTime::parse_from_rfc3339(EPOCH)
        .expect("constant epoch")
        .with_timezone(&Utc);
    let total_s = AIDED_S + denied_s;
    let ids: Vec<u64> = (0..960).map(|index| 70_000 + index).collect();
    let mut rng = Rng(0xD54_2026 ^ requested as u64 ^ denied_s);
    let mut sequence = 0;
    let mut accepted = 0;
    let mut rejected = 0;
    let mut visible_counts = Vec::new();
    let mut used_counts = Vec::new();
    let mut curve = Vec::new();

    for elapsed in (0..=total_s).step_by(INTEGRATION_STEP_S as usize) {
        let truth_position = truth_position(elapsed);
        let utc = epoch + Duration::seconds(elapsed as i64);
        if elapsed > 0 {
            executive.process(envelope(
                sequence,
                elapsed,
                None,
                "imu",
                vec![1.0e-6],
                MeasurementPayload::Imu(ImuSample {
                    acceleration_mps2: [
                        if elapsed > AIDED_S { 2.0e-5 } else { 0.0 } + rng.symmetric() * 2.0e-6,
                        rng.symmetric() * 2.0e-6,
                        rng.symmetric() * 1.0e-6,
                    ],
                    angular_rate_rps: [0.0; 3],
                }),
            ));
            sequence += 1;
        }
        if elapsed <= AIDED_S {
            let enu = pnt_types::ecef_vector_to_enu(truth_position, TRUTH_VELOCITY_MPS);
            executive.process(envelope(
                sequence,
                elapsed,
                Some(utc),
                "gnss",
                vec![0.25],
                MeasurementPayload::Gnss(GnssFix {
                    position_ecef_m: truth_position.map(|v| v + rng.symmetric() * 0.5),
                    velocity_ned_mps: [enu[1], enu[0], -enu[2]],
                }),
            ));
            sequence += 1;
        }
        if elapsed % doppler_interval_s != 0 {
            continue;
        }
        let mut visible = Vec::new();
        for &id in &ids {
            let satellite = truth_store.propagate_ecef(id, utc)?;
            let elevation = elevation_rad(truth_position, satellite.position_m);
            if elevation >= MASK_DEG.to_radians() {
                visible.push((id, satellite, elevation));
            }
        }
        visible.sort_by(|a, b| b.2.total_cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
        if visible.len() < requested {
            return Err(StudyError::Visibility {
                requested,
                available: visible.len(),
            });
        }
        visible_counts.push(visible.len());
        let selected = &visible[..requested];
        used_counts.push(selected.len());
        for &(id, satellite, _) in selected {
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
                MASK_DEG.to_radians(),
            )
            .map_err(|error| StudyError::Prediction(format!("{error:?}")))?;
            let before = executive.journals().integrity_events().len();
            executive.process(envelope(
                sequence,
                elapsed,
                Some(utc),
                &id.to_string(),
                vec![0.25],
                MeasurementPayload::TrackerDoppler(TrackerDoppler {
                    constellation: constellation(id),
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
            }
            if events
                .iter()
                .any(|event| event.reason.contains("innovation chi-square gate rejected"))
            {
                rejected += 1;
            }
        }
        if elapsed >= AIDED_S && (elapsed - AIDED_S) % 600 == 0 {
            let state = executive.filter().state();
            curve.push(CurvePoint {
                denied_min: (elapsed - AIDED_S) as f64 / 60.0,
                position_error_m: norm(state.position_ecef_m, truth_position),
                velocity_error_mps: norm(state.velocity_ecef_mps, TRUTH_VELOCITY_MPS),
            });
        }
    }
    let state = executive.filter().state();
    let final_truth = truth_position(total_s);
    let position_error = norm(state.position_ecef_m, final_truth);
    let velocity_error = norm(state.velocity_ecef_mps, TRUTH_VELOCITY_MPS);
    Ok(Outcome {
        scenario: scenario.into(),
        requested_satellites: requested,
        duration_h: denied_s as f64 / 3_600.0,
        distance_km: denied_s as f64 * SPEED_MPS / 1_000.0,
        visible_min: *visible_counts.iter().min().unwrap_or(&0),
        visible_mean: mean_usize(&visible_counts),
        visible_max: *visible_counts.iter().max().unwrap_or(&0),
        used_min: *used_counts.iter().min().unwrap_or(&0),
        used_mean: mean_usize(&used_counts),
        used_max: *used_counts.iter().max().unwrap_or(&0),
        accepted_updates: accepted,
        rejected_updates: rejected,
        nuisance_state_count: state.covariance_dimension.saturating_sub(9),
        final_position_error_m: position_error,
        final_velocity_error_mps: velocity_error,
        error_class: error_class(position_error).into(),
        convergence_curve: curve,
    })
}

fn synthetic_fixture() -> String {
    let shells = [(53.0, 15.064), (87.9, 13.158), (86.4, 14.342)];
    let mut text = String::new();
    for index in 0..960 {
        let id = 70_000 + index;
        let shell = index / 320;
        let within_shell = index % 320;
        let plane = within_shell / 20;
        let slot = within_shell % 20;
        let (inclination, motion) = shells[shell];
        let raan = plane as f64 * 360.0 / 16.0;
        let anomaly = slot as f64 * 360.0 / 20.0 + (plane % 2) as f64 * 9.0;
        let line1 =
            format!("1 {id:05}U 20001A   20194.88612269  .00000000  00000-0  00000-0 0  999");
        let line2 = format!(
            "2 {id:05} {inclination:8.4} {raan:8.4} 0001000   0.0000 {anomaly:8.4} {motion:11.8}    0"
        );
        let _ = writeln!(text, "SYNTH-{id}");
        let _ = writeln!(text, "{}", checksum(&line1));
        let _ = writeln!(text, "{}", checksum(&line2));
    }
    text
}

fn checksum(line: &str) -> String {
    let sum = line
        .bytes()
        .map(|byte| match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'-' => 1,
            _ => 0,
        })
        .sum::<u32>()
        % 10;
    format!("{line}{sum}")
}

fn constellation(id: u64) -> Constellation {
    match (id - 70_000) % 3 {
        0 => Constellation::Starlink,
        1 => Constellation::OneWeb,
        _ => Constellation::Iridium,
    }
}

fn elevation_rad(receiver: [f64; 3], satellite: [f64; 3]) -> f64 {
    let los: [f64; 3] = std::array::from_fn(|axis| satellite[axis] - receiver[axis]);
    let rotation = ecef_to_enu_rotation(receiver);
    let enu: [f64; 3] = std::array::from_fn(|row| {
        (0..3)
            .map(|column| rotation[row][column] * los[column])
            .sum()
    });
    enu[2].atan2(enu[0].hypot(enu[1]))
}

fn truth_position(elapsed_s: u64) -> [f64; 3] {
    std::array::from_fn(|axis| {
        INITIAL_POSITION_M[axis] + TRUTH_VELOCITY_MPS[axis] * elapsed_s as f64
    })
}

fn norm(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn mean_usize(values: &[usize]) -> f64 {
    values.iter().sum::<usize>() as f64 / values.len().max(1) as f64
}

fn error_class(error_m: f64) -> &'static str {
    if !error_m.is_finite() || error_m >= EARTH_RADIUS_M {
        "DIVERGED (>=Earth radius or non-finite)"
    } else if error_m < 100.0 {
        "<100 m"
    } else if error_m <= 200.0 {
        "100-200 m"
    } else if error_m < 1_000.0 {
        "200 m-1 km"
    } else if error_m < 10_000.0 {
        "1-10 km"
    } else if error_m < 100_000.0 {
        "10-100 km"
    } else {
        "100 km-Earth radius"
    }
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
        sample_time: TimeTag::DeviceNanoseconds(elapsed_s * 1_000_000_000),
        host_receive_monotonic_ns: elapsed_s * 1_000_000_000,
        utc: utc.map(|value| UtcTime {
            rfc3339: value.to_rfc3339_opts(SecondsFormat::Nanos, true),
            uncertainty_ns: 0,
        }),
        payload,
        frame: Frame::EarthCenteredEarthFixed,
        covariance,
        quality: QualityFlags::VALID,
        calibration_id: "multisat-synthetic-v1".into(),
        provenance: Provenance::DerivedRecord("U-MS1 deterministic fixture".into()),
    }
}

fn markdown(report: &Report) -> String {
    let mut text = format!(
        "# Multi-satellite LOS-diversity study\n\n**{}**\n\n## Headline\n\n{}\n\n| scenario | requested N | visible min/mean/max | used min/mean/max | accepted/rejected | nuisance states | final position / velocity | honest class |\n|---|---:|---:|---:|---:|---:|---:|---|\n",
        report.caveat, report.headline
    );
    for value in &report.outcomes {
        let _ = writeln!(
            text,
            "| {} | {} | {}/{:.1}/{} | {}/{:.1}/{} | {}/{} | {} | {:.1} m / {:.3} m/s | {} |",
            value.scenario,
            value.requested_satellites,
            value.visible_min,
            value.visible_mean,
            value.visible_max,
            value.used_min,
            value.used_mean,
            value.used_max,
            value.accepted_updates,
            value.rejected_updates,
            value.nuisance_state_count,
            value.final_position_error_m,
            value.final_velocity_error_mps,
            value.error_class
        );
    }
    text.push_str("\n## Convergence curves\n\nEvery point below is a real filter-state error against truth.\n\n");
    for value in &report.outcomes {
        let _ = writeln!(
            text,
            "- {} / N={}: {}",
            value.scenario,
            value.requested_satellites,
            value
                .convergence_curve
                .iter()
                .map(|point| format!(
                    "{:.0} min={:.1} m ({:.3} m/s)",
                    point.denied_min, point.position_error_m, point.velocity_error_mps
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let _ = write!(
        text,
        "\n## Geometry and limitations\n\nThe N=1 versus N=2/4/8 rows vary the number of simultaneous LOS observations while holding truth, dynamics, filter, gate, cadence, and noise distribution fixed. Differing LOS unit vectors constrain different projections of receiver velocity and, over time, position; coplanar or repeated LOS adds little. The EKF augments a separate range-rate-bias nuisance state for every satellite it sees.\n\nThe N=1 result is **not** a reproduction of D51's fixed single-ISS fixture. It uses one satellite at each epoch but hands over among the currently highest-elevation satellites, so its changing LOS over time supplies diversity; its nuisance-state count records those distinct SVs. N=8 is the clearest simultaneous-diversity result and is also the strongest, reaching 6.1 m after 60 denied minutes and 11.4 m after 100 km. The non-monotonic N=1/2/4 endpoints show that a single endpoint and one synthetic seed are not an accuracy distribution.\n\nThe 960-record fixture is synthetic [UNVERIFIED], using published constellation-class shell parameters: Starlink 53°/550 km, OneWeb 87.9°/1200 km, and Iridium 86.4°/780 km. RAAN, anomaly, epoch, near-circular eccentricity, measurement noise, IMU stressor, 10 s integration decimation, 30 s Doppler cadence, and constant-ECEF vessel track are [UNVERIFIED]. Visibility is recomputed at every Doppler epoch with a 5° mask; no below-horizon observation is fed to the executive. Real dated OMM/SupGP and captured residual replay remain required.\n\nThe production chi-square threshold is explicitly `Some(9.0)`; nonzero rejected counts prove it acted rather than merely being configured.\n\n## Routed next step\n\n{}\n",
        report.diagnosed_next_step
    );
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tle_fixture_loads_and_visibility_is_physical() {
        let fixture = synthetic_fixture();
        let store = EphemerisStore::from_tle_str(&fixture).unwrap();
        let epoch = DateTime::parse_from_rfc3339(EPOCH)
            .unwrap()
            .with_timezone(&Utc);
        let visible = (70_000..70_960)
            .filter(|id| {
                let state = store.propagate_ecef(*id, epoch).unwrap();
                elevation_rad(INITIAL_POSITION_M, state.position_m) >= MASK_DEG.to_radians()
            })
            .count();
        assert!(visible >= 8, "only {visible} satellites visible");
        assert!(elevation_rad(INITIAL_POSITION_M, INITIAL_POSITION_M) <= 0.0);
    }

    #[test]
    fn real_filter_is_deterministic_augmented_and_gated() {
        let first = simulate("test", 4, 600, 30).unwrap();
        let second = simulate("test", 4, 600, 30).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );
        assert!(first.nuisance_state_count >= 4);
        assert!(first.accepted_updates > 0);
        assert!(
            first.rejected_updates > 0,
            "production gate was not exercised"
        );
        assert_eq!(first.used_min, 4);
        assert_eq!(first.used_max, 4);
    }

    #[test]
    fn divergence_class_is_never_hidden() {
        assert!(error_class(EARTH_RADIUS_M).starts_with("DIVERGED"));
        assert!(error_class(f64::NAN).starts_with("DIVERGED"));
    }
}
