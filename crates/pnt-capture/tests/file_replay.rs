use num_complex::Complex64;
use pnt_capture::{
    write_capture, CaptureError, CaptureMetadata, FileIqSource, Gap, IqSource,
    MetadataRequirements, ENDIANNESS, FORMAT_VERSION, REPRESENTATION,
};
use pnt_tracker::{
    synth::{BpskReference, SynthConfig, Synthesizer},
    TrackOutcome, TrackerConfig,
};

const SAMPLE_RATE_HZ: f64 = 8_192.0;
const BLOCK_SIZE: usize = 256;

fn metadata(sample_count: u64) -> CaptureMetadata {
    CaptureMetadata {
        format_version: FORMAT_VERSION,
        representation: REPRESENTATION.to_owned(),
        endianness: ENDIANNESS.to_owned(),
        sample_rate_hz: SAMPLE_RATE_HZ,
        centre_frequency_hz: 1_575_420_000.0,
        bandwidth_hz: 5_000_000.0,
        gain_db: vec![17.0, 19.0],
        channels: vec![0, 1],
        external_reference: true,
        first_sample_monotonic_ns: 2_000_000_000,
        first_sample_utc_rfc3339: Some("2026-07-24T00:00:00Z".to_owned()),
        sample_count_per_channel: sample_count,
        gaps: Vec::new(),
        calibration_id: "antennas-v3".to_owned(),
        configuration_id: "capture-test-v1".to_owned(),
    }
}

#[test]
fn synth_write_read_track_matches_direct_detections() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("segment.iq");
    let reference = BpskReference::pn(BLOCK_SIZE, 0x1234);
    let mut synth = Synthesizer::new(
        SynthConfig {
            sample_rate_hz: SAMPLE_RATE_HZ,
            initial_offset_hz: 480.0,
            offset_ramp_hz_per_s: 10.0,
            delay_samples: 31,
            cn0_db_hz: 70.0,
            seed: 77,
        },
        reference.clone(),
    );
    let blocks: Vec<Vec<Complex64>> = (0..4).map(|_| synth.next_block()).collect();
    let channel_zero: Vec<_> = blocks.iter().flatten().copied().collect();
    let channel_one: Vec<_> = channel_zero
        .iter()
        .map(|sample| Complex64::new(-sample.im, sample.re))
        .collect();
    write_capture(
        &path,
        &metadata(channel_zero.len() as u64),
        &[channel_zero.clone(), channel_one.clone()],
    )
    .expect("write capture");

    let config = TrackerConfig {
        sample_rate_hz: SAMPLE_RATE_HZ,
        min_frequency_hz: -4_080.0,
        max_frequency_hz: 4_080.0,
        frequency_bin_hz: 32.0,
        detection_threshold: 32.0,
        tracking_half_width_hz: 128.0,
    };
    let mut direct = config
        .clone()
        .build(reference.samples.clone())
        .expect("tracker");
    let mut replay = config.build(reference.samples).expect("tracker");
    let mut source =
        FileIqSource::open(&path, BLOCK_SIZE, &MetadataRequirements::default()).expect("open");

    for (index, original) in blocks.iter().enumerate() {
        let block = source.next_block().expect("read").expect("block");
        assert_eq!(block.channels, vec![0, 1]);
        assert_eq!(block.samples[1].len(), BLOCK_SIZE);
        for (actual, expected) in block.samples[1].iter().zip(
            original
                .iter()
                .map(|sample| Complex64::new(-sample.im, sample.re)),
        ) {
            assert!((actual - expected).norm() < 2.0e-7);
        }
        let timestamp = 2_000_000_000 + index as u64 * 31_250_000;
        assert_eq!(block.first_sample_monotonic_ns, timestamp);
        let expected = direct.process_block(original, timestamp);
        let actual = replay.process_block(&block.samples[0], block.first_sample_monotonic_ns);
        match (expected, actual) {
            (TrackOutcome::Detection(left), TrackOutcome::Detection(right)) => {
                assert!((left.correlation_peak_hz - right.correlation_peak_hz).abs() < 1.0e-4);
                assert_eq!(left.sample_time_ns, right.sample_time_ns);
                assert_eq!(left.delay_samples, right.delay_samples);
            }
            pair => panic!("detection mismatch: {pair:?}"),
        }
    }
    assert!(source.next_block().expect("EOF").is_none());
}

#[test]
fn gap_and_overrun_are_surfaced_and_advance_time() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("gap.iq");
    let mut metadata = metadata(8);
    metadata.sample_rate_hz = 1_000.0;
    metadata.gaps = vec![Gap {
        sample_index: 3,
        missing_samples: 3,
        overrun: true,
    }];
    let channel = vec![Complex64::new(0.0, 0.0); 8];
    write_capture(&path, &metadata, &[channel.clone(), channel]).expect("write");
    let mut source = FileIqSource::open(&path, 5, &MetadataRequirements::default()).expect("open");
    let first = source.next_block().expect("read").expect("first");
    let second = source.next_block().expect("read").expect("second");
    assert_eq!(first.samples[0].len(), 3);
    assert!(first.discontinuities_before.is_empty());
    assert_eq!(second.discontinuities_before, metadata.gaps);
    assert_eq!(second.first_sample_monotonic_ns, 2_006_000_000);
}

#[test]
fn metadata_mismatch_and_truncated_iq_are_hard_errors() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("bad.iq");
    let channel = vec![Complex64::new(0.0, 0.0); 8];
    write_capture(&path, &metadata(8), &[channel.clone(), channel]).expect("write");
    let requirements = MetadataRequirements {
        sample_rate_hz: Some(10_000.0),
        ..MetadataRequirements::default()
    };
    assert!(matches!(
        FileIqSource::open(&path, 4, &requirements),
        Err(CaptureError::Metadata(reason)) if reason.contains("sample rate mismatch")
    ));

    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("open");
    file.set_len(8).expect("truncate");
    assert!(matches!(
        FileIqSource::open(&path, 4, &MetadataRequirements::default()),
        Err(CaptureError::Codec(reason)) if reason.contains("metadata requires")
    ));
}
