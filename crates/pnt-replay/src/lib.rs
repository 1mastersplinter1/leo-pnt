//! Deterministic aided/withheld replay and truth-referenced scoring.

use fusion_executive::{Executive, RoutingDestination};
use pnt_config::{Config, GnssAuthority};
use pnt_estimator::FilterStub;
use pnt_integrity::{AuthorityParams, AuthoritySupervisor};
use pnt_journal::{
    IntegrityEvent, JournalError, MeasurementJournalRecord, MeasurementReader, MemoryJournals,
    RunManifest, TruthJournalRecord, TruthReader,
};
use pnt_time::ClockService;
use pnt_types::{
    ecef_vector_to_enu, GnssFix, MeasurementEnvelope, MeasurementPayload, SolutionEpoch,
};
use serde::{Deserialize, Serialize};
use std::{fmt, fs::File, io, path::Path};

const REPORT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug)]
pub enum ReplayError {
    Io(io::Error),
    Journal(JournalError),
    Manifest(String),
    UnsupportedMode,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ReplayError {}

impl From<io::Error> for ReplayError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JournalError> for ReplayError {
    fn from(value: JournalError) -> Self {
        Self::Journal(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayRun {
    pub mode: GnssAuthority,
    pub epochs: Vec<SolutionEpoch>,
    pub integrity_events: Vec<IntegrityEvent>,
    pub input_measurement_count: u64,
    pub fusion_routes: u64,
    pub gnss_fusion_routes: u64,
    pub gnss_truth_routes: u64,
    pub measurement_updates: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub n: u64,
    pub mean: Option<f64>,
    pub rms: Option<f64>,
    pub p50: Option<f64>,
    pub p95: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub mode: String,
    pub fusion_routes: u64,
    pub gnss_fusion_routes: u64,
    pub gnss_truth_routes: u64,
    pub measurement_updates: u64,
    pub matched_epochs: u64,
    pub excluded_no_near_truth: u64,
    pub horizontal_position_error_m: ErrorStatistics,
    pub horizontal_speed_error_mps: ErrorStatistics,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparisonSummary {
    pub horizontal_position_error_m: ErrorStatistics,
    pub horizontal_speed_error_mps: ErrorStatistics,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayReport {
    pub schema_version: u16,
    pub run_uuid: String,
    pub config_hash: String,
    pub max_truth_offset_ns: u64,
    pub input_measurement_count: u64,
    pub aided: RunSummary,
    pub withheld: RunSummary,
    /// Per-matched-epoch aided error minus withheld error.
    pub comparison: ComparisonSummary,
}

struct RecordedClock {
    timestamps: std::vec::IntoIter<u64>,
}

impl RecordedClock {
    fn new(measurements: &[MeasurementEnvelope]) -> Self {
        Self {
            timestamps: measurements
                .iter()
                .map(|value| value.host_receive_monotonic_ns)
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}

impl ClockService for RecordedClock {
    fn ingress_monotonic_ns(&mut self) -> u64 {
        self.timestamps.next().unwrap_or(0)
    }
}

/// Replays one already-loaded immutable measurement stream through a fresh executive.
fn replay_loaded(
    measurements: &[MeasurementEnvelope],
    mode: GnssAuthority,
) -> Result<ReplayRun, ReplayError> {
    if mode == GnssAuthority::Off {
        return Err(ReplayError::UnsupportedMode);
    }
    let clock = RecordedClock::new(measurements);
    let mut executive = Executive::new(
        Config {
            gnss_authority: mode,
            oneweb_enabled: false,
        },
        clock,
        FilterStub::default(),
        AuthoritySupervisor::fail_closed(AuthorityParams::default()),
        MemoryJournals::default(),
    );
    let mut fusion_routes = 0;
    let mut gnss_fusion_routes = 0;
    let mut gnss_truth_routes = 0;
    for measurement in measurements.iter().cloned() {
        let is_gnss = matches!(measurement.payload, MeasurementPayload::Gnss(_));
        let routes = executive.process(measurement);
        let fusion = routes
            .iter()
            .filter(|route| **route == RoutingDestination::Fusion)
            .count() as u64;
        fusion_routes += fusion;
        if is_gnss {
            gnss_fusion_routes += fusion;
            gnss_truth_routes += routes
                .iter()
                .filter(|route| **route == RoutingDestination::TruthJournal)
                .count() as u64;
        }
    }
    let epochs = executive.take_solution_epochs();
    let measurement_updates = executive.filter().measurement_updates();
    let integrity_events = executive.journals().integrity_events().to_vec();
    Ok(ReplayRun {
        mode,
        epochs,
        integrity_events,
        input_measurement_count: measurements.len() as u64,
        fusion_routes,
        gnss_fusion_routes,
        gnss_truth_routes,
        measurement_updates,
    })
}

/// Opens a run and replays its measurement stream once in the chosen aided/withheld mode.
///
/// # Errors
///
/// Returns an error if the journal cannot be opened or decoded, or if `off` is requested.
pub fn replay_directory(
    path: impl AsRef<Path>,
    mode: GnssAuthority,
) -> Result<ReplayRun, ReplayError> {
    let measurements = read_measurements(path.as_ref())?;
    replay_loaded(&measurements, mode)
}

/// Reads the input once, replays exact clones in both modes, and scores both against truth.
///
/// # Errors
///
/// Returns an error if the manifest or either journal stream cannot be opened or decoded.
pub fn replay_paired(
    path: impl AsRef<Path>,
    max_truth_offset_ns: u64,
) -> Result<ReplayReport, ReplayError> {
    let path = path.as_ref();
    let manifest: RunManifest = serde_json::from_reader(File::open(path.join("manifest.json"))?)
        .map_err(|error| ReplayError::Manifest(error.to_string()))?;
    let measurements = read_measurements(path)?;
    let truth = read_truth(path)?;
    // Both executions receive clones from this single immutable vector. No mode-dependent
    // file read or preprocessing can change their raw input stream.
    let aided = replay_loaded(&measurements, GnssAuthority::Production)?;
    let withheld = replay_loaded(&measurements, GnssAuthority::RecordedOnly)?;
    let aided_errors = epoch_errors(&aided.epochs, &truth, max_truth_offset_ns);
    let withheld_errors = epoch_errors(&withheld.epochs, &truth, max_truth_offset_ns);
    let comparison =
        comparison_errors(&aided.epochs, &withheld.epochs, &truth, max_truth_offset_ns);
    Ok(ReplayReport {
        schema_version: REPORT_SCHEMA_VERSION,
        run_uuid: manifest.run_uuid,
        config_hash: manifest.config_hash,
        max_truth_offset_ns,
        input_measurement_count: measurements.len() as u64,
        aided: summarize_run(&aided, &aided_errors),
        withheld: summarize_run(&withheld, &withheld_errors),
        comparison: ComparisonSummary {
            horizontal_position_error_m: statistics(comparison.0),
            horizontal_speed_error_mps: statistics(comparison.1),
        },
    })
}

fn read_measurements(path: &Path) -> Result<Vec<MeasurementEnvelope>, ReplayError> {
    MeasurementReader::open(path)?
        .filter_map(|record| match record {
            Ok(MeasurementJournalRecord::Envelope(value)) => Some(Ok(value)),
            Ok(MeasurementJournalRecord::Integrity(_) | MeasurementJournalRecord::Epoch(_)) => None,
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}

fn read_truth(path: &Path) -> Result<Vec<MeasurementEnvelope>, ReplayError> {
    TruthReader::open(path)?
        .filter_map(|record| match record {
            Ok(TruthJournalRecord::Envelope(value)) => Some(Ok(value)),
            Ok(TruthJournalRecord::Epoch(_)) => None,
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}

struct Errors {
    position: Vec<f64>,
    speed: Vec<f64>,
    excluded: u64,
}

fn epoch_errors(
    epochs: &[SolutionEpoch],
    truth: &[MeasurementEnvelope],
    max_offset: u64,
) -> Errors {
    let mut errors = Errors {
        position: Vec::new(),
        speed: Vec::new(),
        excluded: 0,
    };
    for epoch in epochs {
        if let Some(fix) = nearest_truth(epoch.monotonic_ns, truth, max_offset) {
            let ecef_error = std::array::from_fn(|index| {
                epoch.state.position_ecef_m[index] - fix.position_ecef_m[index]
            });
            let enu_error = ecef_vector_to_enu(fix.position_ecef_m, ecef_error);
            errors.position.push(enu_error[0].hypot(enu_error[1]));
            errors.speed.push(
                (epoch.state.horizontal_velocity_ned_mps[0] - fix.velocity_ned_mps[0])
                    .hypot(epoch.state.horizontal_velocity_ned_mps[1] - fix.velocity_ned_mps[1]),
            );
        } else {
            errors.excluded += 1;
        }
    }
    errors
}

fn comparison_errors(
    aided: &[SolutionEpoch],
    withheld: &[SolutionEpoch],
    truth: &[MeasurementEnvelope],
    max_offset: u64,
) -> (Vec<f64>, Vec<f64>) {
    let mut position = Vec::new();
    let mut speed = Vec::new();
    for left in aided {
        let Some(right) = withheld
            .iter()
            .find(|epoch| epoch.monotonic_ns == left.monotonic_ns)
        else {
            continue;
        };
        let Some(fix) = nearest_truth(left.monotonic_ns, truth, max_offset) else {
            continue;
        };
        let score = |epoch: &SolutionEpoch| {
            let delta = std::array::from_fn(|index| {
                epoch.state.position_ecef_m[index] - fix.position_ecef_m[index]
            });
            let enu = ecef_vector_to_enu(fix.position_ecef_m, delta);
            let pos = enu[0].hypot(enu[1]);
            let vel = (epoch.state.horizontal_velocity_ned_mps[0] - fix.velocity_ned_mps[0])
                .hypot(epoch.state.horizontal_velocity_ned_mps[1] - fix.velocity_ned_mps[1]);
            (pos, vel)
        };
        let (left_pos, left_vel) = score(left);
        let (right_pos, right_vel) = score(right);
        position.push(left_pos - right_pos);
        speed.push(left_vel - right_vel);
    }
    (position, speed)
}

fn nearest_truth(
    timestamp: u64,
    truth: &[MeasurementEnvelope],
    max_offset: u64,
) -> Option<GnssFix> {
    truth
        .iter()
        .filter_map(|value| match value.payload {
            MeasurementPayload::Gnss(fix) => Some((value.host_receive_monotonic_ns, fix)),
            _ => None,
        })
        .min_by_key(|(truth_time, _)| (timestamp.abs_diff(*truth_time), *truth_time))
        .filter(|(truth_time, _)| timestamp.abs_diff(*truth_time) <= max_offset)
        .map(|(_, fix)| fix)
}

fn summarize_run(run: &ReplayRun, errors: &Errors) -> RunSummary {
    RunSummary {
        mode: mode_name(run.mode).to_owned(),
        fusion_routes: run.fusion_routes,
        gnss_fusion_routes: run.gnss_fusion_routes,
        gnss_truth_routes: run.gnss_truth_routes,
        measurement_updates: run.measurement_updates,
        matched_epochs: errors.position.len() as u64,
        excluded_no_near_truth: errors.excluded,
        horizontal_position_error_m: statistics(errors.position.clone()),
        horizontal_speed_error_mps: statistics(errors.speed.clone()),
    }
}

const fn mode_name(mode: GnssAuthority) -> &'static str {
    match mode {
        GnssAuthority::Production => "production",
        GnssAuthority::RecordedOnly => "recorded_only",
        GnssAuthority::Off => "off",
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn statistics(mut values: Vec<f64>) -> ErrorStatistics {
    if values.is_empty() {
        return ErrorStatistics {
            n: 0,
            mean: None,
            rms: None,
            p50: None,
            p95: None,
            max: None,
        };
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let rms = (values.iter().map(|value| value * value).sum::<f64>() / n).sqrt();
    values.sort_by(f64::total_cmp);
    let percentile = |p: f64| {
        let index = p * (values.len() - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;
        values[lower] + (values[upper] - values[lower]) * index.fract()
    };
    ErrorStatistics {
        n: values.len() as u64,
        mean: Some(mean),
        rms: Some(rms),
        p50: Some(percentile(0.5)),
        p95: Some(percentile(0.95)),
        max: values.last().copied(),
    }
}

#[cfg(test)]
mod tests;
