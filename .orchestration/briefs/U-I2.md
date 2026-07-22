# Brief U-I2 — Integration: wire predictor/gate/journal/arm/output into the executive

Contract version: v3 → you author v4. Workspace: dedicated worktree, branch `unit/U-I2`.
Commit here; never merge to main. Read first: `.orchestration/CONTRACTS.md` (v2+v3),
`docs/design/ARCHITECTURE.md` (modules 6–12, build order), `.orchestration/DECISIONS.md`
(D10, D13, D15, D21, D22 are all binding work items for you),
`.orchestration/reports/U-C1-review-summary.md` (F1/F2/F5/F6/F7),
`.orchestration/reports/U-E1.md` and `U-F1.md` (their "for U-I2" sections).

## Goal
1. **CONTRACTS v4** (append): pin the Doppler observable convention per D22-N1 — define
   exactly what `predicted_range_rate_mps` contains (recommendation: pure geometric range
   rate from pnt-predictor, satellite-side only; ALL receiver clock terms live in the
   estimator's H·x) and reconcile `update_doppler` / `update_doppler_for_receiver` to that
   single definition with a test where primary clock drift is nonzero (the blind spot Opus
   found). Also: ArmCommand routing semantics (to authority, never fusion), the OneWeb
   survey-gate config key (`oneweb_enabled: bool`, default false, D15-F1), and the
   NDJSON solution-epoch output schema (must match tools/mavlink_bridge README's input
   schema field-for-field — read it).
2. **Executive wiring** (fusion-executive + a new `pnt-gate` crate if boundaries demand):
   TrackerDoppler path becomes ingress → ephemeris lookup (pnt-ephemeris store, age gate)
   → Doppler prediction (pnt-predictor, receiver state from current FilterState, NED→ECEF
   conversion + lever-arm hook with [UNVERIFIED] zero-lever-arm default) → innovation gate
   (estimator chi-square hook) → EKF update → journal. Rejects (gate, age, Orbcomm-D10,
   OneWeb-disabled, off-mode GNSS) become journaled integrity events, never silent drops
   (D15-F2). ArmCommand routes to the authority/integrity port, never fusion (U-F1 L5).
   Solution epochs carry the v3 accuracies and stream as NDJSON lines via a writer the
   executive owns (module 12 seam; stdout or a sink trait — testable).
3. **Tests** (TDD): off-mode GNSS dropped at process() level (D15-F5); production GNSS
   dual-route fusion+truth; Heading/SpeedThroughWater routes; OneWeb rejected while
   disabled and routed when enabled; ArmCommand never reaches filter.update; end-to-end:
   fixture ephemeris + synthetic receiver → predicted Doppler → accepted update → epoch
   with finite accuracies → valid NDJSON line matching the bridge schema; reject-path
   journal assertions.
4. **Carried residuals**: restore the t=0 Vallado assertion alongside t=360 in
   pnt-ephemeris (D21); add the [UNVERIFIED] improved-mode-validation note (D21); add the
   geocentric-vs-geodetic latitude comment in pnt-types' ENU rotation (D22-N2).

## Method
TDD; gate: PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings
&& cargo fmt --all -- --check. Keep crate boundaries aligned with ARCHITECTURE modules.

## Files owned
`crates/**`, root `Cargo.toml`, the `## v4` section of `.orchestration/CONTRACTS.md`,
`.orchestration/reports/U-I2.md`. Not `tools/**` (owned by U-M1.1, in flight), not `docs/`.

## Report
Per-item dispositions for every D15/D21/D22/U-E1/U-F1 carried item, evidence, [UNVERIFIED] list.
