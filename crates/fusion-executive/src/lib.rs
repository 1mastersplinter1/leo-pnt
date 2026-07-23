//! The sole runtime orchestrator.

use chrono::{DateTime, Utc};
use pnt_config::{Config, GnssAuthority};
use pnt_ephemeris::EphemerisStore;
use pnt_estimator::{DopplerRangeRateUpdate, Estimator, FilterStub, UpdateResult};
use pnt_integrity::{IntegrityAuthorityGate, IntegrityStub};
use pnt_journal::{IntegrityEvent, JournalSinks, MemoryJournals};
use pnt_predictor::{geometric_range_rate_linearisation, predict, ReceiverState, SatelliteState};
use pnt_time::{ClockService, ManualClock};
use pnt_types::{Constellation, MeasurementEnvelope, MeasurementPayload, SolutionEpoch};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingDestination {
    Fusion,
    TruthJournal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingTable {
    pub gnss: Vec<RoutingDestination>,
    pub non_gnss: Vec<RoutingDestination>,
}

pub struct Executive<C, E, I, J> {
    config: Config,
    clock: C,
    filter: E,
    integrity: I,
    journals: J,
    solution_epochs: Vec<SolutionEpoch>,
    solution_lines: Vec<String>,
    doppler: Option<DopplerPipeline>,
}

pub struct DopplerPipeline {
    store: EphemerisStore,
    elevation_mask_rad: Option<f64>,
    pub chi_square_threshold: Option<f64>,
}

impl DopplerPipeline {
    #[must_use]
    pub fn new(store: EphemerisStore) -> Self {
        Self {
            store,
            elevation_mask_rad: Some(5.0_f64.to_radians()),
            chi_square_threshold: Some(9.0),
        }
    }

    /// Explicitly disables elevation screening, primarily for geometry-independent tests.
    #[must_use]
    pub fn without_elevation_mask(mut self) -> Self {
        self.elevation_mask_rad = None;
        self
    }

    #[must_use]
    /// Configures the screening angle at a degrees-safe API boundary.
    ///
    /// # Panics
    ///
    /// Panics when `degrees` is non-finite or outside the physical elevation range
    /// `-90..=90`.
    pub fn with_elevation_mask_degrees(mut self, degrees: f64) -> Self {
        assert!(degrees.is_finite() && (-90.0..=90.0).contains(&degrees));
        self.elevation_mask_rad = Some(degrees.to_radians());
        self
    }
}

impl<C, E, I, J> Executive<C, E, I, J>
where
    C: ClockService,
    E: Estimator,
    I: IntegrityAuthorityGate,
    J: JournalSinks,
{
    pub fn new(config: Config, clock: C, filter: E, integrity: I, journals: J) -> Self {
        Self {
            config,
            clock,
            filter,
            integrity,
            journals,
            solution_epochs: Vec::new(),
            solution_lines: Vec::new(),
            doppler: None,
        }
    }

    #[must_use]
    pub fn with_doppler_pipeline(mut self, pipeline: DopplerPipeline) -> Self {
        self.doppler = Some(pipeline);
        self
    }

    #[must_use]
    pub fn routing_table(authority: GnssAuthority) -> RoutingTable {
        let gnss = match authority {
            GnssAuthority::Production => {
                vec![RoutingDestination::Fusion, RoutingDestination::TruthJournal]
            }
            GnssAuthority::RecordedOnly => vec![RoutingDestination::TruthJournal],
            GnssAuthority::Off => Vec::new(),
        };
        RoutingTable {
            gnss,
            non_gnss: vec![RoutingDestination::Fusion],
        }
    }

    pub fn process(&mut self, mut envelope: MeasurementEnvelope) -> Vec<RoutingDestination> {
        envelope.host_receive_monotonic_ns = self.clock.ingress_monotonic_ns();
        let routes = self.routes_for(&envelope.payload);
        if routes.is_empty() {
            self.journals.write_measurement(&envelope);
            self.reject(&envelope, Self::rejection_reason(&envelope.payload));
        }
        for route in &routes {
            match route {
                RoutingDestination::Fusion => self.dispatch_to_fusion(&envelope),
                RoutingDestination::TruthJournal => self.journals.write_truth(&envelope),
            }
        }
        routes
    }

    fn routes_for(&self, payload: &MeasurementPayload) -> Vec<RoutingDestination> {
        match payload {
            MeasurementPayload::Gnss(_) => Self::routing_table(self.config.gnss_authority).gnss,
            MeasurementPayload::TrackerDoppler(observation)
                if observation.constellation == Constellation::Orbcomm =>
            {
                Vec::new()
            }
            MeasurementPayload::TrackerDoppler(observation)
                if observation.constellation == Constellation::OneWeb
                    && !self.config.oneweb_enabled =>
            {
                Vec::new()
            }
            MeasurementPayload::ArmCommand(_) => vec![RoutingDestination::Fusion],
            _ => Self::routing_table(self.config.gnss_authority).non_gnss,
        }
    }

    fn dispatch_to_fusion(&mut self, envelope: &MeasurementEnvelope) {
        self.journals.write_measurement(envelope);
        if let MeasurementPayload::ArmCommand(command) = &envelope.payload {
            self.integrity.arm_command(command);
            return;
        }
        if let MeasurementPayload::Imu(imu) = &envelope.payload {
            self.filter.propagate(*imu);
            return;
        }
        if matches!(envelope.payload, MeasurementPayload::TrackerDoppler(_)) {
            self.process_doppler(envelope);
            return;
        }
        self.filter.update(envelope);
        self.emit_epoch(envelope.host_receive_monotonic_ns, &envelope.source_id.0);
    }

    fn process_doppler(&mut self, envelope: &MeasurementEnvelope) {
        let MeasurementPayload::TrackerDoppler(observation) = envelope.payload else {
            return;
        };
        let result = (|| {
            let pipeline = self
                .doppler
                .as_ref()
                .ok_or("Doppler pipeline unavailable".to_owned())?;
            let norad_id = envelope
                .source_id
                .0
                .parse::<u64>()
                .map_err(|_| "source_id is not a NORAD id".to_owned())?;
            let utc = envelope
                .utc
                .as_ref()
                .ok_or("Doppler observation has no UTC".to_owned())?;
            let query = DateTime::parse_from_rfc3339(&utc.rfc3339)
                .map_err(|error| error.to_string())?
                .with_timezone(&Utc);
            let satellite = pipeline
                .store
                .propagate_ecef(norad_id, query)
                .map_err(|error| error.to_string())?;
            let state = self.filter.state();
            // [UNVERIFIED] No surveyed lever arm is wired yet; receiver position/velocity are
            // therefore the vessel-reference state (zero lever-arm hook).
            let prediction = predict(
                SatelliteState {
                    position_ecef_m: satellite.position_m,
                    velocity_ecef_mps: satellite.velocity_mps,
                },
                ReceiverState {
                    position_ecef_m: state.position_ecef_m,
                    velocity_ecef_mps: state.velocity_ecef_mps,
                    clock_drift_mps: 0.0,
                },
                0.0,
                observation.nominal_carrier_hz,
                pipeline
                    .elevation_mask_rad
                    .unwrap_or(-std::f64::consts::FRAC_PI_2),
            )
            .map_err(|error| format!("prediction rejected: {error:?}"))?;
            let measured =
                -observation.correlation_peak_hz * 299_792_458.0 / observation.nominal_carrier_hz;
            let jacobian = geometric_range_rate_linearisation(
                SatelliteState {
                    position_ecef_m: satellite.position_m,
                    velocity_ecef_mps: satellite.velocity_mps,
                },
                ReceiverState {
                    position_ecef_m: state.position_ecef_m,
                    velocity_ecef_mps: state.velocity_ecef_mps,
                    clock_drift_mps: 0.0,
                },
            )
            .map_err(|error| format!("linearisation rejected: {error:?}"))?;
            let variance_hz2 = envelope
                .covariance
                .first()
                .copied()
                .unwrap_or(1.0)
                .max(f64::EPSILON);
            let scale = 299_792_458.0 / observation.nominal_carrier_hz;
            let update = DopplerRangeRateUpdate {
                satellite_id: envelope.source_id.0.clone(),
                measured_range_rate_mps: measured,
                predicted_range_rate_mps: prediction.range_rate_mps,
                core_jacobian: jacobian,
                variance_mps2: variance_hz2 * scale * scale,
                chi_square_threshold: pipeline.chi_square_threshold,
                satellite_bias_variance_mps2: 100.0,
            };
            Ok::<UpdateResult, String>(self.filter.update_predicted_doppler(&update))
        })();
        match result {
            Ok(update) if update.accepted => {
                self.journals.write_integrity(IntegrityEvent {
                    monotonic_ns: envelope.host_receive_monotonic_ns,
                    source_id: envelope.source_id.0.clone(),
                    reason: "Doppler innovation accepted".to_owned(),
                });
                self.emit_epoch(envelope.host_receive_monotonic_ns, &envelope.source_id.0);
            }
            Ok(_) => self.reject(envelope, "innovation chi-square gate rejected"),
            Err(reason) => self.reject(envelope, &reason),
        }
    }

    fn emit_epoch(&mut self, monotonic_ns: u64, source_id: &str) {
        let state = self.filter.state();
        let steering_authorised = self.integrity.steering_authorised(&state, monotonic_ns);
        let epoch = SolutionEpoch::new(monotonic_ns, state, steering_authorised);
        let Some(line) = epoch_json(&epoch) else {
            self.journals.write_integrity(IntegrityEvent {
                monotonic_ns,
                source_id: source_id.to_owned(),
                reason: "solution epoch contains a non-finite value".to_owned(),
            });
            return;
        };
        self.solution_lines.push(line);
        self.solution_epochs.push(epoch);
    }

    fn rejection_reason(payload: &MeasurementPayload) -> &'static str {
        match payload {
            MeasurementPayload::Gnss(_) => "GNSS disabled by authority mode",
            MeasurementPayload::TrackerDoppler(value)
                if value.constellation == Constellation::Orbcomm =>
            {
                "Orbcomm independent receiver clock is not provisioned"
            }
            MeasurementPayload::TrackerDoppler(value)
                if value.constellation == Constellation::OneWeb =>
            {
                "OneWeb survey gate disabled"
            }
            _ => "measurement has no route",
        }
    }

    fn reject(&mut self, envelope: &MeasurementEnvelope, reason: &str) {
        self.journals.write_integrity(IntegrityEvent {
            monotonic_ns: envelope.host_receive_monotonic_ns,
            source_id: envelope.source_id.0.clone(),
            reason: reason.to_owned(),
        });
    }

    #[must_use]
    pub const fn filter(&self) -> &E {
        &self.filter
    }

    #[must_use]
    pub const fn journals(&self) -> &J {
        &self.journals
    }

    #[must_use]
    pub const fn integrity(&self) -> &I {
        &self.integrity
    }

    pub fn take_solution_epochs(&mut self) -> Vec<SolutionEpoch> {
        std::mem::take(&mut self.solution_epochs)
    }
    pub fn take_solution_lines(&mut self) -> Vec<String> {
        std::mem::take(&mut self.solution_lines)
    }
}

