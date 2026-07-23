# U-N1 report — authority-contract numeric-freeze proposal

Contract read: v5.1 (`AuthorityParams`). Authored 2026-07-23.
Deliverable: `docs/design/PARAMS_PROPOSAL.md`. No code edits, no commit, no web research.

## What changed

- Authored `docs/design/PARAMS_PROPOSAL.md`: one section per `AuthorityParams` field
  (6 protection limits + t_lease/t_dr/t_eph + caution band/revoke + 2 dwells + T_ack) with
  proposed value, derivation chain (cited to acceptance profiles, D17a, U-T1 tracker sigmas,
  EKF/DRMS conventions, human factors), two-sided sensitivity, and validation plan.
- Completed the U-T1 **F9** tracker detection-threshold PFA analysis (D36 route): Fisher-g
  model, validated against the measured noise-only quantiles, PFA at threshold 32, threshold
  policy.
- Machine-readable TOML appendix headed PROPOSED — NOT FROZEN.
- Status/subordination section; every value tagged [UNVERIFIED until validated].

## Key derivations (VERIFIED against source docs/code)

- **Supervisor metric is `horizontal_accuracy_m()` (metres)** — confirmed by reading
  `crates/pnt-integrity/src/lib.rs` (line ~492: `let metric = solution.state.horizontal_accuracy_m()`).
  This pins `caution_enter/caution_clear/revoke_threshold` as **metre** horizontal-accuracy
  values, not dimensionless fractions. Ordering clear<enter<revoke per SAFETY_CASE §3.2.
- **PFA model checked numerically** (python): Fisher-g independent-cell prediction gives
  Q = 12.06 / 16.19 / 20.14 at P = 0.5 / 0.01 / 2.5e-4 vs the measured 11.5 / 15.7 / 20.0. Match
  to ~0.5 in Q. **The 4000-block quantiles are NOT my measurement and NOT reproducible**: they
  are the U-T1 deep-review seat's in-worktree probes, cited from
  `.orchestration/reports/U-T1-review-opus-measurements.md` (commit 54005dd, probes removed;
  committed suite has 24 blocks only). PFA(32) ≈ 5.30e-9 is stated as an **analytic-model figure
  only** — 4000 samples cannot validate a ~5e-9 probability, so the earlier "conservative bracket
  ~1e-8" and "very safe" claims are **withdrawn** (fix round). Correlation direction corrected:
  positive inter-cell dependence makes the independent-cell model **conservative** (overestimate),
  so inflation rests only on the separate non-ideal-reference/interference caveats.

## Decisions taken (proposals, flagged as such in the doc)

- Coverage factor **k = 2** on horizontal position (one-sigma DRMS metric; 2-DRMS ≈ 98%
  containment) to satisfy SAFETY_CASE §0 "no looser than the acceptance profile implies".
  Velocity mapped per-axis 95% (factor √2/1.96 = 0.721 on the per-axis acceptance); heading
  treated as acceptance = ~2σ (PL = half). All three flagged as coupled to the unfrozen
  acceptance-percentile definition (open issue 5.2).
- Proposed values: aided/denied position 12/100 m; velocity 0.014/0.028 m/s; heading
  0.01745/0.04363 rad (1.0°/2.5°); t_lease 1.0 s; t_dr 120 s; t_eph 21600 s (6 h, from
  baseline); caution 60/75, revoke 100 m; dwell_clear 5 s; dwell_rearm 10 s; T_ack 10 s.
- revoke_threshold set = denied PL_pos (100 m); corrected to note it backstops a finite-but-loose
  limit (NOT an absent one — is_complete() already fail-closes absent limits) and the `<` vs `<=`
  boundary makes the scalar marginally tighter at exactly 100.0 m.
- Per-source freshness deadlines (fix round, new §2.4): IMU 0.10 s, magnetometer 0.50 s,
  speed-log 1.00 s, ephemeris = t_eph. Registered §5.4: these are NOT AuthorityParams fields,
  so is_complete() does not enforce them — contracts-owner v6 action item.
- Tracker threshold: retain fixture default 32; production value [UNVERIFIED], re-derive per
  policy §4.5. Not an AuthorityParams field (excluded from struct TOML except as a comment).
- TOML flattened (fix round) to mirror the real struct: 9 scalar fields top-level with exact
  Rust names, aided/denied sub-tables; verified it deserializes field-for-field (tomllib check).

## Assumptions

- Acceptance-profile numbers treated as high-confidence error bounds (their percentile is
  [UNVERIFIED] in the baseline); k factors stand in until frozen.
- t_dr sized on a conservative ~0.05 m/s post-loss residual velocity-error scale (no replay
  data) — displacement-hull only.
- Tracker 4 Hz freq-error scale (U-T1) is fixture geometry (Fs=8192, 32 Hz grid); used only as
  an order-of-magnitude range-rate scale, not a frozen sigma.
- PFA i.i.d.-exponential delay-bin model assumes ideal delta-autocorrelation reference; real
  PSS/SSS/Iridium/Orbcomm sidelobes fatten the tail (stated caveat A1).

## Open uncertainties / weakest evidentiary support

- **t_dr (120 s)** — no replayed LEO revisit-gap statistics; single weakest timer.
- **Aided velocity PL (0.014 m/s)** — achievability unproven; demands tens–hundreds of fused
  Doppler observations; 4 Hz scale is fixture-only.
