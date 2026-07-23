//! Deterministic synthetic maritime mission generation and paired replay study.
#![allow(clippy::cast_precision_loss, clippy::too_many_lines)]

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use pnt_ephemeris::{EphemerisError, EphemerisStore};
use pnt_journal::{FileJournals, JournalError, RunManifest, RunMetadata};
use pnt_predictor::{predict, ReceiverState, SatelliteState};
use pnt_replay::{
    replay_paired, replay_paired_configured, ReceiverPrior, ReplayDopplerConfig, ReplayError,
    ReplayReport, RunSummary,
};
use pnt_tracker::synth::{BpskReference, SynthConfig, Synthesizer};
use pnt_tracker::{ConfigError, EnvelopeMetadata, TrackOutcome, TrackerConfig};
use pnt_types::{
    Constellation, Frame, GnssFix, Heading, ImuSample, MeasurementEnvelope, MeasurementPayload,
    Provenance, QualityFlags, SourceId, SpeedThroughWater, TimeTag, UtcTime, SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::{f64::consts::FRAC_PI_2, path::Path};

const EARTH_RADIUS_M: f64 = 6_378_137.0;
const NORAD_ID: u64 = 25_544;
const CARRIER_HZ: f64 = 1_600_000_000.0;
const TRACKER_SAMPLE_RATE_HZ: f64 = 131_072.0;
const TRACKER_REFERENCE_LEN: usize = 256;
const TRACKER_TOLERANCE_HZ: f64 = 4.0;
const TLE: &str = include_str!("../../pnt-ephemeris/tests/fixtures/iss.tle");

#[derive(Debug, thiserror::Error)]
pub enum MissionError {
    #[error("invalid mission configuration: {0}")]
    InvalidConfig(String),
    #[error(transparent)]
    Journal(#[from] JournalError),
    #[error(transparent)]
    Ephemeris(#[from] EphemerisError),
    #[error(transparent)]
    Replay(#[from] ReplayError),
    #[error(transparent)]
    TrackerConfig(#[from] ConfigError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Optional coordinated-turn dynamics. `None` retains the historical mission byte-for-byte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinatedTurnConfig {
    /// Commanded yaw rate during the middle ten percent of the mission.
    pub rate_rad_s: f64,
}

/// Seeded synthetic planing-hull wave/slam stand-in.
///
/// This is **[UNVERIFIED vs real planing data]**. Bursts use a bounded, zero-mean full-cycle
/// vertical acceleration and a pitch-coupled horizontal component. They are not a sea-state model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveSlamConfig {
    pub burst_rate_hz: f64,
    pub duration_s: f64,
    pub vertical_peak_mps2: f64,
    pub pitch_coupling: f64,
}

/// Optional linear speed scaling for IMU white noise and fixed bias.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeedScaledImuConfig {
    pub reference_speed_mps: f64,
    pub noise_per_speed_ratio: f64,
    pub bias_per_speed_ratio: f64,
}

/// Configuration values are deliberately modest by default so the smoke command is quick.
#[derive(Clone, Debug, PartialEq)]
pub struct MissionConfig {
    pub seed: u64,
    pub duration_s: u64,
    pub imu_rate_hz: u64,
    pub speed_through_water_mps: f64,
    pub current_north_mps: f64,
    pub current_east_mps: f64,
    pub imu_bias_mps2: [f64; 3],
    pub imu_noise_std_mps2: f64,
    pub heading_noise_std_rad: f64,
    pub speed_noise_std_mps: f64,
    pub gnss_noise_std_m: f64,
    pub doppler_noise_std_hz: f64,
    pub elevation_mask_rad: f64,
    /// Whole-second cadence for tracker-in-loop observations.
    pub doppler_interval_s: u64,
    /// Optional element-epoch offset for ephemeris-aging campaigns.
    pub ephemeris_start_age_s: Option<u64>,
    pub coordinated_turn: Option<CoordinatedTurnConfig>,
    pub wave_slam: Option<WaveSlamConfig>,
    pub speed_scaled_imu: Option<SpeedScaledImuConfig>,
}

impl Default for MissionConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            duration_s: 180,
            imu_rate_hz: 100,
            speed_through_water_mps: 3.0,
            current_north_mps: 0.25,
            current_east_mps: -0.10,
            imu_bias_mps2: [2.0e-4, -1.0e-4, 0.0],
            imu_noise_std_mps2: 5.0e-4,
            heading_noise_std_rad: 0.01,
            speed_noise_std_mps: 0.02,
            gnss_noise_std_m: 0.5,
            doppler_noise_std_hz: 2.0,
            elevation_mask_rad: 5.0_f64.to_radians(),
            doppler_interval_s: 1,
            ephemeris_start_age_s: None,
            coordinated_turn: None,
            wave_slam: None,
            speed_scaled_imu: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissionSummary {
    pub seed: u64,
    pub duration_s: u64,
    pub measurement_count: u64,
    pub truth_count: u64,
    pub doppler_count: u64,
    pub constant_heading_samples: u64,
    pub turn_samples: u64,
    pub tracker_in_loop_count: u64,
    pub tracker_max_direct_error_hz: Option<f64>,
    pub synthetic_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StudyReport {
    pub caveat: String,
    pub mission: MissionSummary,
    pub replay: ReplayReport,
    pub four_way: FourWayTable,
    pub attribution: StudyAttribution,
    pub qualitative_demonstration: QualitativeDemonstration,
    pub integration_gaps: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FourWayTable {
    pub denied_dr_only: RunSummary,
    pub denied_prior_only: RunSummary,
    pub denied_prior_with_doppler: RunSummary,
    pub denied_no_prior_with_doppler: RunSummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StudyAttribution {
    /// Caller-provided initialization; this is truth-equivalent for the synthetic fixture.
    pub disclosed_receiver_prior: ReceiverPrior,
    /// DR-only RMS minus prior-only RMS. Positive values mean the prior reduced error.
    pub prior: RmsContribution,
    /// Prior-only RMS minus prior+Doppler RMS. Positive values mean Doppler reduced error.
    pub doppler_given_prior: RmsContribution,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RmsContribution {
    pub position_rms_reduction_m: f64,
    pub speed_rms_reduction_mps: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QualitativeDemonstration {
    pub aided_smaller_than_withheld: bool,
    pub doppler_rich_constant_heading_present: bool,
    pub outage_or_turn_present: bool,
}

/// Generate a `FileJournals` run whose metadata and record ordering depend only on `config`.
///
/// # Errors
///
/// Returns a typed error when ephemeris parsing, journal creation/writes, or finalization fails.
///
/// # Panics
///
/// Panics only if the compile-time fixture unexpectedly lacks its declared NORAD identifier.
pub fn generate_mission(
    output: impl AsRef<Path>,
    config: &MissionConfig,
) -> Result<MissionSummary, MissionError> {
    validate_config(config)?;
    let output = output.as_ref();
    let store = EphemerisStore::from_tle_str(TLE)?;
    let epoch = store.epoch(NORAD_ID).expect("fixture satellite exists");
    let start = config.ephemeris_start_age_s.map_or_else(
        || find_visible_start(&store, epoch),
        |age_s| epoch + Duration::seconds(i64::try_from(age_s).unwrap_or(i64::MAX)),
    );
    let metadata = RunMetadata {
        run_uuid: format!("synthetic-mission-{:016x}", config.seed),
        created_utc_rfc3339: Some(start.to_rfc3339_opts(SecondsFormat::Nanos, true)),
        monotonic_epochs: Vec::new(),
        config_hash: config_fingerprint(config),
        calibration_ids: vec!["synthetic-cal-v1".into()],
        software_revision: "pnt-mission-v1".into(),
        hardware_setup: "synthetic displacement-hull vessel".into(),
        ephemeris_snapshot_id: "fixture-iss-2020-194".into(),
    };
    let mut journals = FileJournals::create(output, metadata, 64 * 1024 * 1024)?;
    let mut rng = DeterministicRng::new(config.seed);
    let dt = 1.0 / config.imu_rate_hz as f64;
    let total = config.duration_s.saturating_mul(config.imu_rate_hz);
    let mut sequence = 0_u64;
    let mut measurement_count = 0_u64;
    let mut truth_count = 0_u64;
    let mut doppler_count = 0_u64;
    let mut tracker_in_loop_count = 0_u64;
    let mut tracker_max_direct_error_hz: Option<f64> = None;
    let tracker_reference = BpskReference::pn(TRACKER_REFERENCE_LEN, config.seed ^ 0x5452_4143);
    let mut tracker = None;
    let mut turn_samples = 0_u64;
    let mut constant_heading_samples = 0_u64;
    let mut local_position = [0.0; 3];
    let mut disturbance_velocity = [0.0; 3];
    let mut previous_velocity = {
        let velocity = velocity_ne(0.0, config);
        [velocity[0], velocity[1], 0.0]
    };

    for tick in 0..=total {
        let seconds = tick as f64 * dt;
        let (heading, turn_rate) =
            heading_profile(seconds, config.duration_s as f64, config.coordinated_turn);
        if turn_rate.abs() > f64::EPSILON {
            turn_samples += 1;
        } else {
            constant_heading_samples += 1;
        }
        let commanded_velocity = velocity_ne(heading, config);
        let slam = wave_slam_acceleration(seconds, config.seed, config.wave_slam);
        if tick > 0 {
            for axis in 0..3 {
                disturbance_velocity[axis] += slam[axis] * dt;
            }
        }
        let velocity = [
            commanded_velocity[0] + disturbance_velocity[0],
            commanded_velocity[1] + disturbance_velocity[1],
            disturbance_velocity[2],
        ];
        if tick > 0 {
            for axis in 0..3 {
                local_position[axis] += 0.5 * (previous_velocity[axis] + velocity[axis]) * dt;
            }
        }
        let acceleration_local = [
            (velocity[0] - previous_velocity[0]) / dt,
            (velocity[1] - previous_velocity[1]) / dt,
            (velocity[2] - previous_velocity[2]) / dt,
        ];
        previous_velocity = velocity;
        let timestamp = tick.saturating_mul(1_000_000_000 / config.imu_rate_hz);
        let utc = start + Duration::nanoseconds(i64::try_from(timestamp).unwrap_or(i64::MAX));
        let truth_position =
            local_to_ecef_up(local_position[0], local_position[1], local_position[2]);
        let velocity_ecef = local_vector_to_ecef(velocity[0], velocity[1], velocity[2]);
        let acceleration_ecef = local_vector_to_ecef(
            acceleration_local[0],
            acceleration_local[1],
            acceleration_local[2],
        );
        let (imu_noise, imu_bias_scale) = scaled_imu(config);
        let imu = ImuSample {
            acceleration_mps2: std::array::from_fn(|axis| {
                acceleration_ecef[axis]
                    + config.imu_bias_mps2[axis] * imu_bias_scale
                    + imu_noise * rng.normal()
            }),
            angular_rate_rps: [0.0, 0.0, turn_rate],
        };
        journals.try_write_measurement(&envelope(
            sequence,
            timestamp,
            None,
            "imu",
            Frame::Sensor,
            vec![imu_noise.powi(2)],
            MeasurementPayload::Imu(imu),
        ))?;
        sequence += 1;
        measurement_count += 1;

        if tick % config.imu_rate_hz == 0 {
            let truth = envelope(
                sequence,
                timestamp,
                Some(utc),
                "gnss",
                Frame::EarthCenteredEarthFixed,
                vec![config.gnss_noise_std_m.powi(2)],
                MeasurementPayload::Gnss(GnssFix {
                    position_ecef_m: std::array::from_fn(|axis| {
                        truth_position[axis] + config.gnss_noise_std_m * rng.normal()
                    }),
                    velocity_ned_mps: [velocity[0], velocity[1], -velocity[2]],
                }),
            );
            journals.try_write_measurement(&truth)?;
            // The physically separate truth stream is noise-free.
            journals.try_write_truth(&envelope(
                sequence,
                timestamp,
                Some(utc),
                "truth",
                Frame::EarthCenteredEarthFixed,
                vec![0.0],
                MeasurementPayload::Gnss(GnssFix {
                    position_ecef_m: truth_position,
                    velocity_ned_mps: [velocity[0], velocity[1], -velocity[2]],
                }),
            ))?;
            sequence += 1;
            measurement_count += 1;
            truth_count += 1;

            for payload in [
                MeasurementPayload::Heading(Heading {
                    radians: heading + config.heading_noise_std_rad * rng.normal(),
                }),
                MeasurementPayload::SpeedThroughWater(SpeedThroughWater {
                    metres_per_second: config.speed_through_water_mps
                        + config.speed_noise_std_mps * rng.normal(),
                }),
            ] {
                journals.try_write_measurement(&envelope(
                    sequence,
                    timestamp,
                    None,
                    "marine-sensor",
                    Frame::VesselReference,
                    vec![0.01],
                    payload,
                ))?;
                sequence += 1;
                measurement_count += 1;
            }

            if tick % config.imu_rate_hz.saturating_mul(config.doppler_interval_s) != 0 {
                continue;
            }
            // Mission truth may span the graduated-aging study envelope. This propagation is
            // truth generation, while acceptance/inflation remains the executive's decision.
            let satellite = store
                .propagate_ecef_with_age(NORAD_ID, utc, Duration::hours(30))?
                .state;
            if let Ok(prediction) = predict(
                SatelliteState {
                    position_ecef_m: satellite.position_m,
                    velocity_ecef_mps: satellite.velocity_mps,
                },
                ReceiverState {
                    position_ecef_m: truth_position,
                    velocity_ecef_mps: velocity_ecef,
                    clock_drift_mps: 0.0,
                },
                0.0,
                CARRIER_HZ,
                config.elevation_mask_rad,
            ) {
                let predicted_hz = prediction.correlation_peak_hz;
                // Decimated endurance studies represent independent acquisition opportunities;
                // reacquire around the current prediction rather than carrying a 30-minute-old
                // narrow tracking window. The historical one-second path is unchanged.
                if config.doppler_interval_s > 1 {
                    tracker = None;
                }
                let receiver = match tracker.as_mut() {
                    Some(receiver) => receiver,
                    None => tracker.insert(
                        TrackerConfig {
                            sample_rate_hz: TRACKER_SAMPLE_RATE_HZ,
                            min_frequency_hz: predicted_hz - 512.0,
                            max_frequency_hz: predicted_hz + 512.0,
                            frequency_bin_hz: 32.0,
                            detection_threshold: TrackerConfig::DEFAULT_DETECTION_THRESHOLD,
                            tracking_half_width_hz: 128.0,
                        }
                        .build(tracker_reference.samples.clone())?,
                    ),
                };
                let mut synthesizer = Synthesizer::new(
                    SynthConfig {
                        sample_rate_hz: TRACKER_SAMPLE_RATE_HZ,
                        initial_offset_hz: predicted_hz,
                        offset_ramp_hz_per_s: 0.0,
                        delay_samples: 37,
                        cn0_db_hz: 90.0,
                        seed: config.seed ^ tick ^ 0x4951_5041,
                    },
                    tracker_reference.clone(),
                );
                if let TrackOutcome::Detection(detection) =
                    receiver.process_block(&synthesizer.next_block(), timestamp)
                {
                    let error_hz = (detection.correlation_peak_hz - predicted_hz).abs();
                    tracker_max_direct_error_hz = Some(
                        tracker_max_direct_error_hz.map_or(error_hz, |prior| prior.max(error_hz)),
                    );
                    journals.try_write_measurement(&detection.into_envelope(EnvelopeMetadata {
                        norad_catalog_id: &NORAD_ID.to_string(),
                        sequence,
                        host_receive_monotonic_ns: timestamp,
                        utc: UtcTime {
                            rfc3339: utc.to_rfc3339_opts(SecondsFormat::Nanos, true),
                            uncertainty_ns: 0,
                        },
                        constellation: Constellation::Iridium,
                        nominal_carrier_hz: CARRIER_HZ,
                        frequency_variance_hz2: TRACKER_TOLERANCE_HZ.powi(2),
                        calibration_id: "synthetic-cal-v1",
                        capture_record: "seeded-pnt-tracker-iq-pass",
                    }))?;
                    sequence += 1;
                    measurement_count += 1;
                    doppler_count += 1;
                    tracker_in_loop_count += 1;
                }
            }
        }
    }
    journals.finalize()?;
    Ok(MissionSummary {
        seed: config.seed,
        duration_s: config.duration_s,
        measurement_count,
        truth_count,
        doppler_count,
        constant_heading_samples,
        turn_samples,
        tracker_in_loop_count,
        tracker_max_direct_error_hz,
        synthetic_only: true,
    })
}

/// Generate a mission, run the existing paired replay API, and write `replay-report.json`.
///
/// # Errors
///
/// Returns a typed generation, replay, journal I/O, or JSON serialization error.
///
/// # Panics
///
/// Panics if a generated mission yields zero scored position or speed epochs — an internal
/// invariant of the generator (every mission emits truth-matched epochs), not a caller
/// condition.
pub fn run_study(
    output: impl AsRef<Path>,
    config: &MissionConfig,
) -> Result<StudyReport, MissionError> {
    let output = output.as_ref();
    let mission = generate_mission(output, config)?;
    let replay = replay_paired(output, 1)?;
    let receiver_prior = ReceiverPrior {
        position_ecef_m: local_to_ecef(0.0, 0.0),
        velocity_ecef_mps: local_vector_to_ecef(
            config.speed_through_water_mps + config.current_north_mps,
            config.current_east_mps,
            0.0,
        ),
        position_variance_m2: [1.0; 3],
        velocity_variance_mps2: [1.0; 3],
    };
    let doppler_config = ReplayDopplerConfig {
        ephemeris_tle: TLE.to_owned(),
        // The denied filter intentionally has no GNSS-derived geodetic initialization.
        // Disable elevation screening while retaining the same ephemeris and measurements.
        elevation_mask_degrees: None,
        chi_square_threshold: None,
        receiver_prior: Some(receiver_prior),
    };
    let doppler_replay = replay_paired_configured(output, 1, Some(&doppler_config))?;
    let mut prior_only_config = doppler_config.clone();
    prior_only_config.chi_square_threshold = Some(0.0);
    let prior_only_replay = replay_paired_configured(output, 1, Some(&prior_only_config))?;
    let mut no_prior_doppler_config = doppler_config.clone();
    no_prior_doppler_config.receiver_prior = None;
    let no_prior_doppler_replay =
        replay_paired_configured(output, 1, Some(&no_prior_doppler_config))?;
    let aided = replay.aided.horizontal_position_error_m.rms;
    let withheld = replay.withheld.horizontal_position_error_m.rms;
    let dr = &replay.withheld;
    let prior_only = &prior_only_replay.withheld;
    let prior_with_doppler = &doppler_replay.withheld;
    let contribution = |baseline: &RunSummary, treatment: &RunSummary| RmsContribution {
        position_rms_reduction_m: baseline
            .horizontal_position_error_m
            .rms
            .expect("generated mission has scored position epochs")
            - treatment
                .horizontal_position_error_m
                .rms
                .expect("generated mission has scored position epochs"),
        speed_rms_reduction_mps: baseline
            .horizontal_speed_error_mps
            .rms
            .expect("generated mission has scored speed epochs")
            - treatment
                .horizontal_speed_error_mps
                .rms
                .expect("generated mission has scored speed epochs"),
    };
    let prior_contribution = contribution(dr, prior_only);
    let doppler_given_prior_contribution = contribution(prior_only, prior_with_doppler);
    let report = StudyReport {
        caveat: "SYNTHETIC DEMONSTRATION ONLY — not a performance claim; real-signal behavior is [UNVERIFIED].".into(),
        four_way: FourWayTable {
            denied_dr_only: replay.withheld.clone(),
            denied_prior_only: prior_only_replay.withheld,
            denied_prior_with_doppler: doppler_replay.withheld,
            denied_no_prior_with_doppler: no_prior_doppler_replay.withheld,
        },
        attribution: StudyAttribution {
            disclosed_receiver_prior: receiver_prior,
            prior: prior_contribution,
            doppler_given_prior: doppler_given_prior_contribution,
        },
        qualitative_demonstration: QualitativeDemonstration {
            aided_smaller_than_withheld: aided.zip(withheld).is_some_and(|(a, w)| a < w),
            doppler_rich_constant_heading_present: mission.doppler_count > 0
                && mission.constant_heading_samples > 0,
            outage_or_turn_present: mission.turn_samples > 0,
        },
        integration_gaps: vec![],
        mission,
        replay,
    };
    std::fs::write(
        output.join("replay-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )
    .map_err(JournalError::Io)?;
    Ok(report)
}

fn heading_profile(
    seconds: f64,
    duration: f64,
    coordinated: Option<CoordinatedTurnConfig>,
) -> (f64, f64) {
    let turn_start = duration * 0.45;
    let turn_end = duration * 0.55;
    if seconds < turn_start {
        (0.0, 0.0)
    } else if seconds < turn_end {
        let legacy_rate = FRAC_PI_2 / (turn_end - turn_start).max(1.0);
        let rate = coordinated.map_or(legacy_rate, |value| value.rate_rad_s);
        let heading = rate * (seconds - turn_start);
        (heading, rate)
    } else {
        let legacy_rate = FRAC_PI_2 / (turn_end - turn_start).max(1.0);
        let rate = coordinated.map_or(legacy_rate, |value| value.rate_rad_s);
        (
            coordinated.map_or(FRAC_PI_2, |_| rate * (turn_end - turn_start)),
            0.0,
        )
    }
}

fn validate_config(config: &MissionConfig) -> Result<(), MissionError> {
    if !(0.0..=15.5).contains(&config.speed_through_water_mps) {
        return Err(MissionError::InvalidConfig(
            "speed_through_water_mps must be in 0..=15.5".into(),
        ));
    }
    if config.imu_rate_hz == 0 {
        return Err(MissionError::InvalidConfig(
            "imu_rate_hz must be non-zero".into(),
        ));
    }
    if config.doppler_interval_s == 0 {
        return Err(MissionError::InvalidConfig(
            "doppler_interval_s must be non-zero".into(),
        ));
    }
    if let Some(turn) = config.coordinated_turn {
        if !turn.rate_rad_s.is_finite() || turn.rate_rad_s == 0.0 {
            return Err(MissionError::InvalidConfig(
                "coordinated turn rate must be finite and non-zero".into(),
            ));
        }
    }
    if let Some(wave) = config.wave_slam {
        if !wave.burst_rate_hz.is_finite()
            || wave.burst_rate_hz < 0.0
            || !wave.duration_s.is_finite()
            || wave.duration_s <= 0.0
            || !wave.vertical_peak_mps2.is_finite()
            || wave.vertical_peak_mps2 < 0.0
            || !wave.pitch_coupling.is_finite()
        {
            return Err(MissionError::InvalidConfig(
                "wave/slam values must be finite, with non-negative rate/peak and positive duration"
                    .into(),
            ));
        }
    }
    if let Some(scaling) = config.speed_scaled_imu {
        if !scaling.reference_speed_mps.is_finite()
            || scaling.reference_speed_mps <= 0.0
            || !scaling.noise_per_speed_ratio.is_finite()
            || !scaling.bias_per_speed_ratio.is_finite()
        {
            return Err(MissionError::InvalidConfig(
                "speed-scaled IMU values must be finite and reference speed must be positive"
                    .into(),
            ));
        }
    }
    Ok(())
}

fn scaled_imu(config: &MissionConfig) -> (f64, f64) {
    config
        .speed_scaled_imu
        .map_or((config.imu_noise_std_mps2, 1.0), |scaling| {
            let ratio = config.speed_through_water_mps / scaling.reference_speed_mps - 1.0;
            (
                config.imu_noise_std_mps2 * (1.0 + scaling.noise_per_speed_ratio * ratio).max(0.0),
                (1.0 + scaling.bias_per_speed_ratio * ratio).max(0.0),
            )
        })
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn wave_slam_acceleration(seconds: f64, seed: u64, config: Option<WaveSlamConfig>) -> [f64; 3] {
    let Some(config) = config else {
        return [0.0; 3];
    };
    // One deterministic Bernoulli opportunity per burst duration. Hashing the interval makes
    // the result independent of sample rate and avoids perturbing the legacy RNG stream.
    let interval = (seconds / config.duration_s).floor() as u64;
    let phase = seconds.rem_euclid(config.duration_s) / config.duration_s;
    let mut hash = interval ^ seed ^ 0x5741_5645_534c_414d;
    hash ^= hash >> 30;
    hash = hash.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    hash ^= hash >> 27;
    hash = hash.wrapping_mul(0x94d0_49bb_1331_11eb);
    hash ^= hash >> 31;
    let unit = hash as f64 / u64::MAX as f64;
    let probability = (config.burst_rate_hz * config.duration_s).clamp(0.0, 1.0);
    if unit >= probability {
        return [0.0; 3];
    }
    let vertical = config.vertical_peak_mps2 * (std::f64::consts::TAU * phase).cos();
    [vertical * config.pitch_coupling, 0.0, vertical]
}

fn velocity_ne(heading: f64, config: &MissionConfig) -> [f64; 2] {
    [
        config.speed_through_water_mps * heading.cos() + config.current_north_mps,
        config.speed_through_water_mps * heading.sin() + config.current_east_mps,
    ]
}

// The fixture origin is the equator/prime meridian: north=ECEF z, east=ECEF y. The small
// local displacement is projected back to the spherical sea surface, keeping altitude zero.
fn local_to_ecef(north: f64, east: f64) -> [f64; 3] {
    local_to_ecef_up(north, east, 0.0)
}

fn local_to_ecef_up(north: f64, east: f64, up: f64) -> [f64; 3] {
    let latitude = north / EARTH_RADIUS_M;
    let longitude = east / EARTH_RADIUS_M;
    let radius = EARTH_RADIUS_M + up;
    [
        radius * latitude.cos() * longitude.cos(),
        radius * latitude.cos() * longitude.sin(),
        radius * latitude.sin(),
    ]
}

fn local_vector_to_ecef(north: f64, east: f64, up: f64) -> [f64; 3] {
    [up, east, north]
}

fn envelope(
    sequence: u64,
    timestamp: u64,
    utc: Option<DateTime<Utc>>,
    source: &str,
    frame: Frame,
    covariance: Vec<f64>,
    payload: MeasurementPayload,
) -> MeasurementEnvelope {
    MeasurementEnvelope {
        schema_version: SCHEMA_VERSION,
        source_id: SourceId(source.into()),
        sequence,
        sample_time: TimeTag::HostMonotonicNanoseconds(timestamp),
        host_receive_monotonic_ns: timestamp,
        utc: utc.map(|value| UtcTime {
            rfc3339: value.to_rfc3339_opts(SecondsFormat::Nanos, true),
            uncertainty_ns: 0,
        }),
        payload,
        frame,
        covariance,
        quality: QualityFlags::VALID,
        calibration_id: "synthetic-cal-v1".into(),
        provenance: Provenance::DerivedRecord(format!("mission:{sequence}")),
    }
}

fn find_visible_start(store: &EphemerisStore, epoch: DateTime<Utc>) -> DateTime<Utc> {
    let receiver = ReceiverState {
        position_ecef_m: local_to_ecef(0.0, 0.0),
        velocity_ecef_mps: [0.0; 3],
        clock_drift_mps: 0.0,
    };
    (-21_600..=21_600)
        .step_by(10)
        .find_map(|seconds| {
            let time = epoch + Duration::seconds(seconds);
            let satellite = store.propagate_ecef(NORAD_ID, time).ok()?;
            predict(
                SatelliteState {
                    position_ecef_m: satellite.position_m,
                    velocity_ecef_mps: satellite.velocity_mps,
                },
                receiver,
                0.0,
                CARRIER_HZ,
                10.0_f64.to_radians(),
            )
            .ok()
            .map(|_| time)
        })
        .unwrap_or(epoch)
}

fn config_fingerprint(config: &MissionConfig) -> String {
    format!(
        "synthetic-v1:{:016x}:{}:{}:{:.6}:{:.6}:{:.6}",
        config.seed,
        config.duration_s,
        config.imu_rate_hz,
        config.speed_through_water_mps,
        config.current_north_mps,
        config.current_east_mps
    )
}

struct DeterministicRng(u64);

impl DeterministicRng {
    const fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15)
    }

    fn uniform(&mut self) -> f64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        (value as f64 + 1.0) / (u64::MAX as f64 + 2.0)
    }

    fn normal(&mut self) -> f64 {
        (-2.0 * self.uniform().ln()).sqrt() * (std::f64::consts::TAU * self.uniform()).cos()
    }
}

/// Read the finalized manifest, useful for capture round-trip checks.
///
/// # Errors
///
/// Returns an I/O or JSON decoding error when the manifest cannot be read.
pub fn read_manifest(path: impl AsRef<Path>) -> Result<RunManifest, MissionError> {
    Ok(serde_json::from_reader(
        std::fs::File::open(path.as_ref().join("manifest.json")).map_err(JournalError::Io)?,
    )?)
}

#[cfg(test)]
mod high_speed_tests {
    use super::*;

    #[test]
    fn slam_is_seeded_bounded_zero_mean_and_pitch_coupled() {
        let config = WaveSlamConfig {
            burst_rate_hz: 10.0,
            duration_s: 1.0,
            vertical_peak_mps2: 4.0,
            pitch_coupling: 0.2,
        };
        let first = wave_slam_acceleration(0.5, 4, Some(config));
        let second = wave_slam_acceleration(0.5, 4, Some(config));
        assert_eq!(first.map(f64::to_bits), second.map(f64::to_bits));
        assert!(first[2].abs() <= 4.0);
        assert!((first[0] - first[2] * 0.2).abs() < f64::EPSILON);
        let mean = (0..1_000)
            .map(|sample| wave_slam_acceleration(f64::from(sample) / 1_000.0, 4, Some(config))[2])
            .sum::<f64>()
            / 1_000.0;
        assert!(mean.abs() < 1.0e-12);
    }

    #[test]
    fn envelope_accepts_exploratory_thirty_knots_and_rejects_more() {
        let mut config = MissionConfig {
            speed_through_water_mps: 15.43,
            ..MissionConfig::default()
        };
        assert!(validate_config(&config).is_ok());
        config.speed_through_water_mps = 15.51;
        assert!(matches!(
            validate_config(&config),
            Err(MissionError::InvalidConfig(_))
        ));
    }

    #[test]
    fn speed_scaled_imu_changes_noise_and_bias_at_high_speed() {
        let scaling = SpeedScaledImuConfig {
            reference_speed_mps: 5.0,
            noise_per_speed_ratio: 0.5,
            bias_per_speed_ratio: 0.25,
        };
        let reference = MissionConfig {
            speed_through_water_mps: 5.0,
            speed_scaled_imu: Some(scaling),
            ..MissionConfig::default()
        };
        let faster = MissionConfig {
            speed_through_water_mps: 10.0,
            ..reference.clone()
        };
        assert_eq!(scaled_imu(&reference), (reference.imu_noise_std_mps2, 1.0));
        assert_eq!(
            scaled_imu(&faster),
            (reference.imu_noise_std_mps2 * 1.5, 1.25)
        );
    }

    #[test]
    fn integrated_slam_acceleration_recovers_disturbance_velocity() {
        let config = WaveSlamConfig {
            burst_rate_hz: 10.0,
            duration_s: 1.0,
            vertical_peak_mps2: 4.0,
            pitch_coupling: 0.2,
        };
        let dt = 0.001;
        let mut velocity = [0.0; 3];
        for sample in 0..1_000 {
            let acceleration = wave_slam_acceleration(f64::from(sample) * dt, 4, Some(config));
            for axis in 0..3 {
                velocity[axis] += acceleration[axis] * dt;
            }
        }
        assert!(velocity.iter().all(|value| value.abs() < 1.0e-12));
    }

    #[test]
    fn configured_turn_rate_controls_centripetal_acceleration() {
        let config = MissionConfig {
            speed_through_water_mps: 10.0,
            coordinated_turn: Some(CoordinatedTurnConfig { rate_rad_s: 0.1 }),
            ..MissionConfig::default()
        };
        let (_, rate) = heading_profile(46.0, 100.0, config.coordinated_turn);
        assert!((rate - 0.1).abs() < f64::EPSILON);
        assert!((config.speed_through_water_mps * rate - 1.0).abs() < f64::EPSILON);
    }
}
