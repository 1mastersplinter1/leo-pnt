# pnt-mission

Deterministic synthetic full-stack rehearsal. The generator creates constant-heading legs
separated by a coordinated 90-degree turn, applies a configurable water current, and derives
the ECEF acceleration and yaw rate supplied to the IMU from successive truth velocities.
It records IMU, heading, speed-log, noisy GNSS, noise-free GNSS truth, and fixture-ephemeris
correlation-peak Doppler in the same `FileJournals` directory used by capture/replay.

Run the smoke study with:

```text
cargo run -p pnt-mission --bin mission-study -- --seed 1 --out /tmp/mission-run
```

The emitted `four_way` table reports four denied-mode rows from the same immutable journal —
`denied_dr_only`, `denied_prior_only` (receiver prior, Doppler suppressed),
`denied_prior_with_doppler`, and `denied_no_prior_with_doppler` — alongside the paired
`replay` block (which carries the aided run). The `attribution` block discloses the
caller-supplied receiver prior (truth-equivalent for this synthetic fixture) and separately
attributes the prior's and Doppler's RMS contributions, so neither can masquerade as the
other. The configured replay reconstructs the mission's fixture ephemeris and elevation
mask, so the Doppler rows assimilate journaled tracker Doppler. Honest caveat: in this
synthetic geometry/tuning, Doppler assimilation improves position RMS given the prior but
**degrades speed RMS against the same-initialization baseline** (mechanism [UNVERIFIED],
pending an estimator tuning study). This is a synthetic demonstration of integration, not
a real-signal or operational performance claim; real-signal behavior remains unverified.

## High-speed envelope

`MissionConfig` accepts speeds through 10.3 m/s (20 kn). Its opt-in `coordinated_turn`
commands yaw rate during the turn; `wave_slam` creates seeded, bounded half-sine vertical
bursts with a configurable pitch-coupled horizontal term; and `speed_scaled_imu` linearly
scales IMU white noise and fixed bias from a reference speed. All default to `None`, leaving
legacy output and RNG draw order unchanged. The wave/slam model is a synthetic stand-in
**[UNVERIFIED vs real planing data]**, not a calibrated sea-state, hull, or mount model.
