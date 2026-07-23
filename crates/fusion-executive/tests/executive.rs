use fusion_executive::{DopplerPipeline, Executive, RoutingDestination};
use pnt_config::{Config, GnssAuthority};
use pnt_estimator::{Estimator, FilterStub};
use pnt_integrity::{AuthorityParams, AuthoritySupervisor, IntegrityStub, ProtectionLimits};
use pnt_journal::MemoryJournals;
use pnt_time::ManualClock;
use pnt_types::{
    AckCommand, ArmAction, ArmCommand, Constellation, FilterState, Frame, GnssFix, Heading,
    ImuSample, MeasurementEnvelope, MeasurementPayload, Provenance, QualityFlags, SourceId,
    SpeedThroughWater, TimeTag, TrackerDoppler, UtcTime,
};

fn envelope(sequence: u64, payload: MeasurementPayload) -> MeasurementEnvelope {
    MeasurementEnvelope {
        schema_version: 2,
        source_id: SourceId("synthetic".into()),
        sequence,
        sample_time: TimeTag::DeviceNanoseconds(sequence),
        host_receive_monotonic_ns: sequence,
        utc: None,
        payload,
        frame: Frame::VesselReference,
        covariance: vec![1.0],
        quality: QualityFlags::VALID,
        calibration_id: "cal-1".into(),
        provenance: Provenance::SourceRecord("test".into()),
    }
}

#[test]
fn bad_gnss_authority_is_a_hard_error() {
    let error = Config::parse("gnss_authority = sometimes").unwrap_err();
    assert!(error.to_string().contains("sometimes"));
    assert!(
        !Config::parse("gnss_authority = off")
            .unwrap()
            .oneweb_enabled
    );
    assert!(
        Config::parse("gnss_authority = off\noneweb_enabled = true")
            .unwrap()
            .oneweb_enabled
    );
}

#[test]
fn recorded_only_sends_gnss_to_truth_but_not_fusion() {
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::RecordedOnly,
            oneweb_enabled: false,
        },
        ManualClock::default(),
        FilterStub::default(),
        IntegrityStub,
        MemoryJournals::default(),
    );
    let routes = executive.process(envelope(1, MeasurementPayload::Gnss(GnssFix::default())));
    assert_eq!(routes, vec![RoutingDestination::TruthJournal]);
    assert_eq!(executive.filter().measurement_updates(), 0);
    assert_eq!(executive.journals().truth_records().len(), 1);
}

#[test]
fn production_gnss_is_dual_routed_and_off_is_journalled_as_a_reject() {
    let mut production = Executive::test_stub(GnssAuthority::Production);
    assert_eq!(
        production.process(envelope(1, MeasurementPayload::Gnss(GnssFix::default()))),
        vec![RoutingDestination::Fusion, RoutingDestination::TruthJournal]
    );
    assert_eq!(production.journals().truth_records().len(), 1);
    assert!(production.filter().measurement_updates() > 0);

    let mut off = Executive::test_stub(GnssAuthority::Off);
    assert!(off
        .process(envelope(1, MeasurementPayload::Gnss(GnssFix::default())))
        .is_empty());
    assert_eq!(off.filter().measurement_updates(), 0);
    assert_eq!(off.journals().integrity_events().len(), 1);
    assert_eq!(off.journals().measurement_records().len(), 1);
}

#[test]
fn heading_and_speed_route_to_real_updates() {
    let mut executive = Executive::test_stub(GnssAuthority::Off);
    executive.process(envelope(
        1,
        MeasurementPayload::Heading(Heading { radians: 0.2 }),
    ));
    executive.process(envelope(
        2,
        MeasurementPayload::SpeedThroughWater(SpeedThroughWater {
            metres_per_second: 1.0,
        }),
    ));
    assert_eq!(executive.filter().measurement_updates(), 2);
    assert_eq!(executive.take_solution_epochs().len(), 2);
}

