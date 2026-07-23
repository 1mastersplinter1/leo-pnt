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

The emitted headline table is a synthetic demonstration, not a real-signal performance
claim. Tracker-in-loop generation and Doppler assimilation in paired replay are explicitly
reported integration gaps in this checkout.
