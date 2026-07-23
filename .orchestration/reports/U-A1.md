# U-A1 report — authority supervisor

Contract authored: v5. Branch: `unit/U-A1`.

## Result

Replaced the unconditional `IntegrityStub` on the default executable construction path with
`AuthoritySupervisor`. The supervisor is deterministic, consumes injected monotonic
nanoseconds only, starts disarmed, implements the G1--G4 conjunction and fail-closed numeric
register, renews only on sequence advance plus G2/G3, implements the complete §2.2 state
matrix, and exposes only authority/state/alarm/transition outputs. It has no manoeuvre output.

The legacy stub remains available only through the explicitly named `test_stub` constructor.
`default_fail_closed` constructs the real supervisor with all parameters unset, and its
executive test proves that even arm plus a solution produces `steering_authorised=false`.

## SAFETY_CASE §1--§3 disposition

| Clause | Code location | Test/evidence |
|---|---|---|
| §1 fail-closed parameter gate | `pnt-integrity::AuthorityParams::is_complete`, `AuthoritySupervisor::output` | `every_missing_parameter_is_fail_closed` removes each of all 15 fields |
| §1 G1 human arm latch | `AuthoritySupervisor::arm_command`; fault entry clears latch | `revocation_is_latched_until_ack_dwell_and_fresh_arm`; executive `arm_command_reaches_authority_and_never_filter_update` |
| §1 G2 profile PL, heading covariance, DR age, ephemeris age | `AuthoritySupervisor::evaluate_solution` | fail-closed, boundary, latch, and randomized invariant tests |
| §1 G3 calibration | injected `with_calibration_validator`; missing/empty always rejected | randomized invariant test varies missing/matched ID |
| §1 G4 deadline | `lease_deadline_ns`; `tick`; equality expires | `lease_is_non_circular_and_requires_sequence_advance` checks deadline ±1 ns |
| §1.1 non-circular renewal | `advanced && new_g2 && new_g3`, independent of current G4 | `lease_is_non_circular_and_requires_sequence_advance` |
| §2.1 revocation semantics | `AuthorityOutput` has no vehicle-control or manoeuvre variant | type inspection; fault tests assert false authority |
| §2.2 total matrix | `matrix_successor`, `simultaneous_successor`, destination `output` | `exhaustive_safety_case_section_2_2_matrix`: all 66 named cells plus guarded cells and priority cases |
| §2.2 quiet disarm and latched warning/escalation | transition function and fault-first processing | matrix test; latch test |
| §3.1 live watchdog | monotonic `tick`; no wall-clock access | lease boundary test |
| §3.2 caution hysteresis, clear/re-arm dwell, ACK escalation | supervisor timers and matrix events | `dwell_and_ack_boundaries_are_exact`; latch test checks re-arm at −1 ns and exact dwell |
| G invariant | final authority conjunction plus state destination | hand-rolled deterministic random sequences, 6,368 ticks total |
| Executive wiring | `Executive::emit_epoch` calls `solution` before authority; ArmCommand only integrity | `default_real_supervisor_is_fail_closed` and existing routing tests |

## Deviations / limits

No deviation from the requested supervisor state/event matrix is known.

- `[UNVERIFIED]` Numeric values remain deliberately unset in default construction. v5 makes
  that state fail-closed; this unit does not invent deployment values.
- `[UNVERIFIED]` Per-source freshness evaluation remains upstream integrity-monitor work. The
  brief's required `AuthorityParams` field list does not define individual source names or
  fields; this supervisor consumes the resulting G2 epoch integrity plus the explicitly
  required DR and ephemeris ages.
- `[UNVERIFIED]` The executive currently conveys `ephemeris_age_s = 0` only after its existing
  ephemeris store has accepted the observation through its own configured age gate. The real
  age is not exposed by the existing ephemeris-store API. Default authority remains closed,
  and direct supervisor inputs/tests exercise `t_eph`; deployment wiring should convey the
  measured age when that API exists.
- `[UNVERIFIED]` The executive does not yet journal `AuthorityOutput.transition`; the output
  is available from the supervisor, while the current journal contract has no authority-state
  event schema. This does not affect authority decisions.
- Case-A MAVLink hand-to-helm commands remain outside this unit and intentionally absent from
  the supervisor output type, per §2.1 and the brief.

## Gate evidence

Run in `/home/od/work/leo-pnt-wt-UA1` on 2026-07-23:

```text
cargo test
PASS: 48 tests (16 executive, 6 ephemeris, 13 estimator, 6 authority,
4 predictor, 3 types) plus all doc tests

cargo clippy --all-targets -- -D warnings
PASS

cargo fmt --all -- --check
PASS
```
