(Sol verify pass on U-S1 rework. Verdict: FAIL.)

1. **Resolved** — watchdog guarantee is now correctly limited to companion-alive faults; whole-process death is explicitly outside its control (`SAFETY_CASE.md:188–207, 306–312`).

2. **Resolved** — the invalid claim that `t_lease < GPS_TIMEOUT_MS` guarantees revocation across process/link failure is withdrawn and scoped to Case A (`202–207, 314–315`).

3. **Not resolved** — Case A supplies only candidate commands/modes and explicitly admits that “hand to helm” remains unproven (`176–186, 407–410`). The exact mode, RC arbitration, command acknowledgement/failure behavior, and actuator transition required by the original finding are still undefined.

4. **Resolved** — startup and intentional disarm now enter a quiet DISARMED state rather than the warning ladder (`211–231`).

5. **Not resolved** — G1 withdrawal is covered, but the claim that every Boolean transition has exactly one outcome is false (`233–239`). The table does not define G1 falling from WARNING/ESCALATED/LATCHED-SAFE, G2/G3/G4 recovery in those states, G4 expiry while DISARMED, or acknowledgement/recovery races.

6. **Resolved** — renewal is non-circular: `R` depends on sequence advance plus G2/G3, while G4 is only the deadline comparison; acquisition and expiry boundaries are defined (`89–100`).

7. **Resolved** — DR-propagated frames explicitly renew G4, while loss of absolute observations ends authority through G2 using `t_dr` or protection-limit breach (`102–117, 250`).

8. **Resolved** — NEES is correctly classified as offline/truth-dependent; online NIS is identified as covariance-coupled, redundancy residuals have limited coverage, and the remaining online gap is acknowledged (`381`).

9. **Resolved** — kill-cord coverage is separated by human-failure case, unconscious-at-helm is explicitly UNCONTROLLED, and operational control plus trial-authority acceptance is required (`345–366, 386, 420–423`).

10. **Not resolved** — the table adds operators, units and missing-data behavior, but the protection limits, timers, freshness limits, dwell values and exact test thresholds remain `[UNVERIFIED]` (`119–138, 395–400`). The authority contract therefore still is not numerically executable before steering.

11. **Resolved** — link detection is now alarm/logging only and is no longer claimed to control ArduPilot over the failed channel (`196–200, 384`).

12. **Resolved** — the companion components are placed in one process/board fault domain; H6(a) is honestly UNCONTROLLED and no same-domain integrity monitor/watchdog is cited as independent mitigation (`274–296, 385, 415–418`). H7 is likewise honestly uncontrolled where appropriate.

13. **Resolved** — hysteresis now has ordered, distinct entry and clear thresholds, a clear dwell, and latched revocation/re-arm behavior (`223–224, 321–333, 382`).

14. **Resolved** — the report withdraws the “always revokes first” guarantee and distinguishes Case A from Case B (`.orchestration/reports/U-S1.md:14–25`).

### NEW findings

- **Blocker — high confidence — `docs/design/SAFETY_CASE.md:190–205, 309–315, 384, 411–413`**  
  The document still asserts that silence on `GPS_INPUT` causes ArduPilot, at 4000 ms, to take a configured “non-manoeuvre GPS-failsafe” action and thereby bounds authority. `GPS_TIMEOUT_MS` is a GPS-backend data timeout; the document provides no evidence that Rover treats it as a dedicated control-authority timeout or immediately changes mode/actuation. Official Rover failsafe documentation describes radio, battery and GCS failsafes, while GPS loss affects position/EKF availability and behavior depends on mode, EKF state and configuration—not simply on this backend timeout. Calling the action bounded while merely marking the parameters `[UNVERIFIED]` remains an asserted central safety mechanism. [ArduPilot Rover failsafes](https://ardupilot.org/rover/docs/rover-failsafes.html)  
  **How to verify:** On the pinned Rover firmware, stop only `GPS_INPUT` in every intended autonomous mode. Record GPS backend status, EKF source/validity, mode, navigation demand, servo outputs and RC override from last input through at least 10 seconds. Identify the exact source path and parameters producing any transition. Until proven, describe Case B as uncontrolled by the current design rather than as a bounded non-manoeuvre handoff.

- **Major — high confidence — `docs/design/SAFETY_CASE.md:6–8, 255–258, 306–312`; `docs/design/DESIGN_BASELINE.md:146`**  
  The subordinate safety case correctly says an in-process watchdog cannot handle companion process death, but the governing baseline still requires that watchdog to expire authority on a companion process stall. Thus the documents remain normatively inconsistent; under the safety case’s own precedence rule, the erroneous baseline claim governs.  
  **How to verify:** Revise/approve the baseline to distinguish estimator-only stall, live-supervisor failure, whole-process death and board/power loss, then trace each class to an independently executing responder and test.

- **Major — high confidence — `docs/design/SAFETY_CASE.md:218–239`**  
  The state machine claims complete edge coverage but omits multiple state/event combinations and race precedence. This could yield implementation-dependent alarm, latch or re-arm behavior.  
  **How to verify:** Produce a total state/event matrix for every state and every G1–G4 edge, ACK, arm/re-arm and simultaneous event; define priority and prove every cell has exactly one next state, authority result and annunciation.

**FAIL**
