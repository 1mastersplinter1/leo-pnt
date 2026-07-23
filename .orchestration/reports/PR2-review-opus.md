# Adversarial review — PR #2 "Current (SOG≠STW) estimation state (U2)"

**Scope reviewed:** `git diff 67961f2...HEAD` on `origin/feat/current-state-u2` (worktree `/home/od/work/leo-pnt-pr2`). 7 files, +1085/−14. Base `67961f2` = PR#1 head (band-aware fusion + Huber cost). **This PR is stacked on PR#1 and must integrate after it.**

**Gate run (mine):** `cargo build --workspace` ✅; `cargo test --workspace` ✅ (all crates green — estimator 23, smoother 8, executive 22+7, mission 4); `cargo clippy --workspace --all-targets` ✅ clean; `cargo fmt --check` ✅. **But `cargo build --workspace --locked` FAILS** (see F3).

**Scope red flag up front:** the PR is titled "U2" but the delta actually ships U2 **plus U4b (soft-prior), U4c (ZUPT), U4d (smoother crate), and U4e (reseed gate + executive seam)**. Five plan-units in one "U2" PR. The plan (`docs/CODE_CHANGE_PLAN.md` §U4) explicitly says "U4d/U4e do not merge until the information rule, kill-switch dual path, and lag-aligned reseed gate are implemented and tested" — and by the crate's own admission the lag-alignment (N5) is *not* implemented. Bundling this under a "U2" title understates what is being asked to merge.

---

## Positive findings — what is actually correct (verified)

The central design claim **holds**: putting current in the fixed core genuinely eliminates the index-shift bug class. I verified the index arithmetic everywhere:

- `CORE_DIM 9→11`, `CURRENT_E=9`, `CURRENT_N=10`; POS/VEL/HEADING/CLOCK keep 0..8 (`lib.rs:14-20`). Augmentation appends at `x.len()` (`augment_state`, `lib.rs:534`), so every dynamic state lands at index ≥ 11. `retire_satellite_bias`'s shift bookkeeping (`if *slot > index`, `lib.rs:243-255`) can therefore never touch 9/10. Test `current_lives_in_the_fixed_core_at_stable_indices` (`lib.rs:1067`) asserts this under an augment→reserve→retire sequence. **Claim (B3) verified.**
- Doppler Jacobian correctly narrowed to `DOPPLER_JACOBIAN_DIM=9` and zero-padded onto 9/10 (`update_doppler` / `update_doppler_for_receiver`, `lib.rs:263,288`). Predictor's `[f64;9]` signature therefore does **not** need to change — the estimator absorbs the pad internally. Test `a_doppler_jacobian_does_not_observe_current` (`lib.rs:1080`) confirms a pure-clock Doppler leaves current untouched. The receiver-clock drift remap (`h[slot.drift_index] += h[CLOCK_DRIFT]`) is unaffected. **N1 predictor concern verified resolved.**
- `F` is identity on the current rows and `Q` adds only a diagonal RW term (`propagate`, `lib.rs:627-631`); current is block-decoupled from everything. So with the model **off**, the two states are inert and cannot degrade the existing solution — the opt-out claim (c) is real.
- Accuracy helpers stride by the **instance** `covariance_dimension` (=11+), not the `CORE_DIMENSION` constant (`pnt-types/src/lib.rs:206,216,225,280`), so DRMS/vertical accuracy stay correct with the widened core. `state()` sets `covariance_dimension: self.x.len()` (`lib.rs:698`). **Accuracy-helper validity verified.**
- `apply_reseed` overwrites the core block and zeroes core↔augmented cross-covariances, yielding a block-diagonal PSD joint; test checks symmetry + min-eigenvalue (`lib.rs:485-523`). Sound and conservative.
- All measurement Jacobians (water-velocity components, doppler, msl) still pass central-difference at 2e-6 (`all_measurement_jacobians_match_central_difference`).

---

## Blocking findings

### F1 — Normative-baseline violation, unrevised. **Severity: HIGH · Confidence: HIGH**
`docs/design/DESIGN_BASELINE.md:113-116` (Status: **normative**): *"The online estimator shall **not** carry a water-current state in the baseline… This makes current observable without enlarging the core state; a future first-class current state **requires an explicit update model and baseline revision.**"* This PR adds current as first-class core states and **does not touch DESIGN_BASELINE.md** (confirm: `git diff --name-only 67961f2...HEAD` lists only 7 code files). The baseline still forbids exactly what the code now does. The "explicit update model" half is (partially) met in code; the mandatory "baseline revision" half is absent.
**Verify:** the file list above; `DESIGN_BASELINE.md` line 9 ("this document governs until explicitly revised").

