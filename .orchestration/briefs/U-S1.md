# Brief U-S1 — Safety case

Contract version: v1. Read first: `.orchestration/CONTRACTS.md`, `docs/HANDOFF_PROMPT_BLADERF.md`
(especially failure mode 9 and "Fixed constraints"), `docs/design/DESIGN_BASELINE.md`,
`docs/design/ARCHITECTURE.md` (approved baseline), `.orchestration/DECISIONS.md`.

## Goal
`docs/design/SAFETY_CASE.md` — handoff deliverable 6. Must state, precisely and testably:
1. **What grants steering authority**: the full conjunction of conditions (solution integrity
   within the active profile's protection limits, calibration validity per the baseline's
   extrinsics rule, watchdog liveness, human arm action) and where each is evaluated
   (companion process, upstream of MAVLink — ArduPilot's 100 m clamp makes its EKF variance a
   censored view; restate why the authority gate cannot live in ArduPilot).
2. **What revokes it**: per degradation row of the baseline; revocation semantics on a manned
   fast boat (never auto-RTL/Loiter/disarm — an unannounced manoeuvre at speed is the hazard;
   revocation means stop steering + alarm + hand to helm).
3. **The backstop when the human does not respond**: supervisor monotonic watchdog so authority
   cannot outlive the solution; un-acknowledged alarm escalation ladder (state stages, timing,
   and the terminal state); the physical, controller-independent override (helm kill-cord)
   as the final layer outside software.
4. **Hazard table**: hazard → cause → mitigation → residual risk, covering at minimum: stale
   solution steering, optimistic covariance (wrong-Jacobian class faults), authority flapping,
   silent estimator halt vs authority-alive inversion, MAVLink link loss mid-authority,
   GPS_INPUT spoof/failure of the companion process itself, human-override failure.
Mark anything not derivable from the baseline/handoff as [UNVERIFIED] or estimate. This
document is subordinate to DESIGN_BASELINE.md and must say so.

## Files owned
Only: `docs/design/SAFETY_CASE.md`, `.orchestration/reports/U-S1.md`. No code, no git commit, no web research.

## Report
`.orchestration/reports/U-S1.md`: decisions taken, assumptions, open uncertainties, contract version.
