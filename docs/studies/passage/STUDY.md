# Passage endurance study

> **Correction (2026-07-23, D51): estimator caveat.** The absolute error magnitudes in this
> study (hard 3637 m, graduated 1633 m) are produced by a deliberately simple study estimator
> whose Doppler velocity correction is hard-clamped, so it CANNOT diverge. Adversarial review
> of U-H2 (docs/studies/highspeed) showed that the *production* error-state EKF, run on the
> same single-satellite (ISS-TLE-only) denied passage with the production chi-square gate,
> stays bounded but at ~tens-of-km error — NOT the 100-200 m class — because a single-satellite
> range-rate geometry is near-unobservable for position. **What this study validly demonstrates
> is the RELATIVE point — graduated ephemeris aging keeps observations flowing past 6 h where
> the hard gate cuts them off — NOT an absolute accuracy claim.** The 100-200 m denied class
> requires multi-satellite geometry, which the current fixtures (one TLE) do not provide.
> [UNVERIFIED] pending a multi-satellite fixture study.


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
