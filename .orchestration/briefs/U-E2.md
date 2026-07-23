# Brief U-E2 — Synthetic end-to-end mission capstone

Contract version: v5.1. Workspace: dedicated worktree, branch `unit/U-E2`. Commit here;
never merge to main. Read first: `docs/design/DESIGN_BASELINE.md` (velocity-instrument
framing, 10–20 min position observability, constant-heading legs), `crates/pnt-tracker`
README (synth + tracker API), `crates/pnt-ephemeris` + `pnt-predictor` (fixture ephemeris,
prediction API), `crates/pnt-replay` README, `.orchestration/DECISIONS.md` D35 (binding
carried items for you).

## Goal
New crate `crates/pnt-mission` (sim + study binary/lib) that closes the loop end-to-end on
synthetic data — the full-stack rehearsal of the research campaign:
1. **Mission generator**: a vessel trajectory (constant-heading legs + turns, configurable
   speed/current), IMU stream consistent with the trajectory (derive accelerations from the
   motion; add configurable bias/noise), speed-log and heading measurements (with noise),
   GNSS fixes from the true trajectory (truth), and **LEO Doppler observations synthesized
   from the fixture ephemeris**: for each visible satellite pass (use pnt-ephemeris fixtures
   + pnt-predictor geometry from the TRUE position), generate correlation-peak Doppler
   observations with configurable noise — optionally by running pnt-tracker's synth+tracker
   in the loop for a subset (prove the tracker's output feeds the pipeline), directly
   synthesized for the bulk (faster). All deterministic from a seed.
2. **Recording**: write the whole mission into a U-J1 FileJournals run directory (measurements
   + truth), exactly as a sea capture would be.
3. **Study**: run pnt-replay's paired replay on that directory; produce the aided-vs-withheld
   report — the synthetic-data rehearsal of the headline result. Assert the physics
   qualitatively: withheld error bounded during Doppler-rich constant-heading legs, growing
   during outages/turns; aided error small throughout. Numbers are synthetic-config-dependent
   — state them as demonstration, NOT performance claims [UNVERIFIED vs real signals].
4. **D35 carried items (binding)**: direct tests on pnt-replay's comparison table via your
   mission fixture — hand-derivable comparison values on a small case, sign convention
   asserted; Production-mode bit-exactness repeat; input-identity count assertion. Implement
   these as tests in YOUR crate exercising pnt-replay's public API (do not edit pnt-replay;
   if its API blocks a counted comparison-exclusion field, document for integration).

## Tests (TDD)
Mission determinism (same seed → bit-identical run directory); journal round-trip of a
generated mission; the paired-replay assertions of goal 3 on a small deterministic mission;
the D35 items; a smoke binary (`cargo run -p pnt-mission --bin mission-study -- --seed N
--out DIR`) that emits the run directory + replay report JSON.
Gate: PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check.

## Files owned
`crates/pnt-mission/**`, root `Cargo.toml` member line, `.orchestration/reports/U-E2.md`.
No edits to other crates — API gaps go in the report.

## Report
Evidence, mission-physics derivations (IMU consistency), the synthetic headline table with
the demonstration-not-claim caveat, API gaps, [UNVERIFIED] list.
