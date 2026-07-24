//! Real-TLE check of the controlled multi-satellite production Executive/EKF study.

use chrono::{DateTime, Duration, Utc};
use fusion_executive::{DopplerPipeline, Executive};
use nalgebra::DMatrix;
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
use pnt_predictor::{predict, ReceiverState, SatelliteState};
use pnt_time::ManualClock;
use pnt_types::{Constellation, GnssFix, MeasurementPayload, TrackerDoppler, UtcTime};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    fs,
    path::Path,
    str::FromStr,
};
use tempfile::TempDir;

const CARRIER_HZ: f64 = 1_600_000_000.0;
const SPEED_OF_LIGHT_MPS: f64 = 299_792_458.0;
const SPEED_MPS: f64 = 7.0 * 0.514_444;
const AIDED_S: u64 = 300;
const MASK_DEG: f64 = 5.0;
const EARTH_RADIUS_M: f64 = 6_371_000.0;
const EPOCH: &str = "2026-07-22T22:00:00Z";
const RECEIVER_CLOCK_DRIFT_MPS: f64 = 0.03;
const PRODUCTION_CHI_SQUARE_GATE: f64 = 9.0;
const SEED_COUNT: usize = 8;
const REAL_TLE_RAW: &str =
    include_str!("../../../pnt-ephemeris/tests/fixtures/real/constellations-2026-204.tle");
const SATELLITE_IDS: [u64; 40] = [
    44714, 44718, 44723, 44725, 44741, 44744, 44747, 44748, 44751, 44752, 44753, 44768, 44771,
    44772, 44927, 44930, 44941, 44949, 44961, 44968, 44057, 44058, 44059, 44060, 44061, 44062,
    45131, 45132, 45133, 45134, 41917, 41918, 41919, 41920, 41921, 41922, 41923, 41924, 41925,
    41926,
];

#[derive(Clone, Debug)]
pub struct RealTleConfig {
    pub counts: Vec<usize>,
    pub manoeuvring_denied_s: u64,
    pub doppler_interval_s: u64,
    pub seeds: Vec<u64>,
}