#[test]
fn authority_modes_change_routing_table_not_processing_graph() {
    let production =
        Executive::<ManualClock, FilterStub, IntegrityStub, MemoryJournals>::routing_table(
            GnssAuthority::Production,
        );
    let recorded =
        Executive::<ManualClock, FilterStub, IntegrityStub, MemoryJournals>::routing_table(
            GnssAuthority::RecordedOnly,
        );
    let off = Executive::<ManualClock, FilterStub, IntegrityStub, MemoryJournals>::routing_table(
        GnssAuthority::Off,
    );
    assert_eq!(production.non_gnss, recorded.non_gnss);
    assert_eq!(recorded.non_gnss, off.non_gnss);
    assert_ne!(production.gnss, recorded.gnss);
    assert_ne!(recorded.gnss, off.gnss);
}

#[test]
fn covariance_grows_on_every_imu_tick_without_measurements() {
    let mut executive = Executive::test_stub(GnssAuthority::Off);
    for sequence in 1..=3 {
        executive.process(envelope(
            sequence,
            MeasurementPayload::Imu(ImuSample {
                acceleration_mps2: [1.0, 0.0, 0.0],
                angular_rate_rps: [0.0; 3],
            }),
        ));
    }
    assert_eq!(executive.filter().propagations(), 3);
    assert_eq!(executive.filter().covariance_growth_count(), 3);
    assert_eq!(executive.filter().measurement_updates(), 0);
}

#[test]
fn synthetic_imu_and_measurement_emit_a_solution_epoch() {
    let mut executive = Executive::test_stub(GnssAuthority::Production);
    executive.process(envelope(
        1,
        MeasurementPayload::Imu(ImuSample {
            acceleration_mps2: [0.5, 0.0, 0.0],
            angular_rate_rps: [0.0; 3],
        }),
    ));
    executive.process(envelope(2, MeasurementPayload::Gnss(GnssFix::default())));
    let epochs = executive.take_solution_epochs();
    assert_eq!(epochs.len(), 2);
    assert_eq!(epochs[0].monotonic_ns, 1);
    assert_eq!(epochs[1].monotonic_ns, 2);
}

#[test]
fn default_real_supervisor_is_fail_closed() {
    let mut executive = Executive::default_fail_closed(GnssAuthority::Production);
    executive.process(envelope(
        1,
        MeasurementPayload::ArmCommand(ArmCommand {
            action: ArmAction::Arm,
            host_monotonic_ns: 0,
            source_id: SourceId("helm".into()),
        }),
    ));
    executive.process(envelope(2, MeasurementPayload::Gnss(GnssFix::default())));
    assert!(!executive.take_solution_epochs()[0].steering_authorised);
}

#[test]
fn orbcomm_is_rejected_before_fusion_by_default() {
    let mut executive = Executive::test_stub(GnssAuthority::Production);
    let routes = executive.process(envelope(
        1,
        MeasurementPayload::TrackerDoppler(TrackerDoppler {
            constellation: Constellation::Orbcomm,
            correlation_peak_hz: 12.0,
            nominal_carrier_hz: 137_000_000.0,
        }),
    ));
    assert!(routes.is_empty());
    assert_eq!(executive.filter().measurement_updates(), 0);
    assert_eq!(executive.journals().integrity_events().len(), 1);
    assert_eq!(executive.journals().measurement_records().len(), 1);
}

#[test]
fn orbcomm_is_rejected_before_an_installed_doppler_pipeline() {
    let store =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap();
    let mut executive = Executive::test_stub(GnssAuthority::Production)
        .with_doppler_pipeline(DopplerPipeline::new(store).without_elevation_mask());
    let routes = executive.process(envelope(
        1,
        MeasurementPayload::TrackerDoppler(TrackerDoppler {
            constellation: Constellation::Orbcomm,
            correlation_peak_hz: 12.0,
            nominal_carrier_hz: 137_000_000.0,
        }),
    ));
    assert!(routes.is_empty());
    assert_eq!(executive.filter().measurement_updates(), 0);
    assert_eq!(executive.journals().integrity_events().len(), 1);
    assert!(executive.journals().integrity_events()[0]
        .reason
        .contains("not provisioned"));
}

