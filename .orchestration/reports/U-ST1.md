# U-ST1 — endurance studies

## Outcome

Implemented `pnt-studies::endurance` and ran both requested eight-seed sweeps through
the production `Executive`, real `FilterStub` EKF, and production Doppler chi-square
gate (`Some(9.0)`). The measured curves are committed in
`docs/studies/endurance/results.json` and rendered in `STUDY.md`. No multisat or
highspeed source was edited.

This is a synthetic controlled experiment, not a validated performance prediction.
The constellation, oscillator labels and injected values, dynamics, noise, biases,
cadence, and leg choices are all marked `[UNVERIFIED]`. A fixed eight-satellite LEO
cohort could not survive endurance intervals. The final fixture therefore uses a
deterministic, synthetic three-MEO-shell grid and selects the lowest-ID eight
satellites above the production 5-degree mask at each epoch. Handovers are measured
and disclosed.

## Honest measured curves

All errors below are endpoint horizontal errors from EKF state versus generator truth.
Each tier contains the same eight deterministic seeds.

### Leg duration (clock injection fixed at 1e-9)

| leg | mean | p50 | p95 | spread | accepted/rejected mean | handovers mean |
|---:|---:|---:|---:|---:|---:|---:|
| 10 min | 180,340.6 m | 175,579.9 m | 222,206.0 m | 154,782.6–222,206.0 m | 252.2/83.8 | 1.0 |
| 20 min | 158,913.8 m | 154,084.6 m | 198,493.0 m | 136,676.9–198,493.0 m | 487.5/168.5 | 1.0 |
| 30 min | 129,087.1 m | 126,011.6 m | 156,740.4 m | 114,083.8–156,740.4 m | 719.5/256.5 | 1.0 |
| 45 min | 69,870.2 m | 66,262.0 m | 89,463.7 m | 60,875.0–89,463.7 m | 1,070.8/385.2 | 2.2 |
| 60 min | 12,680.4 m | 11,683.6 m | 20,594.8 m | 9,294.8–20,594.8 m | 1,416.4/519.6 | 3.0 |

D55/D57's directional “longer legs help” claim holds strongly in this fixture:
10-to-60-minute p95 improves by 201,611.2 m (90.7%). It does not establish usable
accuracy. Zero of five tiers meets 500 m at p50, zero meets 500 m at p95, and zero
meets D56's 750 m p95 threshold. The 60-minute p95 remains 20.6 km. The duration
lever is also partly confounded by accumulated handovers, which rise at longer legs.

### Clock discipline (30-minute leg)

| label `[UNVERIFIED]` | fractional injection | mean | p50 | p95 | spread | accepted/rejected mean |
|---|---:|---:|---:|---:|---:|---:|
| rubidium | 1e-11 | 129,085.2 m | 126,009.7 m | 156,739.1 m | 114,081.9–156,739.1 m | 719.5/256.5 |
| good OCXO | 1e-9 | 129,087.1 m | 126,011.6 m | 156,740.4 m | 114,083.8–156,740.4 m | 719.5/256.5 |
| poor reference | 1e-7 | 127,758.2 m | 125,064.7 m | 147,317.7 m | 114,279.3–147,317.7 m | 797.6/178.4 |

The Rb-labelled tier buys only 1.3 m p95 relative to the OCXO-labelled tier
(approximately 0.0008%), which is negligible beside the 114–157 km seed spread.
This study therefore provides no evidence for paying for Rb over a good OCXO.

The poor-reference tier is paradoxically 9.4 km better at p95. It also changes the
gate outcome substantially (about 78 more accepted and 78 fewer rejected updates per
seed), so this is not evidence that a poor clock is physically superior. It is an
honest non-monotonic result showing that the constant clock-drift stand-in and
innovation gating interact; a stochastic oscillator truth/error model and captured
residual replay are needed before a BOM decision.

## Guardrails and verification

- Real EKF state is compared with generator truth; no formula, clamp, target fitting,
  replacement estimator, or truth-fed correction is used.
- The gate threshold is the production value `9.0`; both accepted and rejected
  integrity events are reported.
- Tests cover deterministic repeatability, production-default gate equality and a
  clock-stressed run that genuinely produces gate rejections.
- Individual seed endpoints are retained in JSON.
- D55/D57 are cross-referenced in the generated study.

## `[UNVERIFIED]` list

- Synthetic 1,920-satellite, three-MEO-shell TLE grid and deterministic cohort
  selection.
- Receiver clock fractional injections and the Rb/good-OCXO/poor labels. The
  injection is constant drift, not oscillator phase/frequency stochastic behavior.
- Constant-heading 10/20/30/45/60-minute legs, 30-second Doppler cadence, 7 kn speed,
  wave/slam and speed-scaled IMU parameters.
- Per-SV transmit biases and deterministic tracker noise/outlier process.

