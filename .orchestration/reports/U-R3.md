# U-R3 report — paired replay harness

Contract: v5.1. Branch: `unit/U-R3`.

## What changed

- Added workspace crate `pnt-replay`.
- Added deterministic single-mode replay using `MeasurementReader`, a fresh `Executive`,
  and a replay clock that returns each recorded ingress monotonic timestamp.
- Added paired replay that reads the measurement stream once and passes clones of the same
  ordered vector to `production` and `recorded_only`. Mode changes only Executive routing.
- Added nearest-truth matching with a caller-specified inclusive maximum offset. Ties choose
  the earlier truth. Unmatched solution epochs are excluded and counted per run.
- Added truth-referenced horizontal position and horizontal speed errors. No estimate is
  used as truth. Added n/mean/RMS/p50/p95/max statistics and aided-minus-withheld comparison
  statistics at timestamps emitted by both runs.
- Added serde JSON report provenance: schema version, run UUID, config hash, input count,
  modes and maximum truth offset. The complete schema and percentile convention are in the
  crate README.

## TDD evidence and fixture derivation

The synthetic fixture is written and finalized with `FileJournals`; replay reads the actual
segment files. At an ECEF point on the positive x axis, ECEF y is local east and ECEF z is
local north. Truth fixes use north/east velocity `(2, 1) m/s` and positions displaced in
those axes. A first IMU impulse moves the default Earth-centred filter off the radial line,
so denied horizontal drift is measurable; near-zero-covariance GNSS then pulls the aided
filter close to truth.

The direct statistics test uses `[0, 3, 4]`: mean `7/3`, RMS `sqrt(25/3)`, p50 `3`, linearly
interpolated p95 `3.9`, max `4`. Separate tests prove GNSS route/update count differences,
truth-gap exclusions, bit-exact repeated epochs and integrity events, and JSON provenance
round-trip.

Exact gate executed successfully after formatting:

```text
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

Workspace result: 70 tests passed (including 5 `pnt-replay` tests), 0 failed; doc tests
passed; strict Clippy and rustfmt check passed.

## API gaps

No changes to `fusion-executive` or `pnt-journal` were required. `ClockService` permits a
recorded-time clock, and Executive routing destinations, filter counters, epochs, journals,
and readers provide the required seams.

## [UNVERIFIED]

- No real U-J1 capture was available in this unit workspace, so operation on real capture
  scale and real headline values remain `[UNVERIFIED]`.
- The report's linear percentile convention and maximum truth-offset value are documented
  and deterministic, but the trial-wide convention/threshold still requires project-level
  freeze as stated by the design baseline `[UNVERIFIED]`.
- Floating-point determinism is verified in-process on the current backend; cross-platform
  bit identity remains `[UNVERIFIED]`.
