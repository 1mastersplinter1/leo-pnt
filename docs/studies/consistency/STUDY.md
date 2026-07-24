# Covariance-consistency diagnosis (D68/D69)

**SYNTHETIC COVARIANCE-CONSISTENCY DIAGNOSIS [UNVERIFIED]. NEES is the real production Executive + real FilterStub EKF covariance/state (public API) versus generator truth. No value is clamped, formula-generated, or target-fitted. DIAGNOSIS ONLY: no estimator/executive/mission code is modified; testing an actual fix is out of scope.**

DIAGNOSIS ONLY. This study drives the real production `Executive` + real `FilterStub` EKF through their public API and reads the public `FilterState`/covariance to compute Normalized Estimation Error Squared (NEES) per state group against generator truth. It characterizes the D68/D69 overconfidence so the estimator fix (landed separately, serially) has a targeted spec; it does NOT modify or test the estimator.

Cross-reference: D43 (aided/short ~7x PESSIMISTIC covariance), D68 (long-denied 7-70x OVERCONFIDENT), D69 (endurance gate close). NEES > dof => overconfident; NEES < dof => pessimistic; NEES ~ dof => consistent.

## Fixture

- 960 satellites, synthetic [UNVERIFIED]. The verified endurance/multi-satellite study's 960-satellite three-shell synthetic LEO Walker grid, reproduced unchanged (private to endurance.rs, so copied not imported).
  - Starlink-class: ~550 km, 53.0 deg, 15.064 rev/day
  - OneWeb-class: ~1200 km, 87.9 deg, 13.158 rev/day
  - Iridium-class: ~780 km, 86.4 deg, 14.342 rev/day

## 1. NEES by state group

Cross-seed MEAN NEES per group in three windows (aided prime, denied early third, denied late/steady third) against the chi-square expectation (= dof) and the two-sided 95% consistency band for the mean over the seed set. The overconfidence factor is denied-late NEES / dof.

| group | dof | 95% band | aided | denied early | denied late | factor | verdict |
|---|---:|---|---:|---:|---:|---:|---|
| position | 3 | [1.5, 4.9] | 2.42 | 66.66 | 165.72 | 55.2x | OVERCONFIDENT |
| velocity | 3 | [1.5, 4.9] | 0.52 | 66.70 | 140.77 | 46.9x | OVERCONFIDENT |
| heading | 1 | [0.3, 2.2] | 0.00 | 0.00 | 0.00 | 0.0x | PESSIMISTIC |
| clock-drift | 1 | [0.3, 2.2] | 0.09 | 0.09 | 0.24 | 0.2x | PESSIMISTIC |
| aggregate | 8 | [5.5, 11.0] | 3.16 | 135.76 | 259.38 | 32.4x | OVERCONFIDENT |

## 2. Temporal / handover correlation

Does the inconsistency spike at handover epochs (per-SV-bias null-space hypothesis) or grow smoothly (clock-coupling / linearisation)?

- Handover epochs: 304, steady epochs: 656.
- Position handover spike ratio (detrended): 0.93.
- Aggregate handover spike ratio (detrended): 1.39.
- Aggregate NEES vs elapsed-time correlation: 0.75.
- Nuisance-count vs elapsed-time correlation: 0.99.

The dominant (position) overconfidence does NOT spike at handover (0.93x local trend, ~1); it grows SMOOTHLY with denial time (position/aggregate NEES-vs-time correlation 0.75), favouring a continuous mechanism (never-retired-bias accumulation, clock coupling, or linearisation) over discrete per-handover jumps. A secondary handover-correlated excursion IS present in the aggregate/velocity block (1.39x at handover), so a per-handover component exists alongside the smooth position growth. AMBIGUITY: the never-retired per-SV biases accumulate near-continuously (nuisance-count-vs-time correlation 0.99, one new SV roughly every epoch under sticky handover), so smooth-time growth and bias-count growth are collinear and cannot be told apart from this characterization alone -- the estimator-side retirement experiment (Section 5) is required to disambiguate.


## 3. Mechanism evidence (covariance structure)

- Position block is overconfident by construction of the finding: denied-late filter horizontal sigma averages 326 m (bounded, not km-scale) while the true horizontal error averages 3239 m -- the covariance stays small while the error does not.
- The most overconfident state group is position (denied-late NEES/dof 55.2x). Comparing per-group factors localizes where the covariance is most wrong.
- Per-SV nuisance-bias states are NEVER retired in the pipeline: their count grows from 8 to 70 over the denied leg (retire_satellite_bias is unit-test-only). Max nuisance variance moves from 2.2 to 26.9 m^2/s^2 (a fresh variance-100 state is minted per newly seen SV and only shrinks under updates); the growing augmented null-space is a direct estimation-consistency suspect.
- Position-clock-drift coupling: denied-late max |correlation| between a position axis and the clock-drift state is -0.04 (WEAK: the overconfidence is NOT concentrated in a position-clock cross term -- it lives in the position/velocity diagonal blocks, consistent with the accumulating per-SV-bias null-space rather than a two-state clock/position coupling); clock-drift sigma is 0.8448 m/s.
- Clock-bias variance sits at 7.18e6 m^2 (Doppler-unobservable; capped, excluded from NEES). It is inert for the position overconfidence but confirms the clock block carries an unobserved direction. Note the clock-drift group is itself PESSIMISTIC (NEES/dof < 1), so the fix must be group-specific, not a global inflation.

