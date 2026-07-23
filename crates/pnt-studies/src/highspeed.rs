//! Deterministic D46/D47 high-speed mission study through the real generator, executive and EKF.

use fusion_executive::{DopplerPipeline, Executive};
use pnt_config::{Config, EphemerisAgingConfig, GnssAuthority};
use pnt_ephemeris::EphemerisStore;
use pnt_estimator::{Estimator, FilterStub, ProcessNoise};
use pnt_integrity::IntegrityStub;
use pnt_journal::{
    MeasurementJournalRecord, MeasurementReader, MemoryJournals, TruthJournalRecord, TruthReader,
};
use pnt_mission::{
    generate_mission, CoordinatedTurnConfig, MissionConfig, SpeedScaledImuConfig, WaveSlamConfig,
};
use pnt_time::ManualClock;
use pnt_types::{FilterState, GnssFix, MeasurementEnvelope, MeasurementPayload, TimeTag};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Write, fs, path::Path};
use tempfile::TempDir;

const KNOT_MPS: f64 = 0.514_444;
const DENIED_DISTANCE_M: f64 = 100_000.0;
const AIDED_STEADY_S: u64 = 300;
const EPHEMERIS_CEILING_S: f64 = 30.0 * 3_600.0;
const MANOEUVRE_RATE_RAD_S: f64 = 3.0_f64.to_radians();
const STEADY_COVARIANCE_RELATIVE_DELTA: f64 = 0.01;
const EARTH_RADIUS_M: f64 = 6_371_000.0;
const CONVERGED_POSITION_ERROR_M: f64 = 500.0;
const CONVERGED_VELOCITY_ERROR_MPS: f64 = 0.5;
const TLE: &str = include_str!("../../pnt-ephemeris/tests/fixtures/iss.tle");

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProcessNoiseScale {
    pub acceleration: f64,
    pub turn_rate: f64,
    pub clock_drift: f64,
    pub nuisance_random_walk: f64,
}

