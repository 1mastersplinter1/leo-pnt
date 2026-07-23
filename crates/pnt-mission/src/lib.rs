//! Deterministic synthetic maritime mission generation and paired replay study.
#![allow(clippy::cast_precision_loss, clippy::too_many_lines)]

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use pnt_ephemeris::{EphemerisError, EphemerisStore};
use pnt_journal::{FileJournals, JournalError, RunManifest, RunMetadata};
use pnt_predictor::{predict, ReceiverState, SatelliteState};
use pnt_replay::{replay_paired, ReplayError, ReplayReport};
use pnt_types::{
    Constellation, Frame, GnssFix, Heading, ImuSample, MeasurementEnvelope, MeasurementPayload,
    Provenance, QualityFlags, SourceId, SpeedThroughWater, TimeTag, TrackerDoppler, UtcTime,
    SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::{f64::consts::FRAC_PI_2, path::Path};

const EARTH_RADIUS_M: f64 = 6_378_137.0;
const NORAD_ID: u64 = 25_544;
const CARRIER_HZ: f64 = 1_600_000_000.0;
const TLE: &str = include_str!("../../pnt-ephemeris/tests/fixtures/iss.tle");

#[derive(Debug, thiserror::Error)]
pub enum MissionError {
    #[error(transparent)]
    Journal(#[from] JournalError),
    #[error(transparent)]
    Ephemeris(#[from] EphemerisError),
    #[error(transparent)]
    Replay(#[from] ReplayError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
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
    pub synthetic_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StudyReport {
    pub caveat: String,
    pub mission: MissionSummary,
    pub replay: ReplayReport,
    pub qualitative_demonstration: QualitativeDemonstration,
    pub integration_gaps: Vec<String>,
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
    let output = output.as_ref();
    let store = EphemerisStore::from_tle_str(TLE)?;
    let epoch = store.epoch(NORAD_ID).expect("fixture satellite exists");
    let start = find_visible_start(&store, epoch);
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
    let mut turn_samples = 0_u64;
    let mut constant_heading_samples = 0_u64;
    let mut north = 0.0;
    let mut east = 0.0;
    let mut previous_velocity = velocity_ne(0.0, config);

    for tick in 0..=total {
        let seconds = tick as f64 * dt;
        let (heading, turn_rate) = heading_profile(seconds, config.duration_s as f64);
        if turn_rate.abs() > f64::EPSILON {
            turn_samples += 1;
        } else {
            constant_heading_samples += 1;
        }
        let velocity = velocity_ne(heading, config);
        if tick > 0 {
            north += 0.5 * (previous_velocity[0] + velocity[0]) * dt;
            east += 0.5 * (previous_velocity[1] + velocity[1]) * dt;
        }
        let acceleration_ne = [
            (velocity[0] - previous_velocity[0]) / dt,
            (velocity[1] - previous_velocity[1]) / dt,
        ];
        previous_velocity = velocity;
        let timestamp = tick.saturating_mul(1_000_000_000 / config.imu_rate_hz);
        let utc = start + Duration::nanoseconds(i64::try_from(timestamp).unwrap_or(i64::MAX));
        let truth_position = local_to_ecef(north, east);
        let velocity_ecef = local_vector_to_ecef(velocity[0], velocity[1]);
        let acceleration_ecef = local_vector_to_ecef(acceleration_ne[0], acceleration_ne[1]);
        let imu = ImuSample {
            acceleration_mps2: std::array::from_fn(|axis| {
                acceleration_ecef[axis]
                    + config.imu_bias_mps2[axis]
                    + config.imu_noise_std_mps2 * rng.normal()
            }),
            angular_rate_rps: [0.0, 0.0, turn_rate],
        };
        journals.try_write_measurement(&envelope(
            sequence,
            timestamp,
            None,
            "imu",
            Frame::Sensor,
            vec![config.imu_noise_std_mps2.powi(2)],
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
                    velocity_ned_mps: [velocity[0], velocity[1], 0.0],
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
                    velocity_ned_mps: [velocity[0], velocity[1], 0.0],
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

            let satellite = store.propagate_ecef(NORAD_ID, utc)?;
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
                journals.try_write_measurement(&envelope(
                    sequence,
                    timestamp,
                    Some(utc),
                    &NORAD_ID.to_string(),
                    Frame::AntennaPhaseCenter,
                    vec![config.doppler_noise_std_hz.powi(2)],
                    MeasurementPayload::TrackerDoppler(TrackerDoppler {
                        constellation: Constellation::Iridium,
                        correlation_peak_hz: prediction.correlation_peak_hz
                            + config.doppler_noise_std_hz * rng.normal(),
                        nominal_carrier_hz: CARRIER_HZ,
                    }),
                ))?;
                sequence += 1;
                measurement_count += 1;
                doppler_count += 1;
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
        tracker_in_loop_count: 0,
        synthetic_only: true,
    })
}

/// Generate a mission, run the existing paired replay API, and write `replay-report.json`.
///
/// # Errors
///
/// Returns a typed generation, replay, journal I/O, or JSON serialization error.
pub fn run_study(
    output: impl AsRef<Path>,
    config: &MissionConfig,
) -> Result<StudyReport, MissionError> {
    let output = output.as_ref();
    let mission = generate_mission(output, config)?;
    let replay = replay_paired(output, 1)?;
    let aided = replay.aided.horizontal_position_error_m.rms;
    let withheld = replay.withheld.horizontal_position_error_m.rms;
    let report = StudyReport {
        caveat: "SYNTHETIC DEMONSTRATION ONLY — not a performance claim; real-signal behavior is [UNVERIFIED].".into(),
        qualitative_demonstration: QualitativeDemonstration {
            aided_smaller_than_withheld: aided.zip(withheld).is_some_and(|(a, w)| a < w),
            doppler_rich_constant_heading_present: mission.doppler_count > 0
                && mission.constant_heading_samples > 0,
            outage_or_turn_present: mission.turn_samples > 0,
        },
        integration_gaps: vec![
            "pnt-tracker is absent from this checkout; tracker-in-loop count is zero".into(),
            "pnt-replay does not attach an EphemerisStore/DopplerPipeline, so recorded Doppler is rejected during paired replay".into(),
            "pnt-replay exposes no comparison-pair exclusion count (D35 integration field)".into(),
        ],
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

fn heading_profile(seconds: f64, duration: f64) -> (f64, f64) {
    let turn_start = duration * 0.45;
    let turn_end = duration * 0.55;
    if seconds < turn_start {
        (0.0, 0.0)
    } else if seconds < turn_end {
        let rate = FRAC_PI_2 / (turn_end - turn_start).max(1.0);
        (rate * (seconds - turn_start), rate)
    } else {
        (FRAC_PI_2, 0.0)
    }
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
    let latitude = north / EARTH_RADIUS_M;
    let longitude = east / EARTH_RADIUS_M;
    [
        EARTH_RADIUS_M * latitude.cos() * longitude.cos(),
        EARTH_RADIUS_M * latitude.cos() * longitude.sin(),
        EARTH_RADIUS_M * latitude.sin(),
    ]
}

fn local_vector_to_ecef(north: f64, east: f64) -> [f64; 3] {
    [0.0, east, north]
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
