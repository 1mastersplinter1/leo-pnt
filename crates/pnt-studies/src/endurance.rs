//! Endurance sweeps through the production executive, chi-square gate, and EKF.

use chrono::{DateTime, Duration, Utc};
use fusion_executive::{DopplerPipeline, Executive};
use pnt_config::{Config, EphemerisAgingConfig, GnssAuthority};
use pnt_ephemeris::EphemerisStore;
use pnt_estimator::{Estimator, FilterStub, ProcessNoise};
use pnt_integrity::IntegrityStub;
use pnt_journal::{
    MeasurementJournalRecord, MeasurementReader, MemoryJournals, TruthJournalRecord, TruthReader,
};
use pnt_mission::{generate_mission, MissionConfig, SpeedScaledImuConfig, WaveSlamConfig};
use pnt_predictor::{predict, ReceiverState, SatelliteState};
use pnt_time::ManualClock;
use pnt_types::{Constellation, GnssFix, MeasurementPayload, TrackerDoppler};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    fs,
    path::Path,
};
use tempfile::TempDir;

const CARRIER_HZ: f64 = 1_600_000_000.0;
const SPEED_OF_LIGHT_MPS: f64 = 299_792_458.0;
const SPEED_MPS: f64 = 7.0 * 0.514_444;
const AIDED_S: u64 = 300;
const MASK_DEG: f64 = 5.0;
const EARTH_RADIUS_M: f64 = 6_371_000.0;
const PRODUCTION_CHI_SQUARE_THRESHOLD: f64 = 9.0;
const SATELLITE_COUNT: usize = 8;
const MINIMUM_SEEDS: usize = 8;

#[derive(Clone, Debug)]
pub struct EnduranceConfig {
    pub leg_durations_min: Vec<u64>,
    pub clock_fractional_stabilities: Vec<f64>,
    pub clock_leg_duration_min: u64,
    pub doppler_interval_s: u64,
    pub seeds: Vec<u64>,
}