impl ProcessNoiseScale {
    #[must_use]
    pub fn apply(self, base: ProcessNoise) -> ProcessNoise {
        ProcessNoise {
            acceleration_variance: base.acceleration_variance * self.acceleration,
            turn_rate_variance: base.turn_rate_variance * self.turn_rate,
            clock_drift_variance: base.clock_drift_variance * self.clock_drift,
            nuisance_random_walk_variance: base.nuisance_random_walk_variance
                * self.nuisance_random_walk,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SpeedRegime {
    pub name: String,
    pub speed_kn: f64,
    pub process_noise_scale: ProcessNoiseScale,
    pub wave_peak_mps2: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HighSpeedConfig {
    pub seed: u64,
    pub aided_steady_s: u64,
    pub denied_distance_m: f64,
    pub ephemeris_ceiling_s: f64,
    pub regimes: Vec<SpeedRegime>,
}

impl Default for HighSpeedConfig {
    fn default() -> Self {
        Self {
            seed: 0xD47_2026,
            aided_steady_s: AIDED_STEADY_S,
            denied_distance_m: DENIED_DISTANCE_M,
            ephemeris_ceiling_s: EPHEMERIS_CEILING_S,
            regimes: vec![
                regime("displacement", 7.0, [1.0, 1.0, 1.0, 1.0], 6.10),
                regime("planing", 20.0, [6.0, 4.0, 1.0, 2.0], 6.10),
                regime("exploratory", 30.0, [10.0, 7.0, 1.0, 3.0], 11.22),
            ],
        }
    }
}

fn regime(name: &str, speed_kn: f64, scale: [f64; 4], wave_peak_mps2: f64) -> SpeedRegime {
    SpeedRegime {
        name: name.into(),
        speed_kn,
        process_noise_scale: ProcessNoiseScale {
            acceleration: scale[0],
            turn_rate: scale[1],
            clock_drift: scale[2],
            nuisance_random_walk: scale[3],
        },
        wave_peak_mps2,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LossState {
    pub position_error_m: f64,
    pub velocity_error_mps: f64,
    pub position_covariance_trace_m2: f64,
    pub covariance_dimension: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Convergence {
    pub time_s: Option<f64>,
    pub truth_distance_m: Option<f64>,
    pub criterion: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PassageResult {
    pub regime: String,
    pub speed_kn: f64,
    pub denied_distance_km: f64,
    pub denied_duration_h: f64,
    pub loss_state: LossState,
    pub position_error_rms_m: f64,
    pub position_error_p95_m: f64,
    pub landfall_position_error_m: f64,
    pub velocity_error_rms_mps: f64,
    pub velocity_error_p95_mps: f64,
    pub position_error_class: String,
    pub ephemeris_age_h: f64,
    pub ephemeris_age_margin_h: f64,
    pub accepted_doppler_updates: u64,
    pub rejected_doppler_updates: u64,
    pub aged_doppler_updates: u64,
    pub manoeuvre_convergence: Convergence,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub schema_version: u32,
    pub caveat: String,
    pub scenario: String,
    pub wave_model: String,
    pub results: Vec<PassageResult>,
    pub d50_consistency: String,
    pub integration_notes: Vec<String>,
}

/// Runs the generator-produced missions through `Executive<FilterStub>` and writes artifacts.
///
/// # Errors
///
/// Returns a mission, journal, executive-fixture, I/O, or JSON error.
pub fn run(output: impl AsRef<Path>, config: &HighSpeedConfig) -> Result<Report, StudyError> {
    let mut results = Vec::with_capacity(config.regimes.len());
    for speed in &config.regimes {
        results.push(simulate(speed, config)?);
    }
    let report = Report {
        schema_version: 3,
        caveat: "SYNTHETIC CAPABILITY/PLUMBING DEMONSTRATION [UNVERIFIED]; not a navigation-performance or denied-authority claim.".into(),
        scenario: "Aided until covariance steady state is verified at 300 s, then GNSS denied for 100 km at each of 7/20/30 kn; the real EKF uses the production chi-square gate (9.0), wave/slam is on, and graduated ephemeris aging is on.".into(),
        wave_model: "Zero-mean full-cycle acceleration integrated into both truth and IMU. 100-450 ms duration and 0.44 g RMS anchor are R5-sourced; 0.25 s, opportunity rate, pitch coupling, speed scaling, and mapping 0.44 g RMS to 6.10 m/s^2 sinusoidal peak are [UNVERIFIED]. The 30 kn 1.84x peak scale is [UNVERIFIED].".into(),
        results,
        d50_consistency: "Consistent with D50: this synthetic run demonstrates plumbing only. It does not support denied operation at 20 kn; 30 kn remains aided/manual-only and exploratory, with no denied autonomous authority.".into(),
        integration_notes: vec![
            "This ISS-TLE-only fixture supplies one satellite. Single-satellite range-rate geometry is near-unobservable for position, so its bounded roughly-ten-to-tens-of-kilometres errors are not the 100-200 m multi-satellite class.".into(),
            "D51 reconciliation: U-P1's smaller absolute errors came from a deliberately clamped toy PassageEstimator that cannot diverge; this study uses the real EKF with its production gate. U-P1 remains evidence only for the relative graduated-vs-hard aging comparison.".into(),
            "Position, velocity, covariance and any valid reconvergence are read from the real EKF state against generator truth; divergent runs are explicitly flagged and receive no reconvergence metric.".into(),
            "The 90-degree turn runs at a sharp, realistic 3 deg/s. Distance-to-reconvergence is accumulated from truth positions after that actual turn, independently of elapsed time.".into(),
            "Graduated aging is exercised by the executive Doppler pipeline; the 30 h ceiling and synthetic TLE aging remain [UNVERIFIED] per D43/D45.".into(),
            "OPEN / REQUIRED: build and run a multi-satellite fixture study before claiming the 100-200 m denied position class.".into(),
        ],
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
    Mission(#[from] pnt_mission::MissionError),
    #[error(transparent)]
    Journal(#[from] pnt_journal::JournalError),
    #[error(transparent)]
    Ephemeris(#[from] pnt_ephemeris::EphemerisError),
    #[error("generated mission has no truth at the loss instant")]
    MissingLossTruth,
    #[error("generated mission has no filter sample at the denied endpoint")]
    MissingEndpoint,
    #[error("aided covariance did not reach steady state before GNSS loss")]
    AidedCovarianceNotSteady,
}

#[derive(Clone)]
struct Sample {
    elapsed_s: f64,
    truth: GnssFix,
    state: FilterState,
    position_error_m: f64,
    velocity_error_mps: f64,
}

#[allow(clippy::too_many_lines)]
fn simulate(regime: &SpeedRegime, study: &HighSpeedConfig) -> Result<PassageResult, StudyError> {
    let speed_mps = regime.speed_kn * KNOT_MPS;
    let denied_s = (study.denied_distance_m / speed_mps).ceil() as u64;
    let duration_s = study.aided_steady_s + denied_s;
    let mission_dir = TempDir::new()?;
    let mission = MissionConfig {
        seed: study.seed,
        duration_s,
        imu_rate_hz: 20,
        speed_through_water_mps: speed_mps,
        imu_bias_mps2: [2.0e-4, -1.0e-4, 1.0e-4],
        imu_noise_std_mps2: 5.0e-4,
        gnss_noise_std_m: 0.5,
        elevation_mask_rad: -std::f64::consts::FRAC_PI_2,
        coordinated_turn: Some(CoordinatedTurnConfig {
            rate_rad_s: MANOEUVRE_RATE_RAD_S,
        }),
        wave_slam: Some(WaveSlamConfig {
            burst_rate_hz: 0.08,
            duration_s: 0.25,
            vertical_peak_mps2: regime.wave_peak_mps2,
            pitch_coupling: 0.18,
        }),
        speed_scaled_imu: Some(SpeedScaledImuConfig {
            reference_speed_mps: 7.0 * KNOT_MPS,
            noise_per_speed_ratio: 0.12,
            bias_per_speed_ratio: 0.08,
        }),
        doppler_interval_s: 1_800,
        ephemeris_start_age_s: Some(0),
        ..MissionConfig::default()
    };
    generate_mission(mission_dir.path(), &mission)?;

    let truth = load_truth(mission_dir.path())?;
    let process_noise = regime.process_noise_scale.apply(ProcessNoise::default());
    let mut pipeline =
        DopplerPipeline::new(EphemerisStore::from_tle_str(TLE)?).without_elevation_mask();
    pipeline.chi_square_threshold = Some(9.0);
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Production,
            oneweb_enabled: false,
            ephemeris_aging: EphemerisAgingConfig::default(),
        },
        ManualClock::default(),
        FilterStub::new(0.05, process_noise),
        IntegrityStub,
        MemoryJournals::default(),
    )
    .with_doppler_pipeline(pipeline);

    let records = MeasurementReader::open(mission_dir.path())?;
    let mut samples = Vec::new();
    let mut active_second = None;
    for record in records {
        let MeasurementJournalRecord::Envelope(envelope) = record? else {
            continue;
        };
        let timestamp_ns = monotonic_ns(&envelope);
        if active_second.is_some_and(|value| value != timestamp_ns) {
            capture_sample(
                active_second.expect("checked Some"),
                &truth,
                &executive,
                &mut samples,
            );
        }
        active_second = Some(timestamp_ns);
        if matches!(envelope.payload, MeasurementPayload::Gnss(_))
            && timestamp_ns > study.aided_steady_s.saturating_mul(1_000_000_000)
        {
            continue;
        }
        executive.process(envelope);
    }
    if let Some(timestamp_ns) = active_second {
        capture_sample(timestamp_ns, &truth, &executive, &mut samples);
    }

    let loss = samples
        .iter()
        .find(|sample| (sample.elapsed_s - study.aided_steady_s as f64).abs() < f64::EPSILON)
        .ok_or(StudyError::MissingLossTruth)?;
    verify_aided_steady_state(&samples, study.aided_steady_s)?;
    let denied: Vec<_> = samples
        .iter()
        .filter(|sample| sample.elapsed_s >= study.aided_steady_s as f64)
        .collect();
    let endpoint = denied.last().ok_or(StudyError::MissingEndpoint)?;
    let position: Vec<_> = denied
        .iter()
        .map(|sample| sample.position_error_m)
        .collect();
    let velocity: Vec<_> = denied
        .iter()
        .map(|sample| sample.velocity_error_mps)
        .collect();
    let events = executive.journals().integrity_events();
    let accepted_doppler_updates = events
        .iter()
        .filter(|event| event.reason == "Doppler innovation accepted")
        .count() as u64;
    let rejected_doppler_updates = events
        .iter()
        .filter(|event| event.reason.contains("innovation chi-square gate rejected"))
        .count() as u64;
    let aged_doppler_updates = events
        .iter()
        .filter(|event| event.reason.contains("applied sigma_add"))
        .count() as u64;
    let ephemeris_age_s = duration_s as f64;

    Ok(PassageResult {
        regime: regime.name.clone(),
        speed_kn: regime.speed_kn,
        denied_distance_km: study.denied_distance_m / 1_000.0,
        denied_duration_h: denied_s as f64 / 3_600.0,
        loss_state: LossState {
            position_error_m: loss.position_error_m,
            velocity_error_mps: loss.velocity_error_mps,
            position_covariance_trace_m2: covariance_trace(&loss.state),
            covariance_dimension: loss.state.covariance_dimension,
        },
        position_error_rms_m: rms(&position),
        position_error_p95_m: percentile(&position, 0.95),
        landfall_position_error_m: endpoint.position_error_m,
        velocity_error_rms_mps: rms(&velocity),
        velocity_error_p95_mps: percentile(&velocity, 0.95),
        position_error_class: error_class(endpoint.position_error_m).into(),
        ephemeris_age_h: ephemeris_age_s / 3_600.0,
        ephemeris_age_margin_h: (study.ephemeris_ceiling_s - ephemeris_age_s) / 3_600.0,
        accepted_doppler_updates,
        rejected_doppler_updates,
        aged_doppler_updates,
        manoeuvre_convergence: measure_convergence(&samples, duration_s, MANOEUVRE_RATE_RAD_S),
    })
}

fn load_truth(path: &Path) -> Result<BTreeMap<u64, GnssFix>, StudyError> {
    let mut truth = BTreeMap::new();
    for record in TruthReader::open(path)? {
        let TruthJournalRecord::Envelope(envelope) = record? else {
            continue;
        };
        if let MeasurementPayload::Gnss(fix) = envelope.payload {
            truth.insert(monotonic_ns(&envelope), fix);
        }
    }
    Ok(truth)
}

fn capture_sample(
    timestamp_ns: u64,
    truth: &BTreeMap<u64, GnssFix>,
    executive: &Executive<ManualClock, FilterStub, IntegrityStub, MemoryJournals>,
    samples: &mut Vec<Sample>,
) {
    let Some(truth) = truth.get(&timestamp_ns).copied() else {
        return;
    };
    let state = executive.filter().state();
    samples.push(Sample {
        elapsed_s: timestamp_ns as f64 / 1.0e9,
        position_error_m: norm_difference(state.position_ecef_m, truth.position_ecef_m),
        velocity_error_mps: norm_difference(
            state.velocity_ecef_mps,
            ned_to_ecef(truth.position_ecef_m, truth.velocity_ned_mps),
        ),
        truth,
        state,
    });
}

fn monotonic_ns(envelope: &MeasurementEnvelope) -> u64 {
    match envelope.sample_time {
        TimeTag::HostMonotonicNanoseconds(value) | TimeTag::DeviceNanoseconds(value) => value,
    }
}

fn ned_to_ecef(position: [f64; 3], ned: [f64; 3]) -> [f64; 3] {
    let enu = [ned[1], ned[0], -ned[2]];
    let rotation = pnt_types::ecef_to_enu_rotation(position);
    std::array::from_fn(|column| (0..3).map(|row| rotation[row][column] * enu[row]).sum())
}

fn norm_difference(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn covariance_trace(state: &FilterState) -> f64 {
    (0..3)
        .map(|axis| state.covariance[axis * state.covariance_dimension + axis])
        .sum()
}

fn rms(values: &[f64]) -> f64 {
    (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
}

fn percentile(values: &[f64], fraction: f64) -> f64 {
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    ordered[((ordered.len() - 1) as f64 * fraction).round() as usize]
}

fn verify_aided_steady_state(samples: &[Sample], loss_s: u64) -> Result<(), StudyError> {
    let window_start = loss_s.saturating_sub(30) as f64;
    let traces = samples
        .iter()
        .filter(|sample| sample.elapsed_s >= window_start && sample.elapsed_s <= loss_s as f64)
        .map(|sample| covariance_trace(&sample.state))
        .collect::<Vec<_>>();
    let Some(&first) = traces.first() else {
        return Err(StudyError::AidedCovarianceNotSteady);
    };
    let min = traces.iter().copied().fold(f64::INFINITY, f64::min);
    let max = traces.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let relative_delta = (max - min) / first.abs().max(f64::EPSILON);
    if traces.len() < 2 || relative_delta > STEADY_COVARIANCE_RELATIVE_DELTA {
        return Err(StudyError::AidedCovarianceNotSteady);
    }
    Ok(())
}

fn error_class(error_m: f64) -> &'static str {
    if error_m < 25.0 {
        "<25 m"
    } else if error_m < 100.0 {
        "25-100 m"
    } else if error_m < 500.0 {
        "100-500 m"
    } else if error_m < 1_000.0 {
        "500 m-1 km"
    } else if error_m < 10_000.0 {
        "1-10 km"
    } else if error_m < 100_000.0 {
        "10-100 km"
    } else if error_m < 1_000_000.0 {
        "100-1000 km"
    } else if error_m < EARTH_RADIUS_M {
        "1000 km-Earth radius"
    } else {
        "DIVERGED (>=Earth radius)"
    }
}

fn measure_convergence(samples: &[Sample], duration_s: u64, turn_rate_rad_s: f64) -> Convergence {
    if samples
        .iter()
        .any(|sample| sample.position_error_m >= EARTH_RADIUS_M)
    {
        return Convergence {
            time_s: None,
            truth_distance_m: None,
            criterion: "not reported: filter diverged beyond Earth radius".into(),
        };
    }
    let turn_start = 0.45 * duration_s as f64;
    let turn_end = turn_start + std::f64::consts::FRAC_PI_2 / turn_rate_rad_s.abs();
    let baseline = samples
        .iter()
        .find(|sample| sample.elapsed_s >= (turn_start - 1.0).max(0.0))
        .map(|sample| (sample.position_error_m, sample.velocity_error_mps));
    let Some(baseline) = baseline else {
        return Convergence {
            time_s: None,
            truth_distance_m: None,
            criterion: "not reported: no pre-turn filter sample".into(),
        };
    };
    if baseline.0 > CONVERGED_POSITION_ERROR_M || baseline.1 > CONVERGED_VELOCITY_ERROR_MPS {
        return Convergence {
            time_s: None,
            truth_distance_m: None,
            criterion: format!(
                "not reported: filter was not converged before turn ({:.1} m / {:.3} m/s)",
                baseline.0, baseline.1
            ),
        };
    }
    let thresholds = ((baseline.0 * 1.2).max(25.0), (baseline.1 * 1.2).max(0.5));
    let post: Vec<_> = samples
        .iter()
        .filter(|sample| sample.elapsed_s >= turn_end)
        .collect();
    let converged = post.windows(5).find(|window| {
        window.iter().all(|sample| {
            sample.position_error_m <= thresholds.0 && sample.velocity_error_mps <= thresholds.1
        })
    });
    let Some(window) = converged else {
        return Convergence {
            time_s: None,
            truth_distance_m: None,
            criterion: format!(
                "not recovered for 5 s below pre-turn thresholds {:.1} m / {:.3} m/s",
                thresholds.0, thresholds.1
            ),
        };
    };
    let target = window[0];
    let distance = post
        .windows(2)
        .take_while(|pair| pair[1].elapsed_s <= target.elapsed_s)
        .map(|pair| norm_difference(pair[0].truth.position_ecef_m, pair[1].truth.position_ecef_m))
        .fold(0.0, |total, step| total + step);
    Convergence {
        time_s: Some(target.elapsed_s - turn_end),
        truth_distance_m: Some(distance),
        criterion: format!(
            "first 5 s below pre-turn thresholds {:.1} m / {:.3} m/s",
            thresholds.0, thresholds.1
        ),
    }
}

fn markdown(report: &Report) -> String {
    let mut text = format!(
        "# High-speed good-fix-loss study\n\n**{}**\n\n{}\n\n{}\n\n| tier | denied time | loss error / covariance trace | landfall error class | velocity RMS | ephemeris age / margin | gate accepted / rejected (aged accepted) | reconvergence time / truth distance |\n|---|---:|---:|---:|---:|---:|---:|---:|\n",
        report.caveat, report.scenario, report.d50_consistency
    );
    for result in &report.results {
        let convergence = match (
            result.manoeuvre_convergence.time_s,
            result.manoeuvre_convergence.truth_distance_m,
        ) {
            (Some(time), Some(distance)) => format!("{time:.0} s / {distance:.0} m"),
            _ => result.manoeuvre_convergence.criterion.clone(),
        };
        let _ = writeln!(
            text,
            "| {} ({:.0} kn) | {:.2} h | {:.2} m / {:.2} m² | {:.2} m ({}) | {:.3} m/s | {:.2} h / {:+.2} h | {} / {} ({} aged) | {} |",
            result.regime,
            result.speed_kn,
            result.denied_duration_h,
            result.loss_state.position_error_m,
            result.loss_state.position_covariance_trace_m2,
            result.landfall_position_error_m,
            result.position_error_class,
            result.velocity_error_rms_mps,
            result.ephemeris_age_h,
            result.ephemeris_age_margin_h,
            result.accepted_doppler_updates,
            result.rejected_doppler_updates,
            result.aged_doppler_updates,
            convergence
        );
    }
    text.push_str("\n## Model and interpretation\n\n");
    text.push_str(&report.wave_model);
    text.push_str("\n\n");
    for note in &report.integration_notes {
        let _ = writeln!(text, "- {note}");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_classes_remain_honest_at_large_scales() {
        assert_eq!(error_class(750.0), "500 m-1 km");
        assert_eq!(error_class(75_000.0), "10-100 km");
        assert_eq!(error_class(750_000.0), "100-1000 km");
        assert_eq!(error_class(2_000_000.0), "1000 km-Earth radius");
        assert_eq!(error_class(7_000_000.0), "DIVERGED (>=Earth radius)");
    }

    #[test]
    fn divergence_has_no_reconvergence_metric() {
        let samples = vec![Sample {
            elapsed_s: 100.0,
            truth: GnssFix {
                position_ecef_m: [0.0; 3],
                velocity_ned_mps: [0.0; 3],
            },
            state: FilterState::default(),
            position_error_m: 7_000_000.0,
            velocity_error_mps: 1.0,
        }];
        let convergence = measure_convergence(&samples, 100, 1.0);
        assert_eq!(convergence.time_s, None);
        assert_eq!(convergence.truth_distance_m, None);
        assert!(convergence.criterion.contains("diverged"));
    }

    #[test]
    fn real_pipeline_is_deterministic_and_holds_distance_constant() {
        let config = HighSpeedConfig {
            denied_distance_m: 20.0,
            ..HighSpeedConfig::default()
        };
        let first = config
            .regimes
            .iter()
            .map(|regime| simulate(regime, &config).unwrap())
            .collect::<Vec<_>>();
        let second = config
            .regimes
            .iter()
            .map(|regime| simulate(regime, &config).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(first, second);
        assert!(first
            .iter()
            .all(|result| (result.denied_distance_km - 0.02).abs() < f64::EPSILON));
        assert!(first
            .iter()
            .all(|result| result.loss_state.covariance_dimension >= 9));
    }

    #[test]
    fn process_noise_is_config_driven() {
        let base = ProcessNoise::default();
        let scaled = HighSpeedConfig::default().regimes[1]
            .process_noise_scale
            .apply(base);
        assert!(
            (scaled.acceleration_variance - base.acceleration_variance * 6.0).abs() < f64::EPSILON
        );
    }
}
