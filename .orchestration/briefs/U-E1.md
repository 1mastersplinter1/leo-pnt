# Brief U-E1 — Ephemeris propagator + Doppler predictor (library crates only)

Contract version: v2. Workspace: you are in a dedicated git worktree on branch `unit/U-E1`.
Commit here; never merge to main. Read first: `.orchestration/CONTRACTS.md` (v2),
`docs/design/DESIGN_BASELINE.md`, `docs/design/ARCHITECTURE.md` (modules 6–7),
`.orchestration/reports/U-C1-review-summary.md` (F7: you interpose LATER via U-I2 — this
unit builds libraries only).

## Goal
Two new workspace crates. **Do not modify fusion-executive, pnt-estimator, pnt-types, or any
existing crate** — if you believe an existing type must change, write the needed change into
your report for the integration unit instead.

1. `crates/pnt-ephemeris`: parse CelesTrak SupGP/OMM (JSON) and TLE records from LOCAL files
   (no network in code or tests — fixtures under the crate's `tests/fixtures/`); store per-
   satellite ephemeris with epoch; SGP4 propagation to a query time (you MAY depend on the
   `sgp4` crate — record its version and validate against its shipped Vallado/reference test
   vectors in your tests); TEME→ECEF conversion (document the Earth-rotation model you use and
   its error bound; mark simplifications [UNVERIFIED] if not literature-anchored); age gate:
   reject queries when ephemeris age exceeds a configurable limit (default 6 h per
   DESIGN_BASELINE — cite it), returning a typed error, never a silent extrapolation.
2. `crates/pnt-predictor`: given satellite ECEF state, receiver ECEF position/velocity,
   receiver clock drift, per-SV nuisance bias, and nominal carrier Hz → predicted
   correlation-peak Doppler (Hz) and range rate (m/s), plus line-of-sight unit vector,
   elevation, and range. Include an elevation mask. Sign conventions must be stated in doc
   comments and pinned by tests (approaching satellite ⇒ positive Doppler — verify against
   the physics, don't trust this sentence).

## Method — TDD, strictly
Failing test first, minimal implementation, green, commit; conventional messages. Gate:
`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`
(toolchain: PATH="$HOME/.cargo/bin:$PATH"). Analytic values in tests must come from cited
sources or independent computation shown in the test comment — never from running the code
under test. Include at least one end-to-end test: real fixture ephemeris + fixed receiver →
Doppler curve over a pass with physically-sane properties asserted (sign flip at closest
approach, |Doppler| bounds from orbital velocity, smoothness).

## Files owned
`crates/pnt-ephemeris/**`, `crates/pnt-predictor/**`, root `Cargo.toml` members list ONLY
(the two added lines), `.orchestration/reports/U-E1.md`.

## Report
`.orchestration/reports/U-E1.md`: evidence (test output), dependency versions, models chosen
+ error bounds, changes you need from other crates (for U-I2), assumptions, [UNVERIFIED] items.
