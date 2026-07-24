# Adversarial Review — Unit U-RT1.1 (real-SupGP constellation geometry check)

**Worktree** `/home/od/work/leo-pnt-wt-URT1b`, branch `unit/U-RT1`, commit `12dee3d`. Gate reproduced: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, and all 6 `realtle` tests **PASS**; the study binary regenerates `results.json`/`STUDY.md` **byte-identical** to the committed files (determinism confirmed). No attribution trailers on the commit. Worktree left clean.

## Machinery honesty — verified GOOD

- **Real EKF, not a toy.** `FilterStub` (crates/pnt-estimator/src/lib.rs:87) is the production error-state EKF renamed for source-compat ("Kept under the historical name…"), with covariance propagation, nuisance/clock augmentation, and Doppler updates — the same filter as the D57 multisat study. Not the D51 clamped `PassageEstimator`. No clamp/formula/target-fit.
- **Production gate on.** `chi_square_threshold = Some(9.0)` wired to the pipeline (mod.rs:443), asserted by `production_gate_is_on`.
- **No N=8 target-fit — honestly reported N=7 max.** The negative result is real: `fixture_size_and_n7_cohort_are_locked` (mod.rs:1137) asserts N=8 is *unreachable* across all 8 seeds. Genuine honesty credit.
- **Three review fixes all present and tested.** F1 (R4 mis-attribution) corrected in report/comments/tests (mod.rs:1110); F2 (cohort-size lock) implemented (mod.rs:1137); F3 (IDs derived at runtime) — `fixture_satellite_ids` parses NORAD numbers from the fixture text (mod.rs:720), no hardcoded list remains.
- `[UNVERIFIED]` discipline on TLE/SupGP currency, synthetic dynamics/clock/noise, and representativeness is thorough.

## CENTRAL FINDING — the 36–52 m is DEAD-RECKONING COAST, and the study overclaims it as real geometry

**Severity: CRITICAL (verdict-determining). Confidence: HIGH — reproduced quantitatively.**

I instrumented a temporary copy of `simulate` to record filter position error at every denied-leg epoch (t=300→600 s), seed[0], reverted after. Endpoint values matched `results.json` exactly (N=1 → 88.16 m = seed_position_errors_m[0]; N=7 → 39.62 m), so the trace is the study's own filter behavior:

| t (s) | N=0 INS-only (my control) | N=1 (unobservable) | N=7 |
|---|---|---|---|
| 300 (denial) | 0.29 | 0.29 | 0.29 |
| 360 | 20.7 | 20.4 | 18.2 |
| 450 | 50.6 | 49.1 | 51.4 |
| 540 | 80.0 | 73.0 | 36.4 |
| 600 (endpoint) | **99.45** | **88.16** | **39.62** |

This settles every sub-question in the brief:

- **(a) Smoking gun confirmed.** The prior at denial is **sub-meter** (~0.3 m — 300 s of aided GPS at 0.5 m noise). Error **GROWS monotonically** over the leg; it never converges. N=1 (infinite GDOP, position-unobservable) tracks the pure-INS coast almost exactly (88 m vs 99 m). The endpoint numbers are a *growing coast trajectory sampled at t=600 s*, not an observability floor. This is inertial/DR coast from a GPS-good prior, exactly as the brief suspected.
- **(b) Geometry-independence confirmed at the decision boundary.** A **zero-satellite INS-only run reaches 99 m — still "<100 m," still under the 500 m target.** N=1-unobservable = 52.5 m mean; N=7 = 35.8 m. Satellites *modestly arrest* the coast (N=7 bends down after ~450 s where INS-only runs to 99 m), roughly halving the endpoint — real but secondary. GDOP ranges infinite→8.75 while the pass/fail against 500 m is **independent of geometry**: every tier, including unobservable N=1 and zero-sat INS, clears it. That is the coast-dominated signature.
- **(c) Short-leg prior-dominated regime confirmed.** 5-min leg from a sub-meter prior — the exact D51/D55/U-MS1/U-H2 confound class, **not** the D55/D57 10–20 min regime where the prior has decayed and Doppler *must* solve position.
- **(d) Structurally coast-limited.** N=7 cap + short SupGP validity + a 5-min leg cannot test multi-sat observability. The study should say this plainly.

