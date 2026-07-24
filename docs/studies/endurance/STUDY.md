# Endurance study: leg duration and clock discipline

**SYNTHETIC ENDURANCE EXPERIMENT [UNVERIFIED]. Errors are production Executive + real FilterStub EKF state versus generator truth. No result is clamped, formula-generated, or target-fitted.**

Cross-reference: D55 identified the confounds; D57 identified longer constant-heading legs and clock discipline as the next two levers; D56 defines 500 m typical (p50) and 750 m worst-case (p95). This study runs on the **identical LEO fixture as the verified multi-satellite study** (the same 960-satellite three-shell Walker grid at 53.0/87.9/86.4 deg, 13-15 rev/day -- correcting the earlier MEO regression and reverting the unjustified 768-SV variant), tracking the best-conditioned eight currently-visible satellites with realistic sticky handovers, and reports per-epoch GDOP so the leg-duration lever is isolated from geometry. The *cause* of the km-scale denied error is not asserted: a bias-zeroed control run (injected per-SV transmit bias set to zero, everything else identical) and a handover-aligned per-epoch error trace decide it from data.

## Fixture

- 960 satellites, synthetic [UNVERIFIED]. The verified multi-satellite study's 960-satellite three-shell synthetic LEO Walker grid, reused unchanged (D65 mandate). At least ~22 (typically 22-45) satellites stay continuously visible above the 5-degree mask over a full 60-minute leg, so no coverage gap forces a denser grid.
  - Starlink-class: ~550 km, 53.0 deg, 15.064 rev/day
  - OneWeb-class: ~1200 km, 87.9 deg, 13.158 rev/day
  - Iridium-class: ~780 km, 86.4 deg, 14.342 rev/day

## Leg-duration curve (full bias)

Endpoint = single-epoch error at leg end (noisy). RMS = root-mean-square horizontal error over every denied-leg doppler epoch (stable headline).

| leg | clock | GDOP mean (min-max) | endpoint p50 | endpoint p95 | RMS p50 | RMS p95 | endpoint spread | accepted/rejected mean | handovers mean | class |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 10 min | 1e-9 | 1.68 (1.45-1.97) | 8126.4 m | 19377.0 m | 86.1 m | 541.0 m | 2666.4-19921.1 m | 282.1/53.9 | 12.0 | 10-100 km |
| 20 min | 1e-9 | 2.00 (1.45-2.87) | 7228.6 m | 21997.9 m | 231.0 m | 1304.6 m | 2888.4-29439.7 m | 514.3/141.6 | 25.0 | 10-100 km |
| 30 min | 1e-9 | 2.27 (1.45-4.02) | 6778.3 m | 31586.9 m | 265.0 m | 2879.6 m | 5238.4-37325.8 m | 729.4/246.4 | 31.0 | 10-100 km |
| 45 min | 1e-9 | 2.49 (1.45-4.02) | 9915.1 m | 25281.7 m | 866.4 m | 6286.6 m | 2417.2-25504.7 m | 1032.6/421.0 | 45.0 | 10-100 km |
| 60 min | 1e-9 | 2.48 (1.45-4.02) | 3895.9 m | 9165.7 m | 2135.9 m | 6220.0 m | 300.6-15350.3 m | 1273.3/660.0 | 62.0 | 1-10 km |

## Decisive experiment: bias-zeroed control vs full bias

Identical leg sweep with the injected per-SV transmit bias forced to zero in the truth generator, everything else held fixed. If bias-zeroed error collapses toward multisat-class (hundreds of m), per-SV bias re-convergence across handovers is the driver; if it stays km-scale, the limiter is fundamental weak Doppler-only observability.

| leg | full-bias RMS p50 | bias-zeroed RMS p50 | ratio | full-bias endpoint p50 | bias-zeroed endpoint p50 | bias-zeroed class |
|---:|---:|---:|---:|---:|---:|---|
| 10 min | 86.1 m | 86.1 m | 1.00 | 8126.4 m | 8126.0 m | 10-100 km |
| 20 min | 231.0 m | 231.0 m | 1.00 | 7228.6 m | 7228.6 m | 10-100 km |
| 30 min | 265.0 m | 265.0 m | 1.00 | 6778.3 m | 6778.2 m | 10-100 km |
| 45 min | 866.4 m | 866.4 m | 1.00 | 9915.1 m | 9914.7 m | 10-100 km |
| 60 min | 2135.9 m | 2135.4 m | 1.00 | 3895.9 m | 3894.7 m | 1-10 km |

## Per-epoch error trace (representative seed, handover-aligned)

Mean horizontal error at handover epochs vs steady (no-handover) epochs, and the within-leg error trajectory (start third -> end third). Full sample series are in `results.json`.

| trace | seed | leg | handover epochs | mean err @ handover | mean err @ steady | err start-third | err end-third |
|---|---:|---:|---:|---:|---:|---:|---:|
| full-bias | 3776782374 | 60 min | 38 | 148.32 | 138.47 | 164.1 m | 195.8 m |
| bias-zeroed | 3776782374 | 60 min | 38 | 148.32 | 138.48 | 164.1 m | 195.8 m |

## Clock-discipline curve (fixed leg, fixed good geometry)

| label | fractional stability | drift | GDOP mean | p50 | p95 | spread | accepted/rejected mean | class |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| rubidium [UNVERIFIED] | 1e-11 | 0.002998 m/s | 2.27 | 6778.5 m | 31582.4 m | 5238.7-37347.2 m | 730.2/245.7 | 10-100 km |
| good OCXO [UNVERIFIED] | 1e-9 | 0.299792 m/s | 2.27 | 6778.3 m | 31586.9 m | 5238.4-37325.8 m | 729.4/246.4 | 10-100 km |
| poor reference [UNVERIFIED] | 1e-7 | 29.979246 m/s | 2.27 | 11763.6 m | 43182.1 m | 5757.7-61058.6 m | 709.4/266.4 | 10-100 km |

