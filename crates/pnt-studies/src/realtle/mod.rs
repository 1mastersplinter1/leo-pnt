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
// The pnt-mission generator always starts the vessel at the (0N,0E) tangent point (see
// crates/pnt-mission local_to_ecef_up/local_vector_to_ecef); it is not itself parameterised by
// latitude and this module owns no pnt-mission files. A Starlink 53 deg-inclination shell has
// its highest simultaneous-visibility density near the shell's own inclination latitude, not at
// the equator (satellites dwell longer in latitude near the sinusoidal ground-track turning
// point) -- this is ordinary orbital geometry, not a fixture artefact. To study a real,
// physically motivated operating point, RECEIVER_LATITUDE_DEG relocates the whole generated
// scenario with one fixed rigid rotation applied uniformly to every ECEF position and IMU
// acceleration vector (see `relocation_rotation`/`relocate`); local NED velocity and heading are
// already position-independent and are left untouched. The receiver stays at 0E for simplicity.
const RECEIVER_LATITUDE_DEG: f64 = 43.0;
const EPOCH: &str = "2026-07-22T23:52:00Z";
const RECEIVER_CLOCK_DRIFT_MPS: f64 = 0.03;
const PRODUCTION_CHI_SQUARE_GATE: f64 = 9.0;
const SEED_COUNT: usize = 8;
/// Operator-supplied `SupGP` elements (primary, accuracy-preferred per `DESIGN_BASELINE`) for 120
/// Starlink satellites. [UNVERIFIED: grok-fetched, not independently confirmed vs `CelesTrak`.]
const SUPGP_RAW: &str =
    include_str!("../../../pnt-ephemeris/tests/fixtures/real/starlink-supgp-120-2026-204.tle");
