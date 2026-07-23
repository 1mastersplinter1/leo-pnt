# Passage endurance study

**SYNTHETIC ONLY — D43 CAVEAT:** D43 applies: synthetic epoch aging aliases orbital phase and is availability evidence only, not validation of real SupGP error growth.

Nine hours at 6 kn covers 100.01 km; GNSS is lost at hour 2 and ephemeris is cached at departure. The same seed (`22589824271730501`) and generated measurements drive both policies through the integrated executive, SGP4 propagation, Doppler prediction, estimator update, and integrity journaling paths. The 9 h mission is honestly decimated to one 1 s integration/measurement step; it is not an IMU-rate endurance run.

## Measured result

| handling | accepted / rejected Doppler | Doppler through | measured final 3D position error | position class |
|---|---:|---:|---:|---|
| hard 6 h | 1083 / 540 | 6.0 h | 3636.8 m | dead-reckoning (>1 NM error) |
| graduated, 30 h ceiling | 1623 / 0 | 9.0 h | 1632.6 m | passage-held (<1 NM error) |

The values above are computed from each executive filter's final state against the seeded mission truth; no endpoint error law is imposed.

## `[UNVERIFIED]`

The seeded IMU bias/noise, Doppler noise, SGP4 error curve, LOS-rate mapping, 1 s decimation, 30 h ceiling, and position-class proxy remain `[UNVERIFIED]`. Real-SupGP aging, constellation availability, real tracker residuals, sensor-rate execution, and at-sea replay are required before parameter freeze.