impl Default for EnduranceConfig {
    fn default() -> Self {
        Self {
            leg_durations_min: vec![10, 20, 30, 45, 60],
            clock_fractional_stabilities: vec![1.0e-11, 1.0e-9, 1.0e-7],
            clock_leg_duration_min: 30,
            doppler_interval_s: 30,
            seeds: (0..MINIMUM_SEEDS)
                .map(|index| 0xE11D_2026_u64 + index as u64)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub schema_version: u16,
    pub caveat: String,
    pub controls: Controls,
    pub leg_duration_curve: Vec<Outcome>,
    pub clock_discipline_curve: Vec<Outcome>,
    pub conclusions: Vec<String>,
    pub unverified: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Controls {
    pub seed_count: usize,
    pub seed_values: Vec<u64>,
    pub simultaneous_los: usize,
    pub doppler_interval_s: u64,
    pub chi_square_threshold: f64,
    pub gate_enabled: bool,
    pub geometry: String,
    pub dynamics: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Outcome {
    pub lever: String,
    pub leg_duration_min: u64,
    pub clock_fractional_stability: f64,
    pub clock_drift_mps: f64,
    pub horizontal_error_mean_m: f64,
    pub horizontal_error_p50_m: f64,
    pub horizontal_error_p95_m: f64,
    pub horizontal_error_min_m: f64,
    pub horizontal_error_max_m: f64,
    pub accepted_updates_mean: f64,
    pub rejected_updates_mean: f64,
    pub handovers_mean: f64,
    pub seed_horizontal_errors_m: Vec<f64>,
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
    #[error("only {available} satellites visible at {elapsed_s}s; need eight")]
    Visibility { elapsed_s: u64, available: usize },
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
    horizontal_error_m: f64,
    accepted: u64,
    rejected: u64,
    handovers: u64,
}

/// Runs both endurance sweeps and writes measured JSON and Markdown.
///
/// # Errors
///
/// Returns a mission, journal, ephemeris, prediction, I/O, or JSON error.
///
/// # Panics
///
/// Panics if fewer than eight seeds are supplied or the sweep axes are empty.
#[allow(clippy::too_many_lines)]
pub fn run(output: impl AsRef<Path>, config: &EnduranceConfig) -> Result<Report, StudyError> {
    assert!(
        config.seeds.len() >= MINIMUM_SEEDS,
        "at least eight seeds required"
    );
    assert!(
        !config.leg_durations_min.is_empty(),
        "leg sweep cannot be empty"
    );
    assert!(
        !config.clock_fractional_stabilities.is_empty(),
        "clock sweep cannot be empty"
    );
    let max_leg_min = config
        .leg_durations_min
        .iter()
        .copied()
        .chain(std::iter::once(config.clock_leg_duration_min))
        .max()
        .unwrap();
    let fixture = synthetic_fixture();
    let mut leg_results: BTreeMap<u64, Vec<SeedResult>> = BTreeMap::new();
    let mut clock_results: BTreeMap<u64, Vec<SeedResult>> = BTreeMap::new();

    for &seed in &config.seeds {
        let mission_dir = TempDir::new()?;
        generate_mission(
            mission_dir.path(),
            &mission_config(seed, max_leg_min * 60, config.doppler_interval_s),
        )?;
        let truth = load_truth(mission_dir.path())?;
        for &duration_min in &config.leg_durations_min {
            leg_results.entry(duration_min).or_default().push(simulate(
                mission_dir.path(),
                &truth,
                &fixture,
                duration_min * 60,
                1.0e-9,
                config.doppler_interval_s,
                seed,
            )?);
        }
        for (index, &fractional) in config.clock_fractional_stabilities.iter().enumerate() {
            clock_results
                .entry(index as u64)
                .or_default()
                .push(simulate(
                    mission_dir.path(),
                    &truth,
                    &fixture,
                    config.clock_leg_duration_min * 60,
                    fractional,
                    config.doppler_interval_s,
                    seed,
                )?);
        }
    }

    let leg_duration_curve = config
        .leg_durations_min
        .iter()
        .map(|duration| aggregate("leg-duration", *duration, 1.0e-9, &leg_results[duration]))
        .collect::<Vec<_>>();
    let clock_discipline_curve = config
        .clock_fractional_stabilities
        .iter()
        .enumerate()
        .map(|(index, fractional)| {
            aggregate(
                "clock-discipline",
                config.clock_leg_duration_min,
                *fractional,
                &clock_results[&(index as u64)],
            )
        })
        .collect::<Vec<_>>();
    let conclusions = conclusions(&leg_duration_curve, &clock_discipline_curve);
    let report = Report {
        schema_version: 1,
        caveat: "SYNTHETIC ENDURANCE EXPERIMENT [UNVERIFIED]. Errors are production Executive + real FilterStub EKF state versus generator truth. No result is clamped, formula-generated, or target-fitted.".into(),
        controls: Controls {
            seed_count: config.seeds.len(),
            seed_values: config.seeds.clone(),
            simultaneous_los: SATELLITE_COUNT,
            doppler_interval_s: config.doppler_interval_s,
            chi_square_threshold: PRODUCTION_CHI_SQUARE_THRESHOLD,
            gate_enabled: true,
            geometry: "At every Doppler epoch, the lowest-ID eight satellites above the 5-degree mask are used. Cohort handovers are counted; the same deterministic selection is used for every lever tier.".into(),
            dynamics: "constant commanded heading at 7 kn with wave/slam and speed-scaled IMU; no coordinated turn".into(),
        },
        leg_duration_curve,
        clock_discipline_curve,
        conclusions,
        unverified: vec![
            "Synthetic 1920-satellite three-MEO-shell TLE grid and lowest-ID visibility selection.".into(),
            "10/20/30/45/60 minute constant-heading leg choices and 30-second Doppler cadence.".into(),
            "Injected receiver clock fractional stabilities: 1e-11 (Rb label), 1e-9 (good OCXO label), and 1e-7 (poor label); constant drift is a stand-in, not an oscillator stochastic model.".into(),
            "Per-SV fixed transmit biases, deterministic measurement noise/outlier process, maritime IMU, wave/slam, and speed assumptions.".into(),
        ],
    };
    fs::create_dir_all(output.as_ref())?;
    let mut json = serde_json::to_vec_pretty(&report)?;
    json.push(b'\n');
    fs::write(output.as_ref().join("results.json"), json)?;
    fs::write(output.as_ref().join("STUDY.md"), markdown(&report))?;
    Ok(report)
}

fn mission_config(seed: u64, denied_s: u64, doppler_interval_s: u64) -> MissionConfig {
    MissionConfig {
        seed,
        duration_s: AIDED_S + denied_s,
        imu_rate_hz: 1,
        speed_through_water_mps: SPEED_MPS,
        imu_bias_mps2: [2.0e-4, -1.0e-4, 1.0e-4],
        imu_noise_std_mps2: 5.0e-4,
        gnss_noise_std_m: 0.5,
        coordinated_turn: None,
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
        doppler_interval_s,
        elevation_mask_rad: -std::f64::consts::FRAC_PI_2,
        ..MissionConfig::default()
    }
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

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn simulate(
    path: &Path,
    truth: &BTreeMap<u64, TruthSample>,
    fixture: &str,
    denied_s: u64,
    clock_fractional: f64,
    doppler_interval_s: u64,
    seed: u64,
) -> Result<SeedResult, StudyError> {
    let mut pipeline = DopplerPipeline::new(
        EphemerisStore::from_tle_str(fixture)?.with_max_age(Duration::hours(12)),
    )
    .with_elevation_mask_degrees(MASK_DEG);
    pipeline.chi_square_threshold = Some(PRODUCTION_CHI_SQUARE_THRESHOLD);
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
        FilterStub::new(1.0, ProcessNoise::default()),
        IntegrityStub,
        MemoryJournals::default(),
    )
    .with_doppler_pipeline(pipeline);
    let store = EphemerisStore::from_tle_str(fixture)?.with_max_age(Duration::hours(12));
    let mut sequence = 10_000_000_u64;
    let mut previous = BTreeSet::new();
    let mut handovers = 0_u64;

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
            || elapsed_s > AIDED_S + denied_s
            || !elapsed_s.is_multiple_of(doppler_interval_s)
        {
            continue;
        }
        let sample = &truth[&(elapsed_s * 1_000_000_000)];
        let satellites = visible_cohort(&store, sample, elapsed_s)?;
        let current = satellites.iter().copied().collect::<BTreeSet<_>>();
        if !previous.is_empty() {
            handovers += previous.difference(&current).count() as u64;
        }
        previous = current;
        let receiver_velocity =
            ned_to_ecef(sample.fix.position_ecef_m, sample.fix.velocity_ned_mps);
        for id in satellites {
            let satellite = store.propagate_ecef(id, sample.utc)?;
            let prediction = predict(
                SatelliteState {
                    position_ecef_m: satellite.position_m,
                    velocity_ecef_mps: satellite.velocity_mps,
                },
                ReceiverState {
                    position_ecef_m: sample.fix.position_ecef_m,
                    velocity_ecef_mps: receiver_velocity,
                    clock_drift_mps: clock_fractional * SPEED_OF_LIGHT_MPS,
                },
                sv_bias_hz(id, seed),
                CARRIER_HZ,
                MASK_DEG.to_radians(),
            )
            .map_err(|error| StudyError::Prediction(format!("{error:?}")))?;
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
    }
    let endpoint = truth
        .get(&((AIDED_S + denied_s) * 1_000_000_000))
        .ok_or(StudyError::MissingTruth)?;
    let state = executive.filter().state();
    let events = executive.journals().integrity_events();
    Ok(SeedResult {
        horizontal_error_m: horizontal_error(state.position_ecef_m, endpoint.fix.position_ecef_m),
        accepted: events
            .iter()
            .filter(|event| event.reason == "Doppler innovation accepted")
            .count() as u64,
        rejected: events
            .iter()
            .filter(|event| event.reason.contains("innovation chi-square gate rejected"))
            .count() as u64,
        handovers,
    })
}

fn visible_cohort(
    store: &EphemerisStore,
    sample: &TruthSample,
    elapsed_s: u64,
) -> Result<Vec<u64>, StudyError> {
    let mut visible = Vec::with_capacity(SATELLITE_COUNT);
    for id in 70_000..71_920 {
        let satellite = store.propagate_ecef(id, sample.utc)?;
        if elevation_rad(sample.fix.position_ecef_m, satellite.position_m) >= MASK_DEG.to_radians()
        {
            visible.push(id);
            if visible.len() == SATELLITE_COUNT {
                return Ok(visible);
            }
        }
    }
    Err(StudyError::Visibility {
        elapsed_s,
        available: visible.len(),
    })
}

fn aggregate(lever: &str, duration_min: u64, fractional: f64, seeds: &[SeedResult]) -> Outcome {
    let errors = seeds
        .iter()
        .map(|result| result.horizontal_error_m)
        .collect::<Vec<_>>();
    let p95 = percentile(&errors, 0.95);
    Outcome {
        lever: lever.into(),
        leg_duration_min: duration_min,
        clock_fractional_stability: fractional,
        clock_drift_mps: fractional * SPEED_OF_LIGHT_MPS,
        horizontal_error_mean_m: mean(&errors),
        horizontal_error_p50_m: percentile(&errors, 0.50),
        horizontal_error_p95_m: p95,
        horizontal_error_min_m: errors.iter().copied().fold(f64::INFINITY, f64::min),
        horizontal_error_max_m: errors.iter().copied().fold(f64::NEG_INFINITY, f64::max),
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
        handovers_mean: mean(
            &seeds
                .iter()
                .map(|result| result.handovers as f64)
                .collect::<Vec<_>>(),
        ),
        seed_horizontal_errors_m: errors,
        error_class: error_class(p95).into(),
    }
}

fn conclusions(legs: &[Outcome], clocks: &[Outcome]) -> Vec<String> {
    let first = &legs[0];
    let last = &legs[legs.len() - 1];
    let leg_delta = first.horizontal_error_p95_m - last.horizontal_error_p95_m;
    let rubidium = clocks
        .iter()
        .min_by(|a, b| {
            a.clock_fractional_stability
                .total_cmp(&b.clock_fractional_stability)
        })
        .unwrap();
    let ocxo = clocks
        .iter()
        .min_by(|a, b| {
            (a.clock_fractional_stability - 1.0e-9)
                .abs()
                .total_cmp(&(b.clock_fractional_stability - 1.0e-9).abs())
        })
        .unwrap();
    vec![
        format!(
            "D55/D57 longer-leg check: p95 changes from {:.1} m at {} min to {:.1} m at {} min (signed improvement {:.1} m).",
            first.horizontal_error_p95_m,
            first.leg_duration_min,
            last.horizontal_error_p95_m,
            last.leg_duration_min,
            leg_delta
        ),
        format!(
            "500 m robustness: {}/{} leg tiers meet p50 <=500 m; {}/{} meet p95 <=500 m (the adopted D56 worst-case threshold is 750 m).",
            legs.iter().filter(|value| value.horizontal_error_p50_m <= 500.0).count(),
            legs.len(),
            legs.iter().filter(|value| value.horizontal_error_p95_m <= 500.0).count(),
            legs.len()
        ),
        format!(
            "Rb-vs-OCXO [UNVERIFIED labels]: at {} min, p95 is {:.1} m at {:.0e} versus {:.1} m at {:.0e}; signed Rb benefit {:.1} m.",
            rubidium.leg_duration_min,
            rubidium.horizontal_error_p95_m,
            rubidium.clock_fractional_stability,
            ocxo.horizontal_error_p95_m,
            ocxo.clock_fractional_stability,
            ocxo.horizontal_error_p95_m - rubidium.horizontal_error_p95_m
        ),
    ]
}

fn markdown(report: &Report) -> String {
    let mut text = format!(
        "# Endurance study: leg duration and clock discipline\n\n**{}**\n\nCross-reference: D55 identified the confounds; D57 identified longer constant-heading legs and clock discipline as the next two levers. D56 defines 500 m typical (p50) and 750 m worst-case (p95), while the requested stricter 500 m p95 check is also shown.\n\n## Leg-duration curve\n\n| leg | clock | mean | p50 | p95 | spread | accepted/rejected mean | handovers mean | class |\n|---:|---:|---:|---:|---:|---:|---:|---:|---|\n",
        report.caveat
    );
    for value in &report.leg_duration_curve {
        let _ = writeln!(
            text,
            "| {} min | {:.0e} | {:.1} m | {:.1} m | {:.1} m | {:.1}-{:.1} m | {:.1}/{:.1} | {:.1} | {} |",
            value.leg_duration_min,
            value.clock_fractional_stability,
            value.horizontal_error_mean_m,
            value.horizontal_error_p50_m,
            value.horizontal_error_p95_m,
            value.horizontal_error_min_m,
            value.horizontal_error_max_m,
            value.accepted_updates_mean,
            value.rejected_updates_mean,
            value.handovers_mean,
            value.error_class
        );
    }
    text.push_str("\n## Clock-discipline curve\n\n| label | fractional stability | drift | mean | p50 | p95 | spread | accepted/rejected mean | class |\n|---|---:|---:|---:|---:|---:|---:|---:|---|\n");
    for value in &report.clock_discipline_curve {
        let label = if value.clock_fractional_stability <= 1.0e-11 {
            "rubidium [UNVERIFIED]"
        } else if value.clock_fractional_stability <= 1.0e-9 {
            "good OCXO [UNVERIFIED]"
        } else {
            "poor reference [UNVERIFIED]"
        };
        let _ = writeln!(
            text,
            "| {label} | {:.0e} | {:.6} m/s | {:.1} m | {:.1} m | {:.1} m | {:.1}-{:.1} m | {:.1}/{:.1} | {} |",
            value.clock_fractional_stability,
            value.clock_drift_mps,
            value.horizontal_error_mean_m,
            value.horizontal_error_p50_m,
            value.horizontal_error_p95_m,
            value.horizontal_error_min_m,
            value.horizontal_error_max_m,
            value.accepted_updates_mean,
            value.rejected_updates_mean,
            value.error_class
        );
    }
    text.push_str("\n## Honest answers\n\n");
    for conclusion in &report.conclusions {
        let _ = writeln!(text, "- {conclusion}");
    }
    let _ = write!(
        text,
        "\n## Controls\n\n- Seeds: {:?}; individual endpoint errors are retained in `results.json`.\n- Real path: production `Executive` and `FilterStub` EKF state versus truth.\n- Gate: production chi-square threshold `Some({:.1})`; rejection counts above are measured integrity events.\n- Geometry: {} This permits handovers because no fixed eight-SV cohort survives an endurance leg; duration therefore also changes accumulated handovers.\n- Dynamics: {} [UNVERIFIED].\n- No formula, error clamp, target fitting, or replacement estimator is used.\n\n## [UNVERIFIED] inputs\n\n",
        report.controls.seed_values,
        report.controls.chi_square_threshold,
        report.controls.geometry,
        report.controls.dynamics
    );
    for item in &report.unverified {
        let _ = writeln!(text, "- {item}");
    }
    text
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

fn synthetic_fixture() -> String {
    let shells = [(55.0, 2.0056), (56.0, 2.0060), (64.8, 2.1310)];
    let mut text = String::new();
    for index in 0..1_920 {
        let id = 70_000 + index;
        let shell = index / 640;
        let within_shell = index % 640;
        let plane = within_shell / 20;
        let slot = within_shell % 20;
        let (inclination, motion) = shells[shell];
        let raan = plane as f64 * 360.0 / 32.0;
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

fn horizontal_error(estimated: [f64; 3], truth: [f64; 3]) -> f64 {
    let delta: [f64; 3] = std::array::from_fn(|axis| estimated[axis] - truth[axis]);
    let rotation = pnt_types::ecef_to_enu_rotation(truth);
    let east = (0..3)
        .map(|axis| rotation[0][axis] * delta[axis])
        .sum::<f64>();
    let north = (0..3)
        .map(|axis| rotation[1][axis] * delta[axis])
        .sum::<f64>();
    east.hypot(north)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn short_fixture() -> (TempDir, BTreeMap<u64, TruthSample>, String, u64) {
        let seed = 0xE11D_2026;
        let directory = TempDir::new().unwrap();
        generate_mission(directory.path(), &mission_config(seed, 120, 30)).unwrap();
        let truth = load_truth(directory.path()).unwrap();
        (directory, truth, synthetic_fixture(), seed)
    }

    #[test]
    fn core_simulation_is_deterministic() {
        let (directory, truth, fixture, seed) = short_fixture();
        let first = simulate(directory.path(), &truth, &fixture, 120, 1.0e-9, 30, seed).unwrap();
        let second = simulate(directory.path(), &truth, &fixture, 120, 1.0e-9, 30, seed).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn production_gate_is_enabled_and_rejects_injected_outliers() {
        let store = EphemerisStore::from_tle_str(&synthetic_fixture()).unwrap();
        assert_eq!(
            DopplerPipeline::new(store).chi_square_threshold,
            Some(PRODUCTION_CHI_SQUARE_THRESHOLD)
        );
        let (directory, truth, fixture, seed) = short_fixture();
        let result = simulate(directory.path(), &truth, &fixture, 120, 1.0e-7, 30, seed).unwrap();
        assert!(
            result.rejected > 0,
            "clock-stressed observations must exercise the gate"
        );
    }

    #[test]
    fn divergence_class_is_never_hidden() {
        assert!(error_class(EARTH_RADIUS_M).starts_with("DIVERGED"));
        assert!(error_class(f64::NAN).starts_with("DIVERGED"));
    }
}
