# U-T1 report — correlation Doppler tracker

Contract: v5.1 (TrackerDoppler envelope v2, Doppler convention v4, NORAD/UTC amendment
v4.1). Completed 2026-07-23.

## What changed

- Added workspace crate `pnt-tracker` with configurable-reference FFT correlation across
  frequency bins and every circular time offset, followed by adjacent-sample phase-slope
  refinement.
- Added stateful block tracking that extrapolates measured drift to narrow the next search
  but only emits a current-block measured peak.
- Added typed `Detection`, `NoDetection`, and `TrackOutcome`. Threshold failure has no
  envelope conversion and cannot fabricate an observation.
- Added a constructor for v2/v4.1 `TrackerDoppler` envelopes. It requires UTC and a decimal
  NORAD catalogue ID and reports correlation offset from nominal.
- Added deterministic seeded PN/BPSK, frequency offset, linear ramp, circular delay, AWGN,
  continuous-phase multi-block synthetic IQ and explicit noise-only simulator mode.
- Documented the production reference plug-in point and real-sequence evidence boundary.

## Numeric method and tolerance derivation

The test fixture uses Fs = 8192 Hz, N = 256, and a 32 Hz coarse search grid. FFT circular
correlation identifies delay at every frequency hypothesis. After unwinding the selected
coarse frequency, the tracker aligns and despreads the samples and takes the argument of
the sum of adjacent-sample conjugate products. Its residual-frequency estimate is
`arg(sum(z[n] conj(z[n-1]))) Fs/(2 pi)`, so it is continuous rather than bin-quantised.

The constant-offset and ramp tests use a conservative 4 Hz error bound (one eighth of the
32 Hz coarse bin) at the tested high/moderate C/N0 values. For a linear ramp the adjacent
phase products span samples 0 through N-1, so their analytic mean epoch is
`block_start + (N-1)/(2 Fs)`. Ramp truth is evaluated at that midpoint. The injected
75 Hz/s ramp moves 2.34375 Hz per 31.25 ms block; the adjacent-block smoothness bound is
the analytic movement plus two independent 4 Hz measurement-error bounds (8 Hz residual
error), excluding a coarse-bin/cycle-slip jump.

The quality statistic is correlation peak power divided by the mean other-delay-bin power
for the winning frequency hypothesis. The explicit default threshold is 32 (15.1 dB) to
account provisionally for the multiple-cell frequency/delay search. Tests demonstrate zero
false observations over 24 deterministic pure-noise blocks at that same threshold, followed
by reacquisition, but do not establish a production probability of false alarm.

## Dependencies

- `rustfft` exactly 6.4.1 (recorded in crate manifest and `Cargo.lock`)
- `num-complex` 0.4.6
- workspace `pnt-types` 0.1.0

## Gate evidence

Executed from `/home/od/work/leo-pnt-wt-UT1` with
`PATH="$HOME/.cargo/bin:$PATH"` on 2026-07-23:

1. `cargo clippy --all-targets -- -D warnings`
   - Result: exit 0; all workspace crates checked, including `pnt-tracker`.
2. `cargo test`
   - Result: exit 0; all workspace unit, integration, and doc tests passed.
   - Tracker result: 6 passed, 0 failed. Coverage includes high/moderate C/N0 offsets,
     negative and near-Nyquist offsets, twelve-block ramp and smoothness, 24 noise blocks
     plus reacquisition, bit-identical synthesis/tracking, three-level monotone quality,
     and custom BPSK references.
3. `cargo fmt --all -- --check`
   - Result: exit 0, no output.

An earlier first numeric run intentionally failed two new tests and exposed that a 9 dB
single-cell-style threshold was too low for a many-cell maximum and that very high C/N0 was
reference-sidelobe-limited for the monotonicity fixture. The threshold was changed to the
explicit provisional 15.1 dB value and monotonicity levels were placed in the noise-sensitive
range; the final gate above is the post-change evidence.

## Integration notes

- Construct one `CorrelationTracker` per independently tracked signal/reference.
- Feed capture blocks whose length equals the supplied reference and stamp each call with
  clock-service monotonic nanoseconds. A later capture adapter must handle segmentation and
  any non-circular framing before this API.
- On `Detection`, call `Detection::into_envelope` with UTC, NORAD ID, nominal carrier,
  constellation, calibration/provenance, and a frequency variance selected by integration.
  Submit that immutable envelope to the executive. On `NoDetection`, submit no tracker
  observation; a health/journal event may be added by the executive integration unit.
- Production capture bandwidth remains the baseline 2.5–5 MHz. The low-rate synthetic test
  fixture is not a production RF setting.

## Assumptions and [UNVERIFIED] items

- **[UNVERIFIED]** Default threshold 32 requires link-budget, search-volume/PFA analysis,
  and replay/real-capture tuning. Twenty-four seeded noise blocks are regression evidence,
  not a false-alarm guarantee.
- **[UNVERIFIED]** No Starlink PSS/SSS, Iridium, or Orbcomm sequence is included. Each needs
  a verified published definition, sample-rate/polarity conventions, and real captures.
- **[UNVERIFIED]** Production uncertainty mapping from quality to `frequency_variance_hz2`
  is not derived here; the envelope constructor requires integration to supply it.
- **[UNVERIFIED]** Real capture framing, oscillator impairments beyond a linear ramp,
  multipath, quantisation, dropouts, and 2.5–5 MHz throughput require replay/hardware work.
- OneWeb is not implemented and remains behind its normative occupancy-survey gate.
