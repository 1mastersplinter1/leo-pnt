//! The sole runtime orchestrator.

mod band_trust;

use band_trust::BandTrust;
use chrono::{DateTime, Utc};
use pnt_config::{Config, GnssAuthority};
use pnt_ephemeris::EphemerisStore;
use pnt_estimator::{DopplerRangeRateUpdate, Estimator, FilterStub, UpdateResult};
use pnt_integrity::{
    AuthorityParams, AuthorityProfile, AuthoritySolution, AuthoritySupervisor,
    IntegrityAuthorityGate, IntegrityStub,
};
use pnt_journal::{IntegrityEvent, JournalSinks, MemoryJournals};
use pnt_predictor::{geometric_range_rate_linearisation, predict, ReceiverState, SatelliteState};
use pnt_smoother::{FilterEstimate, ReseedCandidate, ReseedDecision, ReseedGate};
use pnt_time::{ClockService, ManualClock};
use pnt_types::{Constellation, MeasurementEnvelope, MeasurementPayload, SolutionEpoch};

/// Per-band interference increment folded into [`BandTrust`] when a Doppler observation is
/// rejected by the chi-square gate (U1b). A single rejection barely moves trust (hysteresis);
/// sustained rejections on one band progressively down-weight it.
const DOPPLER_REJECTION_INTERFERENCE: f64 = 0.5;

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
    band_trust: BandTrust,
    smoother_ownership: SmootherOwnership,
    solution_sequence: u64,
    last_absolute_observation_ns: Option<u64>,
}

