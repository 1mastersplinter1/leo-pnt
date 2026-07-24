# Endurance study: leg duration and clock discipline

**SYNTHETIC ENDURANCE EXPERIMENT [UNVERIFIED]. Errors are production Executive + real FilterStub EKF state versus generator truth. No result is clamped, formula-generated, or target-fitted.**

Cross-reference: D55 identified the confounds; D57 identified longer constant-heading legs and clock discipline as the next two levers; D56 defines 500 m typical (p50) and 750 m worst-case (p95). This study runs on the **same LEO regime as the verified multi-satellite study** (three shells at 53.0/87.9/86.4 deg, 13-15 rev/day -- correcting the earlier MEO regression), on a Starlink-scale grid densified for genuinely continuous 60-minute coverage, tracking the best-conditioned eight currently-visible satellites with realistic sticky handovers, and reports per-epoch GDOP so the leg-duration lever is isolated from geometry.

## Fixture

- 768 satellites, synthetic [UNVERIFIED]. Starlink-scale synthetic LEO megaconstellation in the same regime as the verified multi-satellite study; grid densified so >=8 (typically 18-40) satellites stay continuously visible above the 5-degree mask over a full 60-minute leg.
  - Starlink-class: ~550 km, 53.0 deg, 15.064 rev/day
  - OneWeb-class: ~1200 km, 87.9 deg, 13.158 rev/day
  - Iridium-class: ~780 km, 86.4 deg, 14.342 rev/day

## Leg-duration curve

| leg | clock | GDOP mean (min-max) | p50 | p95 | spread | accepted/rejected mean | handovers mean | class |
|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 10 min | 1e-9 | 2.11 (1.47-3.06) | 8551.7 m | 31519.5 m | 3112.6-31813.1 m | 271.8/64.2 | 10.0 | 10-100 km |
| 20 min | 1e-9 | 2.35 (1.47-5.27) | 6837.4 m | 17552.0 m | 3014.4-26311.5 m | 488.5/167.5 | 20.0 | 10-100 km |
| 30 min | 1e-9 | 2.50 (1.47-5.27) | 13535.4 m | 24961.9 m | 4375.7-36485.6 m | 708.1/267.2 | 32.0 | 10-100 km |
| 45 min | 1e-9 | 2.65 (1.47-5.27) | 4316.8 m | 19648.5 m | 1980.2-21558.5 m | 984.3/470.3 | 48.0 | 10-100 km |
| 60 min | 1e-9 | 2.73 (1.47-7.29) | 5067.1 m | 9250.6 m | 33.3-15214.5 m | 1264.0/670.6 | 65.0 | 1-10 km |

## Clock-discipline curve (fixed leg, fixed good geometry)

| label | fractional stability | drift | GDOP mean | p50 | p95 | spread | accepted/rejected mean | class |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| rubidium [UNVERIFIED] | 1e-11 | 0.002998 m/s | 2.50 | 13535.1 m | 24966.6 m | 4375.4-36489.2 m | 708.1/267.2 | 10-100 km |
| good OCXO [UNVERIFIED] | 1e-9 | 0.299792 m/s | 2.50 | 13535.4 m | 24961.9 m | 4375.7-36485.6 m | 708.1/267.2 | 10-100 km |
| poor reference [UNVERIFIED] | 1e-7 | 29.979246 m/s | 2.50 | 11252.6 m | 25871.8 m | 3092.0-38651.6 m | 712.7/262.9 | 10-100 km |

## Honest answers

- Geometry control: per-epoch GDOP stays well-conditioned across every leg (mean 2.11 at 10 min to 2.73 at 60 min, comparable to the multi-satellite good cohort's ~1.8), so the leg-duration and clock levers are measured on fixed, well-conditioned geometry and are not a geometry confound.
- Leg-duration lever (D55/D57): on realistic continuous-handover geometry the denied endpoint error improves with leg length on average but noisily -- p50 8552 m -> 5067 m and p95 31520 m -> 9251 m from 10 to 60 min (p95 improvement 22269 m), with a non-monotonic mid-leg tier because the endpoint metric samples the instantaneous handover geometry.
- Convergence is BIMODAL: at 60 min the best seeds reach 33 m (5 of 16 seeds are <=500 m, the D56 p50 target) while others remain km-scale, so tight denied position is achievable but not reliable -- outcome depends on the handover sequence.
- D56 goal (500 m p50 / 750 m p95): p50<=500 m first met at no tested leg; p95<=750 m first met at no tested leg. Neither lever, on honest handover geometry, robustly delivers the target -- the fixed-cohort 116 m / 554 m does NOT transfer to sustained endurance because continuous handover keeps per-SV bias observability from converging.
- Clock-discipline lever: near-invisible. At 30 min, p95 is 24967 m at 1e-11 (Rb label) versus 24962 m at 1e-9 (OCXO label); signed Rb benefit -4.7 m. Even a 1e-7 poor clock barely moves the result. The common-mode receiver-clock injection is absorbed by the filter's clock/nuisance states, so clock choice is not a usable BOM lever for denied position here.

## Controls

- Seeds: [3776782374, 3776782375, 3776782376, 3776782377, 3776782378, 3776782379, 3776782380, 3776782381, 3776782382, 3776782383, 3776782384, 3776782385, 3776782386, 3776782387, 3776782388, 3776782389]; individual endpoint errors are retained in `results.json`.
- Real path: production `Executive` and `FilterStub` EKF state versus truth.
- Gate: production chi-square threshold `Some(9.0)`; rejection counts above are measured integrity events.
- Geometry: The receiver continuously tracks eight satellites, holding lock on each until it sets below the 5-degree mask (sticky handover, as real hardware does) and refilling freed slots with the geometry-improving visible candidate. Handovers therefore reflect physical setting events; per-epoch GDOP is reported to prove the instantaneous geometry stays well-conditioned throughout every leg. Because no fixed eight-SV cohort survives an endurance leg from LEO, handovers are physically required; the identical geometry schedule is reused across every leg and clock tier, so the levers vary against fixed, well-conditioned geometry.
- Dynamics: constant commanded heading at 7 kn with speed-scaled IMU noise and horizontal bias; sub-second wave-slam disabled to keep long-leg truth physical; no coordinated turn [UNVERIFIED].
- No formula, error clamp, target fitting, or replacement estimator is used.

## [UNVERIFIED] inputs

- Synthetic 768-satellite three-shell LEO Walker grid (53.0/87.9/86.4 deg at 15.064/13.158/14.342 rev/day) and sticky best-N-visible handover selection.
- 10/20/30/45/60 minute constant-heading leg choices and 30-second Doppler cadence.
- Injected receiver clock fractional stabilities: 1e-11 (Rb label), 1e-9 (good OCXO label), and 1e-7 (poor label); constant common-mode drift is a stand-in, not an oscillator stochastic model.
- Per-SV fixed transmit biases, deterministic measurement noise/outlier process, maritime IMU bias/noise, and speed assumptions; sub-second wave-slam disabled for long-leg truth stability.