## 4. Reconcile with D43 (regime crossover)

- Aided position NEES/dof: 0.81.
- Denied-early position NEES/dof: 22.22.
- Denied-late position NEES/dof: 55.24.
- Crossover (position NEES/dof first > 1 in denial): ~420s.

RECONCILES D43 and D68 in one trace: aided-prime position NEES/dof is 0.81x (roughly consistent), while denied-late is 55.2x (OVERCONFIDENT). The crossover from consistent/pessimistic to overconfident occurs at elapsed ~420s in the denied leg. D43's pessimism (aided/short) and D68's overconfidence (long-denied) are the SAME covariance-consistency defect seen at two operating points: the filter's covariance does not track the true error as the observability regime shifts from tightly-aided to weakly-observable Doppler-only.


## 5. Estimator-fix spec

- States needing consistency correction: ["position (denied-late NEES/dof 55.2x)", "velocity (denied-late NEES/dof 46.9x)"].
- Per-SV bias retirement across handover implicated: true.
- Q retuning indicated: true.
- NEES-consistency correction indicated: true.

**Problem statement.** Over long GPS-denied LEO-Doppler legs the EKF is covariance-INCONSISTENT: the observable state groups ["position (denied-late NEES/dof 55.2x)", "velocity (denied-late NEES/dof 46.9x)"] report a covariance far tighter than their true error (position denied-late sigma ~326 m vs true error ~3239 m). The characterization localizes the overconfidence to the position block and implicates the never-retired per-SV nuisance-bias augmentation (count grows 8->70 over the leg, never retired). The position-clock-drift cross-covariance is WEAK (|corr| ~-0.04), so the overconfidence is NOT concentrated in a position-clock cross term; it sits in the position/velocity blocks themselves as they absorb Doppler information that the growing augmented bias null-space should have retained. The regime crossover from D43's aided pessimism (0.81x) to D68's denied overconfidence (55.2x) at ~420s confirms this is one consistency defect across operating points. Clock-drift and heading are separately PESSIMISTIC (NEES/dof < 1), so the correction must be state-group-specific, not a global covariance scale. The estimator fix must make the reported position/velocity covariance track the true error in the weakly-observable Doppler-only regime.