#[test]
fn oneweb_is_gated_by_config() {
    let observation = MeasurementPayload::TrackerDoppler(TrackerDoppler {
        constellation: Constellation::OneWeb,
        correlation_peak_hz: 0.0,
        nominal_carrier_hz: 1.0e9,
    });
    let mut disabled = Executive::test_stub(GnssAuthority::Off);
    assert!(disabled
        .process(envelope(1, observation.clone()))
        .is_empty());
    assert!(disabled.journals().integrity_events()[0]
        .reason
        .contains("OneWeb"));
    assert_eq!(disabled.journals().measurement_records().len(), 1);
    let mut enabled = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Off,
            oneweb_enabled: true,
        },
        ManualClock::default(),
        FilterStub::default(),
        IntegrityStub,
        MemoryJournals::default(),
    );
    let store =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap();
    let query = store.epoch(25544).unwrap();
    enabled = enabled.with_doppler_pipeline(DopplerPipeline::new(store));
    let mut routed = envelope(1, observation);
    routed.source_id = SourceId("25544".into());
    routed.utc = Some(UtcTime {
        rfc3339: query.to_rfc3339(),
        uncertainty_ns: 1,
    });
    assert_eq!(enabled.process(routed), vec![RoutingDestination::Fusion]);
    assert_eq!(enabled.journals().measurement_records().len(), 1);
    assert_eq!(enabled.journals().integrity_events().len(), 1);
}

#[derive(Default)]
struct ArmGate {
    commands: Vec<ArmCommand>,
    acknowledgements: Vec<AckCommand>,
}
impl pnt_integrity::IntegrityAuthorityGate for ArmGate {
    fn steering_authorised(&mut self, _: &FilterState, _: u64) -> bool {
        true
    }
    fn arm_command(&mut self, command: &ArmCommand) {
        self.commands.push(command.clone());
    }
    fn acknowledge(&mut self, command: &AckCommand) {
        self.acknowledgements.push(command.clone());
    }
}

#[test]
fn arm_command_reaches_authority_and_never_filter_update() {
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Off,
            oneweb_enabled: false,
        },
        ManualClock::default(),
        FilterStub::default(),
        ArmGate::default(),
        MemoryJournals::default(),
    );
    executive.process(envelope(
        1,
        MeasurementPayload::ArmCommand(ArmCommand {
            action: ArmAction::Arm,
            host_monotonic_ns: 1,
            source_id: SourceId("helm".into()),
        }),
    ));
    assert_eq!(executive.filter().measurement_updates(), 0);
    assert_eq!(executive.integrity().commands.len(), 1);
    assert_eq!(executive.journals().measurement_records().len(), 1);
}

#[test]
fn acknowledge_reaches_authority_and_never_filter_update() {
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Off,
            oneweb_enabled: false,
        },
        ManualClock::default(),
        FilterStub::default(),
        ArmGate::default(),
        MemoryJournals::default(),
    );
    executive.process(envelope(
        1,
        MeasurementPayload::AckCommand(AckCommand {
            host_monotonic_ns: 1,
            source_id: SourceId("helm".into()),
        }),
    ));
    assert_eq!(executive.filter().measurement_updates(), 0);
    assert_eq!(executive.integrity().acknowledgements.len(), 1);
    assert_eq!(executive.journals().measurement_records().len(), 1);
}

#[derive(Debug)]
struct ScriptedClock(std::collections::VecDeque<u64>);

impl pnt_time::ClockService for ScriptedClock {
    fn ingress_monotonic_ns(&mut self) -> u64 {
        self.0.pop_front().expect("clock script exhausted")
    }
}

fn dr_params() -> AuthorityParams {
    AuthorityParams {
        aided: ProtectionLimits {
            horizontal_position_m: Some(1.0e9),
            horizontal_velocity_mps: Some(1.0e9),
            heading_rad: Some(1.0e9),
        },
        denied: ProtectionLimits {
            horizontal_position_m: Some(1.0e9),
            horizontal_velocity_mps: Some(1.0e9),
            heading_rad: Some(1.0e9),
        },
        t_lease_s: Some(2.0e-9),
        t_dr_s: Some(5.0e-9),
        t_eph_s: Some(1.0),
        dwell_clear_s: Some(0.0),
        dwell_rearm_s: Some(0.0),
        caution_enter: Some(1.0e9),
        caution_clear: Some(1.0e9),
        revoke_threshold: Some(1.0e10),
        t_ack_s: Some(1.0),
    }
}