impl Executive<ManualClock, FilterStub, IntegrityStub, MemoryJournals> {
    #[must_use]
    pub fn test_default(gnss_authority: GnssAuthority) -> Self {
        Self::new(
            Config {
                gnss_authority,
                oneweb_enabled: false,
            },
            ManualClock::default(),
            FilterStub::default(),
            IntegrityStub,
            MemoryJournals::default(),
        )
    }
}

fn epoch_json(epoch: &SolutionEpoch) -> Option<String> {
    let s = &epoch.state;
    let accuracies = [
        epoch.horizontal_accuracy_m(),
        epoch.speed_accuracy_mps(),
        epoch.vertical_accuracy_m(),
    ];
    if s.position_ecef_m
        .iter()
        .chain(s.velocity_ecef_mps.iter())
        .chain(s.horizontal_velocity_ned_mps.iter())
        .chain(
            [
                s.heading_rad,
                s.receiver_clock_bias_m,
                s.receiver_clock_drift_mps,
            ]
            .iter(),
        )
        .chain(s.covariance.iter())
        .chain(accuracies.iter())
        .any(|value| !value.is_finite())
    {
        return None;
    }
    Some(format!("{{\"monotonic_ns\":{},\"state\":{{\"position_ecef_m\":[{},{},{}],\"horizontal_velocity_ned_mps\":[{},{}],\"heading_rad\":{},\"receiver_clock_bias_m\":{},\"receiver_clock_drift_mps\":{}}},\"steering_authorised\":{},\"horiz_accuracy_m\":{},\"speed_accuracy_mps\":{},\"vert_accuracy_m\":{},\"msl_alt_m\":0.0}}", epoch.monotonic_ns, s.position_ecef_m[0], s.position_ecef_m[1], s.position_ecef_m[2], s.horizontal_velocity_ned_mps[0], s.horizontal_velocity_ned_mps[1], s.heading_rad, s.receiver_clock_bias_m, s.receiver_clock_drift_mps, epoch.steering_authorised, accuracies[0], accuracies[1], accuracies[2]))
}
