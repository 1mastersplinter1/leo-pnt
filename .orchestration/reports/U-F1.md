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

## U-F1.1 fix-round dispositions

The fix round was developed as a tests-only red commit (`da83854`) followed by the
implementation commit (`686fa7a`).

| Finding | Disposition |
|---|---|
| Opus H1 / Sonnet 4 | Fixed. A shared ECEF-to-ENU rotation at the epoch position projects the full position covariance, including ECEF cross-covariances. Horizontal accuracy is ENU E/N DRMS and vertical accuracy is ENU U one-sigma. An independent 56°N/12°E test checks both projections. |
| Opus H2 | Fixed. The growth test starts both filters with zero covariance and compares production Q against Q=0. Fault injection temporarily changed default acceleration variance from `0.04` to `0.0`; the named test failed at its Q-attributable `> 0.01` assertion (0 passed, 1 failed), after which `0.04` was restored. |
| Opus M1 | Fixed. Heading, GNSS, and MSL finite differences invoke `update_heading`, `update_gnss`, and `update_msl_altitude` on the real augmented filter rather than rebuilding H. |
| Opus M2 / Sonnet 1 | Fixed. Receiver-specific Doppler is exercised through `update_doppler_for_receiver`; its primary-to-receiver drift remap and prediction are finite-difference checked. |
| Opus M3 / Sonnet 5 | Fixed. `horizontal_velocity_ned_mps` is now north/east from the shared local rotation, not ECEF X/Y. |
| Opus M4 | Fixed. Speed-through-water predicts the ENU horizontal norm; its real update path has an augmented-filter FD regression that requires the ECEF Z contribution at this location. |
| Opus M5 | Fixed by document+bound. Clock bias remains for future pseudorange/STL and baseline compatibility. Primary and registered receiver bias variance is capped at `1e8 m²`; full two-state clock Q is applied. The cap and coefficient are `[UNVERIFIED]` pending replay tuning. |
| Opus L1 / Sonnet 2 | Fixed. Transition FD runs after registering a receiver and multiple nuisances and verifies primary and receiver bias/drift coupling across the full augmented dimension. |
| Opus L2 | Documented in contracts v3: denied-mode ECEF vertical velocity is weakly observable; U-M1 retains the baseline `vd = 0` plus consistent nonzero vertical-accuracy rule. Dynamics/noise remain `[UNVERIFIED]`. |
| Opus L3 | Retained as informational. Symmetry/PSD remain debug assertions; no production failure-policy change was requested or justified in this unit. |
| Opus L4 | Fixed. Primary and independent clocks use the standard integrated drift-noise Q terms `[dt³/3, dt²/2; dt²/2, dt]`; coefficients remain `[UNVERIFIED]`. |
| Opus L5 | Explicit U-I2 handoff retained: the unchanged executive still routes `ArmCommand` through its fallback Fusion route; U-I2 must journal/validate/route it to authority and must not treat it as an estimator measurement. |
| Opus I2 | Fixed. `DopplerRangeRateUpdate::satellite_bias_variance_mps2` supplies augmentation variance; a regression observes the requested `47.0` variance through the real update. |
| Opus I3 | Documented in contracts v3. GNSS uses six sequential independent 1-DOF scalar NIS gates; the threshold is per component, not a joint 6-DOF threshold. |
| Sonnet 1 | Fixed. Direct real-path coverage now includes GNSS, MSL, and receiver Doppler. `Estimator::update()` has a table-driven integration case for all six payload variants; Heading, SpeedThroughWater, and GNSS update, while Imu (propagated by the executive), TrackerDoppler (requires predictor output), and ArmCommand (U-I2 authority route) are deliberate no-ops here. |
| Sonnet 3 | Fixed. The augmented fixture creates three nuisances, retires the middle one, and verifies both surviving nuisance indices plus the receiver registry and covariance dimension. |
| Sonnet 6 | Fixed by the separate red tests commit followed by implementation. |
| Sonnet 7 | No change required; the reviewed loss of `Copy` is harmless because covariance is dynamically owned. |
| Sonnet 8 / Opus I1 | No change required; branch-staleness noise was benign. `main` was merged first as mandated. |

## U-F1.1 verification evidence

Final gate from the repository root, with `PATH="$HOME/.cargo/bin:$PATH"` and the
`fusion-executive` sources unmodified in this fix round:

```text
$ cargo test
fusion-executive integration tests: 6 passed, 0 failed
pnt-estimator: 12 passed, 0 failed
pnt-types: 3 passed, 0 failed
all other unit and doc-test targets: passed
Finished test profile; exit 0

$ cargo clippy --all-targets -- -D warnings
Checked pnt-types, pnt-integrity, pnt-estimator, pnt-journal, fusion-executive
Finished dev profile; exit 0

$ cargo fmt --all -- --check
(no output; exit 0)

$ cargo build --workspace
Finished dev profile; exit 0
```

The required Q=0 red run was separate from the final gate and produced:

```text
running 1 test
test tests::dead_reckoning_grows_position_variance_by_magnitude ... FAILED
assertion failed: with_q.covariance()[(0, 0)]
    > without_q.covariance()[(0, 0)] + 0.01
test result: FAILED. 0 passed; 1 failed; 11 filtered out
```
