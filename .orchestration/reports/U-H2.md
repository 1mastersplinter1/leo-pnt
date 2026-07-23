# U-H2 report

Implemented the high-speed mission envelope and deterministic D46 passage campaign.

## Headline

Same 500 km route: 7 kn takes 38.57 h (37.57 h denied), with synthetic position
RMS/P95/landfall 43.87/59.47/60.95 m and velocity RMS/P95 0.040/0.047 m/s; 20 kn takes
13.50 h (12.50 h denied), with 98.49/134.59/138.01 m and 0.087/0.112 m/s. Margin to the
assumed 30 h ephemeris ceiling is -8.57 h versus +16.50 h.

The 20 kn / 24 h case covers 888.96 km, with position RMS/P95/landfall
132.43/181.61/186.27 m, velocity RMS/P95 0.109/0.143 m/s, and 6.00 h margin. The brief's
20 kn, 24 h, ~500 km tuple is arithmetically inconsistent, so same-distance and
same-duration cases are both retained.

| Regime | Turn 1 | Turn 2 | Turn 3 |
|---|---:|---:|---:|
| 7 kn | 80.60 s / 0.290 km | 80.60 s / 0.290 km | 80.60 s / 0.290 km |
| 20 kn | 114.19 s / 1.175 km | 114.19 s / 1.175 km | 114.19 s / 1.175 km |

## Implementation and integration

`MissionConfig` supports 0–10.3 m/s, optional configurable coordinated turns, speed-scaled
IMU noise/bias, and seeded configurable wave/slam bursts. Defaults disable new behavior and
preserve the legacy RNG sequence. Existing bit-identical directory tests pass.

The study exposes config-driven multipliers for all four estimator `ProcessNoise` terms:
7 kn `[1,1,1,1]`, 20 kn `[6,4,1,2]`. U-H1 and the D43 real-IMU follow-up must replace these
provisional values.

U-P1 overlap: no graduated-aging/passage module exists here. The standalone `highspeed`
module assumes the ordered 30 h ceiling; the current 6 h binary gate cannot support the
cases. Wave/slam parameters, speed-scaled IMU behavior, process-noise multipliers, synthetic
accuracy, and 30 h aged-SupGP behavior remain **[UNVERIFIED]**.

## Verification

- `cargo test -p pnt-mission -p pnt-studies` — pass.
- Full workspace test, clippy, and formatting gates recorded after final changes.
