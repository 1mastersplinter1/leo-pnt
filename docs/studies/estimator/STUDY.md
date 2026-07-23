# Estimator validation study

**SYNTHETIC ONLY.** Full deterministic run: `true`.

## Consistency

NEES(6) mean 0.870 (ideal 6), 95% coverage 100.0% (nominal 95%). Doppler NIS mean 0.738, coverage 97.8%. Verdict: pessimistic: covariance is materially wider than observed errors.

## D39 velocity degradation

Prior-only velocity RMS: 0.2506 m/s; baseline Doppler: 0.3753 m/s (along LOS 0.3075, across LOS 0.2153). Two mechanisms are evidenced. The current replay prior path is structurally confounded: variance 1 against initial variance 1 gives gain 0.5 and retains 3189068 m radial ECEF error. After removing that confound, Doppler still raises velocity RMS from 0.2506 to 0.3753 m/s entirely in the LOS component (across-LOS changes from 0.2243 to 0.2153 m/s): the default Q is 100 times the injected acceleration-error variance, so the filter repeatedly trusts noisy scalar range rate over the cleaner DR velocity. Matching Q yields 0.2365 m/s.

| Treatment | velocity RMS (m/s) | along-LOS RMS | across-LOS RMS | horizontal position RMS (m) |
|---|---:|---:|---:|---:|
| prior-only | 0.2506 | 0.1116 | 0.2243 | 94.65 |
| controlled baseline | 0.3753 | 0.3075 | 0.2153 | 166.41 |
| current replay prior path | 3835.2588 | 2749.4897 | 2673.8580 | 2010706.06 |
| near-fixed nuisance bias | 0.3594 | 0.2883 | 0.2146 | 153.41 |
| matched velocity process noise | 0.2365 | 0.1142 | 0.2071 | 79.35 |

Fed-R sweep — R=0.01: 0.8483; R=0.14: 0.3753; R=1: 0.3272; R=10: 0.2487.

Acceleration-Q sweep — Qa=0.0004: 0.2365; Qa=0.004: 0.3084; Qa=0.04: 0.3753; Qa=0.4: 0.6643.

Geometry sweep — epoch+0s: 0.3753; epoch+600s: 0.3908; epoch+1200s: 0.3947; epoch+1800s: 0.3798.

Observation-period sweep — period=1s: 0.3753; period=2s: 0.3693; period=5s: 0.4168; period=10s: 0.6814.

Routed action: Route to the next pnt-replay/estimator integration unit: add an atomic FilterStub state/covariance initialization API, use it for ReceiverPrior, and regression-test radial ECEF error as well as horizontal replay scores. Do not tune Doppler around the half-radius state.

## Position observability

The stub reproduces only the relative 20-minute emergence: Doppler is worse at 2 min (5.74 vs 0.81 m RMS) but better at 30 min (149.59 vs 199.81 m). Absolute RMS does not converge; it grows throughout. Turn reset observed: false. The stub has no heading-to-velocity coupling or manoeuvre covariance reset, so it cannot reproduce the predicted reset mechanism.

| duration (min) | prior-only RMS (m) | prior+Doppler RMS (m) |
|---:|---:|---:|
| 2 | 0.81 | 5.74 |
| 5 | 5.30 | 15.28 |
| 10 | 22.11 | 34.51 |
| 15 | 49.97 | 58.90 |
| 20 | 88.84 | 86.95 |
| 25 | 138.77 | 114.48 |
| 30 | 199.81 | 149.59 |

Turn windows: pre 28.52 m, first two minutes after 43.87 m, final two minutes 125.24 m.

## Stale ephemeris

Threshold 9 first rejects at least 95% at 1 h; the 6 h case rejects 100%. This supports the gate being no looser than 6 h for this deliberately phase-shifted TLE fixture, but does not validate a real SupGP age-error curve.

| epoch offset | innovation mean (m/s) | innovation RMS (m/s) | threshold-9 rejection |
|---:|---:|---:|---:|
| 0 | 0.00 | 0.00 | 0.0% |
| 1 | -4706.45 | 5413.71 | 99.8% |
| 6 | -3468.19 | 4395.29 | 100.0% |
| 24 | -2604.21 | 3825.29 | 100.0% |

## Scope

- Real-signal measurement distributions and oscillator errors remain unverified.
- The single ISS fixture is a geometry sensitivity instrument, not constellation availability evidence.
- The replay API does not expose process-noise or nuisance-bias configuration; this campaign exercises the same estimator and predictor directly.
