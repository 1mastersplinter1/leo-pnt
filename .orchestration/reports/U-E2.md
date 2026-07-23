# U-E2 — synthetic end-to-end mission capstone

Status: implemented within the owned-file boundary. The generator, journal capture, paired
study, CLI, deterministic tests, and all feasible D35 assertions are present in
`crates/pnt-mission`.

## Evidence

- `cargo test -p pnt-mission`: 4 integration tests pass (determinism, journal round-trip,
  paired rehearsal, and D35 public-API assertions).
- `cargo clippy -p pnt-mission --all-targets -- -D warnings`: pass.
- Smoke: `cargo run -p pnt-mission --bin mission-study -- --seed 23 --duration 20 --out DIR`:
  2,085 measurement records, 21 independent truth records, and 21 fixture-ephemeris Doppler
  observations; JSON run report emitted.
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
and velocity. Seeded noise is then added in hertz. GNSS measurement records are noisy while
the physically separate truth journal is noise-free.

## Synthetic headline table

Seed 23, 20-second smoke fixture:

| Replay | Position RMS (m) | Speed RMS (m/s) | Matched epochs |
|---|---:|---:|---:|
| Aided / production | 0.525 | 0.478 | 84 |
| GNSS-withheld / recorded_only | 18.584 | 3.323 | 63 |
| Aided minus withheld mean | -14.700 | -2.888 | 84 comparison pairs |

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

1. No `pnt-tracker` crate exists in this checkout. The optional tracker synth+tracker path
   therefore cannot be linked; `tracker_in_loop_count` is honestly zero.
2. `pnt-replay::replay_paired` creates executives without an `EphemerisStore` /
   `DopplerPipeline`, and has no injection/configuration parameter. Generated Doppler records
   are consequently rejected as “Doppler pipeline unavailable.” This blocks the requested
   replay proof of Doppler-rich versus outage/turn behavior without editing `pnt-replay`,
   which the brief forbids.
3. `pnt_replay::ComparisonSummary` exposes no comparison-pair exclusion count. Only aided and
   withheld `excluded_no_near_truth` counts are public.
4. Replay uses its estimator's fixed 0.01-second propagation interval. The mission therefore
   emits IMU at 100 Hz to remain consistent; replay does not derive `dt` from journal times.

## [UNVERIFIED]

- Real-IQ tracker behavior and tracker-to-mission integration.
- Real-signal Doppler noise, nuisance bias, multipath, visibility, and ephemeris error.
- Withheld position observability or boundedness from the generated Doppler until replay can
  attach its public Doppler pipeline.
- All numeric headline values beyond this deterministic synthetic demonstration.
- Spherical sea-surface/local-frame approximation versus WGS-84 passage dynamics.