#[test]
fn imu_dr_fill_renews_lease_until_absolute_observation_exceeds_t_dr() {
    let clock = ScriptedClock([0, 1, 2, 3, 4, 5, 6, 7].into());
    let supervisor =
        AuthoritySupervisor::with_calibration_validator(dr_params(), |id| id == "cal-1");
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Production,
            oneweb_enabled: false,
        },
        clock,
        FilterStub::default(),
        supervisor,
        MemoryJournals::default(),
    );
    executive.process(envelope(
        0,
        MeasurementPayload::ArmCommand(ArmCommand {
            action: ArmAction::Arm,
            host_monotonic_ns: 0,
            source_id: SourceId("helm".into()),
        }),
    ));
    executive.process(envelope(1, MeasurementPayload::Gnss(GnssFix::default())));
    for sequence in 2..=7 {
        executive.process(envelope(
            sequence,
            MeasurementPayload::Imu(ImuSample::default()),
        ));
    }
    let epochs = executive.take_solution_epochs();
    assert_eq!(epochs.len(), 7, "GNSS plus every IMU propagate tick emits");
    assert!(epochs.iter().take(6).all(|epoch| epoch.steering_authorised));
    assert!(
        !epochs[6].steering_authorised,
        "t_dr, not the continuously renewed t_lease, revokes authority"
    );
    assert_eq!(
        executive.integrity().state(),
        pnt_integrity::AuthorityState::Warning
    );
}

#[test]
fn fixture_doppler_updates_filter_and_emits_bridge_schema_ndjson() {
    let store =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap();
    let query = store.epoch(25544).unwrap();
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Production,
            oneweb_enabled: false,
        },
        ManualClock::default(),
        FilterStub::default(),
        IntegrityStub,
        MemoryJournals::default(),
    )
    .with_doppler_pipeline(DopplerPipeline::new(store).without_elevation_mask());
    let receiver = [3_518_304.71, 784_390.70, 5_244_191.85];
    let mut fix = envelope(
        1,
        MeasurementPayload::Gnss(GnssFix {
            position_ecef_m: receiver,
            velocity_ned_mps: [0.0; 3],
        }),
    );
    // Unit variance keeps the fixture geometry well-defined while leaving a calculable
    // amount of velocity covariance for the deliberately non-zero Doppler innovation.
    fix.covariance = vec![1.0];
    executive.process(fix);
    let state = executive.filter().state();
    let satellite =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap()
            .propagate_ecef(25544, query)
            .unwrap();
    let prediction = pnt_predictor::predict(
        pnt_predictor::SatelliteState {
            position_ecef_m: satellite.position_m,
            velocity_ecef_mps: satellite.velocity_mps,
        },
        pnt_predictor::ReceiverState {
            position_ecef_m: state.position_ecef_m,
            velocity_ecef_mps: state.velocity_ecef_mps,
            clock_drift_mps: 0.0,
        },
        0.0,
        1.6e9,
        -90.0,
    )
    .unwrap();
    let mut doppler = envelope(
        2,
        MeasurementPayload::TrackerDoppler(TrackerDoppler {
            constellation: Constellation::Starlink,
            correlation_peak_hz: -1.6e9 * (prediction.range_rate_mps + 0.5) / 299_792_458.0,
            nominal_carrier_hz: 1.6e9,
        }),
    );
    doppler.source_id = SourceId("25544".into());
    doppler.utc = Some(UtcTime {
        rfc3339: query.to_rfc3339(),
        uncertainty_ns: 1,
    });
    executive.process(doppler);
    assert!(executive.filter().measurement_updates() >= 7);
    let corrected = executive.filter().state();
    let correction_norm = corrected
        .velocity_ecef_mps
        .iter()
        .zip(state.velocity_ecef_mps)
        .map(|(after, before)| (after - before).powi(2))
        .sum::<f64>()
        .sqrt();
    assert!(
        correction_norm > 1.0e-4 && correction_norm < 1.0,
        "unexpected velocity correction {correction_norm}"
    );
    let line = executive.take_solution_lines().pop().unwrap();
    let value: serde_json::Value = serde_json::from_str(&line).unwrap();
    for key in [
        "monotonic_ns",
        "state",
        "steering_authorised",
        "horiz_accuracy_m",
        "speed_accuracy_mps",
        "vert_accuracy_m",
        "msl_alt_m",
    ] {
        assert!(value.get(key).is_some(), "missing {key}");
    }
    for value in [
        value["horiz_accuracy_m"].as_f64().unwrap(),
        value["speed_accuracy_mps"].as_f64().unwrap(),
        value["vert_accuracy_m"].as_f64().unwrap(),
    ] {
        assert!(value.is_finite());
    }
}

