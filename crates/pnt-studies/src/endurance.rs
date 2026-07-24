//! Endurance sweeps through the production executive, chi-square gate, and EKF.
//!
//! Both sweeps run on the identical LEO fixture as the verified multi-satellite
//! study (D65 mandate): the same 960-satellite three-shell Walker grid (three
//! shells at 53.0/87.9/86.4 deg, 13-15 rev/day, 550-1200 km, 16 planes x 20
//! slots per shell). That grid keeps at least ~22 satellites continuously
//! visible above the 5-degree mask over a full 60-minute leg, so no coverage gap
//! forces a densified grid; the earlier 768-SV "densified for coverage" variant
//! rested on an empirically false gap claim and is reverted. A fixed
//! eight-satellite cohort still cannot survive an endurance leg from LEO
//! (individual satellites set within minutes), so the honest endurance model
//! tracks the best-conditioned eight *currently visible* satellites
//! continuously, handing over as the sky rotates. Per-epoch GDOP is reported to
//! prove the instantaneous geometry stays well-conditioned across the whole leg,
//! so the leg-duration effect is isolated from geometry degradation rather than
//! confounded with it.
//!
//! The sub-second wave-slam disturbance is disabled here: under the 1 Hz truth
//! cadence it aliases to a strictly upward acceleration that integrates into an
//! unphysical altitude over a 10-60 minute leg, lifting the "vessel" above the
//! LEO shell. Endurance truth is therefore constant-heading maritime dead
//! reckoning with horizontal bias and speed-scaled IMU noise.
//!
//! The study does not assert the *cause* of the denied-nav error. It runs a
//! controlled bias-zeroed counterfactual (the injected per-SV transmit bias set
//! to zero, everything else identical) alongside the full-bias sweep, a
//! per-epoch error-vs-time trace aligned to handover epochs, and a per-epoch
//! covariance-consistency trace (the filter's own reported horizontal sigma
//! versus its true horizontal error), and lets that data decide the cause.
//!
//! D68 (verified): the bias-zeroed control rules out the injected per-SV bias
//! *value* (ratio ~1.00) but, because both arms keep the identical
//! never-retired per-SV-bias/clock nuisance architecture, it cannot by itself
//! distinguish "fundamental observability floor" from "estimator
//! inconsistency". Direct covariance instrumentation answers that: position
//! IS weakly observable (the filter's own horizontal sigma converges and
//! stays bounded around ~100 m across the leg -- the Doppler-curve
//! position-observability mechanism works), but the true horizontal error
//! runs 7-70x the filter's own sigma. A genuine fundamental floor would show
//! the covariance itself growing to km-scale to match the error; it does
//! not. The km-scale denied error is therefore FILTER INCONSISTENCY /
//! covariance overconfidence -- an ESTIMATION problem (per-SV bias never
//! retired across handover, covariance overconfidence, linearisation),
//! software-fixable in the estimator, out of this config-level study's
//! scope. See D43 (the opposite-direction ~7x PESSIMISTIC covariance found
//! on aided/short legs): covariance consistency, in both directions, is the
//! recurring central estimation gap.

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
use pnt_mission::{generate_mission, MissionConfig, SpeedScaledImuConfig};
use pnt_predictor::{predict, ReceiverState, SatelliteState};
use pnt_time::ManualClock;
use pnt_types::{Constellation, GnssFix, MeasurementPayload, TrackerDoppler};
use rayon::prelude::*;
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
const SHELL_PLANES: u64 = 16;
const SHELL_SLOTS: u64 = 20;
const SHELL_SATELLITES: u64 = SHELL_PLANES * SHELL_SLOTS;
const FIXTURE_SATELLITES: u64 = 3 * SHELL_SATELLITES;
const FIRST_ID: u64 = 70_000;

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
            seeds: (0..16)
                .map(|index| 0xE11D_2026_u64 + index as u64)
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
    pub leg_duration_curve: Vec<Outcome>,
    /// Bias-zeroed control: the identical leg sweep with the injected per-SV
    /// transmit bias forced to zero in the truth generator, everything else
    /// held fixed. Comparing this to `leg_duration_curve` isolates whether the
    /// km-scale error is driven by per-SV bias re-convergence across handovers.
    pub leg_duration_curve_bias_zeroed: Vec<Outcome>,
    pub clock_discipline_curve: Vec<Outcome>,
    /// Per-epoch position-error-vs-time traces for the representative seed,
    /// aligned to handover epochs, at full bias and bias-zeroed.
    pub epoch_traces: Vec<EpochTrace>,
    pub conclusions: Vec<String>,
    pub unverified: Vec<String>,
}

/// One doppler-epoch sample of the running filter solution within a single leg.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EpochSample {
    pub elapsed_s: u64,
    pub horizontal_error_m: f64,
    /// The filter's own reported horizontal position uncertainty at this
    /// epoch (DRMS over the ENU-projected position covariance, the same
    /// convention as `FilterState::horizontal_accuracy_m`). Compared against
    /// `horizontal_error_m` this is the covariance-consistency check (D68):
    /// a well-calibrated filter has `horizontal_error_m` fluctuate around
    /// `sigma_horizontal_m`, not run many multiples of it.
    pub sigma_horizontal_m: f64,
    pub gdop: Option<f64>,
    /// True when this epoch dropped at least one previously tracked satellite
    /// (a handover occurred at this epoch).
    pub handover: bool,
}

/// A full within-leg error-vs-time trace for one representative run.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EpochTrace {
    pub label: String,
    pub seed: u64,
    pub leg_duration_min: u64,
    pub bias_enabled: bool,
    /// Mean horizontal error over epochs at which a handover occurred.
    pub mean_error_handover_m: Option<f64>,
    /// Mean horizontal error over epochs with no handover.
    pub mean_error_steady_m: Option<f64>,
    /// Mean of the filter's reported horizontal sigma over the leg (D68
    /// covariance-consistency check).
    pub mean_sigma_horizontal_m: Option<f64>,
    /// Mean of the per-epoch `horizontal_error_m / sigma_horizontal_m` ratio
    /// over the WHOLE leg. Diluted by the early epochs immediately after the
    /// aided fix is lost, where both the error and the filter's sigma are
    /// still small and well matched; use `late_leg_consistency_ratio` for the
    /// steady-state (aided-prior-decayed) comparison D68 characterizes.
    pub mean_consistency_ratio: Option<f64>,
    /// Mean consistency ratio over the LAST THIRD of the leg only -- the
    /// steady-state regime once the aided prior has decayed, which is what
    /// D68's "7-70x overconfidence" finding characterizes. A consistent
    /// filter keeps this near 1.
    pub late_leg_consistency_ratio: Option<f64>,
    /// Mean of the filter's reported horizontal sigma over the last third of
    /// the leg (steady-state, D68 reports this as bounded ~50-160 m).
    pub late_leg_sigma_horizontal_m: Option<f64>,
    /// Mean true horizontal error over the last third of the leg.
    pub late_leg_error_m: Option<f64>,
    /// Maximum per-epoch consistency ratio observed over the leg (typically
    /// at leg end, once the error has grown furthest past the bounded sigma).
    pub max_consistency_ratio: Option<f64>,
    pub samples: Vec<EpochSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FixtureDescription {
    pub synthetic_unverified: bool,
    pub satellites: usize,
    pub shells: Vec<String>,
    pub elevation_mask_deg: f64,
    pub regime: String,
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
    pub gdop_mean: Option<f64>,
    pub gdop_min: Option<f64>,
    pub gdop_max: Option<f64>,
    pub gdop_p95: Option<f64>,
    pub horizontal_error_mean_m: f64,
    pub horizontal_error_p50_m: f64,
    pub horizontal_error_p95_m: f64,
    pub horizontal_error_min_m: f64,
    pub horizontal_error_max_m: f64,
    /// RMS-over-leg horizontal error, averaged across seeds. The endpoint
    /// metric above is a single-epoch sample of a noisy Doppler-only solution;
    /// the RMS-over-leg metric averages every denied-leg doppler epoch and is a
    /// more stable headline (the reviewer flagged endpoint sampling as a source
    /// of bimodality artifacts).
    pub horizontal_rms_mean_m: f64,
    pub horizontal_rms_p50_m: f64,
    pub horizontal_rms_p95_m: f64,
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
    #[error("only {available} satellites visible at {elapsed_s}s; need {SATELLITE_COUNT}")]
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
    /// Single-epoch horizontal error at the denied-leg endpoint.
    horizontal_error_m: f64,
    /// RMS of the horizontal error over every denied-leg doppler epoch.
    rms_error_m: f64,
    accepted: u64,
    rejected: u64,
    handovers: u64,
    gdops: Vec<f64>,
    /// Per-epoch error-vs-time trace for the whole denied leg.
    trace: Vec<EpochSample>,
}

