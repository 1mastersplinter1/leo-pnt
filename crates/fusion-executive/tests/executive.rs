use fusion_executive::{DopplerPipeline, Executive, RoutingDestination};
use pnt_config::{Config, GnssAuthority};
use pnt_estimator::{Estimator, FilterStub};
use pnt_integrity::IntegrityStub;
use pnt_journal::MemoryJournals;
use pnt_time::ManualClock;
use pnt_types::{
    ArmAction, ArmCommand, Constellation, FilterState, Frame, GnssFix, Heading, ImuSample,
    MeasurementEnvelope, MeasurementPayload, Provenance, QualityFlags, SourceId, SpeedThroughWater,
    TimeTag, TrackerDoppler, UtcTime,
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
    let mut production = Executive::test_default(GnssAuthority::Production);
    assert_eq!(
        production.process(envelope(1, MeasurementPayload::Gnss(GnssFix::default()))),
        vec![RoutingDestination::Fusion, RoutingDestination::TruthJournal]
    );
    assert_eq!(production.journals().truth_records().len(), 1);
    assert!(production.filter().measurement_updates() > 0);

    let mut off = Executive::test_default(GnssAuthority::Off);
    assert!(off
        .process(envelope(1, MeasurementPayload::Gnss(GnssFix::default())))
        .is_empty());
    assert_eq!(off.filter().measurement_updates(), 0);
    assert_eq!(off.journals().integrity_events().len(), 1);
}

#[test]
fn heading_and_speed_route_to_real_updates() {
    let mut executive = Executive::test_default(GnssAuthority::Off);
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
    let mut executive = Executive::test_default(GnssAuthority::Off);
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
    let mut executive = Executive::test_default(GnssAuthority::Production);
    executive.process(envelope(
        1,
        MeasurementPayload::Imu(ImuSample {
            acceleration_mps2: [0.5, 0.0, 0.0],
            angular_rate_rps: [0.0; 3],
        }),
    ));
    executive.process(envelope(2, MeasurementPayload::Gnss(GnssFix::default())));
    let epochs = executive.take_solution_epochs();
    assert_eq!(epochs.len(), 1);
    assert_eq!(epochs[0].monotonic_ns, 2);
}

#[test]
fn orbcomm_is_rejected_before_fusion_by_default() {
    let mut executive = Executive::test_default(GnssAuthority::Production);
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
}

#[test]
fn oneweb_is_gated_by_config() {
    let observation = MeasurementPayload::TrackerDoppler(TrackerDoppler {
        constellation: Constellation::OneWeb,
        correlation_peak_hz: 0.0,
        nominal_carrier_hz: 1.0e9,
    });
    let mut disabled = Executive::test_default(GnssAuthority::Off);
    assert!(disabled
        .process(envelope(1, observation.clone()))
        .is_empty());
    assert!(disabled.journals().integrity_events()[0]
        .reason
        .contains("OneWeb"));
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
    assert_eq!(
        enabled.process(envelope(1, observation)),
        vec![RoutingDestination::Fusion]
    );
}

#[derive(Default)]
struct ArmGate {
    commands: Vec<ArmCommand>,
}
impl pnt_integrity::IntegrityAuthorityGate for ArmGate {
    fn steering_authorised(&mut self, _: &FilterState, _: u64) -> bool {
        true
    }
    fn arm_command(&mut self, command: &ArmCommand) {
        self.commands.push(command.clone());
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
    .with_doppler_pipeline(DopplerPipeline::new(store));
    let receiver = [3_518_304.71, 784_390.70, 5_244_191.85];
    let mut fix = envelope(
        1,
        MeasurementPayload::Gnss(GnssFix {
            position_ecef_m: receiver,
            velocity_ned_mps: [0.0; 3],
        }),
    );
    fix.covariance = vec![f64::EPSILON];
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
            correlation_peak_hz: -1.6e9 * prediction.range_rate_mps / 299_792_458.0,
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
fn stale_ephemeris_rejection_is_journalled() {
    let store =
        pnt_ephemeris::EphemerisStore::from_tle_file("../pnt-ephemeris/tests/fixtures/iss.tle")
            .unwrap();
    let stale_time = store.epoch(25544).unwrap() + chrono::Duration::hours(7);
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