#[test]
fn inflated_ephemeris_is_accepted_and_journalled_as_a_note() {
    let store =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap();
    let query = store.epoch(25544).unwrap() + chrono::Duration::hours(7);
    let aging = pnt_config::EphemerisAgingConfig {
        los_rate_rad_s: 1.0e-6,
        ..pnt_config::EphemerisAgingConfig::default()
    };
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Production,
            oneweb_enabled: false,
        },
        ManualClock::default(),
        FilterStub::default(),
        IntegrityStub,
        MemoryJournals::default(),
    )
    .with_ephemeris_aging(aging)
    .with_doppler_pipeline(DopplerPipeline::new(store).without_elevation_mask());
    let receiver = [3_518_304.71, 784_390.70, 5_244_191.85];
    executive.process(envelope(
        1,
        MeasurementPayload::Gnss(GnssFix {
            position_ecef_m: receiver,
            velocity_ned_mps: [0.0; 3],
        }),
    ));
    let satellite =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap()
            .propagate_ecef_with_age(25544, query, chrono::Duration::hours(30))
            .unwrap()
            .state;
    let prediction = pnt_predictor::predict(
        pnt_predictor::SatelliteState {
            position_ecef_m: satellite.position_m,
            velocity_ecef_mps: satellite.velocity_mps,
        },
        pnt_predictor::ReceiverState {
            position_ecef_m: executive.filter().state().position_ecef_m,
            velocity_ecef_mps: executive.filter().state().velocity_ecef_mps,
            clock_drift_mps: 0.0,
        },
        0.0,
        1.6e9,
        -90.0,
    )
    .unwrap();
    let mut observation = envelope(
        2,
        MeasurementPayload::TrackerDoppler(TrackerDoppler {
            constellation: Constellation::Starlink,
            correlation_peak_hz: prediction.correlation_peak_hz,
            nominal_carrier_hz: 1.6e9,
        }),
    );
    observation.source_id = SourceId("25544".into());
    observation.utc = Some(UtcTime {
        rfc3339: query.to_rfc3339(),
        uncertainty_ns: 1,
    });
    executive.process(observation);
    assert!(executive.journals().integrity_events().iter().any(|event| {
        event.reason.contains("NOTE ephemeris age") && event.reason.contains("applied sigma_add")
    }));
    assert!(executive
        .journals()
        .integrity_events()
        .iter()
        .any(|event| { event.reason == "Doppler innovation accepted" }));
}

#[test]
fn stale_ephemeris_rejection_is_journalled() {
    let store =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap();
    let stale_time = store.epoch(25544).unwrap() + chrono::Duration::hours(31);
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Off,
            oneweb_enabled: false,
        },
        ManualClock::default(),
        FilterStub::default(),
        IntegrityStub,
        MemoryJournals::default(),
    )
    .with_doppler_pipeline(DopplerPipeline::new(store));
    let mut observation = envelope(
        1,
        MeasurementPayload::TrackerDoppler(TrackerDoppler {
            constellation: Constellation::Starlink,
            correlation_peak_hz: 0.0,
            nominal_carrier_hz: 1.6e9,
        }),
    );
    observation.source_id = SourceId("25544".into());
    observation.utc = Some(UtcTime {
        rfc3339: stale_time.to_rfc3339(),
        uncertainty_ns: 1,
    });
    executive.process(observation);
    assert!(executive.journals().integrity_events()[0]
        .reason
        .contains("too old"));
    assert_eq!(executive.filter().measurement_updates(), 0);
}