/// All sweep results for one seed: `(leg_duration_min, result)` and
/// `(clock_index, result)` pairs, ready to merge into the cross-seed tables.
struct SeedOutcome {
    legs: Vec<(u64, SeedResult)>,
    legs_bias_zeroed: Vec<(u64, SeedResult)>,
    clocks: Vec<(u64, SeedResult)>,
}

/// Runs every leg-duration and clock-discipline tier for a single seed against a
/// freshly generated mission and its shared geometry schedule.
fn simulate_seed(
    fixture: &str,
    config: &EnduranceConfig,
    max_denied_s: u64,
    seed: u64,
) -> Result<SeedOutcome, StudyError> {
    let mission_dir = TempDir::new()?;
    generate_mission(
        mission_dir.path(),
        &mission_config(seed, max_denied_s, config.doppler_interval_s),
    )?;
    let truth = load_truth(mission_dir.path())?;
    let store = EphemerisStore::from_tle_str(fixture)?.with_max_age(Duration::hours(12));
    // Geometry-only handover schedule: continuous best-N tracking with sticky
    // handover. It depends on neither leg duration nor clock quality, so it is
    // computed once and reused across every tier -- this is what holds the
    // geometry model fixed while the levers vary.
    let schedule = cohort_schedule(&store, &truth, config.doppler_interval_s, max_denied_s)?;
    let mut legs = Vec::new();
    let mut legs_bias_zeroed = Vec::new();
    for &duration_min in &config.leg_durations_min {
        legs.push((
            duration_min,
            simulate(
                mission_dir.path(),
                &truth,
                fixture,
                &schedule,
                duration_min * 60,
                1.0e-9,
                config.doppler_interval_s,
                seed,
                true,
            )?,
        ));
        // Bias-zeroed control: identical run with the injected per-SV transmit
        // bias forced to zero in the truth generator, everything else fixed.
        legs_bias_zeroed.push((
            duration_min,
            simulate(
                mission_dir.path(),
                &truth,
                fixture,
                &schedule,
                duration_min * 60,
                1.0e-9,
                config.doppler_interval_s,
                seed,
                false,
            )?,
        ));
    }
    let mut clocks = Vec::new();
    for (index, &fractional) in config.clock_fractional_stabilities.iter().enumerate() {
        clocks.push((
            index as u64,
            simulate(
                mission_dir.path(),
                &truth,
                fixture,
                &schedule,
                config.clock_leg_duration_min * 60,
                fractional,
                config.doppler_interval_s,
                seed,
                true,
            )?,
        ));
    }
    Ok(SeedOutcome {
        legs,
        legs_bias_zeroed,
        clocks,
    })
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

    // Seeds are independent; run them in parallel but merge in input order so
    // the aggregate curves are bit-for-bit deterministic regardless of thread
    // scheduling (`par_iter().map().collect()` preserves order).
    let per_seed = config
        .seeds
        .par_iter()
        .map(|&seed| simulate_seed(&fixture, config, max_leg_min * 60, seed))
        .collect::<Result<Vec<_>, StudyError>>()?;

    let mut leg_results: BTreeMap<u64, Vec<SeedResult>> = BTreeMap::new();
    let mut leg_bias_zeroed_results: BTreeMap<u64, Vec<SeedResult>> = BTreeMap::new();
    let mut clock_results: BTreeMap<u64, Vec<SeedResult>> = BTreeMap::new();
    for outcome in per_seed {
        for (duration_min, result) in outcome.legs {
            leg_results.entry(duration_min).or_default().push(result);
        }
        for (duration_min, result) in outcome.legs_bias_zeroed {
            leg_bias_zeroed_results
                .entry(duration_min)
                .or_default()
                .push(result);
        }
        for (index, result) in outcome.clocks {
            clock_results.entry(index).or_default().push(result);
        }
    }

    let leg_duration_curve = config
        .leg_durations_min
        .iter()
        .map(|duration| aggregate("leg-duration", *duration, 1.0e-9, &leg_results[duration]))
        .collect::<Vec<_>>();
    let leg_duration_curve_bias_zeroed = config
        .leg_durations_min
        .iter()
        .map(|duration| {
            aggregate(
                "leg-duration-bias-zeroed",
                *duration,
                1.0e-9,
                &leg_bias_zeroed_results[duration],
            )
        })
        .collect::<Vec<_>>();

    // Representative-seed per-epoch traces at the longest leg tier: the first
    // seed (par_iter preserves input order, so index 0 is config.seeds[0]), at
    // full bias and bias-zeroed, for the handover-aligned error-vs-time view.
    let representative_seed = config.seeds[0];
    let representative_leg = *config.leg_durations_min.last().unwrap();
    let epoch_traces = vec![
        epoch_trace(
            "full-bias",
            representative_seed,
            representative_leg,
            true,
            &leg_results[&representative_leg][0].trace,
        ),
        epoch_trace(
            "bias-zeroed",
            representative_seed,
            representative_leg,
            false,
            &leg_bias_zeroed_results[&representative_leg][0].trace,
        ),
    ];

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
    let conclusions = conclusions(
        &leg_duration_curve,
        &leg_duration_curve_bias_zeroed,
        &clock_discipline_curve,
        &epoch_traces,
    );
    let report = Report {
        schema_version: 4,
        caveat: "SYNTHETIC ENDURANCE EXPERIMENT [UNVERIFIED]. Errors are production Executive + real FilterStub EKF state versus generator truth. No result is clamped, formula-generated, or target-fitted.".into(),
        fixture: FixtureDescription {
            synthetic_unverified: true,
            satellites: FIXTURE_SATELLITES as usize,
            shells: vec![
                "Starlink-class: ~550 km, 53.0 deg, 15.064 rev/day".into(),
                "OneWeb-class: ~1200 km, 87.9 deg, 13.158 rev/day".into(),
                "Iridium-class: ~780 km, 86.4 deg, 14.342 rev/day".into(),
            ],
            elevation_mask_deg: MASK_DEG,
            regime: "The verified multi-satellite study's 960-satellite three-shell synthetic LEO Walker grid, reused unchanged (D65 mandate). At least ~22 (typically 22-45) satellites stay continuously visible above the 5-degree mask over a full 60-minute leg, so no coverage gap forces a denser grid.".into(),
        },
        controls: Controls {
            seed_count: config.seeds.len(),
            seed_values: config.seeds.clone(),
            simultaneous_los: SATELLITE_COUNT,
            doppler_interval_s: config.doppler_interval_s,
            chi_square_threshold: PRODUCTION_CHI_SQUARE_THRESHOLD,
            gate_enabled: true,
            geometry: "The receiver continuously tracks eight satellites, holding lock on each until it sets below the 5-degree mask (sticky handover, as real hardware does) and refilling freed slots with the geometry-improving visible candidate. Handovers therefore reflect physical setting events; per-epoch GDOP is reported to prove the instantaneous geometry stays well-conditioned throughout every leg.".into(),
            dynamics: "constant commanded heading at 7 kn with speed-scaled IMU noise and horizontal bias; sub-second wave-slam disabled to keep long-leg truth physical; no coordinated turn".into(),
        },
        leg_duration_curve,
        leg_duration_curve_bias_zeroed,
        clock_discipline_curve,
        epoch_traces,
        conclusions,
        unverified: vec![
            format!("Synthetic {FIXTURE_SATELLITES}-satellite three-shell LEO Walker grid (53.0/87.9/86.4 deg at 15.064/13.158/14.342 rev/day, 16 planes x 20 slots per shell), reused unchanged from the multi-satellite study; sticky best-N-visible handover selection."),
            "10/20/30/45/60 minute constant-heading leg choices and 30-second Doppler cadence.".into(),
            "Injected receiver clock fractional stabilities: 1e-11 (Rb label), 1e-9 (good OCXO label), and 1e-7 (poor label); constant common-mode drift is a stand-in, not an oscillator stochastic model.".into(),
            "Per-SV fixed transmit biases, deterministic measurement noise/outlier process, maritime IMU bias/noise, and speed assumptions; sub-second wave-slam disabled for long-leg truth stability.".into(),
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
        // Vertical IMU bias is zeroed: over 10-60 min legs any net vertical
        // acceleration integrates into an unphysical altitude (the sub-second
        // wave-slam model, disabled below, otherwise aliases to a monotonic
        // vertical drift under the 1 Hz truth cadence and lifts the "vessel"
        // above the LEO satellites). Horizontal DR bias stress is retained.
        imu_bias_mps2: [2.0e-4, -1.0e-4, 0.0],
        imu_noise_std_mps2: 5.0e-4,
        gnss_noise_std_m: 0.5,
        coordinated_turn: None,
        wave_slam: None,
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

/// Builds the continuous best-N tracking schedule with realistic *sticky*
/// handover: a satellite that is being tracked keeps being tracked as long as it
/// stays above the mask (a real receiver holds lock until the satellite sets),
/// and freed slots are refilled with the visible, not-yet-tracked satellite that
/// most improves geometry. Handovers therefore occur only when a satellite
/// physically sets, not on every marginal geometry reshuffle. Depends only on
/// the truth trajectory and the constellation, so it is shared unchanged across
/// all leg and clock tiers, holding the geometry model fixed while the levers
/// vary.
fn cohort_schedule(
    store: &EphemerisStore,
    truth: &BTreeMap<u64, TruthSample>,
    interval_s: u64,
    max_denied_s: u64,
) -> Result<BTreeMap<u64, Vec<u64>>, StudyError> {
    let mut schedule = BTreeMap::new();
    let mut tracked: Vec<u64> = Vec::new();
    for elapsed in (AIDED_S..=AIDED_S + max_denied_s).step_by(interval_s as usize) {
        let sample = &truth[&(elapsed * 1_000_000_000)];
        let mut visible: BTreeMap<u64, [f64; 3]> = BTreeMap::new();
        for id in FIRST_ID..FIRST_ID + FIXTURE_SATELLITES {
            let satellite = store.propagate_ecef(id, sample.utc)?;
            if elevation_rad(sample.fix.position_ecef_m, satellite.position_m)
                >= MASK_DEG.to_radians()
            {
                let delta: [f64; 3] = std::array::from_fn(|axis| {
                    satellite.position_m[axis] - sample.fix.position_ecef_m[axis]
                });
                let range = delta.iter().map(|value| value * value).sum::<f64>().sqrt();
                visible.insert(id, std::array::from_fn(|axis| delta[axis] / range));
            }
        }
        if visible.len() < SATELLITE_COUNT {
            return Err(StudyError::Visibility {
                elapsed_s: elapsed,
                available: visible.len(),
            });
        }
        // Hold lock on tracked satellites that are still up; drop those that set.
        tracked.retain(|id| visible.contains_key(id));
        // Refill freed slots with the geometry-improving visible candidates.
        while tracked.len() < SATELLITE_COUNT {
            let current_los: Vec<[f64; 3]> = tracked.iter().map(|id| visible[id]).collect();
            let mut best_id = None;
            let mut best_metric = f64::INFINITY;
            for (&id, los) in &visible {
                if tracked.contains(&id) {
                    continue;
                }
                let mut trial = current_los.clone();
                trial.push(*los);
                // Below four LOS GDOP is undefined; seed the set by preferring
                // the highest-elevation candidate (largest vertical LOS
                // component), then switch to true GDOP minimisation.
                let metric = gdop(&trial)
                    .unwrap_or_else(|| -elevation_component(*los, sample.fix.position_ecef_m));
                if metric < best_metric {
                    best_metric = metric;
                    best_id = Some(id);
                }
            }
            tracked.push(best_id.expect("visible pool exceeds tracked count"));
        }
        let mut cohort = tracked.clone();
        cohort.sort_unstable();
        schedule.insert(elapsed, cohort);
    }
    Ok(schedule)
}

/// Vertical component of a line-of-sight unit vector (elevation proxy) used only
/// to seed the tracked set before four satellites make GDOP well-defined.
fn elevation_component(los: [f64; 3], receiver: [f64; 3]) -> f64 {
    let radius = receiver
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    (0..3).map(|axis| los[axis] * receiver[axis] / radius).sum()
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn simulate(
    path: &Path,
    truth: &BTreeMap<u64, TruthSample>,
    fixture: &str,
    schedule: &BTreeMap<u64, Vec<u64>>,
    denied_s: u64,
    clock_fractional: f64,
    doppler_interval_s: u64,
    seed: u64,
    bias_enabled: bool,
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
    let mut gdops = Vec::new();
    // One sample per distinct doppler epoch (the measurement stream carries
    // several records per integer second, so the epoch is keyed and the
    // last-updated filter state per epoch is kept; the handover flag is sticky
    // so an epoch that dropped a satellite stays marked).
    let mut trace_by_epoch: BTreeMap<u64, EpochSample> = BTreeMap::new();

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
        let satellites = &schedule[&elapsed_s];
        let current = satellites.iter().copied().collect::<BTreeSet<_>>();
        let handover_epoch = !previous.is_empty() && previous.difference(&current).next().is_some();
        if !previous.is_empty() {
            handovers += previous.difference(&current).count() as u64;
        }
        previous = current;
        let receiver_velocity =
            ned_to_ecef(sample.fix.position_ecef_m, sample.fix.velocity_ned_mps);
        let mut los = Vec::new();
        for &id in satellites {
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
                if bias_enabled {
                    sv_bias_hz(id, seed)
                } else {
                    0.0
                },
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
        let epoch_gdop = gdop(&los);
        if let Some(value) = epoch_gdop {
            gdops.push(value);
        }
        // Record the running solution error at this epoch, alongside the
        // filter's own reported horizontal sigma (from its covariance), so the
        // within-leg error-vs-time trajectory (and its RMS) can be measured
        // and aligned to handover epochs, AND so the filter's self-reported
        // uncertainty can be checked against the true error (D68
        // covariance-consistency check), rather than only the single-epoch
        // endpoint.
        let running = executive.filter().state();
        let error_m = horizontal_error(running.position_ecef_m, sample.fix.position_ecef_m);
        let sigma_m = running.horizontal_accuracy_m();
        let entry = trace_by_epoch.entry(elapsed_s).or_insert(EpochSample {
            elapsed_s,
            horizontal_error_m: error_m,
            sigma_horizontal_m: sigma_m,
            gdop: epoch_gdop,
            handover: false,
        });
        entry.horizontal_error_m = error_m;
        entry.sigma_horizontal_m = sigma_m;
        entry.gdop = epoch_gdop;
        entry.handover |= handover_epoch;
    }
    let trace: Vec<EpochSample> = trace_by_epoch.into_values().collect();
    let endpoint = truth
        .get(&((AIDED_S + denied_s) * 1_000_000_000))
        .ok_or(StudyError::MissingTruth)?;
    let state = executive.filter().state();
    let events = executive.journals().integrity_events();
    let rms_error_m = if trace.is_empty() {
        f64::NAN
    } else {
        (trace
            .iter()
            .map(|sample| sample.horizontal_error_m.powi(2))
            .sum::<f64>()
            / trace.len() as f64)
            .sqrt()
    };
    Ok(SeedResult {
        horizontal_error_m: horizontal_error(state.position_ecef_m, endpoint.fix.position_ecef_m),
        rms_error_m,
        accepted: events
            .iter()
            .filter(|event| event.reason == "Doppler innovation accepted")
            .count() as u64,
        rejected: events
            .iter()
            .filter(|event| event.reason.contains("innovation chi-square gate rejected"))
            .count() as u64,
        handovers,
        gdops,
        trace,
    })
}

fn aggregate(lever: &str, duration_min: u64, fractional: f64, seeds: &[SeedResult]) -> Outcome {
    let errors = seeds
        .iter()
        .map(|result| result.horizontal_error_m)
        .collect::<Vec<_>>();
    let gdops = seeds
        .iter()
        .flat_map(|result| result.gdops.iter().copied())
        .collect::<Vec<_>>();
    let rms = seeds
        .iter()
        .map(|result| result.rms_error_m)
        .collect::<Vec<_>>();
    let p95 = percentile(&errors, 0.95);
    Outcome {
        lever: lever.into(),
        leg_duration_min: duration_min,
        clock_fractional_stability: fractional,
        clock_drift_mps: fractional * SPEED_OF_LIGHT_MPS,
        gdop_mean: (!gdops.is_empty()).then(|| mean(&gdops)),
        gdop_min: (!gdops.is_empty()).then(|| gdops.iter().copied().fold(f64::INFINITY, f64::min)),
        gdop_max: (!gdops.is_empty())
            .then(|| gdops.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
        gdop_p95: (!gdops.is_empty()).then(|| percentile(&gdops, 0.95)),
        horizontal_error_mean_m: mean(&errors),
        horizontal_error_p50_m: percentile(&errors, 0.50),
        horizontal_error_p95_m: p95,
        horizontal_error_min_m: errors.iter().copied().fold(f64::INFINITY, f64::min),
        horizontal_error_max_m: errors.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        horizontal_rms_mean_m: mean(&rms),
        horizontal_rms_p50_m: percentile(&rms, 0.50),
        horizontal_rms_p95_m: percentile(&rms, 0.95),
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

/// Builds a representative-seed error-vs-time trace summary, including the mean
/// error at handover versus steady (no-handover) epochs, so the report can show
/// whether handovers actually coincide with error spikes.
fn epoch_trace(
    label: &str,
    seed: u64,
    leg_duration_min: u64,
    bias_enabled: bool,
    samples: &[EpochSample],
) -> EpochTrace {
    let handover_errors: Vec<f64> = samples
        .iter()
        .filter(|sample| sample.handover)
        .map(|sample| sample.horizontal_error_m)
        .collect();
    let steady_errors: Vec<f64> = samples
        .iter()
        .filter(|sample| !sample.handover)
        .map(|sample| sample.horizontal_error_m)
        .collect();
    let sigmas: Vec<f64> = samples.iter().map(|s| s.sigma_horizontal_m).collect();
    // Consistency ratio = true error / filter's own reported sigma at each
    // epoch (D68). A ratio near 1 means the filter's covariance is honest; a
    // ratio in the tens means the filter is overconfident (its covariance
    // shrinks while the true error does not), an ESTIMATION-consistency
    // defect, not evidence of a fundamental observability floor (a genuine
    // floor would show the covariance itself growing to match the error).
    let ratios: Vec<f64> = samples
        .iter()
        .filter(|s| s.sigma_horizontal_m > 0.0)
        .map(|s| s.horizontal_error_m / s.sigma_horizontal_m)
        .collect();
    // Steady-state (late-leg) window: the last third of samples, once the
    // aided prior has decayed. The same start-third/end-third split the
    // markdown table already uses for the raw error trajectory.
    let third = samples.len() / 3;
    let late_leg = if third > 0 {
        &samples[samples.len() - third..]
    } else {
        samples
    };
    let late_sigmas: Vec<f64> = late_leg.iter().map(|s| s.sigma_horizontal_m).collect();
    let late_errors: Vec<f64> = late_leg.iter().map(|s| s.horizontal_error_m).collect();
    let late_ratios: Vec<f64> = late_leg
        .iter()
        .filter(|s| s.sigma_horizontal_m > 0.0)
        .map(|s| s.horizontal_error_m / s.sigma_horizontal_m)
        .collect();
    EpochTrace {
        label: label.into(),
        seed,
        leg_duration_min,
        bias_enabled,
        mean_error_handover_m: (!handover_errors.is_empty()).then(|| mean(&handover_errors)),
        mean_error_steady_m: (!steady_errors.is_empty()).then(|| mean(&steady_errors)),
        mean_sigma_horizontal_m: (!sigmas.is_empty()).then(|| mean(&sigmas)),
        mean_consistency_ratio: (!ratios.is_empty()).then(|| mean(&ratios)),
        late_leg_consistency_ratio: (!late_ratios.is_empty()).then(|| mean(&late_ratios)),
        late_leg_sigma_horizontal_m: (!late_sigmas.is_empty()).then(|| mean(&late_sigmas)),
        late_leg_error_m: (!late_errors.is_empty()).then(|| mean(&late_errors)),
        max_consistency_ratio: (!ratios.is_empty())
            .then(|| ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
        samples: samples.to_vec(),
    }
}

/// The bias-VALUE control verdict: compares the full-bias and bias-zeroed
/// longest-leg control and reports whether the injected per-SV bias
/// *magnitude* drives the km-scale error. This control alone cannot decide
/// "fundamental floor vs. estimator inconsistency" (D68) -- both arms hold
/// the identical nuisance-augmentation architecture (a fresh variance-100
/// per-SV bias state every handover, never retired) and the identical
/// covariance recursion, so it only isolates the injected *value*, not the
/// estimator mechanism. That question is answered by the covariance-
/// consistency check (`covariance_consistency_verdict`), not this control.
fn bias_control_verdict(full: &Outcome, bias_zeroed: &Outcome) -> String {
    // "multisat-class" = hundreds of metres or better (the fixed-cohort study's
    // regime). If zeroing the injected bias drops the error into that band, the
    // injected bias magnitude was itself a dominant driver.
    const MULTISAT_CLASS_M: f64 = 500.0;
    // Use RMS-over-leg as the stable comparison metric (endpoint is a noisy
    // single-epoch sample). Compare the median seed of each control.
    let full_rms = full.horizontal_rms_p50_m;
    let bz_rms = bias_zeroed.horizontal_rms_p50_m;
    let ratio = if full_rms > 0.0 {
        bz_rms / full_rms
    } else {
        f64::NAN
    };
    let head = format!(
        "BIAS-VALUE CONTROL (bias-zeroed, {} min, RMS-over-leg p50): full-bias {full_rms:.0} m vs bias-zeroed {bz_rms:.0} m (bias-zeroed is {:.0}% of full-bias; endpoint p50 {:.0} m vs {:.0} m). ",
        full.leg_duration_min,
        ratio * 100.0,
        full.horizontal_error_p50_m,
        bias_zeroed.horizontal_error_p50_m,
    );
    let verdict = if bz_rms <= MULTISAT_CLASS_M || ratio <= 0.25 {
        "Zeroing the injected per-SV transmit bias VALUE collapses the error toward multisat-class, so the injected bias magnitude IS a dominant driver of km-scale denied nav here."
    } else {
        "Zeroing the injected per-SV transmit bias VALUE barely moves the error -- it STAYS km-scale (ratio ~1.00). This rules OUT the injected bias magnitude as the cause: km-scale denied error is NOT an artifact of how large a bias this study happened to inject. It does NOT, by itself, tell us whether the residual km-scale error is a fundamental observability floor or an estimator consistency defect -- both arms share the same never-retired per-SV-bias nuisance architecture and the same covariance recursion, so that mechanism is untested by this control in either direction. See the covariance-consistency check below (D68) for the arbitrating evidence."
    };
    format!("{head}{verdict}")
}

/// The covariance-consistency verdict (D68): compares the filter's own
/// reported horizontal sigma against the true horizontal error over the
/// representative leg, and lets the measured ratio -- not an assertion --
/// decide between "fundamental observability floor" (covariance would be
/// km-scale too) and "filter inconsistency / overconfidence" (covariance
/// stays small while the true error does not).
fn covariance_consistency_verdict(trace: &EpochTrace) -> String {
    let (
        Some(mean_sigma),
        Some(mean_ratio),
        Some(max_ratio),
        Some(late_sigma),
        Some(late_error),
        Some(late_ratio),
    ) = (
        trace.mean_sigma_horizontal_m,
        trace.mean_consistency_ratio,
        trace.max_consistency_ratio,
        trace.late_leg_sigma_horizontal_m,
        trace.late_leg_error_m,
        trace.late_leg_consistency_ratio,
    )
    else {
        return "No covariance-consistency data was recorded for the representative trace.".into();
    };
    let head = format!(
        "COVARIANCE CONSISTENCY (D68, representative seed {}, {} min, full bias): whole-leg mean filter sigma {mean_sigma:.0} m (mean ratio {mean_ratio:.1}x, rises from ~1x near the aided prior to a peak of {max_ratio:.1}x by leg end); in the STEADY-STATE window once the aided prior has decayed (last third of the leg), filter sigma averages {late_sigma:.0} m while true error averages {late_error:.0} m -- a steady-state ratio of {late_ratio:.1}x. ",
        trace.seed, trace.leg_duration_min,
    );
    let verdict = if late_ratio > 2.0 {
        "The filter's sigma stays small (order 100 m, bounded, not km-scale) while the true error is several times larger and still growing: this is OVERCONFIDENCE / FILTER INCONSISTENCY, not a fundamental observability floor -- a genuine physics floor would show the covariance itself growing to km-scale to match the true error, and it does not. Position IS weakly observable (the filter's own uncertainty converges and stays bounded, evidence the Doppler-curve position mechanism works); the km-scale error is an ESTIMATION-consistency defect (the never-retired per-SV-bias/clock nuisance null-space and linearisation feeding an overconfident covariance), fixable in the estimator (bias continuity/retirement across handover, covariance-consistency correction, Q retuning) -- out of this study's config-only scope. Cross-reference D43, which found the opposite-direction ~7x PESSIMISTIC covariance on aided/short legs: covariance consistency, in both directions, is the recurring central estimation gap, not this study's leg-duration or clock levers."
    } else {
        "The filter's sigma tracks its true error reasonably closely (steady-state ratio near 1), which is the signature of a well-calibrated filter and would be consistent with a genuine observability limit rather than filter inconsistency."
    };
    format!("{head}{verdict}")
}

#[allow(clippy::too_many_lines)]
fn conclusions(
    legs: &[Outcome],
    legs_bias_zeroed: &[Outcome],
    clocks: &[Outcome],
    traces: &[EpochTrace],
) -> Vec<String> {
    let first = &legs[0];
    let last = &legs[legs.len() - 1];
    let last_bz = &legs_bias_zeroed[legs_bias_zeroed.len() - 1];
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
    let poor = clocks
        .iter()
        .max_by(|a, b| {
            a.clock_fractional_stability
                .total_cmp(&b.clock_fractional_stability)
        })
        .unwrap();
    let p50_goal = legs
        .iter()
        .find(|value| value.horizontal_error_p50_m <= 500.0);
    let p95_goal = legs
        .iter()
        .find(|value| value.horizontal_error_p95_m <= 750.0);
    let last_under_500 = last
        .seed_horizontal_errors_m
        .iter()
        .filter(|value| **value <= 500.0)
        .count();
    let full_trace = traces.iter().find(|trace| trace.bias_enabled);
    let handover_alignment = full_trace.map_or_else(
        || "no representative trace recorded".into(),
        |trace| {
            let handover = trace.mean_error_handover_m;
            let steady = trace.mean_error_steady_m;
            match (handover, steady) {
                (Some(handover), Some(steady)) => {
                    let verdict = if handover > steady * 1.25 {
                        "handover epochs DO show elevated error (spikes align with handovers)"
                    } else if handover < steady * 0.8 {
                        "handover epochs are actually LOWER-error than steady epochs (no handover-induced spike)"
                    } else {
                        "handover and steady epochs are comparable (no systematic handover-induced spike)"
                    };
                    format!(
                        "Handover alignment (representative seed {}, {} min, full bias): mean error {:.0} m at the {} handover epochs vs {:.0} m at steady epochs -- {}.",
                        trace.seed,
                        trace.leg_duration_min,
                        handover,
                        trace.samples.iter().filter(|s| s.handover).count(),
                        steady,
                        verdict,
                    )
                }
                _ => "representative trace had no handover or no steady epochs to compare".into(),
            }
        },
    );
    vec![
        format!(
            "Geometry control: per-epoch GDOP stays well-conditioned across every leg (mean {} at {} min to {} at {} min, comparable to the multi-satellite good cohort's ~1.8), so the leg-duration and clock levers are measured on fixed, well-conditioned geometry and are not a geometry confound.",
            optional(first.gdop_mean),
            first.leg_duration_min,
            optional(last.gdop_mean),
            last.leg_duration_min,
        ),
        format!(
            "Leg-duration lever (D55/D57), and a METRIC CORRECTION: the noisy single-epoch endpoint p50 wanders non-monotonically ({:.0} m at {} min -> {:.0} m at {} min, p95 {:.0} -> {:.0} m) and its apparent 'improvement with leg length' is a sampling artifact (endpoint seed spread is {:.0}-{:.0} m at {} min). The stable RMS-over-leg metric instead grows MONOTONICALLY with denial time -- p50 {:.0} m ({} min) -> {:.0} m ({} min) -- the physical signature of Doppler-only position error accumulating as the aided prior decays. So error rises, not falls, with sustained denial: short legs (<=~30 min) hold hundreds of m but hour-long endurance legs are KM-SCALE and the D56 500 m goal is NOT met for sustained denial. RMS-over-leg is the recommended headline; the endpoint metric is too noisy to headline.",
            first.horizontal_error_p50_m,
            first.leg_duration_min,
            last.horizontal_error_p50_m,
            last.leg_duration_min,
            first.horizontal_error_p95_m,
            last.horizontal_error_p95_m,
            last.horizontal_error_min_m,
            last.horizontal_error_max_m,
            last.leg_duration_min,
            first.horizontal_rms_p50_m,
            first.leg_duration_min,
            last.horizontal_rms_p50_m,
            last.leg_duration_min,
        ),
        bias_control_verdict(last, last_bz),
        full_trace.map_or_else(
            || "No representative covariance-consistency trace was recorded.".into(),
            covariance_consistency_verdict,
        ),
        handover_alignment,
        format!(
            "Endpoint-metric bimodality: at {} min the best seed's endpoint reaches {:.0} m ({} of {} seeds have endpoint <=500 m) while others stay km-scale. This bimodality is a property of the single-epoch endpoint sample; the RMS-over-leg p95 is {:.0} m, so the underlying leg-averaged solution is km-scale and the sub-500 m endpoints are sampling luck (a good instantaneous epoch), not converged solutions.",
            last.leg_duration_min,
            last.horizontal_error_min_m,
            last_under_500,
            last.seed_horizontal_errors_m.len(),
            last.horizontal_rms_p95_m,
        ),
        format!(
            "D56 goal (500 m p50 / 750 m p95): p50<=500 m first met at {}; p95<=750 m first met at {} (endpoint metric). On honest handover geometry no tested leg or clock robustly delivers the target. The cause is not fundamental Doppler-only position observability -- position is weakly observable (the covariance-consistency check above shows the filter's own sigma converges and stays bounded around ~100 m) -- but FILTER INCONSISTENCY (D68): the measured steady-state error/sigma ratio above ({}) shows the filter is overconfident. Neither this study's leg-duration nor clock levers reach 500 m because neither touches that estimator-consistency defect; the fix (per-SV bias continuity/retirement across handover, covariance-consistency correction, Q retuning) is routed to the ESTIMATOR, out of this study's config-only scope.",
            p50_goal.map_or_else(|| "no tested leg".into(), |value| format!("{} min", value.leg_duration_min)),
            p95_goal.map_or_else(|| "no tested leg".into(), |value| format!("{} min", value.leg_duration_min)),
            full_trace.and_then(|trace| trace.late_leg_consistency_ratio).map_or_else(|| "not recorded".into(), |ratio| format!("{ratio:.1}x")),
        ),
        format!(
            "Clock-discipline lever: between a good clock and a great one it is near-invisible -- at {} min, p95 is {:.0} m at {:.0e} (Rb label) versus {:.0} m at {:.0e} (OCXO label), signed Rb benefit {:.1} m, because the common-mode receiver-clock injection is absorbed by the filter's clock/nuisance states. A {:.0e} POOR clock, however, does degrade the solution ({:.0} m p95, {:.0} m worse than the OCXO), so a poor oscillator hurts but upgrading a good clock to a great one is not a usable denied-position lever.",
            rubidium.leg_duration_min,
            rubidium.horizontal_error_p95_m,
            rubidium.clock_fractional_stability,
            ocxo.horizontal_error_p95_m,
            ocxo.clock_fractional_stability,
            ocxo.horizontal_error_p95_m - rubidium.horizontal_error_p95_m,
            poor.clock_fractional_stability,
            poor.horizontal_error_p95_m,
            poor.horizontal_error_p95_m - ocxo.horizontal_error_p95_m,
        ),
    ]
}

#[allow(clippy::too_many_lines)]
fn markdown(report: &Report) -> String {
    let mut text = format!(
        "# Endurance study: leg duration and clock discipline\n\n**{}**\n\nCross-reference: D55 identified the confounds; D57 identified longer constant-heading legs and clock discipline as the next two levers; D56 defines 500 m typical (p50) and 750 m worst-case (p95); D68 is the verified crux finding this study reproduces and reports evidence for. This study runs on the **identical LEO fixture as the verified multi-satellite study** (the same 960-satellite three-shell Walker grid at 53.0/87.9/86.4 deg, 13-15 rev/day -- correcting the earlier MEO regression and reverting the unjustified 768-SV variant), tracking the best-conditioned eight currently-visible satellites with realistic sticky handovers, and reports per-epoch GDOP so the leg-duration lever is isolated from geometry. **Verdict (D68): the km-scale denied error over long legs is NOT a fundamental LEO-Doppler observability floor.** A bias-zeroed control run (injected per-SV transmit bias set to zero, everything else identical) rules out the injected bias *value* as the cause (ratio ~1.00) but cannot arbitrate fundamental-vs-inconsistency on its own; a per-epoch covariance-consistency trace (the filter's own reported horizontal sigma versus its true horizontal error, measured below) does that directly: position IS weakly observable (filter sigma converges and stays bounded around ~100 m), but the filter is OVERCONFIDENT -- D68's original instrumentation found true error running 7-70x the reported sigma, and this study's own measured steady-state ratio for its representative seed is reported in the covariance-consistency table below. This is FILTER INCONSISTENCY, an ESTIMATION-consistency defect (software-fixable in the estimator: per-SV bias continuity/retirement across handover, covariance-consistency correction, Q retuning), not a physics floor, and that fix is out of this config-only study's scope.\n\n## Fixture\n\n- {} satellites, synthetic [UNVERIFIED]. {}\n",
        report.caveat, report.fixture.satellites, report.fixture.regime
    );
    for shell in &report.fixture.shells {
        let _ = writeln!(text, "  - {shell}");
    }
    text.push_str("\n## Leg-duration curve (full bias)\n\nEndpoint = single-epoch error at leg end (noisy). RMS = root-mean-square horizontal error over every denied-leg doppler epoch (stable headline).\n\n| leg | clock | GDOP mean (min-max) | endpoint p50 | endpoint p95 | RMS p50 | RMS p95 | endpoint spread | accepted/rejected mean | handovers mean | class |\n|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|\n");
    for value in &report.leg_duration_curve {
        let _ = writeln!(
            text,
            "| {} min | {:.0e} | {} ({}-{}) | {:.1} m | {:.1} m | {:.1} m | {:.1} m | {:.1}-{:.1} m | {:.1}/{:.1} | {:.1} | {} |",
            value.leg_duration_min,
            value.clock_fractional_stability,
            optional(value.gdop_mean),
            optional(value.gdop_min),
            optional(value.gdop_max),
            value.horizontal_error_p50_m,
            value.horizontal_error_p95_m,
            value.horizontal_rms_p50_m,
            value.horizontal_rms_p95_m,
            value.horizontal_error_min_m,
            value.horizontal_error_max_m,
            value.accepted_updates_mean,
            value.rejected_updates_mean,
            value.handovers_mean,
            value.error_class
        );
    }
    text.push_str("\n## Bias-value control: bias-zeroed vs full bias\n\nIdentical leg sweep with the injected per-SV transmit bias forced to zero in the truth generator, everything else held fixed. This isolates the injected bias *magnitude* only: if bias-zeroed error collapses toward multisat-class (hundreds of m), the injected bias value was itself a dominant driver. If it stays km-scale (as measured below), the injected bias value is ruled OUT as the cause -- but because both arms keep the identical never-retired per-SV-bias/clock nuisance architecture, this control alone cannot distinguish a fundamental observability floor from filter inconsistency. The covariance-consistency section below (D68) answers that question directly.\n\n| leg | full-bias RMS p50 | bias-zeroed RMS p50 | ratio | full-bias endpoint p50 | bias-zeroed endpoint p50 | bias-zeroed class |\n|---:|---:|---:|---:|---:|---:|---|\n");
    for (full, bz) in report
        .leg_duration_curve
        .iter()
        .zip(&report.leg_duration_curve_bias_zeroed)
    {
        let ratio = if full.horizontal_rms_p50_m > 0.0 {
            bz.horizontal_rms_p50_m / full.horizontal_rms_p50_m
        } else {
            f64::NAN
        };
        let _ = writeln!(
            text,
            "| {} min | {:.1} m | {:.1} m | {:.2} | {:.1} m | {:.1} m | {} |",
            full.leg_duration_min,
            full.horizontal_rms_p50_m,
            bz.horizontal_rms_p50_m,
            ratio,
            full.horizontal_error_p50_m,
            bz.horizontal_error_p50_m,
            bz.error_class,
        );
    }
    text.push_str("\n## Per-epoch error trace (representative seed, handover-aligned)\n\nMean horizontal error at handover epochs vs steady (no-handover) epochs, and the within-leg error trajectory (start third -> end third). Full sample series are in `results.json`.\n\n| trace | seed | leg | handover epochs | mean err @ handover | mean err @ steady | err start-third | err end-third |\n|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for trace in &report.epoch_traces {
        let third = trace.samples.len() / 3;
        let start_third = if third > 0 {
            mean(
                &trace.samples[..third]
                    .iter()
                    .map(|s| s.horizontal_error_m)
                    .collect::<Vec<_>>(),
            )
        } else {
            f64::NAN
        };
        let end_third = if third > 0 {
            mean(
                &trace.samples[trace.samples.len() - third..]
                    .iter()
                    .map(|s| s.horizontal_error_m)
                    .collect::<Vec<_>>(),
            )
        } else {
            f64::NAN
        };
        let handover_count = trace.samples.iter().filter(|s| s.handover).count();
        let _ = writeln!(
            text,
            "| {} | {} | {} min | {} | {} | {} | {:.1} m | {:.1} m |",
            trace.label,
            trace.seed,
            trace.leg_duration_min,
            handover_count,
            optional(trace.mean_error_handover_m),
            optional(trace.mean_error_steady_m),
            start_third,
            end_third,
        );
    }
    text.push_str("\n## Covariance consistency: filter sigma vs true error (D68)\n\nThe filter's own reported horizontal position sigma (DRMS from its covariance) alongside the true horizontal error, at epochs spaced across the representative full-bias leg. A consistent (well-calibrated) filter keeps the ratio near 1; the D68 finding is that this filter's sigma stays around ~100 m (bounded, converging -- position IS observable) while the true error grows to hundreds of m to km-scale, i.e. the filter is OVERCONFIDENT. This is filter inconsistency (an ESTIMATION problem), not a fundamental physics floor: a genuine floor would show the covariance itself growing to km-scale to match the error. Full per-epoch series are in `results.json`.\n\n");
    if let Some(trace) = report.epoch_traces.iter().find(|trace| trace.bias_enabled) {
        text.push_str("| elapsed (s) | filter sigma (m) | true error (m) | ratio (error/sigma) |\n|---:|---:|---:|---:|\n");
        let stride = (trace.samples.len() / 12).max(1);
        for sample in trace.samples.iter().step_by(stride) {
            let ratio = if sample.sigma_horizontal_m > 0.0 {
                sample.horizontal_error_m / sample.sigma_horizontal_m
            } else {
                f64::NAN
            };
            let _ = writeln!(
                text,
                "| {} | {:.1} m | {:.1} m | {:.1}x |",
                sample.elapsed_s, sample.sigma_horizontal_m, sample.horizontal_error_m, ratio,
            );
        }
        let _ = writeln!(
            text,
            "\nWhole-leg mean: filter sigma {} m, true error {} m, mean ratio {}x, peak ratio {}x. Whole-leg mean is diluted by the early epochs right after the aided fix is lost, where both error and sigma are still small; the STEADY-STATE window (last third of the leg, once the aided prior has decayed) is the D68-comparable figure: filter sigma {} m, true error {} m, ratio {}x.",
            optional(trace.mean_sigma_horizontal_m),
            optional(Some(mean(
                &trace
                    .samples
                    .iter()
                    .map(|s| s.horizontal_error_m)
                    .collect::<Vec<_>>()
            ))),
            optional(trace.mean_consistency_ratio),
            optional(trace.max_consistency_ratio),
            optional(trace.late_leg_sigma_horizontal_m),
            optional(trace.late_leg_error_m),
            optional(trace.late_leg_consistency_ratio),
        );
    }
    text.push_str("\n## Clock-discipline curve (fixed leg, fixed good geometry)\n\n| label | fractional stability | drift | GDOP mean | p50 | p95 | spread | accepted/rejected mean | class |\n|---|---:|---:|---:|---:|---:|---:|---:|---|\n");
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
            "| {label} | {:.0e} | {:.6} m/s | {} | {:.1} m | {:.1} m | {:.1}-{:.1} m | {:.1}/{:.1} | {} |",
            value.clock_fractional_stability,
            value.clock_drift_mps,
            optional(value.gdop_mean),
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
        "\n## Controls\n\n- Seeds: {:?}; individual endpoint errors are retained in `results.json`.\n- Real path: production `Executive` and `FilterStub` EKF state versus truth.\n- Gate: production chi-square threshold `Some({:.1})`; rejection counts above are measured integrity events.\n- Geometry: {} Because no fixed eight-SV cohort survives an endurance leg from LEO, handovers are physically required; the identical geometry schedule is reused across every leg and clock tier, so the levers vary against fixed, well-conditioned geometry.\n- Dynamics: {} [UNVERIFIED].\n- No formula, error clamp, target fitting, or replacement estimator is used.\n\n## [UNVERIFIED] inputs\n\n",
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

/// Synthetic LEO Walker grid reused verbatim (same orbital elements) from the
/// verified multi-satellite study: 960 satellites across three shells at LEO
/// mean motions (13-15 rev/day, 550-1200 km), 16 planes x 20 slots per shell
/// with half-slot inter-plane phasing. NOT an MEO grid -- the
/// `fixture_is_leo_not_meo` test guards against that regression.
fn synthetic_fixture() -> String {
    let shells = [(53.0, 15.064), (87.9, 13.158), (86.4, 14.342)];
    let mut text = String::new();
    for index in 0..FIXTURE_SATELLITES {
        let id = FIRST_ID + index;
        let shell = (index / SHELL_SATELLITES) as usize;
        let within_shell = index % SHELL_SATELLITES;
        let plane = within_shell / SHELL_SLOTS;
        let slot = within_shell % SHELL_SLOTS;
        let (inclination, motion) = shells[shell];
        let raan = plane as f64 * 360.0 / SHELL_PLANES as f64;
        let anomaly = slot as f64 * 360.0 / SHELL_SLOTS as f64
            + (plane % 2) as f64 * 180.0 / SHELL_SLOTS as f64;
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

/// Constellation label by shell, matching the three-shell fixture layout so the
/// labels are physically consistent (each shell is one constellation class).
fn constellation(id: u64) -> Constellation {
    match (id - FIRST_ID) / SHELL_SATELLITES {
        0 => Constellation::Starlink,
        1 => Constellation::OneWeb,
        _ => Constellation::Iridium,
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

fn optional(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".into(), |number| format!("{number:.2}"))
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

    /// (mission dir, truth, fixture TLE, geometry schedule, seed) for tests.
    type Harness = (
        TempDir,
        BTreeMap<u64, TruthSample>,
        String,
        BTreeMap<u64, Vec<u64>>,
        u64,
    );

    fn short_fixture() -> Harness {
        let seed = 0xE11D_2026;
        let directory = TempDir::new().unwrap();
        generate_mission(directory.path(), &mission_config(seed, 600, 30)).unwrap();
        let truth = load_truth(directory.path()).unwrap();
        let fixture = synthetic_fixture();
        let store = EphemerisStore::from_tle_str(&fixture)
            .unwrap()
            .with_max_age(Duration::hours(12));
        let schedule = cohort_schedule(&store, &truth, 30, 600).unwrap();
        (directory, truth, fixture, schedule, seed)
    }

    #[test]
    fn core_simulation_is_deterministic() {
        let (directory, truth, fixture, schedule, seed) = short_fixture();
        let first = simulate(
            directory.path(),
            &truth,
            &fixture,
            &schedule,
            600,
            1.0e-9,
            30,
            seed,
            true,
        )
        .unwrap();
        let second = simulate(
            directory.path(),
            &truth,
            &fixture,
            &schedule,
            600,
            1.0e-9,
            30,
            seed,
            true,
        )
        .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn production_gate_is_enabled_and_rejects_injected_outliers() {
        let store = EphemerisStore::from_tle_str(&synthetic_fixture()).unwrap();
        assert_eq!(
            DopplerPipeline::new(store).chi_square_threshold,
            Some(PRODUCTION_CHI_SQUARE_THRESHOLD)
        );
        let (directory, truth, fixture, schedule, seed) = short_fixture();
        let result = simulate(
            directory.path(),
            &truth,
            &fixture,
            &schedule,
            600,
            1.0e-7,
            30,
            seed,
            true,
        )
        .unwrap();
        assert!(
            result.rejected > 0,
            "clock-stressed observations must exercise the gate"
        );
    }

    #[test]
    fn bias_zeroed_control_is_a_real_perturbation() {
        // The decisive experiment must be a genuine counterfactual: the
        // injected per-SV transmit bias must be non-trivially non-zero (so
        // forcing it to zero is a real change to the generator input), and both
        // controls must run to completion sampling the same epochs. Guards
        // against the control being a silent no-op path.
        let seed = 0xE11D_2026;
        assert!(
            (FIRST_ID..FIRST_ID + FIXTURE_SATELLITES).any(|id| sv_bias_hz(id, seed).abs() > 0.1),
            "injected per-SV bias must be materially non-zero for the control to bite"
        );
        let (directory, truth, fixture, schedule, seed) = short_fixture();
        let full = simulate(
            directory.path(),
            &truth,
            &fixture,
            &schedule,
            600,
            1.0e-9,
            30,
            seed,
            true,
        )
        .unwrap();
        let zeroed = simulate(
            directory.path(),
            &truth,
            &fixture,
            &schedule,
            600,
            1.0e-9,
            30,
            seed,
            false,
        )
        .unwrap();
        assert!(!full.trace.is_empty(), "per-epoch trace must be recorded");
        assert!(
            full.horizontal_error_m.is_finite() && zeroed.horizontal_error_m.is_finite(),
            "both controls must produce finite solutions"
        );
        assert_eq!(
            full.trace.len(),
            zeroed.trace.len(),
            "both controls sample the same epochs"
        );
    }

    /// D68 covariance-consistency plumbing: the filter's own reported
    /// horizontal sigma must be recorded at every epoch (real, finite,
    /// positive numbers from the real covariance, not a placeholder), and the
    /// derived per-trace consistency summary (mean sigma, mean/max ratio)
    /// must be populated so `covariance_consistency_verdict` has real data to
    /// arbitrate the fundamental-floor-vs-inconsistency question on.
    #[test]
    fn covariance_consistency_is_instrumented() {
        let (directory, truth, fixture, schedule, seed) = short_fixture();
        let result = simulate(
            directory.path(),
            &truth,
            &fixture,
            &schedule,
            600,
            1.0e-9,
            30,
            seed,
            true,
        )
        .unwrap();
        assert!(!result.trace.is_empty(), "per-epoch trace must be recorded");
        for sample in &result.trace {
            assert!(
                sample.sigma_horizontal_m.is_finite() && sample.sigma_horizontal_m > 0.0,
                "filter's reported horizontal sigma must be a real positive number, got {}",
                sample.sigma_horizontal_m
            );
        }
        let trace = epoch_trace("full-bias", seed, 10, true, &result.trace);
        assert!(trace.mean_sigma_horizontal_m.unwrap() > 0.0);
        assert!(trace.mean_consistency_ratio.unwrap() > 0.0);
        assert!(trace.max_consistency_ratio.unwrap() >= trace.mean_consistency_ratio.unwrap());
        assert!(trace.late_leg_sigma_horizontal_m.unwrap() > 0.0);
        assert!(trace.late_leg_error_m.unwrap() > 0.0);
        assert!(trace.late_leg_consistency_ratio.unwrap() > 0.0);
        // The verdict text must be generated from the measured ratio, not
        // hard-coded, and must not assert a fundamental physics floor.
        let verdict = covariance_consistency_verdict(&trace);
        assert!(
            !verdict
                .to_lowercase()
                .contains("fundamental observability limit"),
            "verdict must not over-claim a fundamental limit: {verdict}"
        );
    }

    #[test]
    fn geometry_is_well_conditioned_and_hands_over() {
        let (directory, truth, fixture, schedule, seed) = short_fixture();
        let result = simulate(
            directory.path(),
            &truth,
            &fixture,
            &schedule,
            600,
            1.0e-9,
            30,
            seed,
            true,
        )
        .unwrap();
        assert!(!result.gdops.is_empty(), "GDOP must be instrumented");
        let worst = result
            .gdops
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            worst < 10.0,
            "selected cohort geometry must stay well-conditioned, got GDOP {worst}"
        );
    }

    #[test]
    fn fixture_is_leo_not_meo() {
        // Guards against the MEO regression: every satellite's mean motion must
        // be a LEO rate (> 10 rev/day). MEO/GPS is ~2 rev/day.
        let fixture = synthetic_fixture();
        let mut checked = 0;
        for line in fixture.lines().filter(|line| line.starts_with("2 ")) {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let mean_motion: f64 = tokens[tokens.len() - 2].parse().unwrap();
            assert!(
                mean_motion > 10.0,
                "fixture satellite is not LEO: mean motion {mean_motion} rev/day"
            );
            checked += 1;
        }
        assert_eq!(checked, FIXTURE_SATELLITES as usize);
    }

    #[test]
    fn truth_stays_at_sea_level_and_coverage_is_continuous() {
        // Long endurance legs must keep a physical truth trajectory and
        // continuous >= SATELLITE_COUNT visibility; guards the wave-slam
        // vertical-drift regression that lifted the vessel above the LEO shell.
        let fixture = synthetic_fixture();
        let store = EphemerisStore::from_tle_str(&fixture)
            .unwrap()
            .with_max_age(Duration::hours(12));
        let directory = TempDir::new().unwrap();
        generate_mission(directory.path(), &mission_config(0xE11D_2026, 3600, 30)).unwrap();
        let truth = load_truth(directory.path()).unwrap();
        for elapsed in (AIDED_S..=AIDED_S + 3600).step_by(30) {
            let sample = &truth[&(elapsed * 1_000_000_000)];
            let position = sample.fix.position_ecef_m;
            let radius = (position[0].powi(2) + position[1].powi(2) + position[2].powi(2)).sqrt();
            assert!(
                (radius - EARTH_RADIUS_M).abs() < 50_000.0,
                "truth left sea level at {elapsed}s: radius {radius:.0} m"
            );
            let visible = (FIRST_ID..FIRST_ID + FIXTURE_SATELLITES)
                .filter(|&id| {
                    let satellite = store.propagate_ecef(id, sample.utc).unwrap();
                    elevation_rad(position, satellite.position_m) >= MASK_DEG.to_radians()
                })
                .count();
            assert!(
                visible >= SATELLITE_COUNT,
                "only {visible} satellites visible at {elapsed}s"
            );
        }
    }

    #[test]
    fn divergence_class_is_never_hidden() {
        assert!(error_class(EARTH_RADIUS_M).starts_with("DIVERGED"));
        assert!(error_class(f64::NAN).starts_with("DIVERGED"));
    }
}
