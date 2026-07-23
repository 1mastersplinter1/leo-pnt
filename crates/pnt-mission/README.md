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

The emitted `three_way` table reports aided, denied dead-reckoning-only, and denied with
Doppler from the same immutable journal. The configured replay reconstructs the mission's
fixture ephemeris and elevation mask, so the final column assimilates journaled tracker
Doppler. This is a synthetic demonstration of integration and qualitative improvement, not
a real-signal or operational performance claim; real-signal behavior remains unverified.
