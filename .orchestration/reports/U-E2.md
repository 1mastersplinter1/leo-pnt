# U-E2 — synthetic end-to-end mission capstone

Status: implemented. The follow-up integrates `pnt-tracker` into the synthetic pass while
preserving the generator, journal capture, paired study, CLI, deterministic tests, and all
feasible D35 assertions in `crates/pnt-mission`.

## Evidence

- `cargo test -p pnt-mission`: 4 integration tests pass (determinism, journal round-trip,
  tracker-in-loop paired rehearsal, and D35 public-API assertions). The paired rehearsal
  asserts tracker capture provenance, a nonzero tracker observation count, one-for-one
  tracker/Doppler counts, <=4 Hz error from the direct true-trajectory prediction, and
  aided position RMS < withheld position RMS.
- `cargo clippy -p pnt-mission --all-targets -- -D warnings`: pass.
- Smoke: `cargo run -p pnt-mission --bin mission-study -- --seed 23 --duration 20 --out DIR`:
  2,085 measurement records, 21 independent truth records, and 21 tracker-derived Doppler
  observations; maximum tracker versus direct predictor error was 2.188 Hz; JSON run report
  emitted.
- The final workspace-wide test/clippy/fmt gate is recorded in the unit commit handoff.

## Mission physics

The through-water velocity in local north/east is

`v_water = speed * [cos(heading), sin(heading)]`.

Ground velocity adds the configured current vector. Constant-heading legs therefore have
constant velocity and zero ideal translational acceleration. Across the coordinated turn,
the generator evaluates successive ground-velocity vectors and supplies

`a_NE(k) = (v_NE(k) - v_NE(k-1)) / dt`,

rotated into ECEF, while the IMU yaw rate is the derivative of the same heading profile.
Trapezoidal integration of those same successive velocities advances truth position. Thus
the noiseless IMU translational and angular components are derived from, and consistent with,
the motion; configured bias and seeded Gaussian noise are added afterward. The small local
trajectory is projected to a spherical zero-altitude surface. That spherical model, rather
than WGS-84 geodetic transport, is a deliberate synthetic-fixture approximation.

The fixture ISS state is propagated through `pnt-ephemeris`; visibility and noiseless
correlation-peak Doppler are calculated by `pnt-predictor` from the true receiver position
and velocity. For the visible pass, each prediction drives a seeded `pnt-tracker` BPSK IQ
block. One stateful tracker processes the blocks and `Detection::into_envelope` supplies the
journal observation in place of the former direct Doppler envelope. The direct prediction
is retained only as a test oracle. GNSS measurement records are noisy while the physically
separate truth journal is noise-free.

## Synthetic headline table

Seed 23, 20-second smoke fixture:

| Replay | Position RMS (m) | Speed RMS (m/s) | Matched epochs |
|---|---:|---:|---:|
| Aided / production | 0.616 | 0.485 | 84 |
| GNSS-withheld / recorded_only | 14.443 | 4.239 | 63 |
| Aided minus withheld mean | -11.373 | -3.523 | 84 comparison pairs |

**SYNTHETIC DEMONSTRATION ONLY — these values are not performance claims; behavior with
real signals is [UNVERIFIED].** Negative comparison values assert the documented
aided-minus-withheld sign convention. The mission contains a Doppler-rich constant-heading
leg and a turn, and the aggregate aided error is smaller. A legitimate segmented claim that
Doppler bounds withheld error during the rich leg cannot be made from the current replay,
because of the public API gap below.

## D35 carried items

- Comparison table is asserted directly through `ReplayReport`, including nonzero comparison
  count and negative aided-minus-withheld means for both position and speed.
- Production replay is repeated and `ReplayRun` is bit-exact.
- Both modes' `input_measurement_count` values equal the generated journal count.
- Per-run truth-match exclusions are asserted. A comparison-pair exclusion count cannot be
  asserted because `ComparisonSummary` contains only the two statistics fields.

## API gaps

1. `pnt-replay::replay_paired` creates executives without an `EphemerisStore` /
   `DopplerPipeline`, and has no injection/configuration parameter. Generated Doppler records
   are consequently rejected as “Doppler pipeline unavailable.” This blocks the requested
   replay proof of Doppler-rich versus outage/turn behavior without editing `pnt-replay`,
   which the brief forbids.
2. `pnt_replay::ComparisonSummary` exposes no comparison-pair exclusion count. Only aided and
   withheld `excluded_no_near_truth` counts are public.
3. Replay uses its estimator's fixed 0.01-second propagation interval. The mission therefore
   emits IMU at 100 Hz to remain consistent; replay does not derive `dt` from journal times.

## [UNVERIFIED]

- Real-signal tracker behavior (the integrated IQ path remains seeded synthetic BPSK/AWGN).
- Real-signal Doppler noise, nuisance bias, multipath, visibility, and ephemeris error.
- Withheld position observability or boundedness from the generated Doppler until replay can
  attach its public Doppler pipeline.
- All numeric headline values beyond this deterministic synthetic demonstration.
- Spherical sea-surface/local-frame approximation versus WGS-84 passage dynamics.