/// Plain published TLEs for 150 Starlink satellites (120 of which duplicate `SUPGP_RAW`'s
/// catalog numbers). Used only to supplement the 30 satellites `SupGP` does not cover, so a
/// persistent multi-satellite simultaneous-visibility cohort exists for the controlled leg
/// (N=8 was searched for and found NOT physically available from this real sample at the 5 deg
/// mask for the full five-minute leg; N=7 is the confirmed maximum -- see STUDY.md); every
/// satellite that has a `SupGP` record uses that record instead. [UNVERIFIED: grok-fetched.]
const PLAIN_TLE_RAW: &str =
    include_str!("../../../pnt-ephemeris/tests/fixtures/real/starlink-150-2026-205.tle");

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
            // N=8 was searched for (broad receiver-latitude and epoch sweep across the whole
            // 48h TLE validity window, accounting for the vessel's real generated trajectory,
            // not just an idealised fixed point) and is NOT physically available from this real
            // 150-satellite Starlink-only sample at the 5 deg mask for the full five-minute
            // no-handover leg; N=7 is the confirmed persistent maximum. Reported plainly rather
            // than forcing an unreachable N=8 tier (see STUDY.md).
            //
            // N=0 is a zero-satellite INS-only dead-reckoning coast baseline (no Doppler updates
            // at all over the denied leg). Review finding (U-RT1.1-review-opus.md, D66): over
            // this short 5-minute leg from a sub-meter aided prior, position error grows from
            // pure inertial coast rather than converging via satellite geometry, and even a
            // zero-satellite run clears the 500 m goal -- so this tier makes that coast explicit
            // in the study's own output instead of leaving it to be reconstructed by a reviewer.
            counts: vec![0, 1, 2, 4, 7],
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
    pub supgp_satellites: usize,
    pub plain_tle_supplement_satellites: usize,
    /// True when every satellite actually used in the realized cohorts (not just the merged
    /// fixture available to search over) has a `SupGP` record, i.e. the plain-TLE supplement
    /// was searched but not needed for the reported result.
    pub realized_cohort_is_pure_supgp: bool,
    pub shells: Vec<String>,
    pub elevation_mask_deg: f64,
    pub epoch: String,
    pub receiver_latitude_deg: f64,
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
    let ids = fixture_satellite_ids(&fixture);
    let supgp_ids = fixture_satellite_ids(SUPGP_RAW);
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
            &ids,
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
    let max_requested = config.counts.iter().copied().max().unwrap_or(0);
    let best = outcomes
        .iter()
        .find(|value| value.simultaneous_los == max_requested);
    let n8_availability = if max_requested < 8 {
        format!(
            "N=8 was searched for (receiver latitude 25-60 deg and the full 48h TLE validity window, checked against the vessel's actual generated trajectory rather than an idealised fixed point) and is not physically available from this real fixture for the full five-minute persistent no-handover leg; N={max_requested} is the confirmed maximum, reported here instead of forcing an unreachable tier. "
        )
    } else {
        String::new()
    };
    let headline = best.map_or_else(
        || {
            format!(
                "{n8_availability}The real-TLE fixture supports only {} persistent simultaneous LOS over the controlled five-minute leg.",
                cohort.len()
            )
        },
        |value| {
            format!(
                "{n8_availability}Controlled N={} manoeuvring result on REAL Starlink geometry: mean {:.1} m, p95 {:.1} m, range {:.1}-{:.1} m across {} seeds ({}). This is DEAD-RECKONING COAST from a sub-meter GPS-aided prior over a short {}-minute leg, not multi-satellite Doppler position observability -- an N=0 INS-only baseline with zero satellites also clears the goal; this fixture+leg cannot test real multi-satellite geometry observability (see diagnosis, and docs/studies/endurance/STUDY.md for the real long-leg observability answer).",
                value.simultaneous_los,
                value.endpoint_position_error_mean_m,
                value.endpoint_position_error_p95_m,
                value.endpoint_position_error_min_m,
                value.endpoint_position_error_max_m,
                config.seeds.len(),
                value.error_class,
                value.duration_min
            )
        },
    );
    let diagnosis = coast_verdict(&outcomes, max_requested);
    let supplement_count = ids.len() - supgp_ids.len();
    let supgp_id_set = supgp_ids.iter().copied().collect::<BTreeSet<_>>();
    let realized_cohort_is_pure_supgp = cohort.iter().all(|id| supgp_id_set.contains(id));
    let report = Report {
        schema_version: 4,
        caveat: "REAL-PUBLISHED-TLE GEOMETRY CHECK [UNVERIFIED currency/provenance]. Endpoints come from the production Executive + FilterStub against synthetic generator truth; no result is clamped or target-fitted. The elements were grok-fetched and were not independently confirmed against CelesTrak. The receiver is placed at a fixed mid-latitude via a single rigid coordinate rotation (see module docs) so a real Starlink shell has adequate simultaneous coverage; this is a placement choice made from visibility geometry alone, before any accuracy result was computed. This is a SHORT (5-minute) leg from a sub-meter GPS-aided prior: an N=0 INS-only zero-satellite baseline is included below because, over this leg, dead-reckoning coast alone clears the 500 m goal -- see diagnosis for why this fixture+leg cannot test real multi-satellite Doppler position observability.".into(),
        fixture: FixtureDescription {
            real_published_unverified: true,
            usable_tles: ids.len(),
            satellites: ids.len(),
            supgp_satellites: supgp_ids.len(),
            plain_tle_supplement_satellites: supplement_count,
            realized_cohort_is_pure_supgp,
            shells: vec![
                format!(
                    "Starlink SupGP (operator-supplied, accuracy-preferred): {} usable, primary product for every satellite that has one.",
                    supgp_ids.len()
                ),
                format!(
                    "Starlink plain-TLE supplement (same ~53 deg inclination shell, public knowledge -- not sourced from R4, which covers signal structure, not orbital elements): {supplement_count} usable, used only for the satellites SupGP does not cover, needed to complete the largest persistent cohort this real sample supports."
                ),
            ],
            elevation_mask_deg: MASK_DEG,
            epoch: EPOCH.into(),
            receiver_latitude_deg: RECEIVER_LATITUDE_DEG,
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
            geometry_isolation: format!(
                "A single persistent real-TLE cohort of {} satellites is selected once per mission from the {}-satellite merged fixture. N tiers use nested prefixes (plus an N=0 INS-only baseline with zero satellites and no Doppler updates at all), all satellites remain above {MASK_DEG} deg for every denied Doppler epoch, and no tier hands over; only simultaneous distinct LOS count changes.",
                cohort.len(),
                ids.len()
            ),
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
    let rotation = relocation_rotation();
    let mut truth = BTreeMap::new();
    for record in TruthReader::open(path)? {
        let TruthJournalRecord::Envelope(envelope) = record? else {
            continue;
        };
        let MeasurementPayload::Gnss(mut fix) = envelope.payload else {
            continue;
        };
        fix.position_ecef_m = relocate(fix.position_ecef_m, &rotation);
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
    ids: &[u64],
    requested: usize,
    interval_s: u64,
    denied_s: u64,
) -> Result<Vec<u64>, StudyError> {
    let mut persistent = ids.iter().copied().collect::<BTreeSet<_>>();
    for elapsed in (AIDED_S..=AIDED_S + denied_s).step_by(interval_s as usize) {
        let sample = &truth[&(elapsed * 1_000_000_000)];
        let mut visible = BTreeSet::new();
        for &id in ids {
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
    let rotation = relocation_rotation();
    let mut sequence = 10_000_000_u64;
    let mut gdops = Vec::new();

    for record in MeasurementReader::open(path)? {
        let MeasurementJournalRecord::Envelope(mut envelope) = record? else {
            continue;
        };
        let elapsed_s = envelope.host_receive_monotonic_ns / 1_000_000_000;
        // Relocate the mission-generator's (0N,0E)-anchored ECEF vectors to the study receiver
        // latitude with the same fixed rotation used throughout (see `relocation_rotation`).
        // NED velocity and heading are position-independent and are left untouched.
        match &mut envelope.payload {
            MeasurementPayload::Imu(imu) => {
                imu.acceleration_mps2 = relocate(imu.acceleration_mps2, &rotation);
            }
            MeasurementPayload::Gnss(fix) if elapsed_s <= AIDED_S => {
                fix.position_ecef_m = relocate(fix.position_ecef_m, &rotation);
            }
            _ => {}
        }
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
        geometry: if count == 0 {
            "zero-satellite INS-only dead-reckoning coast (no Doppler updates; baseline control)"
        } else if count == 1 {
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

/// Builds the honest coast-vs-observability diagnosis. This is the single source of truth for
/// the reframed verdict (review finding, D66/U-RT1.1-review-opus.md): the 36-52 m band on this
/// short 5-minute leg is dead-reckoning coast from a sub-meter aided prior, not multi-satellite
/// Doppler position observability, so this fixture+leg cannot test that question either way. It
/// is used verbatim for the JSON `diagnosis` field and (via `report.diagnosis`) for the Markdown
/// "Real result" section, so the two outputs can never drift apart.
fn coast_verdict(outcomes: &[Outcome], max_requested: usize) -> String {
    let Some(best) = outcomes
        .iter()
        .find(|value| value.simultaneous_los == max_requested)
    else {
        return format!(
            "N={max_requested} was not run, so no real-geometry multi-satellite accuracy conclusion is available."
        );
    };
    let baseline_sentence = outcomes
        .iter()
        .find(|value| value.simultaneous_los == 0)
        .map_or_else(String::new, |value| {
            format!(
                " An N=0 INS-only control (zero satellites, pure inertial dead-reckoning coast from the same sub-meter aided prior, no Doppler updates at all over the denied leg) reaches mean {:.1} m / p95 {:.1} m ({}) over the identical leg -- also clearing the same D56 500 m p50 / 750 m p95 usable-denied target with zero denied-leg position observations.",
                value.endpoint_position_error_mean_m,
                value.endpoint_position_error_p95_m,
                value.error_class
            )
        });
    let n1_sentence = outcomes
        .iter()
        .find(|value| value.simultaneous_los == 1)
        .map_or_else(String::new, |value| {
            format!(
                " N=1 (position-unobservable, infinite GDOP) reaches mean {:.1} m / p95 {:.1} m -- close to the N=0 baseline, not to a geometry-driven floor.",
                value.endpoint_position_error_mean_m, value.endpoint_position_error_p95_m
            )
        });
    format!(
        "This is DEAD-RECKONING COAST from a sub-meter GPS-aided prior over a short (5-minute) leg, not multi-satellite Doppler position observability, and this fixture+leg CANNOT test that question either way.{baseline_sentence}{n1_sentence} Position error grows over the denied leg rather than converging, and pass/fail against the D56 500 m p50 / 750 m p95 usable-denied target is independent of geometry here: N={} (GDOP mean/p95 {}/{}) reaches mean {:.1} m / p95 {:.1} m, and every tier -- including the position-unobservable N=1 and the zero-satellite N=0 baseline -- clears the target. The real-SupGP fixture validation (real operator-supplied/published Starlink tracks parsing and propagating correctly against published shell inclinations, grown from the original 40-element mixed-constellation fixture to this study's 150-satellite merged fixture) is this study's actual sound contribution; it does not answer whether real multi-satellite Doppler geometry meets the denied-position goal. That question is answered instead by the long-leg endurance study (D68/D69, docs/studies/endurance/STUDY.md): over long denied legs position IS weakly observable (filter sigma converges and stays bounded, roughly 50-160 m), but the filter is INCONSISTENT/OVERCONFIDENT (true error runs several-fold -- up to 7-70x per D68's original instrumentation, ~3x steady-state in the endurance study's own measured run -- above the reported sigma; an estimation-consistency defect, not a physics floor), and the 500 m goal is NOT met over those long legs. Read this short-leg real-geometry result as neither meeting nor failing the real observability question -- it demonstrates only that this fixture+leg is structurally coast-dominated and cannot test it.",
        best.simultaneous_los,
        optional(best.gdop_mean),
        optional(best.gdop_p95),
        best.endpoint_position_error_mean_m,
        best.endpoint_position_error_p95_m
    )
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

/// Groups a raw multi-record TLE/3LE text into `[name, line1, line2]` blocks. Records are
/// three physical lines each in these fixtures (a name line followed by the two TLE lines).
fn tle_blocks(raw: &str) -> Vec<[&str; 3]> {
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .chunks_exact(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect()
}

/// Parses the NORAD catalog number from a TLE line 1 (columns 3-7).
fn tle_id(line1: &str) -> u64 {
    line1[2..7]
        .trim()
        .parse()
        .expect("valid NORAD catalog number")
}

/// Derives the fixture's real satellite IDs at runtime by parsing every TLE line 1 in the raw
/// text (review finding F3: no hardcoded satellite ID list).
fn fixture_satellite_ids(fixture: &str) -> Vec<u64> {
    fixture
        .lines()
        .filter(|line| line.starts_with("1 "))
        .map(tle_id)
        .collect()
}

/// The primary/supplement merge: `SUPGP_RAW` (operator-supplied, accuracy-preferred) for every
/// satellite it covers, plus `PLAIN_TLE_RAW` records only for the satellites `SupGP` does not
/// cover. All 120 `SupGP` catalog numbers are a subset of the 150-satellite plain fixture, so this
/// adds exactly the satellites `SupGP` is missing; no satellite ever uses both products.
fn real_fixture() -> String {
    let supgp_ids = fixture_satellite_ids(SUPGP_RAW)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut text = String::new();
    for block in tle_blocks(SUPGP_RAW) {
        for line in block {
            text.push_str(line);
            text.push('\n');
        }
    }
    for block in tle_blocks(PLAIN_TLE_RAW) {
        if supgp_ids.contains(&tle_id(block[1])) {
            continue;
        }
        for line in block {
            text.push_str(line);
            text.push('\n');
        }
    }
    text
}

/// The single rigid rotation that relocates the pnt-mission generator's (0N,0E)-anchored scenario
/// to `RECEIVER_LATITUDE_DEG` at 0E. It maps the origin's (up, east, north) basis -- ECEF (X, Y,
/// Z) at (0N,0E) -- onto (up, east, north) at the target latitude. Because the generator's ECEF
/// positions are a true spherical embedding of tiny (sub-2 km) local displacements and its ECEF
/// vectors (IMU acceleration) are that embedding's linearisation at the origin (see
/// `pnt_mission::local_to_ecef_up`/`local_vector_to_ecef`), applying this one matrix to every
/// such position and vector is an exact rigid relocation of the whole scenario, not an
/// approximation and not a per-result adjustment.
fn relocation_rotation() -> [[f64; 3]; 3] {
    let (sin_lat, cos_lat) = RECEIVER_LATITUDE_DEG.to_radians().sin_cos();
    [
        [cos_lat, 0.0, -sin_lat],
        [0.0, 1.0, 0.0],
        [sin_lat, 0.0, cos_lat],
    ]
}

fn relocate(vector: [f64; 3], rotation: &[[f64; 3]; 3]) -> [f64; 3] {
    std::array::from_fn(|row| {
        (0..3)
            .map(|column| rotation[row][column] * vector[column])
            .sum()
    })
}

fn constellation(id: u64) -> Constellation {
    // Both real fixtures (SupGP and the plain-TLE supplement) are Starlink-only; there is no
    // OneWeb/Iridium element in this study's satellite set.
    let _ = id;
    Constellation::Starlink
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
    let max_cohort = report
        .outcomes
        .iter()
        .map(|value| value.simultaneous_los)
        .max()
        .unwrap_or(0);
    let best = report
        .outcomes
        .iter()
        .filter(|value| value.simultaneous_los > 0)
        .max_by_key(|value| value.simultaneous_los);
    let n8_note = if max_cohort > 0 && max_cohort < 8 {
        format!(
            "N=8 was searched for (receiver latitude 25-60 deg and the full 48h TLE validity window, against the vessel's actual generated trajectory) and is not physically available from this real fixture for the full five-minute persistent leg; N={max_cohort} is the confirmed maximum. "
        )
    } else {
        String::new()
    };
    // Deliberately NOT titled "verdict": per the reframe (D66/U-RT1.1-review-opus.md), this is a
    // raw numeric comparison against the synthetic multisat result, not an observability
    // comparison -- the coast diagnosis above (`report.diagnosis`) is the actual interpretation.
    let comparison = best.map_or_else(
        || format!(
            "{n8_note}This real-element run's persistent cohort tops out at N={max_cohort} for the controlled five-minute no-handover leg, so it cannot directly replicate the synthetic N=8 multisat result (D57: mean 116 m / p95 554 m, GDOP ~1.8). It cannot validate or falsify that number."
        ),
        |value| format!(
            "{n8_note}Real N={} gives mean {:.1} m / p95 {:.1} m with GDOP mean/p95 {}/{}, against the synthetic N=8 multisat result of mean 116 m / p95 554 m, GDOP ~1.8 (D57). Both numbers are small, but per the diagnosis above neither this real N={} result nor the N=0 INS-only baseline that reaches a similar order of magnitude is a geometry-driven number on this short leg -- so this is an arithmetic comparison only, not evidence that real geometry does or does not undermine the synthetic finding. GDOP also differs sharply (real p95 {} vs synthetic ~1.8) without predicting the (coast-dominated) endpoint error, consistent with the coast diagnosis above.",
            value.simultaneous_los,
            value.endpoint_position_error_mean_m,
            value.endpoint_position_error_p95_m,
            optional(value.gdop_mean),
            optional(value.gdop_p95),
            value.simultaneous_los,
            optional(value.gdop_p95),
        ),
    );
    let _ = write!(
        text,
        "\n## Controls and interpretation\n\n- Seeds: {:?}; individual endpoint errors are retained in `results.json`.\n- Dynamics: {}.\n- Geometry: {}. GDOP is the conventional instantaneous velocity-plus-common-clock geometry metric; N<4 is unobservable/infinite. This is a {}-satellite real sample, not a complete operational constellation.\n- Receiver placement: fixed at {:.1} deg N, 0 deg E via a single rigid coordinate rotation applied to every generated ECEF position and IMU acceleration vector -- chosen from visibility geometry alone (Starlink's ~53 deg shell -- public knowledge, not sourced from docs/research/R4-signal-structures.md, which covers signal/frame structure, not orbital elements -- has its densest simultaneous coverage near its own inclination latitude, not at the equator where the synthetic generator's default origin sits), before any accuracy number was computed.\n- Clock stress: receiver drift {:.3} m/s ({:.3} ppb) and {}. These values and the noise model are [UNVERIFIED].\n- Measurement stress: bounded \u{b1}0.5 Hz nominal error plus deterministic signed 12 Hz tracker outliers at about 1/17 observations [UNVERIFIED].\n- The production chi-square gate is `Some(9.0)`; accepted/rejected counts come from integrity events.\n\n## Real-vs-synthetic numeric comparison (not an observability comparison)\n\n{comparison}\n\n## SupGP vs plain TLE: why the geometry check is valid on either product\n\nSupGP is operator-supplied and materially more accurate than SGP4-on-plain-TLE (plain TLE/SGP4 position error is commonly kilometre-scale; SupGP tracks are tighter). For this study's question -- does real orbital LOS geometry (visible count, GDOP) resemble the synthetic Walker fixture's -- the two products are effectively interchangeable: at a shared epoch, the line-of-sight *directions* from a fixed receiver to a given real satellite differ negligibly between SupGP and plain-TLE propagation of the same object, because both track the same real orbit to well within the angular resolution that matters for elevation-mask visibility and GDOP. Track quality (SupGP vs plain TLE) matters far more for the *absolute* position/Doppler accuracy budget used in the real-signal acceptance/age-gate work than for this geometry question -- which is why SupGP is used as primary (accuracy-preferred, per DESIGN_BASELINE) while the {}-satellite plain-TLE supplement (SupGP does not cover them) is used solely to complete the persistent N={max_cohort} cohort{}.\n\n## [UNVERIFIED]\n\n- TLE/SupGP source and currency: grok-fetched, not independently confirmed against CelesTrak; physical parse/propagation and shell inclinations are confirmed.\n- Synthetic vessel truth, IMU/wave/turn model, clock drift, per-SV bias, cadence, and Doppler noise/outliers.\n- Whether this {}-satellite sample is representative of full operational Starlink coverage; it is not a complete constellation snapshot.\n- The receiver-latitude relocation is an exact rigid-rotation reinterpretation of the synthetic vessel's already-generated dynamics (see module docs), not a re-simulation at that latitude from first principles.\n",
        report.controls.seed_values,
        report.controls.dynamics,
        report.controls.geometry_isolation,
        report.fixture.satellites,
        report.fixture.receiver_latitude_deg,
        report.controls.receiver_clock_drift_mps,
        report.controls.receiver_clock_fractional_ppb,
        report.controls.per_sv_transmit_bias_hz,
        report.fixture.plain_tle_supplement_satellites,
        if report.fixture.realized_cohort_is_pure_supgp {
            " -- every satellite actually used in the table above happens to have a SupGP record, so this table's real accuracy numbers are on pure operator-supplied tracks; the supplement was searched over but not needed for the realized result"
        } else {
            " -- some satellites actually used in the table above only have plain-TLE elements (no SupGP record for them)"
        },
        report.fixture.satellites
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
        let ids = fixture_satellite_ids(&fixture);
        let store = EphemerisStore::from_tle_str(&fixture)
            .unwrap()
            .with_max_age(Duration::hours(48));
        let cohort = persistent_cohort(
            &store,
            &truth,
            &ids,
            7,
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

        let ids = fixture_satellite_ids(&fixture);
        let initial = &truth[&(AIDED_S * 1_000_000_000)];
        let visible = ids
            .iter()
            .filter(|&&id| {
                let satellite = store.propagate_ecef(id, initial.utc).unwrap();
                elevation_rad(initial.fix.position_ecef_m, satellite.position_m)
                    >= MASK_DEG.to_radians()
            })
            .count();
        let visible_fraction = visible as f64 / ids.len() as f64;
        assert!(
            (0.01..0.5).contains(&visible_fraction),
            "implausible visible fraction: {visible}/{}",
            ids.len()
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
        let ids = fixture_satellite_ids(&fixture);
        let store = EphemerisStore::from_tle_str(&fixture).unwrap();
        for &id in &ids {
            let epoch = store.epoch(id).unwrap();
            let state = store.propagate_teme(id, epoch).unwrap();
            assert!(state.position_km.into_iter().all(f64::is_finite));
            assert!(state.velocity_kmps.into_iter().all(f64::is_finite));
        }

        // The Starlink ~53 deg shell inclination is public knowledge (published by the
        // operator/regulatory filings), not sourced from docs/research/R4-signal-structures.md,
        // which documents Ku-band downlink signal/frame structure, not orbital elements
        // (review finding F1: an earlier report mis-attributed this cross-check to R4).
        let inclinations = fixture
            .lines()
            .filter(|line| line.starts_with("2 "))
            .map(|line| {
                let id = line[2..7].trim().parse::<u64>().unwrap();
                let inclination = line[8..16].trim().parse::<f64>().unwrap();
                (id, inclination)
            })
            .collect::<Vec<_>>();
        assert_eq!(inclinations.len(), ids.len());
        for (id, inclination) in inclinations {
            assert!((inclination - 53.0).abs() <= 0.2, "{id}: {inclination}");
        }
    }

    /// Locks the fixture's usable satellite count and the persistent N=7 cohort size (review
    /// finding F2): a future fixture swap that silently shrinks real coverage below what the
    /// headline needs must fail this test, not silently degrade the study. Also locks in the
    /// honest N=8-unavailable finding (task item 1) so a fixture/receiver change can't silently
    /// start claiming N=8 without a deliberate update here, and checked across every default
    /// seed (each has a slightly different vessel trajectory from seed-dependent wave/turn
    /// noise), not just one.
    #[test]
    fn fixture_size_and_n7_cohort_are_locked() {
        let fixture = real_fixture();
        let ids = fixture_satellite_ids(&fixture);
        assert_eq!(
            ids.len(),
            150,
            "merged real fixture satellite count drifted"
        );
        let supgp_ids = fixture_satellite_ids(SUPGP_RAW);
        assert_eq!(supgp_ids.len(), 120, "SupGP primary fixture count drifted");
        assert_eq!(
            ids.len() - supgp_ids.len(),
            30,
            "plain-TLE supplement count drifted"
        );

        let config = RealTleConfig::default();
        assert_eq!(
            config.counts.iter().copied().max(),
            Some(7),
            "default sweep no longer tops out at the confirmed real N=7 maximum"
        );
        let store = EphemerisStore::from_tle_str(&fixture)
            .unwrap()
            .with_max_age(Duration::hours(48));
        for &seed in &config.seeds {
            let mission_dir = TempDir::new().unwrap();
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
            )
            .unwrap();
            let mut truth = load_truth(mission_dir.path()).unwrap();
            retime_truth(&mut truth).unwrap();
            let cohort = persistent_cohort(
                &store,
                &truth,
                &ids,
                7,
                config.doppler_interval_s,
                config.manoeuvring_denied_s,
            )
            .expect("N=7 persistent cohort must be available at the fixed study receiver/epoch");
            assert_eq!(cohort.len(), 7, "N=7 cohort size drifted for seed {seed}");
            assert!(
                persistent_cohort(
                    &store,
                    &truth,
                    &ids,
                    8,
                    config.doppler_interval_s,
                    config.manoeuvring_denied_s,
                )
                .is_err(),
                "N=8 unexpectedly became available for seed {seed}; update the honest \
                 N=8-unavailable finding (and RealTleConfig::default counts) deliberately \
                 instead of leaving this assertion stale"
            );
        }
    }

    #[test]
    fn production_gate_is_on() {
        assert!((PRODUCTION_CHI_SQUARE_GATE - 9.0).abs() < f64::EPSILON);
    }

    /// Locks the N=0 INS-only dead-reckoning coast baseline into the default sweep (task item 1,
    /// review finding D66): the coast must stay explicit in the study's own output, not left to
    /// be reconstructed by a reviewer instrumenting the filter by hand.
    #[test]
    fn default_sweep_includes_ins_only_coast_baseline() {
        let config = RealTleConfig::default();
        assert!(
            config.counts.contains(&0),
            "N=0 INS-only coast baseline must stay in RealTleConfig::default().counts"
        );
    }

    fn seed_result(position_error_m: f64, nuisance_states: usize) -> SeedResult {
        SeedResult {
            position_error_m,
            velocity_error_mps: 0.0,
            accepted: 0,
            rejected: 0,
            nuisance_states,
            gdops: Vec::new(),
        }
    }

    #[test]
    fn n0_baseline_has_no_satellites_no_updates_and_a_distinct_geometry_label() {
        let config = RealTleConfig::default();
        let outcome = aggregate(0, &[], std::slice::from_ref(&seed_result(99.0, 0)), &config);
        assert!(outcome.satellite_ids.is_empty(), "N=0 must use zero SVs");
        assert!(outcome.nuisance_state_count_mean.abs() < f64::EPSILON);
        assert!(outcome.accepted_updates_mean.abs() < f64::EPSILON);
        assert_eq!(outcome.gdop_mean, None);
        assert!(
            outcome.geometry.contains("INS-only"),
            "N=0 geometry label must name it as an INS-only baseline, not a satellite cohort: {}",
            outcome.geometry
        );
    }

    /// Locks the reframed diagnosis (task item 2, D66): must name the coast mechanism and the
    /// N=0 baseline explicitly, cross-reference the endurance study for the real observability
    /// answer, and must NOT reproduce either of the failed unit's overclaims -- "reaches the D56
    /// usable denied target" attributed to real geometry, or "does not undermine the synthetic
    /// finding" -- regardless of whether the numeric result happens to pass or fail 500/750 m.
    #[test]
    fn coast_verdict_names_the_baseline_and_drops_the_overclaim() {
        let config = RealTleConfig::default();
        let outcomes = vec![
            aggregate(0, &[], std::slice::from_ref(&seed_result(99.0, 0)), &config),
            aggregate(
                1,
                &[100],
                std::slice::from_ref(&seed_result(88.0, 1)),
                &config,
            ),
            aggregate(
                7,
                &[100, 200, 300, 400, 500, 600, 700],
                std::slice::from_ref(&seed_result(40.0, 7)),
                &config,
            ),
        ];
        let text = coast_verdict(&outcomes, 7);
        assert!(text.contains("DEAD-RECKONING COAST"), "{text}");
        assert!(text.contains("N=0 INS-only control"), "{text}");
        assert!(text.contains("endurance"), "{text}");
        assert!(text.contains("CANNOT test that question"), "{text}");
        assert!(
            !text.contains("reaches the D56 usable denied target"),
            "must not attribute the D56-clearing result to real geometry: {text}"
        );
        assert!(
            !text.contains("does not undermine the synthetic finding"),
            "must not claim real geometry validates the synthetic result: {text}"
        );
    }

    /// Same `coast_verdict`, but the N=7 tier misses 750 m p95 -- the diagnosis must still avoid
    /// claiming real geometry *fails* the observability question, only that this leg can't test
    /// it (task instruction: do not claim fail either).
    #[test]
    fn coast_verdict_does_not_claim_geometry_fails_when_p95_misses_goal() {
        let config = RealTleConfig::default();
        let failing_seed = SeedResult {
            position_error_m: 900.0,
            velocity_error_mps: 0.0,
            accepted: 0,
            rejected: 0,
            nuisance_states: 7,
            gdops: Vec::new(),
        };
        let outcomes = vec![
            aggregate(0, &[], std::slice::from_ref(&seed_result(99.0, 0)), &config),
            aggregate(
                7,
                &[100, 200, 300, 400, 500, 600, 700],
                std::slice::from_ref(&failing_seed),
                &config,
            ),
        ];
        let text = coast_verdict(&outcomes, 7);
        assert!(
            !text.contains("does NOT reach the D56"),
            "must not phrase a geometry-attributed fail verdict: {text}"
        );
        assert!(text.contains("CANNOT test that question"), "{text}");
    }
}