**Where it overclaims (the FAIL):**
- STUDY.md:7 and results.json:166–167 (headline/diagnosis) and mod.rs `diagnose` (633–634): "**N=7 on REAL Starlink geometry reaches the D56 usable denied target** … mean 35.8 m / p95 56.4 m."
- STUDY.md:30 / markdown verdict (mod.rs:906): "**real orbital geometry does not undermine the synthetic finding**."
- Report U-RT1.md:131–132: "N=7 reaches the D56 usable denied target **with wide margin**, and is **numerically better than the synthetic multisat N=8**."

These attribute a target-meeting result to *real multi-sat geometry/observability*. The study's own N=1-unobservable row (results.json:48) — and the zero-sat control it never ran — show the target is met by the prior+coast regardless of the satellites. **Verify:** counts=[1,2,4,7] never includes an INS-only/N=0 baseline (mod.rs:80); endpoint-only error, no error-vs-time (mod.rs:539–545); reproduce the trace above.

Tellingly, the **superseded U-RT1 section already had the correct diagnosis** ("N=1/N=2 … largely reflect short-leg inertial propagation aided by underdetermined Doppler," U-RT1.md:45–46) — U-RT1.1 dropped that framing from the N=7 headline and replaced it with a geometry claim.

## Supporting finding — the GDOP-decoupling flag mis-diagnoses the confound

**Severity: HIGH. Confidence: HIGH.** The study flags that real GDOP p95 (13.86) is worse than synthetic (~1.8) yet real error is smaller, and reports it as an unexplained "[UNVERIFIED] open observation … reported, not explained away" (STUDY.md:9,30; mod.rs:629). The multisat and realtle runs are the **same experiment** — byte-identical stress (RECEIVER_CLOCK_DRIFT_MPS=0.03, AIDED_S=300, denied=300, cadence=30, identical `sv_bias_hz`/12 Hz outliers/seeds), differing **only** in the ephemeris fixture (multisat.rs:30–35,50–51 vs mod.rs). A pure-geometry metric failing to predict error is not mysterious: on a coast/nuisance-dominated short leg the endpoint isn't a geometry measurement at all, so GDOP *cannot* predict it and the cross-fixture "real 36 m < synthetic 554 m" comparison is not an observability comparison. The Sonnet flag smells the right thing but stops at "puzzle" instead of naming the coast confound — it does **not** go far enough. (Note the synthetic multisat study was itself honest here: multisat.rs:522 and its controls note "the five-minute denied leg … is not endurance evidence.")

## Minor findings

- **LOW — "gate on via rejection counts" not evidenced.** `rejected_updates_mean = 0.0` in every tier (results.json). The gate is configured on, but the injected 12 Hz outliers never trip it on this short leg, so — unlike the D55/D57 emphasis — rejection counts do not *demonstrate* the gate is live here. The gate-on assertion rests on the config/test, not on observed rejections.
- **LOW — duplicated per-epoch processing.** Each 30 s Doppler epoch fires ~5× (multiple journal records share the integer second), so `accepted` counts are 55/110/220/385 (= 55×N). Deterministic and consistent across tiers; does not affect the coast conclusion, but the "every 30 s" cadence description slightly understates the actual update rate.
- **INFO — referenced prior review absent.** `.orchestration/reports/U-RT1-review-opus.md` (cited by the brief as "the prior PASS of the machinery") does not exist in the worktree or on `main`. `[UNVERIFIED]` — the machinery PASS could not be cross-read; I verified the machinery directly instead. D64 (also cited) does not exist; DECISIONS.md ends at D63.

## Verdict rationale

The brief's FAIL basis is met exactly: the 36–52 m band is coast-dominated (error grows from a sub-meter prior; unobservable N=1 reaches 52 m; a zero-sat INS-only run reaches 99 m and would "pass"; pass/fail is independent of GDOP). Reporting "real N=7 meets the 500 m goal on real Starlink geometry / does not undermine the synthetic finding" **overclaims** — the study demonstrates short-leg DR coast, not multi-sat Doppler position observability, and does not say so plainly (the correct diagnosis exists only in the superseded section, and the GDOP-decoupling flag mis-frames it as an unexplained puzzle). To PASS, the headline/diagnosis/verdict must state that the result is coast-dominated over a 5-min leg from a GPS-good prior, that even unobservable/zero-satellite geometries meet the 500 m target, and that this fixture+leg cannot test multi-sat observability — as U-MS1.1 eventually did.

FAIL