/// Which estimator owns the measurement stream (the exclusive-information-ownership rule).
///
/// A smoother reseed may only be applied while the smoother owns the measurements — otherwise
/// the EKF has already absorbed them and reseeding double-counts information into the
/// autopilot-facing covariance (review B2/N4). The executive starts in [`EkfOwnsMeasurements`]
/// and — until the smoother measurement routing and kill-switch are built — never leaves it, so
/// `submit_smoother_reseed` is inert in production. The variant exists so the reseed gate is
/// exercised end-to-end in tests without exposing an unguarded overwrite of the live filter.
///
/// [`EkfOwnsMeasurements`]: SmootherOwnership::EkfOwnsMeasurements
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmootherOwnership {
    /// Measurements update the EKF directly (today's data flow). Reseeds are refused.
    EkfOwnsMeasurements,
    /// Measurements feed the smoother only; the EKF predicts and accepts guarded reseeds.
    /// Not yet reachable in production — set only by tests until routing exists.
    SmootherOwnsMeasurements,
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
            band_trust: BandTrust::new(),
            smoother_ownership: SmootherOwnership::EkfOwnsMeasurements,
            solution_sequence: 0,
            last_absolute_observation_ns: None,
        }
    }

    /// Sets which estimator owns the measurement stream. The reseed seam is refused unless the
    /// smoother owns measurements (the exclusive-ownership guard). Currently only reachable from
    /// tests: production measurement routing to the smoother is not yet built, so the executive
    /// never enters `SmootherOwnsMeasurements` on its own.
    #[doc(hidden)]
    pub fn set_smoother_ownership(&mut self, ownership: SmootherOwnership) {
        self.smoother_ownership = ownership;
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
            MeasurementPayload::ArmCommand(_) | MeasurementPayload::AckCommand(_) => {
                vec![RoutingDestination::Fusion]
            }
            _ => Self::routing_table(self.config.gnss_authority).non_gnss,
        }
    }

    fn dispatch_to_fusion(&mut self, envelope: &MeasurementEnvelope) {
        self.journals.write_measurement(envelope);
        if let MeasurementPayload::ArmCommand(command) = &envelope.payload {
            self.integrity.arm_command(command);
            return;
        }
        if let MeasurementPayload::AckCommand(command) = &envelope.payload {
            self.integrity.acknowledge(command);
            return;
        }
        if let MeasurementPayload::Imu(imu) = &envelope.payload {
            self.filter.propagate(*imu);
            self.emit_epoch(envelope, true);
            return;
        }
        if matches!(envelope.payload, MeasurementPayload::TrackerDoppler(_)) {
            self.process_doppler(envelope);
            return;
        }
        self.filter.update(envelope);
        if matches!(envelope.payload, MeasurementPayload::Gnss(_)) {
            self.last_absolute_observation_ns = Some(envelope.host_receive_monotonic_ns);
        }
        self.emit_epoch(envelope, true);
    }

    fn process_doppler(&mut self, envelope: &MeasurementEnvelope) {
        let MeasurementPayload::TrackerDoppler(observation) = envelope.payload else {
            return;
        };
        let band = observation.constellation.band();
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
            // Band-aware fusion (U1): down-weight (inflate variance of) a band whose
            // interference estimate has risen. Chi-square gate is deliberately NOT tightened
            // here — inflating R already shrinks the innovation. Trust defaults to 1.0, so
            // this is a no-op until `band_trust.observe(..)` is fed an interference estimate.
            let base_variance = variance_hz2 * scale * scale;
            let update = DopplerRangeRateUpdate {
                satellite_id: envelope.source_id.0.clone(),
                measured_range_rate_mps: measured,
                predicted_range_rate_mps: prediction.range_rate_mps,
                core_jacobian: jacobian,
                variance_mps2: self
                    .band_trust
                    .scale_variance(observation.constellation.band(), base_variance),
                chi_square_threshold: pipeline.chi_square_threshold,
                satellite_bias_variance_mps2: 100.0,
            };
            Ok::<UpdateResult, String>(self.filter.update_predicted_doppler(&update))
        })();
        // U1b interference proxy: an accepted innovation is a clean per-band observation
        // (interference 0.0); a gate rejection is anomalous, folded in as a small positive
        // interference so a band under sustained interference is progressively down-weighted.
        // Predictor/pipeline errors (Err) are not a band-jamming signal and are left untouched.
        match result {
            Ok(update) if update.accepted => {
                self.band_trust.observe(band, 0.0);
                self.journals.write_integrity(IntegrityEvent {
                    monotonic_ns: envelope.host_receive_monotonic_ns,
                    source_id: envelope.source_id.0.clone(),
                    reason: "Doppler innovation accepted".to_owned(),
                });
                self.last_absolute_observation_ns = Some(envelope.host_receive_monotonic_ns);
                self.emit_epoch(envelope, true);
            }
            Ok(_) => {
                self.band_trust
                    .observe(band, DOPPLER_REJECTION_INTERFERENCE);
                self.reject(envelope, "innovation chi-square gate rejected");
            }
            Err(reason) => self.reject(envelope, &reason),
        }
    }

    fn emit_epoch(&mut self, envelope: &MeasurementEnvelope, observation_integrity: bool) {
        let monotonic_ns = envelope.host_receive_monotonic_ns;
        let state = self.filter.state();
        self.solution_sequence = self.solution_sequence.saturating_add(1);
        self.integrity.solution(
            AuthoritySolution {
                sequence: self.solution_sequence,
                state: &state,
                profile: match self.config.gnss_authority {
                    GnssAuthority::Production => AuthorityProfile::Aided,
                    GnssAuthority::RecordedOnly | GnssAuthority::Off => AuthorityProfile::Denied,
                },
                last_absolute_observation_ns: self.last_absolute_observation_ns,
                ephemeris_age_s: observation_integrity.then_some(0.0),
                calibration_id: Some(&envelope.calibration_id),
            },
            monotonic_ns,
        );
        let steering_authorised = self.integrity.steering_authorised(&state, monotonic_ns);
        let epoch = SolutionEpoch::new(monotonic_ns, state, steering_authorised);
        let Some(line) = epoch_json(&epoch) else {
            self.journals.write_integrity(IntegrityEvent {
                monotonic_ns,
                source_id: envelope.source_id.0.clone(),
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

    /// Submits a fixed-lag smoother reseed candidate (U4e). The candidate is evaluated against
    /// the live filter's navigation-core estimate by the fail-closed [`ReseedGate`]; on accept
    /// the core state/covariance is overwritten (the reseed is the autopilot-facing surface),
    /// on reject the filter is held unchanged and the reason is journalled. Returns whether the
    /// reseed was applied.
    ///
    /// **Ownership guard (B2/N4):** the reseed is refused outright unless the smoother owns the
    /// measurement stream ([`SmootherOwnership::SmootherOwnsMeasurements`]). While the EKF owns
    /// measurements (the only production data flow today) a reseed would reinject information the
    /// EKF already absorbed, over-tightening the autopilot-facing covariance — so it is rejected
    /// and journalled, never applied. This keeps the seam safe until the smoother measurement
    /// routing and kill-switch exist.
    ///
    /// **Not yet production-complete:** even under smoother ownership this path still lacks
    /// state lag-propagation-to-now (N5) and integrity-authority coupling on accept, and
    /// `apply_reseed` does not yet reconcile core↔augmented cross-covariances. It is exercised
    /// end-to-end only in tests; it must not be enabled on a live vessel until those land.
    pub fn submit_smoother_reseed(
        &mut self,
        gate: &ReseedGate,
        candidate: &ReseedCandidate,
    ) -> bool {
        if self.smoother_ownership != SmootherOwnership::SmootherOwnsMeasurements {
            self.journals.write_integrity(IntegrityEvent {
                monotonic_ns: self.clock.ingress_monotonic_ns(),
                source_id: "smoother-reseed".to_owned(),
                reason: "reseed refused: EKF owns measurements (would double-count)".to_owned(),
            });
            return false;
        }
        let (state, covariance) = self.filter.core_estimate();
        let live = FilterEstimate { state, covariance };
        match gate.evaluate(candidate, &live) {
            ReseedDecision::Accept { state, covariance } => {
                self.filter.apply_reseed(&state, &covariance);
                true
            }
            ReseedDecision::Reject(reason) => {
                self.journals.write_integrity(IntegrityEvent {
                    monotonic_ns: self.clock.ingress_monotonic_ns(),
                    source_id: "smoother-reseed".to_owned(),
                    reason: format!("reseed rejected: {reason:?}"),
                });
                false
            }
        }
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
    pub fn test_stub(gnss_authority: GnssAuthority) -> Self {
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

impl Executive<ManualClock, FilterStub, AuthoritySupervisor, MemoryJournals> {
    /// Default executable skeleton construction: real supervisor with an intentionally
    /// incomplete parameter register, hence fail-closed until deployment freezes it.
    #[must_use]
    pub fn default_fail_closed(gnss_authority: GnssAuthority) -> Self {
        Self::new(
            Config {
                gnss_authority,
                oneweb_enabled: false,
            },
            ManualClock::default(),
            FilterStub::default(),
            AuthoritySupervisor::fail_closed(AuthorityParams::default()),
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
