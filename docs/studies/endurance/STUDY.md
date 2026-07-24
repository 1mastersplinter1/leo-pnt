# Endurance study: leg duration and clock discipline

**SYNTHETIC ENDURANCE EXPERIMENT [UNVERIFIED]. Errors are production Executive + real FilterStub EKF state versus generator truth. No result is clamped, formula-generated, or target-fitted.**

Cross-reference: D55 identified the confounds; D57 identified longer constant-heading legs and clock discipline as the next two levers. D56 defines 500 m typical (p50) and 750 m worst-case (p95), while the requested stricter 500 m p95 check is also shown.

## Leg-duration curve

| leg | clock | mean | p50 | p95 | spread | accepted/rejected mean | handovers mean | class |
|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 10 min | 1e-9 | 180340.6 m | 175579.9 m | 222206.0 m | 154782.6-222206.0 m | 252.2/83.8 | 1.0 | 100 km-Earth radius |
| 20 min | 1e-9 | 158913.8 m | 154084.6 m | 198493.0 m | 136676.9-198493.0 m | 487.5/168.5 | 1.0 | 100 km-Earth radius |
| 30 min | 1e-9 | 129087.1 m | 126011.6 m | 156740.4 m | 114083.8-156740.4 m | 719.5/256.5 | 1.0 | 100 km-Earth radius |
| 45 min | 1e-9 | 69870.2 m | 66262.0 m | 89463.7 m | 60875.0-89463.7 m | 1070.8/385.2 | 2.2 | 10-100 km |
| 60 min | 1e-9 | 12680.4 m | 11683.6 m | 20594.8 m | 9294.8-20594.8 m | 1416.4/519.6 | 3.0 | 10-100 km |

## Clock-discipline curve

| label | fractional stability | drift | mean | p50 | p95 | spread | accepted/rejected mean | class |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| rubidium [UNVERIFIED] | 1e-11 | 0.002998 m/s | 129085.2 m | 126009.7 m | 156739.1 m | 114081.9-156739.1 m | 719.5/256.5 | 100 km-Earth radius |
| good OCXO [UNVERIFIED] | 1e-9 | 0.299792 m/s | 129087.1 m | 126011.6 m | 156740.4 m | 114083.8-156740.4 m | 719.5/256.5 | 100 km-Earth radius |
| poor reference [UNVERIFIED] | 1e-7 | 29.979246 m/s | 127758.2 m | 125064.7 m | 147317.7 m | 114279.3-147317.7 m | 797.6/178.4 | 100 km-Earth radius |

## Honest answers

- D55/D57 longer-leg check: p95 changes from 222206.0 m at 10 min to 20594.8 m at 60 min (signed improvement 201611.2 m).
- 500 m robustness: 0/5 leg tiers meet p50 <=500 m; 0/5 meet p95 <=500 m (the adopted D56 worst-case threshold is 750 m).
- Rb-vs-OCXO [UNVERIFIED labels]: at 30 min, p95 is 156739.1 m at 1e-11 versus 156740.4 m at 1e-9; signed Rb benefit 1.3 m.

## Controls

- Seeds: [3776782374, 3776782375, 3776782376, 3776782377, 3776782378, 3776782379, 3776782380, 3776782381]; individual endpoint errors are retained in `results.json`.
- Real path: production `Executive` and `FilterStub` EKF state versus truth.
- Gate: production chi-square threshold `Some(9.0)`; rejection counts above are measured integrity events.
- Geometry: At every Doppler epoch, the lowest-ID eight satellites above the 5-degree mask are used. Cohort handovers are counted; the same deterministic selection is used for every lever tier. This permits handovers because no fixed eight-SV cohort survives an endurance leg; duration therefore also changes accumulated handovers.
- Dynamics: constant commanded heading at 7 kn with wave/slam and speed-scaled IMU; no coordinated turn [UNVERIFIED].
- No formula, error clamp, target fitting, or replacement estimator is used.

## [UNVERIFIED] inputs

- Synthetic 1920-satellite three-MEO-shell TLE grid and lowest-ID visibility selection.
- 10/20/30/45/60 minute constant-heading leg choices and 30-second Doppler cadence.
- Injected receiver clock fractional stabilities: 1e-11 (Rb label), 1e-9 (good OCXO label), and 1e-7 (poor label); constant drift is a stand-in, not an oscillator stochastic model.
- Per-SV fixed transmit biases, deterministic measurement noise/outlier process, maritime IMU, wave/slam, and speed assumptions.
