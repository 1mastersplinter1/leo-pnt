# Brief U-A1 — Authority supervisor (replace the fail-open stub)

Contract version: v4.1 → you author v5. Workspace: dedicated worktree, branch `unit/U-A1`.
Commit here; never merge to main. Read first, in order: `.orchestration/CONTRACTS.md`,
`docs/design/SAFETY_CASE.md` (§1–§3 are your specification — the G1–G4 conjunction, the
fail-closed parameter gate, lease renewal §1.1, the TOTAL state/event matrix §2.2, revocation
semantics, escalation ladder), `docs/design/DESIGN_BASELINE.md` (degradation table),
`.orchestration/DECISIONS.md` (D13, D17, D27 — D27 names this stub the blocking gap).

## Goal
1. **CONTRACTS v5** (append): the authority supervisor contract — an `AuthorityParams` struct
   carrying every §5-registered numeric (protection limits for horizontal position/velocity/
   heading, `t_lease`, `t_dr`, `t_eph`, `dwell_clear`, `dwell_rearm`, `caution_enter`,
   `caution_clear`, `revoke_threshold`, `T_ack`) where each field is `Option<f64>`/typed and
   **any `None` means the supervisor can NEVER grant authority** (the safety case's fail-closed
   gate, implemented literally); the supervisor's inputs (solution epochs with accuracies,
   integrity events, ArmCommand, clock-service monotonic time) and outputs
   (`steering_authorised`, state-transition integrity events, alarm level).
2. **`crates/pnt-integrity`**: implement `AuthoritySupervisor` (the real `IntegrityAuthorityGate`):
   - G1 arm latch from `ArmCommand` (default disarmed; withdraw → disarm; re-arm required
     after any latched revocation).
   - G2 protection-limit test per epoch on the DRMS accuracies vs the active profile's params;
     absolute-observation-age (`t_dr`) test.
   - G3 calibration-ID presence/match hook (accept a validator closure; default = reject-on-missing).
   - G4 lease: renewal event R := sequence-advanced ∧ G2 ∧ G3 (non-circular per §1.1);
     `lease_deadline = now + t_lease`; expiry drops authority.
   - The §2.2 state machine EXACTLY: states and every event edge from the total matrix,
     simultaneous-event priority order, dwell timers, ack/escalation (`T_ack`), latched
     revocation, DISARMED quiet state. Revocation output is only ever
     "stop asserting authority + alarm" — the supervisor has no manoeuvre outputs at all
     (make unrepresentable: no RTL/Loiter/disarm variant exists in its output type).
   - All time injected (monotonic ns argument), zero wall-clock reads — deterministic tests.
3. **Executive wiring** (`crates/fusion-executive`): construct with the real supervisor;
   epochs' `steering_authorised` comes from it; ArmCommand routing (already reaches the
   integrity port) now drives G1. Keep a test constructor for stub behavior where existing
   tests need it — but the default construction must be the real, fail-closed supervisor,
   and existing tests that assumed always-true authority must be updated honestly (assert
   the new correct value), never weakened.

## Tests (TDD, strictly — this is the safety supervisor)
- Exhaustive state/event matrix test: iterate every (state, event) cell from §2.2 and assert
  the successor + authority + annunciation against a table literal transcribed from the doc
  (cite section). A cell mismatch must name the cell.
- Fail-closed: any single `AuthorityParams` field `None` → grant impossible even with perfect
  solution + armed helm.
- Lease non-circularity: expired lease + fresh in-integrity frame + armed → re-grant per §1.1;
  no renewal without sequence advance.
- Dwell/ack timing at exact boundaries (t = dwell, t = dwell±1ns).
- Latch: post-revocation good solution does NOT re-grant without fresh arm.
- Property test (proptest or hand-rolled): random event sequences never yield
  authorised=true while any G-condition is false at that instant.
Gate: PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check.

## Files owned
`crates/pnt-integrity/**`, `crates/fusion-executive/**`, `crates/pnt-types/**` (only if a
supervisor output type needs adding — additive), `## v5` of `.orchestration/CONTRACTS.md`,
`.orchestration/reports/U-A1.md`.

## Report
Dispositions vs SAFETY_CASE §1–§3 clause by clause (a table: clause → code location → test),
deviations flagged loudly, [UNVERIFIED] list, real gate output.
