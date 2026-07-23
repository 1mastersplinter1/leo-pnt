# Brief U-R3 — Replay harness: same log twice, aided vs withheld

Contract version: v5.1. Workspace: dedicated worktree, branch `unit/U-R3`. Commit here;
never merge to main. Read first: `docs/HANDOFF_PROMPT_BLADERF.md` "Verification discipline"
(the same-raw-log-twice delta is the headline result) and failure mode 7 (circular
validation), `docs/design/DESIGN_BASELINE.md` (aided/denied profiles; headline statistics
computed offline from GNSS-withheld runs), `crates/pnt-journal` (FileJournals + readers),
`crates/fusion-executive` public API, `.orchestration/CONTRACTS.md` v2 (journal formats)
+ v4 (NDJSON epoch schema).

## Goal
New crate `crates/pnt-replay`:
1. **Replay driver**: open a U-J1 run directory, stream the measurement journal in order
   through a freshly constructed Executive in a caller-chosen `gnss_authority` mode,
   collecting solution epochs and integrity events. The measurement stream is identical
   in both modes — mode changes routing only (this is the design's core invariant; assert
   it by construction, not by trust).
2. **Paired run**: run the SAME directory twice — `production` (aided) and `recorded_only`
   (withheld) — and compute per-epoch deltas of each run's solution against the TRUTH
   journal (never against each other's estimates — failure mode 7): horizontal position
   error, horizontal speed error, per run, plus the aided-vs-withheld comparison table.
   Nearest-truth-in-time matching with a stated max time offset; epochs without near truth
   are excluded and counted.
3. **Summary artifact**: a serde_json report (schema documented in README): per-run error
   statistics (n, mean, RMS, p50/p95, max), exclusion counts, mode, config hash, run UUID —
   everything needed to quote the headline delta with provenance.
4. **Determinism**: replaying the same directory twice in the same mode must produce
   bit-identical epoch sequences; assert in a test.

## Tests (TDD)
Build a synthetic run directory fixture with FileJournals (IMU stream + GNSS fixes + truth
records with known geometry — e.g. truth follows the GNSS exactly, so aided error ≈ 0 and
withheld error grows): verify (a) aided run consumes GNSS (measurement_updates/GNSS-routed
counts differ between modes) and withheld run routes GNSS to truth only; (b) computed
statistics match hand-computed values on a small fixture (derive in comments); (c)
exclusion logic (truth gap) counts correctly; (d) bit-exact repeat; (e) the summary JSON
round-trips and contains the provenance fields.
Gate: PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check.

## Files owned
`crates/pnt-replay/**`, root `Cargo.toml` member line, `.orchestration/reports/U-R3.md`.
No executive/journal edits — if their APIs are insufficient, document the needed change in
your report for the integration unit instead.

## Report
Evidence, fixture geometry derivation, API gaps found, [UNVERIFIED] list.
