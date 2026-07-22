# Brief U-F1 — EKF core + contracts v3

Contract version: v2 → you author v3. Workspace: dedicated git worktree on branch
`unit/U-F1`. Commit here; never merge to main. Read first: `.orchestration/CONTRACTS.md`,
`docs/design/DESIGN_BASELINE.md` (states, observables, MSL pseudo-measurement, per-SV
nuisance model), `docs/design/ARCHITECTURE.md`, `.orchestration/DECISIONS.md` (D10, D13,
D15), `.orchestration/reports/U-C1-review-summary.md` (F3, F4 are yours to fix).

## Goal
1. **CONTRACTS.md v3 section** (append; do not edit v1/v2): full covariance in
   `FilterState` and per-epoch accuracies (horizontal, speed, vertical) in `SolutionEpoch`
   (resolves U-C1 review F4); an `ArmCommand` bus message (helm arm/disarm with monotonic
   timestamp and source — resolves D13; the executive routes it in U-I2, you only define the
   type); the per-receiver clock extension point required by D10 (define the state-slot
   mechanism; Orbcomm stays ingress-rejected until U-I2 wires it).
2. **`crates/pnt-types`**: implement those v3 type changes compatibly — the workspace must
   still build with the UNMODIFIED fusion-executive. Additive changes only (new fields with
   sensible constructors/defaults). If true compatibility is impossible, stop that sub-item
   and write the required executive change into your report for U-I2 instead.
3. **`crates/pnt-estimator`**: replace the stub internals with a real error-state EKF behind
   the existing trait surface: states = ECEF position (3), ECEF velocity (3), heading,
   receiver clock bias (m) and drift (m/s), plus dynamically augmented/retired per-SV
   nuisance biases (API + tests now, even though no tracker exists yet). Full covariance
   propagation from IMU input with configurable process noise; measurement updates with
   innovation, innovation covariance, and chi-square gate hook for: Doppler range-rate
   (predicted value supplied by caller — define that input struct to match pnt-predictor's
   documented output shape from ARCHITECTURE module 7; do NOT depend on the pnt-predictor
   crate), speed-through-water + heading (their current-vector role per baseline — the
   current is derived downstream, not a state), MSL altitude pseudo-measurement, GNSS
   position/velocity (aided mode only).
4. **Jacobian discipline (the handoff's core demand)**: every analytic measurement Jacobian
   and the state-transition Jacobian verified against central finite differences in tests,
   with stated tolerances; a covariance-growth test asserting MAGNITUDE growth of position
   variance under dead-reckoning (replaces U-C1's count-only hook, review F3); symmetry +
   positive-semidefiniteness asserted after every update in debug builds.

## Method — TDD, strictly
Failing test first; gate: `cargo test && cargo clippy --all-targets -- -D warnings &&
cargo fmt --all -- --check` (PATH="$HOME/.cargo/bin:$PATH"). Pure-Rust linear algebra
(you MAY use `nalgebra`; record version). No filter state without an implemented update
path in this crate (baseline rule).

## Files owned
`crates/pnt-estimator/**`, `crates/pnt-types/**`, the `## v3` section of
`.orchestration/CONTRACTS.md`, `.orchestration/reports/U-F1.md`. Nothing else — you may run
the whole-workspace gate but not edit other crates.

## Report
`.orchestration/reports/U-F1.md`: evidence (test output incl. Jacobian check tolerances),
v3 resolutions list, executive changes needed for U-I2, assumptions, [UNVERIFIED] items.
