# Brief U-H2 — High-speed mission capability + 20 kn / 24 h passage study

Contract version: v5.1. Worktree branch `unit/U-H2`. Commit there; never merge to main; NO
Co-Authored-By/Claude-Session trailers. Read first: DECISIONS.md D45/D46,
crates/pnt-mission (mission generator), crates/pnt-studies (passage module from U-P1 — it
may still be on branch unit/U-P1; if crates/pnt-studies has no passage module in YOUR
checkout, write your study standalone in a new `highspeed` module and note the overlap for
integration), crates/pnt-estimator (process noise config), docs/design/SAFETY_CASE.md §0
speed caveat.

## Goal
1. **Mission generator envelope**: extend MissionConfig for high-speed profiles — speeds to
   10.3 m/s (20 kn), planing-regime dynamics: higher accelerations in turns (coordinated
   turn at configurable rate), a wave/slam disturbance model (bounded random vertical +
   pitch-coupled horizontal acceleration bursts, seeded, magnitude/rate configurable,
   documented as synthetic stand-ins [UNVERIFIED vs real planing data]), and speed-scaled
   IMU noise/bias options. Backward compatible: existing missions bit-identical (prove via
   the existing determinism tests).
2. **Speed-aware process noise**: expose estimator process-noise scaling per mission speed
   regime in the study harness (config-driven, not hardcoded); document that real values
   await U-H1's envelope analysis + real-IMU study (D43 lineage).
3. **The D46 study** (pnt-studies): 20 kn, >= 24 h, ~500 km denied passage: GPS loss at
   t=1 h, ephemeris cached at t=0 (uses graduated aging if present in your checkout, else
   note the dependency), several legs + turns, wave model on. Measure: position error class
   over the passage, convergence after each manoeuvre (time AND distance at 20 kn),
   velocity error, and the ephemeris-age margin at landfall. Compare 7 kn vs 20 kn same
   route-time... no — same DISTANCE, different durations. Deterministic, committed JSON +
   STUDY.md section under docs/studies/highspeed/.
4. **Tests**: TDD; whole-workspace gate green
   (PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check).

## Files owned
crates/pnt-mission/**, crates/pnt-studies/** (highspeed module + bin only),
docs/studies/highspeed/**, .orchestration/reports/U-H2.md.

## Report
Study headline numbers (7 vs 20 kn), convergence-distance table, wave-model definition,
[UNVERIFIED] list, integration notes (U-P1 overlap, U-H1 pending values).
