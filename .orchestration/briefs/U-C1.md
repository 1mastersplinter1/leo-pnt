# Brief U-C1 — Contracts v2 + Rust workspace + fusion executive skeleton

Contract version built against: v1 → you produce v2.

**Workspace: you are already inside a dedicated git worktree on branch `unit/U-C1`. Work here;
do not create branches, do not switch branches, do not touch main.**

Read first, in order: `.orchestration/CONTRACTS.md`, `docs/design/DESIGN_BASELINE.md`,
`docs/design/ARCHITECTURE.md` (the approved baseline — reviews already resolved), and
`.orchestration/DECISIONS.md`. Binding for you: **D10** — Orbcomm observations must not enter
fusion until a second receiver-clock state or per-receiver nuisance term exists; contracts v2
must state this, and the executive's routing must reject Orbcomm measurements by default.

## Goal
1. **CONTRACTS.md v2**: append a `## v2` section fixing what v1 deferred: measurement-bus
   message schema (Rust type names + field types), coordinate frames, on-disk formats,
   the module-owns-time statement, and the rate contract — all derived from the reviewed
   design docs, no new invention. Do not edit the v1 section.
2. **Rust workspace** at repo root: `Cargo.toml` workspace + crates under `crates/`
   following ARCHITECTURE.md module boundaries. Executive-first rule: the crate that exists
   fully is the **fusion executive** — the loop that owns time, propagates the (stub) filter
   on every IMU tick, dispatches bus measurements to (stub) handlers, evaluates the (stub)
   integrity/authority gate, and emits solution epochs. Other modules are trait-typed stubs
   the executive already calls; they grow later, into it.
3. **`gnss_authority` enforcement**: config parsing where `production | recorded_only | off`
   are the only accepted values; `recorded_only` routes GNSS measurements to the truth
   journal sink and never to fusion; an unrecognised value returns a hard error (test this).
   Same executive code path in all three modes (test this: mode changes routing tables only).
4. **Propagation honesty test**: with zero measurements and nonzero IMU input, filter-stub
   covariance hook must be called every tick (test that a covariance-growth counter
   advances with IMU ticks alone).

## Method — TDD, strictly
Red-green per behavior: write the failing test, run it, watch it fail, implement minimally,
watch it pass. Loop `cargo test` and `cargo clippy --all-targets -- -D warnings` until both
are clean. Commit frequently on branch `unit/U-C1` with conventional messages — you MAY
commit in this worktree (supersedes wave-1's no-commit rule); never merge to main.

## Files owned
`Cargo.toml`, `crates/**`, the `## v2` section of `.orchestration/CONTRACTS.md`,
`.orchestration/reports/U-C1.md`. Nothing else — do not touch docs/ or main.

## Acceptance
- `git switch unit/U-C1 && cargo test && cargo clippy --all-targets -- -D warnings` all pass.
- Tests exist and pass for: config rejection of bad `gnss_authority`; recorded_only routing
  (GNSS reaches truth sink, not fusion); propagation-on-IMU-tick; executive end-to-end smoke
  (synthetic IMU + one synthetic measurement in → solution epoch out).
- No filter state or bus message type without a consumer in the executive loop.

## Report
`.orchestration/reports/U-C1.md`: what changed, evidence (actual test-run output pasted),
assumptions, open uncertainties, contract version (v2, which you authored — flag every place
v2 resolved a v1 deferral so review can check it against the design docs).
