# D46 high-speed and extended-passage study

Status: deterministic synthetic study; **[UNVERIFIED]** and not a performance claim.
`results.json` is reproduced by `cargo run -p pnt-studies --bin highspeed-study`.

Ephemeris is cached at t=0, GPS is lost at t=1 h, and four equal-distance legs have three
90° turns. The comparison holds distance at 500 km:

| Case | Distance | Total / denied | Position RMS / P95 / landfall | Velocity RMS / P95 | Ephemeris age / 30 h margin |
|---|---:|---:|---:|---:|---:|
| 7 kn | 500.00 km | 38.57 / 37.57 h | 43.87 / 59.47 / 60.95 m | 0.040 / 0.047 m/s | 38.57 / **-8.57 h** |
| 20 kn | 500.00 km | 13.50 / 12.50 h | 98.49 / 134.59 / 138.01 m | 0.087 / 0.112 m/s | 13.50 / 16.50 h |
| 20 kn endurance | 888.96 km | 24.00 / 23.00 h | 132.43 / 181.61 / 186.27 m | 0.109 / 0.143 m/s | 24.00 / 6.00 h |

At constant 20 kn, 24 h is 889 km; 500 km takes 13.50 h. The brief's tuple cannot be one
constant-speed run, so both cases are committed. The 7 kn comparison also exceeds the
assumed 30 h ceiling and correctly has negative margin. `results.json` classifies position
error at every denied hour as `<25 m`, `25–100 m`, `100–500 m`, or `≥500 m`.

## Manoeuvre convergence

| Case | Turn 1 | Turn 2 | Turn 3 |
|---|---:|---:|---:|
| 7 kn | 80.60 s / 0.290 km | 80.60 s / 0.290 km | 80.60 s / 0.290 km |
| 20 kn | 114.19 s / 1.175 km | 114.19 s / 1.175 km | 114.19 s / 1.175 km |

Convergence is the synthetic model's return to its post-turn steady error envelope; distance
is speed times convergence time.

## Models, lineage, and dependencies

Planing dynamics use seed-hashed burst opportunities at 0.08 Hz. Each accepted burst is a
bounded 0.7 s half-sine with 4.0 m/s² vertical peak and horizontal acceleration 0.18 times
the vertical term. This is **[UNVERIFIED vs real planing data]**, not a sea-state model.

Estimator process noise is config-driven through `SpeedRegime.process_noise_scale`. Relative
to `ProcessNoise::default()`, `[acceleration, turn-rate, clock-drift, nuisance]` scales are
`[1,1,1,1]` at 7 kn and `[6,4,1,2]` at 20 kn. These are provisional inputs pending U-H1 and
the real-IMU study required by D43. The deterministic error envelope combines
square-root-time inertial growth and linear aging; its accuracy numbers are reporting-harness
outputs, not validated navigation predictions.

U-P1's graduated ephemeris aging is absent in this checkout, so this study assumes its
ordered 30 h ceiling for margin accounting. The current `pnt-ephemeris` binary 6 h gate
would stop accepting cached ephemeris before landfall in every headline case.

Still **[UNVERIFIED]**: planing acceleration/slam parameters; installed-IMU noise, bias, and
mount response; process-noise multipliers; synthetic errors against passage truth; and
graduated inflation plus the 30 h ceiling against aged real SupGP data.
