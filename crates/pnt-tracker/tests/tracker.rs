#![allow(clippy::cast_precision_loss)] // All fixture counters are at most 256.

use num_complex::Complex64;
use pnt_tracker::synth::{BpskReference, SynthConfig, Synthesizer};
use pnt_tracker::{TrackOutcome, TrackerConfig};

const SAMPLE_RATE_HZ: f64 = 8_192.0;
const LENGTH: usize = 256;
const BLOCK_NS: u64 = 31_250_000;

fn reference() -> BpskReference {
    BpskReference::pn(LENGTH, 0x1234_5678)
}

fn tracker(threshold: f64) -> pnt_tracker::CorrelationTracker {
    TrackerConfig {
        sample_rate_hz: SAMPLE_RATE_HZ,
        min_frequency_hz: -4_080.0,
        max_frequency_hz: 4_080.0,
        frequency_bin_hz: 32.0,
        detection_threshold: threshold,
        tracking_half_width_hz: 128.0,
    }
    .build(reference().samples)
    .expect("valid fixture")
}

fn synth(offset: f64, ramp: f64, cn0: f64, seed: u64) -> Synthesizer {
    Synthesizer::new(
        SynthConfig {
            sample_rate_hz: SAMPLE_RATE_HZ,
            initial_offset_hz: offset,
            offset_ramp_hz_per_s: ramp,
            delay_samples: 37,
            cn0_db_hz: cn0,
            seed,
        },
        reference(),
    )
}

fn detection(outcome: TrackOutcome) -> pnt_tracker::Detection {
    match outcome {
        TrackOutcome::Detection(value) => value,
        TrackOutcome::NoDetection(value) => panic!("unexpected NoDetection: {value:?}"),
    }
}

#[test]
fn constant_offsets_recovered_at_high_and_moderate_cn0() {
    // The coarse grid is 32 Hz. Adjacent-sample phase refinement estimates the residual
    // continuously (not quantised to a bin); a conservative noise-tested bound is 4 Hz,
    // one eighth of a bin. Near-Nyquist fixtures remain inside the configured ±4080 Hz.
    for cn0_db_hz in [78.0, 62.0] {
        for offset_hz in [-4_030.0, -713.25, 487.5, 4_025.0] {
            let mut generator = synth(offset_hz, 0.0, cn0_db_hz, 91);
            let mut receiver = tracker(32.0);
            let result = detection(receiver.process_block(&generator.next_block(), 0));
            assert!(
                (result.correlation_peak_hz - offset_hz).abs() <= 4.0,
                "cn0={cn0_db_hz}, injected={offset_hz}, measured={result:?}"
            );
        }
    }
}

#[test]
fn ramp_is_tracked_smoothly_for_twelve_blocks() {
    let initial_hz = -900.0;
    let ramp_hz_per_s = 75.0;
    let mut generator = synth(initial_hz, ramp_hz_per_s, 70.0, 44);
    let mut receiver = tracker(32.0);
    let mut prior = None;

    for block in 0..12_u64 {
        let measured = detection(receiver.process_block(&generator.next_block(), block * BLOCK_NS));
        // Adjacent phases span sample times 0..N-1, so their mean epoch is block start +
        // (N-1)/(2 Fs). This analytic midpoint defines the injected instantaneous value.
        let midpoint_s = block as f64 * LENGTH as f64 / SAMPLE_RATE_HZ
            + (LENGTH - 1) as f64 / (2.0 * SAMPLE_RATE_HZ);
        let expected = initial_hz + ramp_hz_per_s * midpoint_s;
        assert!((measured.correlation_peak_hz - expected).abs() <= 4.0);
        if let Some(previous_hz) = prior {
            // Physical change is 2.344 Hz/block; 8 Hz permits two independent 4 Hz error
            // bounds while prohibiting a coarse-bin/cycle-slip jump.
            let change: f64 = measured.correlation_peak_hz - previous_hz;
            assert!((change - ramp_hz_per_s * LENGTH as f64 / SAMPLE_RATE_HZ).abs() <= 8.0);
        }
        prior = Some(measured.correlation_peak_hz);
    }
}

#[test]
fn noise_never_emits_and_signal_reacquires() {
    let mut generator = synth(600.0, 0.0, 65.0, 7);
    let mut receiver = tracker(32.0);
    for block in 0..24_u64 {
        assert!(matches!(
            receiver.process_block(&generator.next_noise_block(), block * BLOCK_NS),
            TrackOutcome::NoDetection(_)
        ));
    }
    assert!(matches!(
        receiver.process_block(&generator.next_block(), 24 * BLOCK_NS),
        TrackOutcome::Detection(_)
    ));
}

#[test]
fn synthesis_and_tracker_are_bit_deterministic() {
    let mut first = synth(-321.0, 20.0, 66.0, 0xfeed);
    let mut second = synth(-321.0, 20.0, 66.0, 0xfeed);
    let first_iq = first.next_block();
    let second_iq = second.next_block();
    assert_eq!(first_iq, second_iq);
    assert!(first_iq.iter().zip(&second_iq).all(|(left, right)| {
        left.re.to_bits() == right.re.to_bits() && left.im.to_bits() == right.im.to_bits()
    }));

    let mut first_tracker = tracker(32.0);
    let mut second_tracker = tracker(32.0);
    assert_eq!(
        first_tracker.process_block(&first_iq, 123),
        second_tracker.process_block(&second_iq, 123)
    );
}

#[test]
fn quality_is_monotone_with_cn0() {
    let qualities: Vec<f64> = [42.0, 50.0, 58.0]
        .into_iter()
        .map(|cn0| {
            let mut generator = synth(350.0, 0.0, cn0, 88);
            detection(tracker(2.0).process_block(&generator.next_block(), 0)).quality
        })
        .collect();
    assert!(
        qualities.windows(2).all(|pair| pair[0] < pair[1]),
        "{qualities:?}"
    );
}

#[test]
fn custom_bpsk_burst_is_supported() {
    let burst = BpskReference::from_chips(&[-1, 1, 1, -1]);
    assert_eq!(
        burst.samples,
        vec![
            Complex64::new(-1.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(-1.0, 0.0)
        ]
    );
}
