# U-A1 report — authority supervisor fix round U-A1.1

Contracts authored: v5 and v5.1. Branch: `unit/U-A1`.

## Result

The production executive uses the fail-closed `AuthoritySupervisor`; the explicitly named
`test_stub` remains test-only. U-A1.1 closes both reviewers' findings around stale arm
edges, live simultaneous-event priority, DR-fill delivery, monotonic time, ACK routing, and
calibration-validator construction. The red-first checkpoint is commit `9c5de70`.

The live supervisor now merges events sharing a monotonic tick and calls
`simultaneous_successor` over the tick's origin state and accumulated events. This wires the
normative six-tier priority into the live path (rather than duplicating guard logic), so a
fault beats disarm independently of call order and annunciation is identical.

The executive emits a propagated solution on every accepted IMU propagation tick. This
chooses the every-tick cadence: no unsupported decimation constant is introduced, and lease
renewal remains controlled by sequence advance plus G2/G3. The executive regression proves
DR frames keep a 2 ns lease renewed through the 5 ns `t_dr` boundary and authority is revoked
only when absolute-observation age exceeds `t_dr`.

## Fix-mandate dispositions

1. **Opus F1 / Sonnet S1 — fixed.** Rising Arm edges are accepted only in `Disarmed` or
   `LatchedSafe`. Fault entry and Disarm clear `pending_arm_edge`. The warning/escalated
   regression presses Arm in each alarm state, ACKs, waits through `dwell_rearm`, and proves
   no grant occurs until a fresh post-latch edge. The randomized generator now includes ACK
   and arm-during-alarm actions and tracks the accepted-arm-since-revocation invariant.
2. **Sonnet S2 — fixed.** `AuthoritySupervisor::apply` performs live same-tick event merging
   through `simultaneous_successor`. `live_supervisor_fault_beats_disarm_in_both_same_tick_orders`
   drives disarm→fault and fault→disarm and asserts equal `Warning` output and loud alarm.
3. **Sonnet S3 — fixed.** IMU propagation calls `emit_epoch`, which reaches both
   `IntegrityAuthorityGate::solution` and NDJSON. The all-LEO-loss/DR-fill regression proves
   `t_dr`, not the continuously renewed `t_lease`, governs final revocation.
4. **Opus F2 — fixed.** All supervisor time-bearing entry points enforce nondecreasing
   nanoseconds. Regression clears G2/G3, latch, pending edge, and lease before a debug
   assertion; equal timestamps remain valid. `backward_time_revokes_but_repeated_time_is_accepted`
   covers both cases.
5. **Opus F3 — fixed.** CONTRACTS v5.1 adds `AckCommand`; the bus type and journal codec are
   covered by type and bit-exact round-trip tests. `IntegrityAuthorityGate::acknowledge` and
   the executive route it only to integrity; the routing test proves no estimator update.
6. **Opus F4 — fixed.** `AuthoritySupervisor::fail_closed` rejects complete parameters, and
   its validator always rejects. The only complete/grant-capable construction path is
   `with_calibration_validator`. The constructor test proves the hard error.
7. **Report accuracy — fixed below.** Formerly overstated rows now cite live, new regression
   tests rather than the free helper or broad indirect coverage.

Opus F5 remains an integration note for U-M1, not a U-A1 defect: U-A1's default-deny
supervisor is retained and its executable wiring test remains green.

## Corrected SAFETY_CASE §1–§3 clause table

| Clause | Code location | Direct test/evidence |
|---|---|---|
| §1 fail-closed 15-field parameter gate | `AuthorityParams::is_complete`; `AuthoritySupervisor::output` | `every_missing_parameter_is_fail_closed` removes each field |
| §1 G1 human arm latch | state-gated `arm_command`; fault/Disarm pending-edge clearing | `arm_edges_during_warning_and_escalated_are_ignored`; `revocation_is_latched_until_ack_dwell_and_fresh_arm`; randomized accepted-arm invariant |
| §1 G2 PL/heading/DR/ephemeris gates | `AuthoritySupervisor::evaluate_solution` | missing-field, boundary, latch, randomized tests; executive `imu_dr_fill_renews_lease_until_absolute_observation_exceeds_t_dr` |
| §1 G3 calibration identity | explicit `with_calibration_validator`; rejecting `fail_closed` constructor | `complete_params_require_an_explicit_calibration_validator`; randomized missing/matched ID cases |
| §1 G4 deadline | `lease_deadline_ns`; `tick`; equality expiry | `lease_is_non_circular_and_requires_sequence_advance` checks deadline −1 ns and equality |
| §1.1 non-circular renewal and DR fill | `advanced && new_g2 && new_g3`; executive IMU `emit_epoch` | lease sequence test; `imu_dr_fill_renews_lease_until_absolute_observation_exceeds_t_dr` |
| §2.1 revocation semantics | `AuthorityOutput` has no vehicle-control/manoeuvre variant | fault/latch/live-priority tests assert false authority and alarm only |
| §2.2 total matrix and simultaneous priority | `matrix_successor`; live `apply` accumulation into `simultaneous_successor` | 66-cell `exhaustive_safety_case_section_2_2_matrix`; `live_supervisor_fault_beats_disarm_in_both_same_tick_orders` |
| §2.2 quiet disarm and latched warning/escalation | destination `AuthorityState::output`; state-gated arm handling | both-order live priority test; warning/escalated stale-arm regression; fresh-arm latch test |
| §3.1 live watchdog/monotonic clock | `observe_time`; `tick`; no wall-clock access | `backward_time_revokes_but_repeated_time_is_accepted`; lease boundary test |
| §3.2 caution/clear/re-arm dwell and ACK escalation | supervisor timers; `AckCommand` executive route | `dwell_and_ack_boundaries_are_exact`; stale-arm/fresh-arm regressions; `acknowledge_reaches_authority_and_never_filter_update` |
| G invariant | final conjunction plus accepted-arm tracking | deterministic randomized test: 6,368 solution/tick iterations including ACK and arm-during-alarm |
| Executive wiring | command-only routes; every update/propagation calls `emit_epoch` | default fail-closed, Arm, ACK, DR-fill, and NDJSON tests |

## Deviations / limits

- `[UNVERIFIED]` Deployment numeric values remain deliberately unset. The production-default
  skeleton is therefore fail-closed and cannot accept a calibration identity.
- `[UNVERIFIED]` Per-source freshness evaluation remains upstream integrity-monitor work.
- `[UNVERIFIED]` The executive currently conveys `ephemeris_age_s = 0` for accepted or
  propagated-integrity epochs because the ephemeris store does not expose measured age.
- `[UNVERIFIED]` Authority transitions are not yet journalled because the journal contract
  has no authority-state event schema; this does not affect decisions.
- Case-A MAVLink hand-to-helm commands remain outside this unit and absent from
  `AuthorityOutput`, as required by §2.1.

## Gate evidence

Run in `/home/od/work/leo-pnt-wt-UA1` on 2026-07-23 with the mandated command:

```text
PATH="$HOME/.cargo/bin:$PATH" cargo test
PASS: 60 tests (18 executive, 6 ephemeris, 13 estimator, 10 authority,
5 journal, 4 predictor, 4 types) plus all doc tests

cargo clippy --all-targets -- -D warnings
PASS: finished dev profile with no warnings

cargo fmt --all -- --check
PASS
```
