# U-ST1.2 — endurance study: revert grid + decisive cause experiment (D67)

## Outcome

The U-ST1.1 endurance headline was CONFIRMED HONEST but FAILED on the CAUSE
(D67). This round makes the two mandated fixes and lets the data — not an
assertion — decide the cause. Real `Executive` + `FilterStub` EKF versus
generator truth, production Doppler chi-square gate `Some(9.0)`, no
clamp/formula/target-fit in either direction. Deterministic: `results.json`
(schema 3) and `STUDY.md` are byte-identical on re-run. Scope: only
`crates/pnt-studies/src/endurance.rs` and `docs/studies/endurance/**` changed
(config-level in `endurance.rs` only; no estimator/mission/multisat edits).
Fixture, noise, clock, and dynamics remain `[UNVERIFIED]`.

## Fix 1 — grid reverted to the multisat 960-SV LEO fixture

The 768-SV "densified for coverage" variant is reverted to the verified
multi-satellite study's 960-satellite grid (D65 mandate). The only change is
`SHELL_SLOTS 16 -> 20`, which makes the endurance grid's orbital elements
identical to `multisat::synthetic_fixture` (16 planes x 20 slots x 3 shells,
half-slot inter-plane phasing, same inclinations/mean-motions). The empirically
false "960 grid leaves equatorial coverage gaps" justification is dropped from
the module doc, `STUDY.md`, and the regime string; the reviewer's measurement
(≥22 visible over 60 min, no gap) is stated instead. The `fixture_is_leo_not_meo`
guard is kept and still checks all 960 satellites.

## Fix 2 — the decisive experiment (settles the cause with data)

Added a **bias-zeroed control** (injected per-SV transmit bias `sv_bias_hz` set
to 0 in the truth generator, everything else identical), a **per-epoch
error-vs-time trace** for a representative seed at full-bias and bias-zeroed, and
**handover-epoch alignment**. Also added an **RMS-over-leg** metric alongside the
noisy single-epoch endpoint. The `conclusions()` text is generated from the
measured numbers and branches on them — no hard-coded cause.

### Bias-zeroed vs full-bias (RMS-over-leg p50, per leg)

| leg   | full-bias RMS p50 | bias-zeroed RMS p50 | ratio |
|------:|------------------:|--------------------:|------:|
| 10min | 86 m              | 86 m                | 1.00  |
| 20min | 236 m             | 236 m               | 1.00  |
| 30min | 271 m             | 271 m               | 1.00  |
| 45min | 866 m             | 866 m               | 1.00  |
| 60min | 2136 m            | 2135 m              | 1.00  |

Zeroing the injected per-SV bias moves the error by <0.1 m at every leg. The
estimator's per-SV nuisance-bias augmentation (variance 100, ~10 Hz σ) absorbs a
constant ~1 Hz transmit bias completely, so the bias is irrelevant to the
position solution.

### Per-epoch / handover alignment (representative seed, 60 min)

Mean error at the 38 handover epochs = 148 m vs 138 m at steady epochs —
comparable, **no handover-induced spikes**. Error grows over the leg
(start-third 164 m → end-third 196 m), the signature of accumulation with time,
not of handover events.

## DATA-DECIDED CAUSE

**RETRACTED:** "km-scale is caused by per-SV transmit-bias re-convergence across
handovers." The bias-zeroed control refutes it decisively (ratio 1.00) and the
handover alignment shows no handover spikes.

**EVIDENCED:** the limiter is **fundamental weak Doppler-only position
observability of a slow (7 kn) receiver once the aided prior decays**. Under the
stable RMS-over-leg metric, error grows MONOTONICALLY with denial time (86 m at
10 min → 2136 m at 60 min) — error *rises* with sustained denial, the opposite
of the endpoint metric's artifactual "improvement with leg length" (the endpoint
is a noisy single-epoch sample; 60-min seed spread 301–15350 m). The km-scale
500-m-not-met headline for long endurance legs HOLDS and is now correctly
attributed.

## Is 500 m reachable by any lever?

- **Leg duration:** no — longer legs are WORSE (accumulation). Short legs
  (≤~30 min) hold hundreds of m RMS but that is not sustained endurance.
- **Clock:** no — 1e-11 vs 1e-9 differ by ~5 m (common-mode absorbed by the
  filter). A 1e-7 poor clock does hurt (+11.6 km p95), so a poor oscillator is a
  downside but upgrading a good clock is not an up-lever.
- **Bias-continuity estimator fix:** no — the control shows bias is already
  absorbed; fixing bias continuity across handover would change nothing.
- **What could:** position-observable aiding, a faster/maneuvering platform
  (stronger Doppler-position observability), shorter re-aiding intervals, or
  accepting km-scale denied nav. This is a fundamental observability limit, not
  an estimation bug.

## Metric change

The headline now leads with **RMS-over-leg** (stable, monotone) and reports the
single-epoch endpoint alongside it, explicitly labelled as noisy. The prior
"bimodality" is explained as endpoint sampling luck against a uniformly km-scale
leg-averaged solution (60-min RMS p95 6220 m).

## Gate

`cargo test` (42 groups ok), `cargo clippy --all-targets -D warnings`, and
`cargo fmt --all --check` all pass. New test `bias_zeroed_control_is_a_real_perturbation`
guards the control is a genuine counterfactual.
