//! The sole runtime orchestrator.

use chrono::{DateTime, Duration, Utc};
use pnt_config::{Config, EphemerisAgingConfig, GnssAuthority};
use pnt_ephemeris::EphemerisStore;
use pnt_estimator::{DopplerRangeRateUpdate, Estimator, FilterStub, UpdateResult};
use pnt_integrity::{
    AuthorityParams, AuthorityProfile, AuthoritySolution, AuthoritySupervisor,
    IntegrityAuthorityGate, IntegrityStub,
};
use pnt_journal::{IntegrityEvent, JournalSinks, MemoryJournals};
use pnt_predictor::{geometric_range_rate_linearisation, predict, ReceiverState, SatelliteState};
use pnt_time::{ClockService, ManualClock};
use pnt_types::{Constellation, MeasurementEnvelope, MeasurementPayload, SolutionEpoch};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingDestination {
    Fusion,
    TruthJournal,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EphemerisAgeBand {
    Fresh,
    Inflated { sigma_add_mps: f64 },
    Rejected,
}

/// Maps the fitted orbit-position error curve into additive range-rate uncertainty.
///
/// The LOS rotation approximation is `sigma_rr = |u_dot| sigma_r`; independent variance
/// already represented at the fresh boundary is removed in quadrature.
#[must_use]
pub fn ephemeris_age_band(age_s: f64, config: EphemerisAgingConfig) -> EphemerisAgeBand {
    if age_s <= config.fresh_age_s {
        return EphemerisAgeBand::Fresh;
    }
    if age_s > config.ceiling_age_s {
        return EphemerisAgeBand::Rejected;
    }
    let orbit_sigma_m = |seconds: f64| {
        1000.0
            * (config.orbit_error_intercept_km
                + config.orbit_error_slope_km_per_h * seconds / 3600.0)
    };
    let current = orbit_sigma_m(age_s);
    let fresh = orbit_sigma_m(config.fresh_age_s);
    EphemerisAgeBand::Inflated {
        sigma_add_mps: config.los_rate_rad_s
            * (current.mul_add(current, -fresh * fresh)).max(0.0).sqrt(),
    }
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
    solution_sequence: u64,
    last_absolute_observation_ns: Option<u64>,
    ephemeris_aging: EphemerisAgingConfig,
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
            solution_sequence: 0,
            last_absolute_observation_ns: None,
            ephemeris_aging: EphemerisAgingConfig::default(),
        }
    }

    #[must_use]
    pub fn with_doppler_pipeline(mut self, pipeline: DopplerPipeline) -> Self {
        self.doppler = Some(pipeline);
        self
    }

    #[must_use]
    pub fn with_ephemeris_aging(mut self, config: EphemerisAgingConfig) -> Self {
        self.ephemeris_aging = config;
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
            self.emit_epoch(envelope, Some(0.0));
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
        self.emit_epoch(envelope, Some(0.0));
    }

    #[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
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
            let ceiling =
                Duration::nanoseconds((self.ephemeris_aging.ceiling_age_s * 1e9).round() as i64);
            let propagated = pipeline
                .store
                .propagate_ecef_with_age(norad_id, query, ceiling)
                .map_err(|error| error.to_string())?;
            let age_s = propagated.age.seconds();
            let sigma_add_mps = match ephemeris_age_band(age_s, self.ephemeris_aging) {
                EphemerisAgeBand::Fresh => 0.0,
                EphemerisAgeBand::Inflated { sigma_add_mps } => sigma_add_mps,
                EphemerisAgeBand::Rejected => {
                    return Err(format!(
                        "ephemeris age {age_s:.9}s exceeds graduated ceiling {:.9}s",
                        self.ephemeris_aging.ceiling_age_s
                    ));
                }
            };
            let satellite = propagated.state;
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
                variance_mps2: variance_hz2 * scale * scale + sigma_add_mps * sigma_add_mps,
                chi_square_threshold: pipeline.chi_square_threshold,
                satellite_bias_variance_mps2: 100.0,
            };
            Ok::<(UpdateResult, f64, f64), String>((
                self.filter.update_predicted_doppler(&update),
                age_s,
                sigma_add_mps,
            ))
        })();
        match result {
            Ok((update, age_s, sigma_add_mps)) if update.accepted => {
                if sigma_add_mps > 0.0 {
                    self.journals.write_integrity(IntegrityEvent {
                        monotonic_ns: envelope.host_receive_monotonic_ns,
                        source_id: envelope.source_id.0.clone(),
                        reason: format!(
                            "NOTE ephemeris age {age_s:.9}s; applied sigma_add {sigma_add_mps:.9} m/s"
                        ),
                    });
                }
                self.journals.write_integrity(IntegrityEvent {
                    monotonic_ns: envelope.host_receive_monotonic_ns,
                    source_id: envelope.source_id.0.clone(),
                    reason: "Doppler innovation accepted".to_owned(),
                });
                self.last_absolute_observation_ns = Some(envelope.host_receive_monotonic_ns);
                self.emit_epoch(envelope, Some(age_s));
            }
            Ok((_, _, _)) => self.reject(envelope, "innovation chi-square gate rejected"),
            Err(reason) => self.reject(envelope, &reason),
        }
    }

    fn emit_epoch(
        &mut self,
        envelope: &MeasurementEnvelope,
        ephemeris_age_s: impl Into<Option<f64>>,
    ) {
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
                ephemeris_age_s: ephemeris_age_s.into(),
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

#[cfg(test)]
mod aging_tests {
    use super::*;

    #[test]
    fn age_bands_have_exact_nanosecond_boundaries() {
        let config = EphemerisAgingConfig::default();
        assert_eq!(
            ephemeris_age_band(config.fresh_age_s, config),
            EphemerisAgeBand::Fresh
        );
        assert!(matches!(
            ephemeris_age_band(config.fresh_age_s + 1e-9, config),
            EphemerisAgeBand::Inflated { .. }
        ));
        assert!(matches!(
            ephemeris_age_band(config.ceiling_age_s, config),
            EphemerisAgeBand::Inflated { .. }
        ));
        assert_eq!(
            ephemeris_age_band(config.ceiling_age_s + 1e-9, config),
            EphemerisAgeBand::Rejected
        );
    }

    #[test]
    fn inflation_matches_finite_difference_geometry_mapping() {
        let config = EphemerisAgingConfig::default();
        let age = 24.0 * 3600.0;
        let EphemerisAgeBand::Inflated { sigma_add_mps } = ephemeris_age_band(age, config) else {
            panic!("expected inflation")
        };
        let orbit_m = |seconds: f64| {
            1000.0
                * (config.orbit_error_intercept_km
                    + config.orbit_error_slope_km_per_h * seconds / 3600.0)
        };
        // Central finite difference of range u(t)·dr with u rotating at configured LOS rate.
        let h = 1.0e-3;
        let rate = |error_m: f64| {
            let range = |t: f64| error_m * (config.los_rate_rad_s * t).sin();
            (range(h) - range(-h)) / (2.0 * h)
        };
        let expected =
            (rate(orbit_m(age)).powi(2) - rate(orbit_m(config.fresh_age_s)).powi(2)).sqrt();
        assert!((sigma_add_mps - expected).abs() < 1.0e-8);
    }
}
