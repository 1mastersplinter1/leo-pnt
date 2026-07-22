# U-C1 report

Contract built against: **v2**, authored by U-C1 from the reviewed v1 design baseline and
architecture.

## What changed

- Added a root Cargo workspace and seven bounded crates: shared bus types, configuration,
  clock/time service, estimator, integrity/authority, journals, and the fusion executive.
- Implemented the executive as the sole orchestrator. Its clock service stamps every
  ingress event; every IMU event propagates the filter stub and advances its observable
  covariance-growth hook; non-IMU accepted measurements dispatch to the estimator,
  integrity/authority gate, and solution-epoch output.
- Implemented strict `gnss_authority` parsing. `production`, `recorded_only`, and `off` are
  the only accepted values, and all modes use one executive type and differ only by routing
  table. `recorded_only` has a truth-journal edge and no fusion edge.
- Enforced D10 at ingress: Orbcomm tracker Doppler has no fusion route until a second
  receiver-clock state or per-receiver clock nuisance term is implemented.
- Added end-to-end tests for bad configuration, GNSS truth-only routing, common processing
  graph/routing tables, propagation honesty, D10 Orbcomm rejection, and synthetic input to
  solution epoch.

## Evidence

The initial red command could not reach compilation because the base environment had no
Rust toolchain:

```text
$ cargo test
/bin/bash: line 1: cargo: command not found
```

An official workspace-local stable toolchain (`rustc 1.97.1`) was then used. Final test run:

```text
$ cargo test
running 6 tests
test authority_modes_change_routing_table_not_processing_graph ... ok
test bad_gnss_authority_is_a_hard_error ... ok
test covariance_grows_on_every_imu_tick_without_measurements ... ok
test orbcomm_is_rejected_before_fusion_by_default ... ok
test recorded_only_sends_gnss_to_truth_but_not_fusion ... ok
test synthetic_imu_and_measurement_emit_a_solution_epoch ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All crate unit tests and doc-tests also passed (zero additional tests). Final lint run:

```text
$ cargo clippy --all-targets -- -D warnings
    Checking pnt-estimator v0.1.0
    Checking pnt-journal v0.1.0
    Checking pnt-integrity v0.1.0
    Checking pnt-config v0.1.0
    Checking fusion-executive v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
```

`cargo fmt --all -- --check` and `git diff --check` completed with no output.

## v2 deferral-resolution checklist

Every v1 deferral is explicitly flagged here for review against the design docs:

1. **Measurement-bus schema resolved:** normative Rust names and field types for
   `MeasurementEnvelope`, `MeasurementPayload`, timestamps, source, covariance, quality,
   calibration, provenance, and supported payloads. D10 is stated at this boundary.
2. **Coordinate frames resolved:** closed `Frame` enumeration and the role of ECEF, NED,
   vessel reference, sensor, antenna phase centre, and frame-independent observations;
   calibration metadata remains authoritative for surveyed transforms.
3. **On-disk formats resolved to the reviewed design's available precision:** run layout,
   segment/record boundaries, mandatory metadata, checksum/index/version rejection,
   truth-journal separation, atomic finalisation and recovery. The design explicitly marks
   binary codecs and ADC packing `[UNVERIFIED]`; v2 preserves those uncertainties rather
   than inventing encodings.
4. **Module-owns-time statement resolved:** clock/time service alone owns runtime time;
   estimator clock states and GNSS UTC do not.
5. **Rate contract resolved:** all reviewed interface rates and behaviours are restated,
   including estimate and `[UNVERIFIED]` labels, IMU-driven propagation, and the rule that
   authority timeout never stops the estimator.

## Assumptions

- The skeleton uses `f64` SI payload values and row-major `Vec<f64>` covariance as the
  minimal typed representation consistent with the reviewed design.
- A solution epoch is emitted after an accepted measurement update in this first slice;
  the later publisher module will schedule propagated 5 Hz fill.
- The in-memory stubs make propagation/update/journal behaviour observable without claiming
  a production filter, persistence codec, predictor, or safety gate.

## Open uncertainties

- Binary codecs, raw ADC packing, SDR sample rate, vessel-axis signs, physical calibration
  content, and all rate values marked estimate remain unverified exactly as stated in the
  reviewed design.
- The production estimator must add an explicit second receiver-clock state or per-receiver
  nuisance model before the D10 Orbcomm rejection route may be changed.
- Bounded queue capacity, overflow event representation, replay persistence, real solution
  integrity, 5 Hz scheduling, and MAVLink publication belong to later connected slices.