### F2 — The current state is *decorative / unobserved* in every wired path. **Severity: HIGH · Confidence: HIGH**
`with_vector_stw()` is the only thing that routes STW into the current-observing model; **it is never called anywhere except the estimator's own unit tests** (`grep -rn with_vector_stw crates/` → only `pnt-estimator/src/lib.rs`). The executive's `update()` path (`lib.rs:660`) calls `update_speed_through_water` with `vector_stw=false`, which falls back to the legacy scalar ground-speed model that never references current (`lib.rs:163-166`). So in the shipped/production configuration the two current states receive process noise and **no measurement** — precisely the "decorative unobservable state" the baseline separately forbids (`DESIGN_BASELINE.md:98,102-103`). The entire value proposition of U2 (recover current, drop position RMS under SOG≠STW) is realized in **zero** integrated tests. This is the "declared-but-unobserved state the baseline forbids" failure mode, present in the default build.
**Verify:** the grep above; trace `Executive::…update` → `update_speed_through_water` → `vector_stw` false branch.

### F3 — `Cargo.lock` not updated; `--locked` build fails. **Severity: MEDIUM-HIGH · Confidence: CERTAIN**
The new `pnt-smoother` member is absent from the committed `Cargo.lock` (`grep pnt-smoother Cargo.lock` → not found). `cargo build --workspace --locked` errors ("cannot update the lock file because --locked was passed"). A repo whose acceptance profile demands **bit-exact replay determinism** (`DESIGN_BASELINE.md:163`) will almost certainly gate CI on `--locked`; this PR breaks it. External-contributor hygiene miss.
**Verify:** the two commands above.

### F4 — Vector-STW cross-track R bug: the exact caveat the plan flagged as a blocker, unaddressed. **Severity: HIGH · Confidence: HIGH**
`update_speed_through_water` applies two sequential scalar updates for the E and N components of `v_ground − current ≈ STW·û_heading`, **both using the same caller `variance`** (`lib.rs:181-184`). That is an isotropic 2-D measurement of the full water-velocity *vector*, which constrains the cross-heading (sideslip) direction just as tightly as the along-heading STW magnitude. The plan is emphatic this must not happen — B1 CONFIRMED-WITH-CAVEAT and the "Vector-STW R caveat" (`CODE_CHANGE_PLAN.md:38-40,136-137,282-284`): *"its R must **not** be set equal to the STW magnitude variance, or it injects fake cross-track information."* The code injects fake cross-track information at full STW precision. The doc comment (`lib.rs:152-154`) acknowledges the issue but the offered mitigation ("a caller expecting sideslip should widen `variance`") widens *both* components equally and cannot down-weight cross-track alone — the API provides no separate cross-track R. So the core new physics of U2 is implemented with the specific defect its own design review identified.
**Verify:** read `lib.rs:168-185`; note both loop iterations pass the single `variance`.

---

## Contract / process findings

### F5 — No CONTRACTS bump, no DECISIONS line, no unit report. **Severity: MEDIUM · Confidence: HIGH**
`.orchestration/CONTRACTS.md:5` ("Changes land here FIRST, with a DECISIONS.md line") and the v1 report contract ("every unit writes `.orchestration/reports/<unit>.md`"). The delta touches none of these. Worse, CONTRACTS v3 (`CONTRACTS.md:133-136`) still asserts *"The nine core slots are ordered position ECEF (0-2)… clock drift (8). Dynamically registered states follow."* — now factually wrong: current at 9/10 is neither one of the documented nine nor a dynamically registered state. A first-class estimator-surface change landed without updating the binding contract that describes that surface.

---

## Moderate / minor findings

