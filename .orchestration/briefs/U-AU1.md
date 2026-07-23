# Brief U-AU1 — Authority reconciliation per D59 (safety-case + params)

Contract v5.1. Worktree branch `unit/U-AU1`. Commit there; never merge to main; NO attribution
trailers. Read first: DECISIONS.md D59 (the two rulings — authoritative), D56/D58 (acceptance
relaxation), D45/D49 (graduated aging), docs/design/SAFETY_CASE.md (§0 acceptance-vs-PL,
§1 G2/G2e/G2p gates, §3 revoke_threshold, §5 register), docs/design/PARAMS_PROPOSAL.md
(t_eph, revoke_threshold, denied PL), docs/design/DESIGN_BASELINE.md (the graduated-aging
amendment + denied acceptance).

## Goal (docs only)
1. **SAFETY_CASE.md** dated amendment (append/annotate per doc-drift discipline; do not rewrite
   reviewed prose): (a) update the stale §0 "<=200 m denied" example and any G2p PL reference
   to the D56/D58 values (500 m p50 / 750 m p95 acceptance; 250 m denied PL); (b) reconcile
   G2e per D59 — state t_eph = 30 h graduated-aging ceiling (not 6 h), with the explicit
   rationale that G2p (age-inflated protection limit) is the continuous accuracy governor and
   G2e is the freshness backstop; mark the old 6 h authority-cliff interpretation superseded
   in place; (c) revoke_threshold 100 m -> 250 m to match the denied PL (D59), superseded in
   place. Keep fail-closed framing — all values remain [UNVERIFIED]/PROPOSED until frozen.
2. **PARAMS_PROPOSAL.md**: set t_eph_s to the 30 h ceiling with D59 rationale (was 6 h; the
   6 h fresh-window for INFLATION still stands in the baseline — distinguish the inflation
   fresh-window from the authority freshness backstop clearly); set revoke_threshold to 250 m
   resolving the §3.1 "retained/overrides" note from U-AC1; update the TOML appendix and the
   validation plans; keep PROPOSED-NOT-FROZEN.
3. Honesty: nothing here grants authority (fail-closed gate holds); state that these
   reconciliations make D56/D45 self-consistent but real-signal freeze evidence is still
   required. Distinguish clearly: G2p accuracy governor (age-inflated PL) vs G2e freshness
   backstop (t_eph=30h) vs the U-P1 inflation fresh-window (6h) — three different 6h/30h uses.

## Files owned
docs/design/SAFETY_CASE.md (amendment), docs/design/PARAMS_PROPOSAL.md, docs/design/DESIGN_BASELINE.md
(only if a G2p cross-ref needs the value corrected), .orchestration/reports/U-AU1.md. No code.

## Report
The three reconciliations, the three-way 6h/30h disambiguation, [UNVERIFIED] list, and a
one-line statement that the fail-closed gate still blocks authority pending real-signal freeze.
