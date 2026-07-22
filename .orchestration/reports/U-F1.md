# U-F1 report — EKF core + contracts v3

## Delivered

- Appended contracts v3: full row-major covariance and core slot order, per-epoch accuracy
  surface, `ArmCommand`, and the independent receiver-clock registry/update mechanism.
- Extended `pnt-types` additively with ECEF velocity, full covariance, accuracy accessors,
  arm-command bus payload, and receiver clock identifiers/slots.
- Replaced estimator stub internals (retaining its name for executive compatibility) with a
  dynamic `nalgebra` 0.33.3 EKF. Core states are ECEF position/velocity, heading, primary
  clock bias/drift. Pass-scoped satellite biases and independent receiver clock pairs can be
  augmented; satellite biases can be retired with covariance and registry reindexing.
- Implemented configurable IMU/process-noise propagation, Joseph-form scalar updates,
  innovations, innovation variance, NIS/chi-square hooks, Doppler predictor input,
  heading, speed-through-water, MSL constraint, and aided-only GNSS position/velocity.
- Debug builds assert covariance symmetry and positive semidefiniteness after propagation
  and every accepted update.

## v3 resolutions

- F3: the test asserts position variance grows by more than 0.9 m² after 100 dead-reckoning
  propagations; the legacy count remains only for unchanged-executive compatibility.
- F4: every filter state carries its complete covariance. A `SolutionEpoch` derives
  horizontal position, speed, and vertical one-sigma accuracy from its own state snapshot.
- D13: `ArmCommand` is a typed bus payload carrying arm/disarm, monotonic time, and source.
- D10: receiver-specific clock bias/drift slots have propagation and a receiver-specific
  Doppler update path. Orbcomm remains rejected by the unchanged executive.

## Verification evidence

Gate run from repository root with `PATH="$HOME/.cargo/bin:$PATH"`:

```text
cargo clippy --all-targets -- -D warnings
Finished `dev` profile ...

cargo fmt --all -- --check
(no output; exit 0)

cargo test
fusion-executive: 6 passed
pnt-estimator: 6 passed
pnt-types: 2 passed
all remaining unit and doc-test targets: passed
```

Jacobian tests use central differences with step `1e-6` and maximum absolute error tolerance
`2e-6`. They cover the state transition, scalar state observations used by heading/GNSS,
speed, and local-plane MSL altitude. Doppler's predictor linearisation is supplied by the
caller by contract and is therefore not re-derived in this crate; augmentation of its
per-SV Jacobian column is tested through an accepted update.

## Executive work required in U-I2

- Route and journal `MeasurementPayload::ArmCommand`; validate the source and freshness and
  deliver it to the authority supervisor. It must not be treated as an estimator update.
- Provision the Orbcomm `ReceiverClockId`, then replace the current unconditional ingress
  rejection only when predictor/gate output is connected to `update_doppler_for_receiver`.
- Migrate `SolutionEpoch` creation to a constructor. Rust struct literals cannot remain
  source-compatible if three required stored accuracy fields are added. This unit therefore
  supplied compatible accessors backed by the epoch's covariance. U-I2 may materialise the
  three values at construction/wire-publication time without changing their definition.
- Convert GNSS NED velocity to ECEF before calling the EKF's aided GNSS update; the v2 bus
  payload and v3 estimator state intentionally retain their declared frames.

## Assumptions and [UNVERIFIED]

- The default IMU interval is 0.01 s, matching the estimated 100 Hz contract. Runtime timing
  integration should eventually supply validated sample intervals. `[UNVERIFIED]`
- Default process-noise numbers and initial variances are engineering placeholders and must
  be tuned with replay data. `[UNVERIFIED]`
- The MSL update treats its supplied ECEF up-vector and scalar as a local tangent-plane
  constraint. The geoid/tide/wave variance remains `[UNVERIFIED]` per the baseline.
- Speed-through-water is a scalar horizontal-speed consistency update; the filter contains
  no current state. The downstream solution module remains responsible for deriving and
  journalling the current vector with heading rotation and covariance.
- No real predictor/tracker integration exists yet, so end-to-end Doppler and Orbcomm paths
  remain `[UNVERIFIED]` despite unit-tested estimator APIs.
