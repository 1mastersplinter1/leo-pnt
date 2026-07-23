use super::*;
use pnt_journal::{FileJournals, JournalSinks, RunMetadata};
use pnt_types::{Frame, ImuSample, Provenance, QualityFlags, SourceId, TimeTag, SCHEMA_VERSION};
use tempfile::TempDir;

fn envelope(sequence: u64, time: u64, payload: MeasurementPayload) -> MeasurementEnvelope {
    MeasurementEnvelope {
        schema_version: SCHEMA_VERSION,
        source_id: SourceId("fixture".into()),
        sequence,
        sample_time: TimeTag::HostMonotonicNanoseconds(time),
        host_receive_monotonic_ns: time,
        utc: None,
        payload,
        frame: Frame::EarthCenteredEarthFixed,
        covariance: vec![1.0e-12],
        quality: QualityFlags::VALID,
        calibration_id: "cal-1".into(),
        provenance: Provenance::CaptureRecord(sequence.to_string()),
    }
}

fn fixture() -> TempDir {
    let directory = TempDir::new().unwrap();
    let mut journals = FileJournals::create(
        directory.path(),
        RunMetadata {
            run_uuid: "run-fixture".into(),
            created_utc_rfc3339: Some("2026-07-23T00:00:00Z".into()),
            monotonic_epochs: Vec::new(),
            config_hash: "sha256:fixture".into(),
            calibration_ids: vec!["cal-1".into()],
            software_revision: "test".into(),
            hardware_setup: "synthetic".into(),
            ephemeris_snapshot_id: "none".into(),
        },
        4096,
    )
    .unwrap();

    // Local geometry is at ECEF x = Earth radius: ECEF y is east and z is north.
    // Truth moves north/east at (2, 1) m/s. A deliberately large first IMU impulse moves
    // the otherwise Earth-centred filter off the radial line, making denied horizontal
    // error observable; tiny-covariance GNSS pulls the aided filter onto truth.
    journals.write_measurement(&envelope(
        0,
        50,
        MeasurementPayload::Imu(ImuSample {
            acceleration_mps2: [0.0, 20_000.0, 0.0],
            angular_rate_rps: [0.0; 3],
        }),
    ));
    for (sequence, time, seconds) in [(1_u64, 100_u64, 1.0), (3, 200, 2.0), (5, 1_000, 3.0)] {
        let truth = envelope(
            sequence,
            time,
            MeasurementPayload::Gnss(GnssFix {
                position_ecef_m: [6_378_137.0, seconds, 2.0 * seconds],
                velocity_ned_mps: [2.0, 1.0, 0.0],
            }),
        );
        journals.write_measurement(&truth);
        journals.write_truth(&truth);
        journals.write_measurement(&envelope(
            sequence + 1,
            time + 1,
            MeasurementPayload::Imu(ImuSample::default()),
        ));
    }
    journals.finalize().unwrap();
    directory
}

#[test]
fn paired_run_routes_gnss_and_scores_truth_independently() {
    let directory = fixture();
    let report = replay_paired(directory.path(), 2).unwrap();
    assert_eq!(report.aided.gnss_fusion_routes, 3);
    assert_eq!(report.withheld.gnss_fusion_routes, 0);
    assert_eq!(report.aided.gnss_truth_routes, 3);
    assert_eq!(report.withheld.gnss_truth_routes, 3);
    assert!(report.aided.measurement_updates > report.withheld.measurement_updates);
    assert!(
        report.aided.horizontal_position_error_m.mean.unwrap()
            < report.withheld.horizontal_position_error_m.mean.unwrap()
    );
    assert_eq!(report.run_uuid, "run-fixture");
    assert_eq!(report.config_hash, "sha256:fixture");
}

#[test]
fn statistics_match_hand_computation() {
    // Values 0, 3, 4: mean=7/3, RMS=sqrt(25/3), p50=3,
    // p95 interpolates 90% from 3 to 4 => 3.9, max=4.
    let value = statistics(vec![0.0, 3.0, 4.0]);
    assert_eq!(value.n, 3);
    assert!((value.mean.unwrap() - 7.0 / 3.0).abs() < 1.0e-12);
    assert!((value.rms.unwrap() - (25.0_f64 / 3.0).sqrt()).abs() < 1.0e-12);
    assert_eq!(value.p50, Some(3.0));
    assert!((value.p95.unwrap() - 3.9).abs() < 1.0e-12);
    assert_eq!(value.max, Some(4.0));
}

#[test]
fn truth_gap_is_excluded_and_counted() {
    let directory = fixture();
    // The last GNSS/IMU pair at 1000/1001 is outside one nanosecond of the earlier truths,
    // but both last epochs do match the truth at 1000. Every GNSS epoch matches exactly and
    // each IMU epoch matches at offset one: tightening to zero excludes all three IMU epochs.
    let report = replay_paired(directory.path(), 0).unwrap();
    assert_eq!(report.aided.matched_epochs, 3);
    assert_eq!(report.aided.excluded_no_near_truth, 4);
    assert_eq!(report.withheld.matched_epochs, 0);
    assert_eq!(report.withheld.excluded_no_near_truth, 4);
}

#[test]
fn repeated_replay_is_bit_exact() {
    let directory = fixture();
    let first = replay_directory(directory.path(), GnssAuthority::RecordedOnly).unwrap();
    let second = replay_directory(directory.path(), GnssAuthority::RecordedOnly).unwrap();
    assert_eq!(first.epochs, second.epochs);
    assert_eq!(first.integrity_events, second.integrity_events);
}

#[test]
fn report_json_round_trips_with_provenance() {
    let directory = fixture();
    let report = replay_paired(directory.path(), 2).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let decoded: ReplayReport = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.schema_version, REPORT_SCHEMA_VERSION);
    assert_eq!(decoded.run_uuid, "run-fixture");
    assert_eq!(decoded.config_hash, "sha256:fixture");
    assert_eq!(decoded.aided.mode, "production");
    assert_eq!(decoded.withheld.mode, "recorded_only");
    assert_eq!(
        decoded.aided.horizontal_position_error_m.n,
        report.aided.horizontal_position_error_m.n
    );
}
