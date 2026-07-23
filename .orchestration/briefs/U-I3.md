# Brief U-I3 — Doppler assimilation in replay: the real denied-mode headline

Contract version: v5.1. Workspace: dedicated worktree, branch `unit/U-I3`. Commit here;
never merge to main. Read first: `.orchestration/DECISIONS.md` D35/D37/D38 (the API gaps
routed to you), `crates/pnt-replay/src/lib.rs` + README, `crates/pnt-mission/src/lib.rs`
(how missions journal tracker Doppler + ephemeris fixture identity), `crates/fusion-executive`
(`with_doppler_pipeline`), `docs/HANDOFF_PROMPT_BLADERF.md` (the headline: satellite-only
denied performance vs truth).

## Goal
1. **`pnt-replay`**: accept an optional Doppler configuration — the caller supplies what a
   `DopplerPipeline` needs (ephemeris records or a ready store + elevation-mask choice) and
   replay constructs each Executive `with_doppler_pipeline`. Modes unchanged; a
   denied-mode replay can now assimilate journaled `TrackerDoppler`. Preserve determinism
   and mode-invariance-of-input by construction. Also: add the counted comparison-pairing
   exclusion field (`ComparisonSummary`), separating no-paired-epoch from no-near-truth
   (D35/Opus LOW-1 + Sonnet MEDIUM), and update the README schema (bump schema_version).
2. **`pnt-mission`**: the study now reports the THREE-way table: aided; denied-DR-only
   (Doppler config absent); denied-with-Doppler (config present, same journal). Assert in
   tests: denied-with-Doppler assimilates (doppler fusion routes > 0, measurement_updates
   greater than DR-only), and its position error improves on DR-only for the
   Doppler-rich mission (state the caveat: synthetic demonstration, not performance).
   Update the README + report; keep every existing test green (adjust schema-version
   expectations).
3. **Do not weaken**: circularity rules (errors vs truth only), bit-exact determinism tests
   (extend to the denied-with-Doppler mode), no-authority framing.

## Method
TDD; gate: PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings
&& cargo fmt --all -- --check.

## Files owned
`crates/pnt-replay/**`, `crates/pnt-mission/**`, `.orchestration/reports/U-I3.md`.
No other crates (executive already exposes what you need — if not, document for follow-up).

## Report
Evidence incl. the new three-way table from a seeded run, dispositions of the D35 carried
items you close, [UNVERIFIED] list, schema-version change notes.