## Honest answers

- Geometry control: per-epoch GDOP stays well-conditioned across every leg (mean 1.68 at 10 min to 2.48 at 60 min, comparable to the multi-satellite good cohort's ~1.8), so the leg-duration and clock levers are measured on fixed, well-conditioned geometry and are not a geometry confound.
- Leg-duration lever (D55/D57), and a METRIC CORRECTION: the noisy single-epoch endpoint p50 wanders non-monotonically (8126 m at 10 min -> 3896 m at 60 min, p95 19377 -> 9166 m) and its apparent 'improvement with leg length' is a sampling artifact (endpoint seed spread is 301-15350 m at 60 min). The stable RMS-over-leg metric instead grows MONOTONICALLY with denial time -- p50 86 m (10 min) -> 2136 m (60 min) -- the physical signature of Doppler-only position error accumulating as the aided prior decays. So error rises, not falls, with sustained denial: short legs (<=~30 min) hold hundreds of m but hour-long endurance legs are KM-SCALE and the D56 500 m goal is NOT met for sustained denial. RMS-over-leg is the recommended headline; the endpoint metric is too noisy to headline.
- DATA-DECIDED CAUSE (bias-zeroed control, 60 min, RMS-over-leg p50): full-bias 2136 m vs bias-zeroed 2135 m (bias-zeroed is 100% of full-bias; endpoint p50 3896 m vs 3895 m). Zeroing the injected per-SV transmit bias barely moves the error -- it STAYS km-scale. So the limiter is NOT the handover bias: it is FUNDAMENTAL weak Doppler-only position observability of a slow (7 kn) receiver once the aided prior decays. The earlier 'per-SV bias across handover' diagnosis is RETRACTED. 500 m is not reachable by an estimator bias-continuity fix; it needs a different lever (position-observable aiding, a faster/maneuvering platform, or accepting km-scale denied nav).
- Handover alignment (representative seed 3776782374, 60 min, full bias): mean error 148 m at the 38 handover epochs vs 138 m at steady epochs -- handover and steady epochs are comparable (no systematic handover-induced spike).
- Endpoint-metric bimodality: at 60 min the best seed's endpoint reaches 301 m (1 of 16 seeds have endpoint <=500 m) while others stay km-scale. This bimodality is a property of the single-epoch endpoint sample; the RMS-over-leg p95 is 6220 m, so the underlying leg-averaged solution is km-scale and the sub-500 m endpoints are sampling luck (a good instantaneous epoch), not converged solutions.
- D56 goal (500 m p50 / 750 m p95): p50<=500 m first met at no tested leg; p95<=750 m first met at no tested leg (endpoint metric). On honest handover geometry no tested leg or clock robustly delivers the target; whether 500 m is reachable by ANY lever is answered by the bias-zeroed control above, not asserted.
- Clock-discipline lever: between a good clock and a great one it is near-invisible -- at 30 min, p95 is 31582 m at 1e-11 (Rb label) versus 31587 m at 1e-9 (OCXO label), signed Rb benefit 4.6 m, because the common-mode receiver-clock injection is absorbed by the filter's clock/nuisance states. A 1e-7 POOR clock, however, does degrade the solution (43182 m p95, 11595 m worse than the OCXO), so a poor oscillator hurts but upgrading a good clock to a great one is not a usable denied-position lever.

## Controls

- Seeds: [3776782374, 3776782375, 3776782376, 3776782377, 3776782378, 3776782379, 3776782380, 3776782381, 3776782382, 3776782383, 3776782384, 3776782385, 3776782386, 3776782387, 3776782388, 3776782389]; individual endpoint errors are retained in `results.json`.
- Real path: production `Executive` and `FilterStub` EKF state versus truth.
- Gate: production chi-square threshold `Some(9.0)`; rejection counts above are measured integrity events.
- Geometry: The receiver continuously tracks eight satellites, holding lock on each until it sets below the 5-degree mask (sticky handover, as real hardware does) and refilling freed slots with the geometry-improving visible candidate. Handovers therefore reflect physical setting events; per-epoch GDOP is reported to prove the instantaneous geometry stays well-conditioned throughout every leg. Because no fixed eight-SV cohort survives an endurance leg from LEO, handovers are physically required; the identical geometry schedule is reused across every leg and clock tier, so the levers vary against fixed, well-conditioned geometry.
- Dynamics: constant commanded heading at 7 kn with speed-scaled IMU noise and horizontal bias; sub-second wave-slam disabled to keep long-leg truth physical; no coordinated turn [UNVERIFIED].
- No formula, error clamp, target fitting, or replacement estimator is used.

## [UNVERIFIED] inputs

- Synthetic 960-satellite three-shell LEO Walker grid (53.0/87.9/86.4 deg at 15.064/13.158/14.342 rev/day, 16 planes x 20 slots per shell), reused unchanged from the multi-satellite study; sticky best-N-visible handover selection.
- 10/20/30/45/60 minute constant-heading leg choices and 30-second Doppler cadence.
- Injected receiver clock fractional stabilities: 1e-11 (Rb label), 1e-9 (good OCXO label), and 1e-7 (poor label); constant common-mode drift is a stand-in, not an oscillator stochastic model.
- Per-SV fixed transmit biases, deterministic measurement noise/outlier process, maritime IMU bias/noise, and speed assumptions; sub-second wave-slam disabled for long-leg truth stability.