fn tracker_envelope(store: &pnt_ephemeris::EphemerisStore, peak_hz: f64) -> MeasurementEnvelope {
    let mut value = envelope(
        2,
        MeasurementPayload::TrackerDoppler(TrackerDoppler {
            constellation: Constellation::Starlink,
            correlation_peak_hz: peak_hz,
            nominal_carrier_hz: 1.6e9,
        }),
    );
    value.source_id = SourceId("25544".into());
    value.utc = Some(UtcTime {
        rfc3339: store.epoch(25544).unwrap().to_rfc3339(),
        uncertainty_ns: 1,
    });
    value
}

#[test]
fn divergent_doppler_is_gate_rejected_without_filter_update() {
    let store =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap();
    let observation = tracker_envelope(&store, 1.0e9);
    let mut executive = Executive::test_stub(GnssAuthority::Production)
        .with_doppler_pipeline(DopplerPipeline::new(store).without_elevation_mask());
    executive.process(envelope(
        1,
        MeasurementPayload::Gnss(GnssFix {
            position_ecef_m: [3_518_304.71, 784_390.70, 5_244_191.85],
            velocity_ned_mps: [0.0; 3],
        }),
    ));
    let updates_before = executive.filter().measurement_updates();
    executive.process(observation);
    assert_eq!(executive.filter().measurement_updates(), updates_before);
    assert!(executive
        .journals()
        .integrity_events()
        .last()
        .unwrap()
        .reason
        .contains("chi-square"));
}

#[test]
fn default_elevation_mask_rejects_below_mask_and_journals_it() {
    let store =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap();
    let query = store.epoch(25544).unwrap();
    let satellite = store.propagate_ecef(25544, query).unwrap();
    let radius = satellite
        .position_m
        .iter()
        .map(|v| v * v)
        .sum::<f64>()
        .sqrt();
    let receiver = satellite.position_m.map(|v| -6_371_000.0 * v / radius);
    let observation = tracker_envelope(&store, 0.0);
    let mut executive = Executive::test_stub(GnssAuthority::Production)
        .with_doppler_pipeline(DopplerPipeline::new(store));
    executive.process(envelope(
        1,
        MeasurementPayload::Gnss(GnssFix {
            position_ecef_m: receiver,
            velocity_ned_mps: [0.0; 3],
        }),
    ));
    let updates_before = executive.filter().measurement_updates();
    executive.process(observation);
    assert_eq!(executive.filter().measurement_updates(), updates_before);
    assert!(executive
        .journals()
        .integrity_events()
        .last()
        .unwrap()
        .reason
        .contains("BelowElevationMask"));
}

#[derive(Default)]
struct PoisonedEstimator;
impl Estimator for PoisonedEstimator {
    fn propagate(&mut self, _: ImuSample) {}
    fn update(&mut self, _: &MeasurementEnvelope) {}
    fn state(&self) -> FilterState {
        FilterState {
            heading_rad: f64::NAN,
            ..FilterState::default()
        }
    }
    fn update_predicted_doppler(
        &mut self,
        _: &pnt_estimator::DopplerRangeRateUpdate,
    ) -> pnt_estimator::UpdateResult {
        unreachable!()
    }
}

#[test]
fn non_finite_epoch_is_not_emitted_and_is_journalled() {
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::Off,
            oneweb_enabled: false,
        },
        ManualClock::default(),
        PoisonedEstimator,
        IntegrityStub,
        MemoryJournals::default(),
    );
    executive.process(envelope(
        1,
        MeasurementPayload::Heading(Heading { radians: 0.2 }),
    ));
    assert!(executive.take_solution_lines().is_empty());
    assert!(executive.take_solution_epochs().is_empty());
    assert!(executive.journals().integrity_events()[0]
        .reason
        .contains("non-finite"));
}