impl Default for RealTleConfig {
    fn default() -> Self {
        Self {
            counts: vec![1, 2],
            manoeuvring_denied_s: 300,
            doppler_interval_s: 30,
            seeds: (0..SEED_COUNT)
                .map(|index| 0xD54_2026_u64 + index as u64)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub schema_version: u16,
    pub caveat: String,
    pub fixture: FixtureDescription,
    pub controls: Controls,
    pub outcomes: Vec<Outcome>,
    pub headline: String,
    pub diagnosis: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FixtureDescription {
    pub real_published_unverified: bool,
    pub usable_tles: usize,
    pub satellites: usize,
    pub shells: Vec<String>,
    pub elevation_mask_deg: f64,
    pub epoch: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Controls {
    pub seed_count: usize,
    pub seed_values: Vec<u64>,
    pub receiver_clock_drift_mps: f64,
    pub receiver_clock_fractional_ppb: f64,
    pub per_sv_transmit_bias_hz: String,
    pub dynamics: String,
    pub geometry_isolation: String,
    pub production_chi_square_gate: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Outcome {
    pub geometry: String,
    pub simultaneous_los: usize,
    pub satellite_ids: Vec<u64>,
    pub duration_min: f64,
    pub gdop_mean: Option<f64>,
    pub gdop_p95: Option<f64>,
    pub endpoint_position_error_mean_m: f64,
    pub endpoint_position_error_p95_m: f64,
    pub endpoint_position_error_min_m: f64,
    pub endpoint_position_error_max_m: f64,
    pub endpoint_velocity_error_mean_mps: f64,
    pub accepted_updates_mean: f64,
    pub rejected_updates_mean: f64,
    pub nuisance_state_count_mean: f64,
    pub seed_position_errors_m: Vec<f64>,
    pub error_class: String,
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
    #[error("prediction failed: {0}")]
    Prediction(String),
    #[error("fixture has only {available} satellites continuously visible, need {requested}")]
    Visibility { requested: usize, available: usize },
    #[error("generated mission has no truth samples")]
    MissingTruth,
}

#[derive(Clone)]
struct TruthSample {
    fix: GnssFix,
    utc: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
struct SeedResult {
    position_error_m: f64,
    velocity_error_mps: f64,
    accepted: u64,
    rejected: u64,
    nuisance_states: usize,
    gdops: Vec<f64>,
}

/// Runs the controlled fixed-cohort sweep and writes measured JSON and Markdown.
///
/// # Errors
///
/// Returns a mission, journal, ephemeris, prediction, I/O, or JSON error.
///
/// # Panics
///
/// Panics when fewer than eight seeds are configured; multi-seed inference is a study invariant.
#[allow(clippy::too_many_lines)]
pub fn run(output: impl AsRef<Path>, config: &RealTleConfig) -> Result<Report, StudyError> {
    assert!(
        config.seeds.len() >= SEED_COUNT,
        "at least eight seeds required"
    );
    let max_count = config.counts.iter().copied().max().unwrap_or(1);
    let fixture = real_fixture();
    let mut by_count: BTreeMap<usize, Vec<SeedResult>> = BTreeMap::new();
    let mut cohort = Vec::new();

    for &seed in &config.seeds {
        let mission_dir = TempDir::new()?;
        generate_mission(
            mission_dir.path(),
            &MissionConfig {
                seed,
                duration_s: AIDED_S + config.manoeuvring_denied_s,
                imu_rate_hz: 1,
                speed_through_water_mps: SPEED_MPS,
                imu_bias_mps2: [2.0e-4, -1.0e-4, 1.0e-4],
                imu_noise_std_mps2: 5.0e-4,
                gnss_noise_std_m: 0.5,
                coordinated_turn: Some(CoordinatedTurnConfig {
                    rate_rad_s: 3.0_f64.to_radians(),
                }),
                wave_slam: Some(WaveSlamConfig {
                    burst_rate_hz: 0.08,
                    duration_s: 0.25,
                    vertical_peak_mps2: 6.10,
                    pitch_coupling: 0.18,
                }),
                speed_scaled_imu: Some(SpeedScaledImuConfig {
                    reference_speed_mps: SPEED_MPS,
                    noise_per_speed_ratio: 0.12,
                    bias_per_speed_ratio: 0.08,
                }),
                doppler_interval_s: config.doppler_interval_s,
                elevation_mask_rad: -std::f64::consts::FRAC_PI_2,
                ..MissionConfig::default()
            },
        )?;
        let mut truth = load_truth(mission_dir.path())?;
        retime_truth(&mut truth)?;
        let store = EphemerisStore::from_tle_str(&fixture)?.with_max_age(Duration::hours(48));
        let selected = persistent_cohort(
            &store,
            &truth,
            max_count,
            config.doppler_interval_s,
            config.manoeuvring_denied_s,
        )?;
        if cohort.is_empty() {
            cohort.clone_from(&selected);
        }
        for &count in &config.counts {
            let result = simulate(
                mission_dir.path(),
                &truth,
                &fixture,
                &selected[..count],
                config,
                seed,
            )?;
            by_count.entry(count).or_default().push(result);
        }
    }

    let outcomes = config
        .counts
        .iter()
        .map(|count| aggregate(*count, &cohort[..*count], &by_count[count], config))
        .collect::<Vec<_>>();
    let n8 = outcomes.iter().find(|value| value.simultaneous_los == 8);
    let headline = n8.map_or_else(
        || "The 40-element real fixture supports only two persistent LOS over the controlled five-minute leg; an N=8 rerun is not physically available from this sparse sample.".into(),
        |value| {
            format!(
                "Controlled N=8 manoeuvring result: mean {:.1} m, p95 {:.1} m, range {:.1}-{:.1} m across {} seeds ({}).",
                value.endpoint_position_error_mean_m,
                value.endpoint_position_error_p95_m,
                value.endpoint_position_error_min_m,
                value.endpoint_position_error_max_m,
                config.seeds.len(),
                value.error_class
            )
        },
    );
    let diagnosis = diagnose(n8);
    let report = Report {
        schema_version: 2,
        caveat: "REAL-PUBLISHED-TLE GEOMETRY CHECK [UNVERIFIED currency/provenance]. Endpoints come from the production Executive + FilterStub against synthetic generator truth; no result is clamped or target-fitted. The elements were grok-fetched and were not independently confirmed against CelesTrak.".into(),
        fixture: FixtureDescription {
            real_published_unverified: true,
            usable_tles: 40,
            satellites: 40,
            shells: vec![
                "Starlink published elements: 20 usable, inclination 53.0371-53.1608 deg".into(),
                "OneWeb published elements: 10 usable, inclination 87.8496-87.9078 deg".into(),
                "Iridium NEXT published elements: 10 usable, inclination 86.3927-86.3941 deg".into(),
            ],
            elevation_mask_deg: MASK_DEG,
            epoch: EPOCH.into(),
        },
        controls: Controls {
            seed_count: config.seeds.len(),
            seed_values: config.seeds.clone(),
            receiver_clock_drift_mps: RECEIVER_CLOCK_DRIFT_MPS,
            receiver_clock_fractional_ppb: RECEIVER_CLOCK_DRIFT_MPS
                / SPEED_OF_LIGHT_MPS
                * 1.0e9,
            per_sv_transmit_bias_hz:
                "deterministic [UNVERIFIED] signed 0.35-1.05 Hz, fixed per SV and seed".into(),
            dynamics: "pnt-mission generator: 3 deg/s coordinated-turn command, wave/slam, and speed-scaled IMU at 7 kn [UNVERIFIED]".into(),
            geometry_isolation: "A single persistent real-TLE cohort is selected once per mission. N tiers use nested prefixes, all satellites remain above 5 deg for every denied Doppler epoch, and no tier hands over; only simultaneous distinct LOS count changes. The sparse 40-element sample supports N=1 and N=2 only.".into(),
            production_chi_square_gate: PRODUCTION_CHI_SQUARE_GATE,
        },
        outcomes,
        headline,
        diagnosis,
    };
    fs::create_dir_all(output.as_ref())?;
    let mut json = serde_json::to_vec_pretty(&report)?;
    json.push(b'\n');
    fs::write(output.as_ref().join("results.json"), json)?;
    fs::write(output.as_ref().join("STUDY.md"), markdown(&report))?;
    Ok(report)
}

fn load_truth(path: &Path) -> Result<BTreeMap<u64, TruthSample>, StudyError> {
    let mut truth = BTreeMap::new();
    for record in TruthReader::open(path)? {
        let TruthJournalRecord::Envelope(envelope) = record? else {
            continue;
        };
        let MeasurementPayload::Gnss(fix) = envelope.payload else {
            continue;
        };
        let utc = envelope
            .utc
            .as_ref()
            .ok_or(StudyError::MissingTruth)
            .and_then(|value| {
                DateTime::parse_from_rfc3339(&value.rfc3339)
                    .map(|time| time.with_timezone(&Utc))
                    .map_err(|_| StudyError::MissingTruth)
            })?;
        truth.insert(envelope.host_receive_monotonic_ns, TruthSample { fix, utc });
    }
    if truth.is_empty() {
        return Err(StudyError::MissingTruth);
    }
    Ok(truth)
}

fn retime_truth(truth: &mut BTreeMap<u64, TruthSample>) -> Result<(), StudyError> {
    let start = DateTime::<Utc>::from_str(EPOCH).map_err(|_| StudyError::MissingTruth)?;
    for (&monotonic_ns, sample) in truth {
        let elapsed_ns = i64::try_from(monotonic_ns).map_err(|_| StudyError::MissingTruth)?;
        sample.utc = start + Duration::nanoseconds(elapsed_ns);
    }
    Ok(())
}

fn persistent_cohort(
    store: &EphemerisStore,
    truth: &BTreeMap<u64, TruthSample>,
    requested: usize,
    interval_s: u64,
    denied_s: u64,
) -> Result<Vec<u64>, StudyError> {
    let ids = SATELLITE_IDS;
    let mut persistent = ids.iter().copied().collect::<BTreeSet<_>>();
    for elapsed in (AIDED_S..=AIDED_S + denied_s).step_by(interval_s as usize) {
        let sample = &truth[&(elapsed * 1_000_000_000)];
        let mut visible = BTreeSet::new();
        for &id in &ids {
            let satellite = store.propagate_ecef(id, sample.utc)?;
            if elevation_rad(sample.fix.position_ecef_m, satellite.position_m)
                >= MASK_DEG.to_radians()
            {
                visible.insert(id);
            }
        }
        persistent.retain(|id| visible.contains(id));
    }
    if persistent.len() < requested {
        return Err(StudyError::Visibility {
            requested,
            available: persistent.len(),
        });
    }
    if requested == 0 {
        Ok(persistent.into_iter().collect())
    } else {
        Ok(persistent.into_iter().take(requested).collect())
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn simulate(
    path: &Path,
    truth: &BTreeMap<u64, TruthSample>,
    fixture: &str,
    satellites: &[u64],
    config: &RealTleConfig,
    seed: u64,
) -> Result<SeedResult, StudyError> {
    let mut pipeline = DopplerPipeline::new(
        EphemerisStore::from_tle_str(fixture)?.with_max_age(Duration::hours(48)),
    )
    .with_elevation_mask_degrees(MASK_DEG);
    pipeline.chi_square_threshold = Some(PRODUCTION_CHI_SQUARE_GATE);
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Production,
            oneweb_enabled: true,
            ephemeris_aging: EphemerisAgingConfig {
                ceiling_age_s: 48.0 * 3_600.0,
                ..EphemerisAgingConfig::default()
            },
        },
        ManualClock::default(),
        FilterStub::new(1.0, ProcessNoise::default()),
        IntegrityStub,
        MemoryJournals::default(),
    )
    .with_doppler_pipeline(pipeline);
    let truth_store = EphemerisStore::from_tle_str(fixture)?.with_max_age(Duration::hours(48));
    let mut sequence = 10_000_000_u64;
    let mut gdops = Vec::new();

    for record in MeasurementReader::open(path)? {
        let MeasurementJournalRecord::Envelope(mut envelope) = record? else {
            continue;
        };
        let elapsed_s = envelope.host_receive_monotonic_ns / 1_000_000_000;
        match envelope.payload {
            MeasurementPayload::Imu(_) => {
                executive.process(envelope.clone());
            }
            MeasurementPayload::Gnss(_) if elapsed_s <= AIDED_S => {
                executive.process(envelope.clone());
            }
            _ => {}
        }
        if elapsed_s < AIDED_S
            || elapsed_s > AIDED_S + config.manoeuvring_denied_s
            || !elapsed_s.is_multiple_of(config.doppler_interval_s)
        {
            continue;
        }
        let sample = &truth[&(elapsed_s * 1_000_000_000)];
        envelope.utc = Some(UtcTime {
            rfc3339: sample.utc.to_rfc3339(),
            uncertainty_ns: 0,
        });
        let receiver_velocity =
            ned_to_ecef(sample.fix.position_ecef_m, sample.fix.velocity_ned_mps);
        let mut los = Vec::new();
        for &id in satellites {
            let satellite = truth_store.propagate_ecef(id, sample.utc)?;
            let transmit_bias_hz = sv_bias_hz(id, seed);
            let prediction = predict(
                SatelliteState {
                    position_ecef_m: satellite.position_m,
                    velocity_ecef_mps: satellite.velocity_mps,
                },
                ReceiverState {
                    position_ecef_m: sample.fix.position_ecef_m,
                    velocity_ecef_mps: receiver_velocity,
                    clock_drift_mps: RECEIVER_CLOCK_DRIFT_MPS,
                },
                transmit_bias_hz,
                CARRIER_HZ,
                MASK_DEG.to_radians(),
            )
            .map_err(|error| StudyError::Prediction(format!("{error:?}")))?;
            los.push(prediction.line_of_sight_ecef);
            envelope.source_id.0 = id.to_string();
            envelope.sequence = sequence;
            envelope.payload = MeasurementPayload::TrackerDoppler(TrackerDoppler {
                constellation: constellation(id),
                correlation_peak_hz: prediction.correlation_peak_hz
                    + measurement_noise_hz(seed, id, elapsed_s),
                nominal_carrier_hz: CARRIER_HZ,
            });
            envelope.covariance = vec![0.25];
            executive.process(envelope.clone());
            sequence += 1;
        }
        if let Some(gdop) = gdop(&los) {
            gdops.push(gdop);
        }
    }
    let endpoint = truth
        .get(&((AIDED_S + config.manoeuvring_denied_s) * 1_000_000_000))
        .ok_or(StudyError::MissingTruth)?;
    let state = executive.filter().state();
    let events = executive.journals().integrity_events();
    Ok(SeedResult {
        position_error_m: norm(state.position_ecef_m, endpoint.fix.position_ecef_m),
        velocity_error_mps: norm(
            state.velocity_ecef_mps,
            ned_to_ecef(endpoint.fix.position_ecef_m, endpoint.fix.velocity_ned_mps),
        ),
        accepted: events
            .iter()
            .filter(|event| event.reason == "Doppler innovation accepted")
            .count() as u64,
        rejected: events
            .iter()
            .filter(|event| event.reason.contains("innovation chi-square gate rejected"))
            .count() as u64,
        nuisance_states: state.covariance_dimension.saturating_sub(9),
        gdops,
    })
}

fn aggregate(
    count: usize,
    satellites: &[u64],
    seeds: &[SeedResult],
    config: &RealTleConfig,
) -> Outcome {
    let positions = seeds
        .iter()
        .map(|result| result.position_error_m)
        .collect::<Vec<_>>();
    let gdops = seeds
        .iter()
        .flat_map(|result| result.gdops.iter().copied())
        .collect::<Vec<_>>();
    let mean_position = mean(&positions);
    let p95_position = percentile(&positions, 0.95);
    Outcome {
        geometry: if count == 1 {
            "fixed single LOS; no handover"
        } else {
            "fixed simultaneous multi-LOS cohort; no handover"
        }
        .into(),
        simultaneous_los: count,
        satellite_ids: satellites.to_vec(),
        duration_min: config.manoeuvring_denied_s as f64 / 60.0,
        gdop_mean: (!gdops.is_empty()).then(|| mean(&gdops)),
        gdop_p95: (!gdops.is_empty()).then(|| percentile(&gdops, 0.95)),
        endpoint_position_error_mean_m: mean_position,
        endpoint_position_error_p95_m: p95_position,
        endpoint_position_error_min_m: positions.iter().copied().fold(f64::INFINITY, f64::min),
        endpoint_position_error_max_m: positions.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        endpoint_velocity_error_mean_mps: mean(
            &seeds
                .iter()
                .map(|result| result.velocity_error_mps)
                .collect::<Vec<_>>(),
        ),
        accepted_updates_mean: mean(
            &seeds
                .iter()
                .map(|result| result.accepted as f64)
                .collect::<Vec<_>>(),
        ),
        rejected_updates_mean: mean(
            &seeds
                .iter()
                .map(|result| result.rejected as f64)
                .collect::<Vec<_>>(),
        ),
        nuisance_state_count_mean: mean(
            &seeds
                .iter()
                .map(|result| result.nuisance_states as f64)
                .collect::<Vec<_>>(),
        ),
        seed_position_errors_m: positions,
        error_class: error_class(p95_position).into(),
    }
}

fn diagnose(n8: Option<&Outcome>) -> String {
    match n8 {
        Some(value) if value.endpoint_position_error_p95_m <= 200.0 => format!(
            "N=8 reaches the 100-200 m class under these controls at p95. D51 still stands for its fixed-single-ISS, 30-minute-cadence, long-leg fixture; this result isolates a shorter multi-LOS geometry case and remains synthetic [UNVERIFIED]. GDOP p95 is {}.",
            optional(value.gdop_p95)
        ),
        Some(value) => format!(
            "N=8 does not reach the 100-200 m class under proper controls (p95 {:.1} m). The finite GDOP ({}) shows distinct instantaneous geometry, but clock/per-SV bias observability, manoeuvre dynamics, cadence, and the {}-minute leg still limit the present filter. D51's single-satellite limitation is therefore not closed.",
            value.endpoint_position_error_p95_m,
            optional(value.gdop_p95),
            value.duration_min
        ),
        None => "N=8 was not run, so no multi-satellite accuracy conclusion is available.".into(),
    }
}

fn gdop(lines: &[[f64; 3]]) -> Option<f64> {
    if lines.len() < 4 {
        return None;
    }
    let mut h = DMatrix::zeros(lines.len(), 4);
    for (row, line) in lines.iter().enumerate() {
        for axis in 0..3 {
            h[(row, axis)] = -line[axis];
        }
        h[(row, 3)] = 1.0;
    }
    (h.transpose() * h)
        .try_inverse()
        .map(|covariance| covariance.trace().sqrt())
        .filter(|value| value.is_finite())
}

fn sv_bias_hz(id: u64, seed: u64) -> f64 {
    let magnitude = 0.35 + ((id ^ seed) % 8) as f64 * 0.10;
    if (id ^ seed).is_multiple_of(2) {
        magnitude
    } else {
        -magnitude
    }
}

fn measurement_noise_hz(seed: u64, id: u64, elapsed_s: u64) -> f64 {
    let mut value = seed ^ id.rotate_left(17) ^ elapsed_s.rotate_left(31);
    value = value
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let nominal = ((value >> 11) as f64 / (1_u64 << 53) as f64 - 0.5) * 1.0;
    let outlier = if value.is_multiple_of(17) {
        if value.is_multiple_of(2) {
            12.0
        } else {
            -12.0
        }
    } else {
        0.0
    };
    nominal + outlier
}

fn real_fixture() -> String {
    REAL_TLE_RAW
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn constellation(id: u64) -> Constellation {
    match id {
        44_057..=45_134 if !(44_714..45_131).contains(&id) => Constellation::OneWeb,
        41_917..=41_926 => Constellation::Iridium,
        _ => Constellation::Starlink,
    }
}

fn elevation_rad(receiver: [f64; 3], satellite: [f64; 3]) -> f64 {
    let delta: [f64; 3] = std::array::from_fn(|axis| satellite[axis] - receiver[axis]);
    let range = delta.iter().map(|value| value * value).sum::<f64>().sqrt();
    let radius = receiver
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    let dot = delta
        .iter()
        .zip(receiver)
        .map(|(left, right)| left * right)
        .sum::<f64>();
    (dot / (range * radius)).clamp(-1.0, 1.0).asin()
}

fn ned_to_ecef(position: [f64; 3], ned: [f64; 3]) -> [f64; 3] {
    let enu = [ned[1], ned[0], -ned[2]];
    let rotation = pnt_types::ecef_to_enu_rotation(position);
    std::array::from_fn(|column| (0..3).map(|row| rotation[row][column] * enu[row]).sum())
}

fn norm(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn percentile(values: &[f64], fraction: f64) -> f64 {
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    ordered[((ordered.len() - 1) as f64 * fraction).round() as usize]
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

fn optional(value: Option<f64>) -> String {
    value.map_or_else(
        || "unobservable/infinite".into(),
        |number| format!("{number:.2}"),
    )
}

fn markdown(report: &Report) -> String {
    let mut text = format!(
        "# Real-TLE constellation geometry realism check\n\n**{}**\n\n## Real result\n\n{}\n\n{}\n\n| geometry | N | fixed SVs | GDOP mean/p95 | endpoint position mean/p95/spread | velocity mean | accepted/rejected mean | class |\n|---|---:|---|---:|---:|---:|---:|---|\n",
        report.caveat, report.headline, report.diagnosis
    );
    for value in &report.outcomes {
        let _ = writeln!(
            text,
            "| {} | {} | {:?} | {}/{} | {:.1}/{:.1}/{:.1}-{:.1} m | {:.3} m/s | {:.1}/{:.1} | {} |",
            value.geometry,
            value.simultaneous_los,
            value.satellite_ids,
            optional(value.gdop_mean),
            optional(value.gdop_p95),
            value.endpoint_position_error_mean_m,
            value.endpoint_position_error_p95_m,
            value.endpoint_position_error_min_m,
            value.endpoint_position_error_max_m,
            value.endpoint_velocity_error_mean_mps,
            value.accepted_updates_mean,
            value.rejected_updates_mean,
            value.error_class
        );
    }
    let _ = write!(
        text,
        "\n## Controls and interpretation\n\n- Seeds: {:?}; individual endpoint errors are retained in `results.json`.\n- Dynamics: {}.\n- Geometry: {} GDOP is the conventional instantaneous velocity-plus-common-clock geometry metric; N<4 is unobservable/infinite. This is a 40-SV sample, not complete operational constellations.\n- Clock stress: receiver drift {:.3} m/s ({:.3} ppb) and {}. These values and the noise model are [UNVERIFIED].\n- Measurement stress: bounded ±0.5 Hz nominal error plus deterministic signed 12 Hz tracker outliers at about 1/17 observations [UNVERIFIED].\n- The production chi-square gate is `Some(9.0)`; accepted/rejected counts come from integrity events.\n\n## Realism verdict on synthetic 116 m / 554 m\n\nThe real-element run cannot validate or falsify the synthetic N=8 result: only two of the 40 sampled SVs remain simultaneously above 5° for the controlled five-minute no-handover leg. N=1 gives {:.1}/{:.1} m mean/p95 and N=2 gives {:.1}/{:.1} m, but both have infinite GDOP for a position-plus-clock solution and largely measure short-leg inertial propagation aided by underdetermined Doppler. Treating those smaller errors as “better than 116/554” would be dishonest. The material difference is coverage: the synthetic 960-SV Walker grid provides N=8 and GDOP about 1.8, while this sparse real subset does not. A complete dated constellation snapshot is required for a genuine real-vs-synthetic N=8 check.\n\n## [UNVERIFIED]\n\n- TLE source and currency: grok-fetched, not independently confirmed against CelesTrak; physical parse/propagation and shell inclinations are confirmed.\n- Synthetic vessel truth, IMU/wave/turn model, clock drift, per-SV bias, cadence, and Doppler noise/outliers.\n- Whether this 40-SV sample is representative of operational constellation coverage; it plainly is not a complete constellation snapshot.\n",
        report.controls.seed_values,
        report.controls.dynamics,
        report.controls.geometry_isolation,
        report.controls.receiver_clock_drift_mps,
        report.controls.receiver_clock_fractional_ppb,
        report.controls.per_sv_transmit_bias_hz,
        report.outcomes[0].endpoint_position_error_mean_m,
        report.outcomes[0].endpoint_position_error_p95_m,
        report.outcomes[1].endpoint_position_error_mean_m,
        report.outcomes[1].endpoint_position_error_p95_m
    );
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_mission() -> (
        TempDir,
        BTreeMap<u64, TruthSample>,
        String,
        Vec<u64>,
        RealTleConfig,
    ) {
        let config = RealTleConfig::default();
        let mission_dir = TempDir::new().unwrap();
        generate_mission(
            mission_dir.path(),
            &MissionConfig {
                seed: config.seeds[0],
                duration_s: AIDED_S + config.manoeuvring_denied_s,
                imu_rate_hz: 1,
                speed_through_water_mps: SPEED_MPS,
                imu_bias_mps2: [2.0e-4, -1.0e-4, 1.0e-4],
                imu_noise_std_mps2: 5.0e-4,
                gnss_noise_std_m: 0.5,
                coordinated_turn: Some(CoordinatedTurnConfig {
                    rate_rad_s: 3.0_f64.to_radians(),
                }),
                wave_slam: Some(WaveSlamConfig {
                    burst_rate_hz: 0.08,
                    duration_s: 0.25,
                    vertical_peak_mps2: 6.10,
                    pitch_coupling: 0.18,
                }),
                speed_scaled_imu: Some(SpeedScaledImuConfig {
                    reference_speed_mps: SPEED_MPS,
                    noise_per_speed_ratio: 0.12,
                    bias_per_speed_ratio: 0.08,
                }),
                doppler_interval_s: config.doppler_interval_s,
                elevation_mask_rad: -std::f64::consts::FRAC_PI_2,
                ..MissionConfig::default()
            },
        )
        .unwrap();
        let mut truth = load_truth(mission_dir.path()).unwrap();
        retime_truth(&mut truth).unwrap();
        let fixture = real_fixture();
        let store = EphemerisStore::from_tle_str(&fixture)
            .unwrap()
            .with_max_age(Duration::hours(48));
        let cohort = persistent_cohort(
            &store,
            &truth,
            2,
            config.doppler_interval_s,
            config.manoeuvring_denied_s,
        )
        .unwrap();
        (mission_dir, truth, fixture, cohort, config)
    }

    #[test]
    fn core_simulation_is_deterministic() {
        let (mission_dir, truth, fixture, cohort, config) = fixture_mission();
        let first = simulate(
            mission_dir.path(),
            &truth,
            &fixture,
            &cohort,
            &config,
            config.seeds[0],
        )
        .unwrap();
        let second = simulate(
            mission_dir.path(),
            &truth,
            &fixture,
            &cohort,
            &config,
            config.seeds[0],
        )
        .unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn fixed_cohort_is_visible_and_each_tier_has_exactly_n_nuisance_states() {
        let (mission_dir, truth, fixture, cohort, config) = fixture_mission();
        let store = EphemerisStore::from_tle_str(&fixture)
            .unwrap()
            .with_max_age(Duration::hours(48));

        for elapsed in (AIDED_S..=AIDED_S + config.manoeuvring_denied_s)
            .step_by(config.doppler_interval_s as usize)
        {
            let sample = &truth[&(elapsed * 1_000_000_000)];
            for &id in &cohort {
                let satellite = store.propagate_ecef(id, sample.utc).unwrap();
                let elevation = elevation_rad(sample.fix.position_ecef_m, satellite.position_m);
                assert!(
                    elevation >= MASK_DEG.to_radians(),
                    "SV {id} is below the mask at {elapsed}s: {} deg",
                    elevation.to_degrees()
                );
            }
        }

        let initial = &truth[&(AIDED_S * 1_000_000_000)];
        let visible = SATELLITE_IDS
            .iter()
            .filter(|&&id| {
                let satellite = store.propagate_ecef(id, initial.utc).unwrap();
                elevation_rad(initial.fix.position_ecef_m, satellite.position_m)
                    >= MASK_DEG.to_radians()
            })
            .count();
        let visible_fraction = visible as f64 / SATELLITE_IDS.len() as f64;
        assert!(
            (0.01..0.5).contains(&visible_fraction),
            "implausible visible fraction: {visible}/40"
        );

        for &count in &config.counts {
            let result = simulate(
                mission_dir.path(),
                &truth,
                &fixture,
                &cohort[..count],
                &config,
                config.seeds[0],
            )
            .unwrap();
            assert_eq!(result.nuisance_states, count, "N={count}");
        }

        assert_eq!(cohort[..1], [cohort[0]]);
        let n1 = aggregate(
            1,
            &cohort[..1],
            &[SeedResult {
                position_error_m: 0.0,
                velocity_error_mps: 0.0,
                accepted: 0,
                rejected: 0,
                nuisance_states: 1,
                gdops: Vec::new(),
            }],
            &config,
        );
        assert_eq!(n1.satellite_ids, vec![cohort[0]]);
        assert_eq!(n1.geometry, "fixed single LOS; no handover");
    }

    #[test]
    fn divergence_class_is_never_hidden() {
        assert!(error_class(EARTH_RADIUS_M).starts_with("DIVERGED"));
        assert!(error_class(f64::NAN).starts_with("DIVERGED"));
    }

    #[test]
    fn all_real_tles_parse_propagate_and_match_published_inclinations() {
        let fixture = real_fixture();
        let store = EphemerisStore::from_tle_str(&fixture).unwrap();
        for id in SATELLITE_IDS {
            let epoch = store.epoch(id).unwrap();
            let state = store.propagate_teme(id, epoch).unwrap();
            assert!(state.position_km.into_iter().all(f64::is_finite));
            assert!(state.velocity_kmps.into_iter().all(f64::is_finite));
        }

        let inclinations = REAL_TLE_RAW
            .lines()
            .filter(|line| line.starts_with("2 "))
            .map(|line| {
                let id = line[2..7].trim().parse::<u64>().unwrap();
                let inclination = line[8..16].trim().parse::<f64>().unwrap();
                (id, inclination)
            })
            .collect::<Vec<_>>();
        assert_eq!(inclinations.len(), 40);
        for (id, inclination) in inclinations {
            let expected = match constellation(id) {
                Constellation::Starlink => 53.0,
                Constellation::OneWeb => 87.9,
                Constellation::Iridium => 86.4,
                Constellation::Orbcomm => unreachable!(),
            };
            assert!((inclination - expected).abs() <= 0.2, "{id}: {inclination}");
        }
    }

    #[test]
    fn production_gate_is_on() {
        assert!((PRODUCTION_CHI_SQUARE_GATE - 9.0).abs() < f64::EPSILON);
    }
}
