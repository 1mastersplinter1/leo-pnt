# pnt-tracker

This crate measures correlation-peak Doppler from complex baseband IQ. It searches a
configurable frequency grid and every circular reference delay using FFT correlation, then
refines frequency from the despread adjacent-sample phase. Tracking extrapolates the last
measured ramp only to choose the next search window; it never creates an observation.
Below the configured peak-to-noise-floor threshold the result is typed `NoDetection`.

The processing bandwidth is selected by the capture configuration; production use must
retain the design baseline's 2.5–5 MHz processing bandwidth. The smaller rates used in unit
tests are numeric fixtures, not a production RF configuration.

## Real-signal plug-in point

Pass the sampled, complex known sequence to `TrackerConfig::build`. The same sequence is
passed as `BpskReference` only when using the synthetic capture generator. Replace that
parameter with a verified, sample-rate-matched Starlink PSS/SSS, Iridium, or Orbcomm known
sequence; the acquisition/tracking engine itself does not encode a constellation sequence.

**[UNVERIFIED — not in U-T1]** No real Starlink PSS/SSS, Iridium, or Orbcomm sequence is
shipped here. Adding one requires a verified published definition, documented resampling
and polarity conventions, and validation against real captures. OneWeb remains forbidden
until its separate occupancy-survey gate passes.

Constraint (review F16): `TrackerConfig::build` currently requires the reference length to be a power of two ≥ 4 (FFT sizing choice, not a rustfft requirement — rustfft supports arbitrary lengths). A real PSS/SSS/Iridium/Orbcomm sequence of arbitrary length must be resampled or zero-padded to a power-of-two length until this restriction is relaxed.

The envelope constructor requires UTC and a decimal NORAD catalogue ID, preserving contract
v4.1. `correlation_peak_hz` is always offset from nominal per v4.

## Dependencies

- `rustfft` 6.4.1
- `num-complex` 0.4.6
- workspace `pnt-types`