- **F6 (LOW-MED):** `pnt-types::FilterState::CORE_DIMENSION` still `=9` (`pnt-types/src/lib.rs:200`) while the estimator core is 11. Harmless today (nothing strides real estimator output by the constant), but it's a latent trap: any future consumer that trusts the constant to mean "estimator core width" or that slices core-vs-nuisance at 9 will silently mis-handle the current states. N1 only *partially* discharged.
- **F7 (LOW-MED):** N3 not done — `new()` leaves the default `DMatrix::identity` init, giving current an **implicit, undocumented σ=1 m/s prior** (`lib.rs:153`). The plan called for an explicit current prior. σ=1 m/s isn't unreasonable for Danish-straits tidal currents, but it's unstated and unjustified.
- **F8 (MEDIUM) — test quality:** the only current test, `vector_stw_makes_current_observable` (`lib.rs:869`), is idealized: ground velocity is *set directly into the state* and never independently pinned by Doppler, so convergence to current=+1 relies on the prior-covariance ratio (CURRENT var 4.0 vs VEL var 1.0), and the test **does not assert velocity integrity** (velocity is free to absorb the discrepancy instead). There is **no** mission/integration test exercising the current state end-to-end, and **no circle-degeneracy regression test** despite the plan calling it "mandatory" (`CODE_CHANGE_PLAN.md:146-147,158`). The legacy scalar `speed_model` also lost its finite-difference Jacobian check (replaced, not added to). So requirement (e) — "a real SOG≠STW scenario with known injected current and recovery verified" — is met only weakly at the unit level and not at all in any wired path.
- **F9 (LOW, informational):** `pnt-smoother` contains **no smoother** — only `ReseedGate` (a pure, well-tested function). U4d's "hand-rolled sparse Gauss-Newton fixed-lag smoother" is not implemented. The crate root is honest about being scaffolding and lists the open blockers ([UNVERIFIED], N5 mean-propagation not done), which is good discipline — but the crate name and the executive `submit_smoother_reseed` seam imply more than exists. The seam is correctly fenced behind `SmootherOwnsMeasurements`, which production never enters, so it is inert and fail-closed today.
- **F10 (LOW):** `Estimator` trait gains `core_estimate`/`apply_reseed` (`lib.rs:31-35`) — a breaking change for any external trait implementor (in-repo `PoisonedEstimator` was updated).
- **F11 (LOW):** stale doc — executive comment says `apply_reseed` "does not yet reconcile core↔augmented cross-covariances" (`fusion-executive/src/lib.rs`), but the estimator implementation *does* zero them (`lib.rs:314-324`). Contradictory documentation.
- **F12 (TRIVIAL):** the `current_random_walk_variance` rationale ("small so it cannot random-walk into velocity", `lib.rs:44-46,627-629`) misattributes the mechanism — velocity is protected by *structural decoupling* in F/Q, not by the RW being small. Behavior is correct; the reasoning in the comment is not.

---

## Duplication / conflict with merged units
No code conflict with D39/replay or existing units; the smoother seam is new and gated. The soft-prior (U4b) explicitly targets the D39/D43 confounding bug and its test reproduces-then-guards it — that part is clean. The conflicts are with the **normative docs** (F1, F5), not with merged code.

---

## MERGE RECOMMENDATION: **needs-rework** (reject in current form)

The engineering is above-average for an external PR — the fixed-core index arithmetic is correct, the opt-out path is genuinely inert, clippy/fmt/tests are green, and the reseed gate is a tidy, well-tested pure function. But it cannot merge as-is:

**Required before merge (blockers):**
1. **F1** — Land a companion **DESIGN_BASELINE.md revision** that explicitly authorizes a first-class current state with its update model, *or* revert to the baseline's solution-module current derivation. A normative-doc violation cannot be merged around.
2. **F2** — Wire `with_vector_stw` into an actual observable path (executive/mission/study) so the state is not decorative in production, and add an **integrated** test proving current recovery + position-RMS improvement under injected SOG≠STW. Until then the states are pure liability the baseline forbids.
3. **F4** — Give the cross-track (sideslip) pseudo-measurement its own model-uncertainty R, distinct from the STW magnitude variance (or implement the 1-state along-track fallback). Add the **circle-degeneracy regression test** the plan mandates.
4. **F3** — Commit an updated `Cargo.lock` including `pnt-smoother`; verify `--locked` builds.
5. **F5** — Bump CONTRACTS (correct the "nine core slots" statement), add the DECISIONS line and the `.orchestration/reports/<unit>.md`.

**Should fix:** F6 (`CORE_DIMENSION` consistency), F7 (explicit current prior), F8 (test depth + restore scalar-model Jacobian check).

**Scope:** split U4b/U4c/U4d/U4e out of a PR titled "U2", or retitle and review them under their own merge criteria — U4e is incomplete by the plan's own gate (N5 lag-propagation absent, self-admitted).

**Integration order:** depends on PR#1 (`67961f2`, band-aware + Huber) — merge strictly after it.

Worktree left clean (restored `Cargo.lock`; no other modifications).