**Disambiguating estimator-side experiments (out of this diagnosis's scope -- require editing pnt-estimator):**

- Enable per-SV bias retirement (call retire_satellite_bias on handover / when an SV sets) in the estimator and re-run this NEES trace: if the denied-late position NEES/dof drops toward 1, the never-retired augmentation is the dominant mechanism. (Out of this diagnosis's scope -- requires editing pnt-estimator.)
- Freeze per-SV bias continuity across handover (carry the estimated bias + its covariance to the same physical SV rather than minting a fresh variance-100 state) and compare NEES: isolates continuity vs retirement.
- Sweep the propagation process noise (acceleration_variance, clock_drift_variance, nuisance_random_walk_variance) and measure whether the smooth NEES-vs-time growth flattens: distinguishes a Q-underfeeding (linearisation/coupling) mechanism from the augmentation mechanism.
- Add a NEES-consistency covariance inflation keyed to the measured denied-late factor and confirm the true error is unchanged while NEES/dof returns to ~1: verifies the correction fixes calibration without touching the (correct) point estimate.
- Repeat the NEES decomposition on a maneuvering (coordinated-turn) truth leg and with real (not synthetic) SoOP elements to check the finding is not an artifact of the constant-heading synthetic fixture [UNVERIFIED].

## Per-epoch NEES trace (cross-seed mean, sampled)

| elapsed (s) | phase | handover frac | pos NEES | agg NEES | true err (m) | sigma_h (m) | nuisance | pos-clk corr |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 30 | aided | 0.00 | 6.35 | 8.60 | 1 | 0 | 0 | 0.00 |
| 180 | aided | 0.00 | 2.29 | 2.94 | 0 | 0 | 0 | 0.00 |
| 330 | denied | 0.00 | 0.05 | 0.15 | 4 | 28 | 8 | 0.03 |
| 480 | denied | 0.00 | 4.04 | 14.20 | 43 | 52 | 10 | 0.04 |
| 630 | denied | 0.00 | 72.14 | 193.38 | 105 | 62 | 12 | 0.02 |
| 780 | denied | 0.00 | 52.59 | 102.06 | 242 | 73 | 14 | -0.04 |
| 930 | denied | 1.00 | 49.31 | 143.12 | 325 | 91 | 22 | -0.04 |
| 1080 | denied | 0.00 | 104.41 | 113.57 | 325 | 59 | 22 | -0.03 |
| 1230 | denied | 0.00 | 135.28 | 188.98 | 442 | 78 | 29 | 0.01 |
| 1380 | denied | 0.00 | 132.55 | 217.13 | 617 | 69 | 30 | -0.02 |
| 1530 | denied | 0.00 | 121.51 | 154.12 | 695 | 87 | 33 | 0.03 |
| 1680 | denied | 0.00 | 177.41 | 294.76 | 930 | 92 | 35 | -0.02 |
| 1830 | denied | 0.00 | 170.25 | 215.01 | 1108 | 104 | 37 | -0.01 |
| 1980 | denied | 0.00 | 160.41 | 197.32 | 1370 | 132 | 38 | -0.01 |
| 2130 | denied | 0.00 | 154.19 | 197.60 | 1694 | 110 | 39 | 0.01 |
| 2280 | denied | 0.00 | 157.24 | 304.94 | 2205 | 163 | 41 | -0.08 |
| 2430 | denied | 1.00 | 139.22 | 199.15 | 2699 | 236 | 43 | -0.10 |
| 2580 | denied | 1.00 | 121.90 | 212.65 | 3032 | 235 | 46 | -0.06 |
| 2730 | denied | 1.00 | 120.99 | 172.85 | 3462 | 278 | 48 | -0.06 |
| 2880 | denied | 1.00 | 157.87 | 182.95 | 4130 | 329 | 51 | -0.06 |
| 3030 | denied | 1.00 | 131.74 | 162.75 | 4383 | 392 | 54 | -0.06 |
| 3180 | denied | 0.00 | 104.07 | 188.41 | 4022 | 418 | 56 | -0.03 |
| 3330 | denied | 0.00 | 186.69 | 236.14 | 3479 | 404 | 57 | -0.04 |
| 3480 | denied | 0.00 | 161.08 | 285.77 | 2370 | 333 | 62 | -0.03 |
| 3630 | denied | 0.00 | 180.80 | 350.45 | 2102 | 224 | 64 | -0.02 |
| 3780 | denied | 0.00 | 199.04 | 375.72 | 2378 | 249 | 68 | -0.04 |

## Controls

- Seeds: [3776782374, 3776782375, 3776782376, 3776782377, 3776782378, 3776782379, 3776782380, 3776782381].
- Real path: production `Executive` and `FilterStub` EKF covariance/state (public API) versus truth.
- Gate: production chi-square threshold `Some(9.0)` (enabled).
- Geometry: Sticky best-eight-visible handover: hold lock until a satellite sets below the 5-degree mask, refill freed slots by GDOP; per-epoch GDOP stays well-conditioned so geometry is not a confound.
- Dynamics: constant commanded heading at 7 kn with speed-scaled IMU noise and horizontal bias; sub-second wave-slam disabled for long-leg truth stability; no coordinated turn [UNVERIFIED].
- Singular covariance sub-blocks skipped: 0.
- No formula, error clamp, target fitting, or replacement estimator is used; the estimator is not modified.

## [UNVERIFIED] inputs

- Synthetic 960-satellite three-shell LEO Walker grid; sticky best-N-visible handover selection.
- 60-minute constant-heading denied leg after 300s aided prime; 30s Doppler cadence, 30s NEES sampling cadence.
- Injected receiver clock fractional stability 1e-9 (constant common-mode drift stand-in), per-SV fixed transmit biases, deterministic measurement noise/outlier process, maritime IMU bias/noise.
- Truth clock drift = fractional * c; clock bias is Doppler-unobservable and excluded from every NEES group (variance tracked for mechanism only). Truth heading is the velocity course; the denied harness applies no heading measurement so heading is an unforced state (its very wide covariance makes its NEES near-zero/pessimistic by construction).
- Harness artifact shared with the endurance study (reproduced to characterize the SAME D68/D69 runs): the Doppler cohort is injected once per measurement envelope at each qualifying second, so seconds carrying more than one envelope apply slightly more than eight updates. This mildly inflates the absolute overconfidence factor; the qualitative finding (position/velocity many-x overconfident, clock/heading pessimistic, smooth growth) is robust to it and matches D68's 7-70x band.

## Honest scope limits

Because testing a fix requires editing `pnt-estimator` (out of scope -- a collaborator owns that file), this study characterizes the defect but cannot prove which remedy closes it. Section 5 lists the estimator-side experiments that would disambiguate the mechanism. Where the handover-vs-smooth attribution is not cleanly separable, the verdict says so rather than forcing a single cause.
