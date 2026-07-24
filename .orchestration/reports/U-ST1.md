# U-ST1 — endurance studies (current state after fix round U-ST1.3)

This report is rewritten in place at each fix round per the brief's file
ownership (`.orchestration/reports/U-ST1.md`); prior-round detail lives in
`.orchestration/reports/U-ST1.1-review-opus.md`, `U-ST1.2.md` /
`U-ST1.2-review-opus.md`, and `DECISIONS.md` D65/D67/D68. This version
reflects the CURRENT, CORRECT state after U-ST1.3's verdict reframe.

## Outcome

The endurance study runs the leg-duration and clock-discipline sweeps through
the production `Executive` + real `FilterStub` EKF, on the identical
960-satellite three-shell LEO Walker fixture as the verified multi-satellite
study (D65 mandate), with continuous best-N sticky-handover tracking and
per-epoch GDOP instrumentation. Two rounds of adversarial review (D65, D67)
fixed a geometry confound and a false coverage-gap claim; a third round (D68,
the CRUX result) found the study's own headline conclusion — "fundamental
observability limit, not an estimation bug" — was itself an over-claim. This
round (U-ST1.3) reframes the verdict to the verified truth and adds a
covariance-consistency metric that reproduces it directly in this study's own
data.

## The reframed verdict (D68, now the headline here)

**KEPT (still holds, unchanged numbers):** on realistic LEO handover geometry
(GDOP 1.68–2.48, well-conditioned, per-epoch instrumented), sustained denied
navigation over long legs is KM-SCALE — the D56 500 m p50 / 750 m p95 goal is
NOT met on any tested leg duration or clock discipline. This is NOT caused by
the injected per-SV bias *value* (bias-zeroed control ratio 1.00, unchanged).
It is NOT fixed by leg duration (RMS-over-leg grows monotonically with denial
time, 86 m at 10 min → 2136 m at 60 min) or by clock quality (Rb vs OCXO ±5 m
on a multi-km error; only a genuinely poor 1e-7 clock hurts).

**CORRECTED (this round's fix):** the prior conclusion — "the limiter is
FUNDAMENTAL weak Doppler-only position observability … not an estimation
bug" — is WRONG and has been removed everywhere (`STUDY.md`, `results.json`
diagnosis/headline, `bias_control_verdict`/`conclusions` logic in
`endurance.rs`). The verified cause (D68): position **is** weakly observable —
the filter's own reported horizontal sigma (from its covariance) converges
and stays bounded, order ~50–160 m, actively shrinking at handover-dense
epochs — proof the Doppler-curve position mechanism works. But the filter is
**overconfident**: true horizontal error runs several to tens of times that
sigma and keeps growing while the sigma does not. A genuine physics floor
would show the covariance itself growing to km-scale to match the error; it
does not. So the km-scale denied error is **filter inconsistency /
covariance overconfidence — an ESTIMATION problem**, not a fundamental
LEO-Doppler observability limit. The fix (per-SV bias continuity/retirement
across handover, covariance-consistency correction, Q retuning) lives in the
**estimator**, out of this config-only study's scope. Cross-reference D43:
the opposite-direction ~7x PESSIMISTIC covariance found on aided/short legs
— covariance consistency, in both directions, is the recurring central
estimation gap, not a leg-duration or clock lever.

## New: covariance-consistency metric (this round's addition)

For the representative seed (3776782374, 60 min leg, full bias), the filter's
own reported horizontal sigma is now recorded alongside the true horizontal
error at every 30 s Doppler epoch (`EpochSample.sigma_horizontal_m` in
`results.json`, and a spaced table in `STUDY.md`). Selected epochs:

| elapsed (s) | filter sigma (m) | true error (m) | ratio |
|---:|---:|---:|---:|
| 300 | 0.5 | 0.6 | 1.2x |
| 900 | 81.6 | 521.4 | 6.4x |
| 1800 | 65.0 | 141.7 | 2.2x |
| 2700 | 45.7 | 54.7 | 1.2x |
| 3600 | 86.0 | 281.6 | 3.3x |
| 3900 (endpoint) | 74.3 | 611.6 | 8.2x |

Whole-leg mean sigma 49.8 m vs mean error 141.6 m (ratio grows from ~1x near
the aided prior to a peak of 8.2x by leg end — diluted early by epochs still
close to the aided fix). The steady-state window (last third of the leg, once
the aided prior has decayed — the D68-comparable regime) is filter sigma
56.7 m vs true error 195.8 m, a **3.1x overconfidence ratio**, directly
reproducing the reviewer's finding (their instrumentation found 7–70x across
seeds/epochs; this run's representative-seed steady-state figure of 3.1x,
rising to 8.2x at the endpoint, is consistent with the low end of that
range).

## Guardrails and verification

- Real EKF state vs generator truth; production gate `Some(9.0)`; no
  clamp/formula/target-fitting in either direction.
- New test `covariance_consistency_is_instrumented` guards the sigma is a
  real, finite, positive number from the filter's actual covariance (not a
  placeholder), and that the auto-generated verdict text is not hard-coded
  and does not over-claim "fundamental observability limit."
- Deterministic: `results.json` (schema 4) and `STUDY.md` reproduce
  byte-identical on re-run.
- Scope: only `crates/pnt-studies/src/endurance.rs` and
  `docs/studies/endurance/**` changed; no estimator/mission/multisat edits.
- No study NUMBER changed except by adding the new consistency metric — the
  leg-duration curve, bias-zeroed control table, clock-discipline curve, and
  per-epoch error trace are byte-identical to the pre-reframe committed
  values (diffed and confirmed).
- Full workspace `cargo test`, `cargo clippy --all-targets -D warnings`, and
  `cargo fmt --all --check` all pass.

## `[UNVERIFIED]` list (unchanged)

- Synthetic 960-satellite three-shell LEO Walker grid and sticky
  best-N-visible handover selection.
- Constant-heading 10/20/30/45/60-min legs, 30 s Doppler cadence, 7 kn speed;
  sub-second wave-slam disabled for long-leg truth stability.
- Receiver clock fractional injections and the Rb/good-OCXO/poor labels.
- Per-SV fixed transmit biases and deterministic tracker noise/outlier
  process.
