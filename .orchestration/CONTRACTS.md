# CONTRACTS.md — v1

Workers state the version they built against. Changes land here FIRST, with a DECISIONS.md line.

## v1 (2026-07-22)

- Repo layout: `docs/design/` (normative docs), `docs/research/` (research outputs), `docs/HANDOFF_PROMPT_BLADERF.md` (source brief), `.orchestration/` (plan/briefs/reports), code in a Rust workspace at repo root (crates under `crates/`, defined in v2 by U-C1).
- Language: English; SI units; all timestamps UTC; absolute dates only (no "yesterday").
- Report contract: every unit writes `.orchestration/reports/<unit>.md` — what changed, evidence (commands run + output), assumptions, open uncertainties. Grok reports additionally split VERIFIED (ran/read it) vs ASSUMED.
- `gnss_authority` config key: `production | recorded_only | off`; `recorded_only` routes GNSS to the truth journal only; unrecognised value raises, never defaults. Same code path in every mode. (Fixed by handoff; restated here as binding.)
- Acceptance criteria are split into `aided` and `denied` profiles — no single position limit applies to both.
- DR timeout governs steering authority only, never estimator execution.
- To be fixed in v2 (by U-C1 from the reviewed U-D1 baseline): measurement-bus message schema, coordinate frames, on-disk formats, module-owns-time statement, rate contract.
