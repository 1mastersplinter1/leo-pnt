# Brief U-ST1 — Endurance studies: leg-duration + clock-discipline sweeps

Contract v5.1. Worktree branch `unit/U-ST1`. Commit there; never merge; NO trailers.
Read first: DECISIONS.md D55/D57/D62 (the two levers: longer legs, better clock), the
existing crates/pnt-studies (multisat + highspeed patterns — real EKF, production gate,
honest error_class, multi-seed), crates/pnt-mission, crates/pnt-estimator.

## Goal (extend pnt-studies with a NEW module `endurance` — do NOT edit multisat/highspeed
modules; only add your module + its bin + the member/mod lines):
1. **Leg-duration sweep**: on the real Executive+EKF with the production chi-square gate and
   the multi-sat cohort (reuse the multisat fixture/machinery via its public items, or a
   local realistic fixture), sweep constant-heading leg duration (e.g. 10/20/30/45/60 min)
   at N=8 sats, >=8 seeds each; measure denied position p50/p95 vs leg duration. Question:
   does the D55 "longer legs help" claim hold, and how much — does p95 drop below the 500m
   goal at longer legs? Report the curve honestly (it may plateau).
2. **Clock-discipline sweep**: hold leg duration fixed, sweep injected receiver-clock
   stability (e.g. rubidium ~1e-11, good OCXO ~1e-9, poor ~1e-7 fractional) in the truth
   generator; measure denied position vs clock quality. Question: how much does a better
   reference buy? This directly informs the BOM Rb-vs-OCXO choice with evidence.
3. **Honesty (mandatory)**: production gate on (rejection counts), real filter state vs
   truth, multi-seed stats (mean+p50+p95+spread), no formula/clamp, no target-fitting —
   report whatever the curves show incl. plateaus/no-improvement. Injected clock/leg values
   [UNVERIFIED]-marked. Cross-reference D55/D57.
4. **Tests**: determinism, the gate genuinely on, whole-workspace gate green.

## Files owned
crates/pnt-studies/src/endurance/** (or endurance.rs), crates/pnt-studies/src/bin/endurance-study.rs,
the endurance mod/member lines ONLY, docs/studies/endurance/**, .orchestration/reports/U-ST1.md.
If you must add a shared helper, put it in your module; do not refactor multisat/highspeed.

## Report
The two curves (leg-duration vs error, clock-quality vs error), the answer to "how robust is
500m / how much do the two levers buy", the Rb-vs-OCXO evidence, [UNVERIFIED] list.
