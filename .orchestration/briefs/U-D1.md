# Brief U-D1 — Design baseline + architecture draft

Contract version: v1 (read `.orchestration/CONTRACTS.md` first).

## Goal
From `docs/HANDOFF_PROMPT_BLADERF.md` (read it fully — it is the sole normative input), produce:

1. `docs/design/DESIGN_BASELINE.md` — the single normative document. Must state: vessel assumptions (displacement hull, manned, Danish-strait operating area as working assumption — flag as assumption), sensor set, the rate contract (per-sensor rates and the 5 Hz GPS_INPUT output), the degradation model (which sensors can drop and what remains), and acceptance criteria split into `aided` and `denied` profiles with limits drawn from the handoff's stated expectations (denied: ~100–200 m position, cm/s-class velocity; aided: operational-grade — justify numbers, mark estimates). Resolve the research-instrument vs operational-navigator tension explicitly: one codebase, mode selected by `gnss_authority`, acceptance profile per mode. State that all other documents are subordinate to this one.
2. `docs/design/ARCHITECTURE.md` — module boundaries, the measurement bus concept, on-disk formats (raw IQ capture, measurement journal, truth journal), and an explicit statement of which module owns time. Build order must follow the handoff's failure modes: fusion executive first, modules grow into it; ephemeris propagator + Doppler predictor before any reject gate; no filter state without a measurement path.

## Files owned
Only: `docs/design/DESIGN_BASELINE.md`, `docs/design/ARCHITECTURE.md`, `.orchestration/reports/U-D1.md`.

## Out of scope
No code. No BOM/pricing. No web research — where you need a fact you cannot derive from the handoff, mark it `[UNVERIFIED]` inline and list it in the report. Do NOT run git commit.

## Acceptance
- Every handoff "Required deliverables" item 1 & 3 element is present.
- No contradiction with the handoff's "What is already known" or "Failure modes" sections; where you disagree with the handoff, say so explicitly in the report with rationale — do not silently comply or silently deviate.
- Estimates marked as estimates; unverified facts marked `[UNVERIFIED]`.

## Report
Write `.orchestration/reports/U-D1.md`: what you produced, key design decisions taken and why, assumptions, open uncertainties, contract version (v1).
