# Brief U-AC1 — Acceptance-profile amendment: usable denied passage target (D56)

Contract v5.1. Worktree branch `unit/U-AC1`. Commit there; never merge to main; NO
Co-Authored-By/Claude-Session trailers. Read first: DECISIONS.md D56 (the decision), D55/D51
(the multi-sat evidence), docs/design/DESIGN_BASELINE.md (Acceptance profiles section — the
normative denied/aided limits), docs/design/PARAMS_PROPOSAL.md (protection-limit derivation,
k=2 mapping, the aided/denied PL sections), docs/design/SAFETY_CASE.md (G2 protection-limit
gate, the acceptance-vs-PL distinction in §0), docs/studies/multisat/STUDY.md (the evidence).

## Goal (docs only; no code)
1. **DESIGN_BASELINE.md** — dated amendment (do NOT rewrite reviewed text; append/annotate
   per the doc-drift discipline): denied horizontal-position acceptance becomes
   **<= 500 m (p50) AND <= 750 m (p95) over a >=100 km constant-heading-dominated passage**,
   citing D56 + the U-MS1.1 controlled evidence (116 m p50 / 554 m p95 at N=8). AIDED profile
   UNCHANGED (<= 25 m) — state explicitly why it stays tight (failure-mode-2). Keep velocity/
   heading denied limits as-is unless the evidence says otherwise (state which). Mark the old
   100-200 m target superseded IN PLACE.
2. **PARAMS_PROPOSAL.md** — re-derive the denied horizontal-position protection limit from the
   new acceptance via the existing k=2 method: denied PL ~= 500/2 = 250 m (p50-referenced) with
   the worst-case bound informing the ceiling; show the derivation, update the §1.1 denied PL
   value and the TOML appendix, keep the [UNVERIFIED]/PROPOSED-NOT-FROZEN framing, update the
   validation plan to reference the multi-sat replay evidence. Reconcile revoke_threshold
   (currently 100 m denied scalar) with the new denied PL — flag if they now disagree.
3. **Authority reconciliation**: note how the relaxed PL interacts with open ruling U-P1-O1
   (t_eph age gate) — both are denied-mode authority tuning; state the combined picture for
   the params/safety-case owners; do NOT edit SAFETY_CASE (not owned) — register the needed
   G2 protection-limit update as an open item.
3b. Honesty: every new number traces to D56/evidence or is marked [UNVERIFIED]; no aided
    relaxation; state plainly that the denied target is now evidence-MET where the old one
    was not, and that real-signal validation remains required.

## Files owned
docs/design/DESIGN_BASELINE.md (amendment), docs/design/PARAMS_PROPOSAL.md (denied PL + TOML),
.orchestration/reports/U-AC1.md. No code, no SAFETY_CASE edit.

## Report
The amended profile, the PL re-derivation, the revoke_threshold/U-P1-O1 reconciliation,
[UNVERIFIED] list, and a one-line statement of what evidence now meets vs what remains open.
