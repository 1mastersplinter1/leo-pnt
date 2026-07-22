# Brief U-D1.1 — Fix pass on design baseline (review round 1)

Contract version: v1. You are revising docs/design/DESIGN_BASELINE.md and docs/design/ARCHITECTURE.md
(authored by a prior session; treat as existing work under revision, fresh eyes).

Read in order: .orchestration/CONTRACTS.md, docs/HANDOFF_PROMPT_BLADERF.md,
the two design docs, then the review findings:
.orchestration/reports/U-D1-review-opus.md (verdict FAIL — blockers F1, F2, F3; F4 must-address)
.orchestration/reports/U-D1-review-sonnet.md (verdict PASS — majors/minors overlap Opus)

## Goal
1. Resolve Opus blockers F1 (Orbcomm receive path), F2 (vertical channel), F3 (lever-arm
   extrinsics) and must-address F4 (observable definition + per-SV frequency treatment) with
   explicit normative statements. Where the handoff does not decide the answer, choose the
   engineering-conservative option, state it as a decision with rationale, and mark residual
   unknowns [UNVERIFIED]. Suggested defaults you may adopt or argue against: F1 — Orbcomm via a
   separate cheap independent receiver (BOM dependency, non-coherent, decorrelates front end);
   F2 — altitude constrained to mean sea level as a pseudo-measurement with tide/wave-scale
   noise, vd published as 0 with matching vert_accuracy; F3 — mandate surveyed antenna
   phase-centre and IMU lever arms as calibration inputs referenced by the measurement
   envelope's calibration ID; F4 — observable is correlation-peak Doppler, per-SV carrier
   offset handled as a per-pass nuisance bias state or epoch-differenced (pick one, justify).
2. Apply Opus minors F5–F10 and Sonnet findings F1 (heading justification), F4 (current-vector
   state-vs-derived ambiguity — resolve it explicitly), F5–F7 where they cost a sentence or two.
   Opus F11 (multipath): add one line to the degradation/integrity discussion.
3. Do not restructure the documents; targeted edits only. Preserve everything the reviews
   verified as correct.

## Files owned
docs/design/DESIGN_BASELINE.md, docs/design/ARCHITECTURE.md, .orchestration/reports/U-D1.1.md. No git commit.

## Acceptance
Every blocker and must-address finding has a visible resolution in the docs; every applied
or rejected finding is listed in the report with a one-line disposition (fixed/rejected+why).

## Report
.orchestration/reports/U-D1.1.md: per-finding disposition table, decisions taken, remaining [UNVERIFIED] items.
