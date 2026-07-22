use fusion_executive::{Executive, RoutingDestination};
use pnt_config::{Config, GnssAuthority};
use pnt_estimator::FilterStub;
use pnt_integrity::IntegrityStub;
use pnt_journal::MemoryJournals;
use pnt_time::ManualClock;
use pnt_types::{
    Constellation, Frame, GnssFix, ImuSample, MeasurementEnvelope, MeasurementPayload, Provenance,
    QualityFlags, SourceId, TimeTag, TrackerDoppler,
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
}

#[test]
fn recorded_only_sends_gnss_to_truth_but_not_fusion() {
    let mut executive = Executive::new(
        Config {
            gnss_authority: GnssAuthority::RecordedOnly,
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
}