- **Dwells and T_ack (5/10/10 s)** — human-factors estimates, zero trial evidence; T_ack
  additionally hull-speed-dependent.
- **Production tracker threshold** — depends on real sequences + production geometry + per-
  constellation link budget (all [UNVERIFIED]); 32 is fixture-only.

## Registered design issues routed out of this unit

- **5.1** Single-scalar caution/revoke vs per-profile ProtectionLimits → aided mode has no
  caution pre-alert. Recommend contracts v6 make the caution band per-profile.
- **5.2** k-factor / acceptance-percentile definition + velocity covariance-shape (anisotropy)
  are a single coupled freeze (baseline/contracts).
- **5.4** (new) AuthorityParams/CONTRACTS v5 lacks per-source freshness fields; is_complete()
  does not enforce them — contracts-owner v6 action item.

## Fix-round disposition (dual review: Sol/codex + Sonnet)

Both seats FAIL. All findings ACCEPTED and fixed; none rejected.

| # | Sev | Seat(s) | Finding | Disposition |
|---|---|---|---|---|
| 1 | HIGH | codex, S-H1 | Per-source freshness deadlines omitted; absent from AuthorityParams so is_complete() cannot enforce them | ACCEPT — new §2.4 (values + derivations), separate TOML block, §5.4 registers the enforcement gap for contracts owner; intro reworded to state whole-register coverage |
| 2 | MED | codex/S-H2 | 4000-block quantiles uncited; trace to review-probe artifact (irreproducible), committed suite = 24 blocks | ACCEPT — cite `U-T1-review-opus-measurements.md`, provenance caveat under §4 header + assumption A5, source list updated |
| 3 | MED | codex/S-M1 | Correlation-direction inverted: positive dependence makes independent-cell model conservative (overestimate), not optimistic | ACCEPT — §4.2 reworked, inflation now rests only on A1/A3 marginal-tail caveats; A2 relabelled conservative |
| 4 | MED | codex | "Very safe" + order-1e-8 bracket unjustifiable from 4000 samples at ~5e-9 | ACCEPT — §4.3 relabels 5.30e-9 analytic-model-only; "very safe"/1e-8 withdrawn |
| 5 | MED | codex/S-M2 | TOML nesting ([timers]/[thresholds]) doesn't match flat struct; no serde adapter | ACCEPT — flattened to exact struct field names; tomllib-verified field-for-field |
| 6 | MED | codex/S-M3 | revoke_threshold can't backstop ABSENT limit (is_complete() already fail-closes) | ACCEPT — §3.1 corrected to finite-but-loose only |
| 7 | MED | codex | Velocity anisotropy: DRMS gate assumes isotropic Gaussian per-axis, not stated at derivation | ACCEPT — §1.2 anisotropy limitation added; §5.2 adds covariance-shape freeze |
| 8 | LOW | codex/S-L2 | Fisher ">10 orders" holds only at Q=32, not median/p99 | ACCEPT — §4.1 scoped to Q=32; validation table uses full sum |
| 9 | LOW | codex/S-L1 | `<=` vs `<` at exactly 100.0 m: profile passes, scalar fails; not "revoke together" | ACCEPT — §1.1/§3.1 corrected; scalar marginally tighter at boundary |
| 10 | LOW | codex/S-LM | "5× nominal" lease margin contradicts ~100× DR-fill (10 ms) renewal cadence | ACCEPT — §2.1 states both cadences (~100× DR-fill, 5× the 5 Hz publication) |

## Verification performed

- Read all briefed sources: SAFETY_CASE §1.2/§3.2/§5, DESIGN_BASELINE (profiles/degradation/
  rate), CONTRACTS v5/v5.1, D17a.md, R1-bladerf-market.md, pnt-tracker README + U-T1 report,
  DECISIONS D17a/D24/D36 (+ context D27-D36).
- Fix round: read both review files in full (U-N1-review-sol.md, U-N1-review-sonnet.md) and the
  cited measurements artifact (U-T1-review-opus-measurements.md).
- Read `crates/pnt-integrity/src/lib.rs` and `crates/pnt-tracker/src/lib.rs` to ground the
  metric definition and the quality statistic against shipped code (not just docs).
- Re-derived the Fisher-g PFA model numerically; validated the corrected TOML parses field-for-
  field against the flat struct (tomllib).
- No commit, no code changes, no web research (gaps marked [UNVERIFIED]).

## Source list (evidence cited)

- `docs/design/SAFETY_CASE.md`, `docs/design/DESIGN_BASELINE.md`, `.orchestration/CONTRACTS.md`
  (v5/v5.1), `tools/sitl/evidence/D17a.md`, `docs/research/R1-bladerf-market.md`,
  `crates/pnt-tracker/README.md`, `.orchestration/reports/U-T1.md`, `.orchestration/DECISIONS.md`.
- `crates/pnt-integrity/src/lib.rs`, `crates/pnt-tracker/src/lib.rs` (shipped code).
- `.orchestration/reports/U-T1-review-opus-measurements.md` — 4000-block noise quantiles and
  discriminator sigmas. **Caveat: review probes (commit 54005dd, removed after run), NOT
  shipped-test evidence, NOT reproducible from the committed 24-block suite; no recorded seed.**
