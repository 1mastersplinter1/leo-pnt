use super::*;
use std::{
    fs::OpenOptions,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct TestDir(PathBuf);
impl TestDir {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "pnt-journal-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn metadata() -> RunMetadata {
    RunMetadata {
        run_uuid: "00000000-0000-4000-8000-000000000001".into(),
        created_utc_rfc3339: Some("2026-01-02T03:04:05Z".into()),
        monotonic_epochs: vec![MonotonicEpoch {
            epoch_id: "boot-1".into(),
            start_monotonic_ns: 10,
            utc_rfc3339: None,
            utc_uncertainty_ns: None,
        }],
        config_hash: "cfg-sha256".into(),
        calibration_ids: vec!["imu-a".into()],
        software_revision: "deadbeef".into(),
        hardware_setup: "two receivers".into(),
        ephemeris_snapshot_id: "eph-7".into(),
    }
}
fn envelope(sequence: u64, payload: MeasurementPayload) -> MeasurementEnvelope {
    MeasurementEnvelope {
        schema_version: SCHEMA_VERSION,
        source_id: SourceId("sensor-a".into()),
        sequence,
        sample_time: TimeTag::DeviceNanoseconds(100 + sequence),
        host_receive_monotonic_ns: 200 + sequence,
        utc: Some(UtcTime {
            rfc3339: "2026-01-02T03:04:05Z".into(),
            uncertainty_ns: 3,
        }),
        payload,
        frame: Frame::Sensor,
        covariance: vec![0.0, -0.0, f64::from_bits(0x7ff8_0000_0000_0042)],
        quality: QualityFlags::VALID,
        calibration_id: "imu-a".into(),
        provenance: Provenance::CaptureRecord("capture-1".into()),
    }
}

#[test]
fn mixed_roundtrip_is_bit_exact_and_truth_is_separate() {
    let dir = TestDir::new();
    let mut w = FileJournals::create(&dir.0, metadata(), 180).unwrap();
    let values = [
        envelope(1, MeasurementPayload::Heading(Heading { radians: -0.0 })),
        envelope(
            2,
            MeasurementPayload::Imu(ImuSample {
                acceleration_mps2: [1., 2., 3.],
                angular_rate_rps: [4., 5., 6.],
            }),
        ),
        envelope(
            3,
            MeasurementPayload::TrackerDoppler(TrackerDoppler {
                constellation: Constellation::Iridium,
                correlation_peak_hz: 12.5,
                nominal_carrier_hz: 1.6e9,
            }),
        ),
        envelope(
            4,
            MeasurementPayload::AckCommand(AckCommand {
                host_monotonic_ns: 204,
                source_id: SourceId("helm".into()),
            }),
        ),
    ];
    for v in &values {
        w.try_write_measurement(v).unwrap();
    }
    w.try_write_integrity(IntegrityEvent {
        monotonic_ns: 99,
        source_id: "bus".into(),
        reason: "gap".into(),
    })
    .unwrap();
    w.try_write_truth(&values[0]).unwrap();
    w.finalize().unwrap();
    let records = MeasurementReader::open(&dir.0)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for (record, expected) in records.iter().take(values.len()).zip(&values) {
        let MeasurementJournalRecord::Envelope(actual) = record else {
            panic!()
        };
        assert_eq!(
            encode_measurement(&MeasurementJournalRecord::Envelope(actual.clone())).unwrap(),
            encode_measurement(&MeasurementJournalRecord::Envelope(expected.clone())).unwrap()
        );
    }
    assert_eq!(records.len(), values.len() + 1);
    let truth = TruthReader::open(&dir.0)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(truth.len(), 1);
    assert_eq!(
        encode_truth(&truth[0]).unwrap(),
        encode_truth(&TruthJournalRecord::Envelope(values[0].clone())).unwrap()
    );
}

#[test]
fn truncated_active_tail_recovers_to_last_record() {
    let dir = TestDir::new();
    let mut w = FileJournals::create(&dir.0, metadata(), 10_000).unwrap();
    let a = envelope(1, MeasurementPayload::Heading(Heading { radians: 1. }));
    let b = envelope(2, MeasurementPayload::Heading(Heading { radians: 2. }));
    w.try_write_measurement(&a).unwrap();
    w.try_write_measurement(&b).unwrap();
    let p = segment_paths(&dir.0, StreamKind::Measurement, "tmp")
        .unwrap()
        .pop()
        .unwrap();
    let len = fs::metadata(&p).unwrap().len();
    OpenOptions::new()
        .write(true)
        .open(&p)
        .unwrap()
        .set_len(len - 5)
        .unwrap();
    std::mem::forget(w);
    let (w, reports) = FileJournals::open(&dir.0, 10_000).unwrap();
    assert_eq!(reports.len(), 1);
    assert!(reports[0].recovered_len < reports[0].original_len);
    assert_eq!(reports[0].reason, RecoveryReason::TruncatedPayload);
    drop(w);
    let records = MeasurementReader::open(&dir.0)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(
        encode_measurement(&records[0]).unwrap(),
        encode_measurement(&MeasurementJournalRecord::Envelope(a)).unwrap()
    );
}

#[test]
fn checksum_corruption_in_final_segment_is_hard_error() {
    let dir = TestDir::new();
    let mut w = FileJournals::create(&dir.0, metadata(), 10_000).unwrap();
    w.try_write_measurement(&envelope(
        1,
        MeasurementPayload::Heading(Heading { radians: 1. }),
    ))
    .unwrap();
    w.finalize().unwrap();
    let p = segment_paths(&dir.0, StreamKind::Measurement, "seg")
        .unwrap()
        .pop()
        .unwrap();
    let mut f = OpenOptions::new().read(true).write(true).open(&p).unwrap();
    f.seek(SeekFrom::End(-1)).unwrap();
    f.write_all(&[0]).unwrap();
    let err = MeasurementReader::open(&dir.0)
        .unwrap()
        .next()
        .unwrap()
        .unwrap_err();
    assert!(matches!(
        err,
        JournalError::CorruptRecord {
            reason: "checksum mismatch",
            ..
        }
    ));
}

#[test]
fn finalisation_is_atomic_and_manifest_is_complete() {
    let dir = TestDir::new();
    let mut w = FileJournals::create(&dir.0, metadata(), 10_000).unwrap();
    w.try_write_measurement(&envelope(
        1,
        MeasurementPayload::Heading(Heading { radians: 1. }),
    ))
    .unwrap();
    assert!(segment_paths(&dir.0, StreamKind::Measurement, "seg")
        .unwrap()
        .is_empty());
    assert_eq!(
        segment_paths(&dir.0, StreamKind::Measurement, "tmp")
            .unwrap()
            .len(),
        1
    );
    let manifest = w.finalize().unwrap();
    assert_eq!(manifest.run_uuid, metadata().run_uuid);
    assert_eq!(manifest.monotonic_epochs, metadata().monotonic_epochs);
    assert_eq!(manifest.files.len(), 1);
    assert!(!manifest.files[0].crc32.is_empty());
    assert!(segment_paths(&dir.0, StreamKind::Measurement, "tmp")
        .unwrap()
        .is_empty());
    let disk: RunManifest =
        serde_json::from_reader(File::open(dir.0.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(disk, manifest);
}

#[test]
fn unknown_required_stream_schema_is_rejected() {
    let dir = TestDir::new();
    let mut w = FileJournals::create(&dir.0, metadata(), 10_000).unwrap();
    w.try_write_measurement(&envelope(
        1,
        MeasurementPayload::Heading(Heading { radians: 1. }),
    ))
    .unwrap();
    w.finalize().unwrap();
    let p = segment_paths(&dir.0, StreamKind::Measurement, "seg")
        .unwrap()
        .pop()
        .unwrap();
    let mut f = OpenOptions::new().write(true).open(&p).unwrap();
    f.seek(SeekFrom::Start(5)).unwrap();
    f.write_all(&999_u16.to_le_bytes()).unwrap();
    assert!(matches!(
        MeasurementReader::open(&dir.0).unwrap().next().unwrap(),
        Err(JournalError::UnknownSchemaVersion { version: 999, .. })
    ));
}
