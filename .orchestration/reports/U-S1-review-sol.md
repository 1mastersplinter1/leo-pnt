(Reviewer: Sol/codex, fresh context, deep seat. Verdict: FAIL.)

## Findings

1. **Blocker — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:164`  
   **Claim:** The companion’s internal watchdog guarantees that “authority cannot outlive the solution,” including when the whole companion stalls.  
   **Finding:** A watchdog implemented inside the companion cannot execute revocation, publish a state change, or raise alarms after the whole process stalls. ArduPilot will retain the last injected state until its own timeout or another independently executing mechanism intervenes. This contradicts lines 168–171 and H4/H5. The architecture likewise places the watchdog within the companion supervisor, not in an independent process/controller.  
   **HOW TO VERIFY:** In SITL, grant authority, then `SIGSTOP` or power-cut the companion. Record actuator/mode behavior and `GPS_INPUT` state for at least 5 seconds. Demonstrate which independently executing component revokes control before ArduPilot’s timeout. If none does, add an ArduPilot-side heartbeat/lease, separate hardware/process watchdog, or narrow the guarantee.

2. **Blocker — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:179`  
   **Claim:** `t_lease < GPS_TIMEOUT_MS` ensures “the companion always revokes before ArduPilot’s own GPS timeout.”  
   **Finding:** This timing inequality has no causal effect during companion failure or MAVLink link loss: the revocation cannot reach ArduPilot over a failed process/link. ArduPilot only observes missing `GPS_INPUT` and reacts after its own timeout. H5 itself concedes an unavoidable window of up to approximately four seconds at line 241, directly contradicting “always revokes before.” The baseline says continuous publication is motivated by the timeout; it does not require or establish that an internal companion lease must beat it.  
   **HOW TO VERIFY:** Break the MAVLink connection while authority is granted and timestamp: last accepted `GPS_INPUT`, internal lease expiry, any message received by ArduPilot conveying revocation, ArduPilot timeout, mode transition, and actuator behavior. Unless ArduPilot receives an independent revocation before 4000 ms, remove the claim and treat its timeout/non-manoeuvre configuration as the actual boundary.

3. **Blocker — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:113`  
   **Claim:** On revocation, ArduPilot is commanded to or held in a manual/helm-controlled state without an autonomous manoeuvre.  
   **Finding:** The document does not define a verified mechanism that performs this transition. `GPS_INPUT` supplies navigation data; it is not itself a steering-authority or manual-control handoff interface. The document’s own register admits that the mechanism remains unresolved at lines 262–264. Therefore “stop steering + hand to helm” is not presently a testable safety action, and the central revocation claim is unsubstantiated.  
   **HOW TO VERIFY:** Specify the exact MAVLink command, ArduPilot mode, RC/actuator arbitration, timeout behavior, and required parameter set. Test integrity failure, lease expiry, process death, and link loss in pinned-firmware SITL plus hardware-in-the-loop. Confirm no commanded turn, loiter, RTL, disarm, throttle change, or actuator discontinuity occurs and that physical helm input immediately dominates.

4. **Major — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:191`  
   **Claim:** S2 is entered whenever any G1–G4 condition is false.  
   **Finding:** G1 is false by default at process start and becomes false when the helm deliberately withdraws authorization. As written, normal startup and intentional disarming cause a loud warning, then maximal continuous escalation if unacknowledged. This conflates “not armed” with a fault-driven revocation and makes the escalation ladder internally inconsistent.  
   **HOW TO VERIFY:** Execute the state machine from cold startup with no arm action, then arm and deliberately disarm. Confirm expected states and annunciation. Define a separate DISARMED state and reserve S2/S3 for loss of a previously granted lease or another hazardous fault.

5. **Major — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:125`  
   **Claim:** Two revocation trigger classes cover the full authority conjunction.  
   **Finding:** The listed classes cover G2/G3 and G4 but omit G1 withdrawal, despite lines 59–61 saying any false term revokes. The escalation table does treat G1 false as S2, producing inconsistent revocation semantics.  
   **HOW TO VERIFY:** Construct a transition table for every condition changing true→false and false→true. Include explicit-arm withdrawal, startup-unarmed, protection-limit breach, calibration/ephemeris fault, and lease expiry. Require exactly one defined authority and alarm outcome for each transition.

6. **Major — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:166`  
   **Claim:** The lease is renewed by a fresh solution satisfying “G2+G3+G4.”  
   **Finding:** G4 is defined as the lease already being unexpired because it was renewed. Making G4 a prerequisite for renewal is circular and leaves initial lease acquisition and boundary-time behavior undefined. Depending on implementation, an expired lease could never become live again even after a fresh arm action, or the implementation could silently differ from the safety case.  
   **HOW TO VERIFY:** Formalize renewal as an event predicate independent of current lease validity, such as `fresh && G2 && G3`, then separately define `G4 := monotonic_now < lease_deadline`. Test startup, exact-deadline arrival, expiry, recovery, and explicit re-arm.

7. **Major — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:237`  
   **Claim:** A few-hundred-millisecond lease renewed only by a “fresh” solution safely handles loss of all LEO observations.  
   **Finding:** The baseline explicitly permits 5 Hz propagated dead-reckoned fill between absolute observations and expects LEO observations to be sparse. “Fresh solution” is not defined tightly enough to distinguish a newly propagated DR solution from a new absolute/integrity observation. If every propagated frame renews the lease, loss of all LEO can continue renewing indefinitely until G2 grows; if only absolute solutions renew it, normal operation may revoke within approximately one second. The degradation row’s “G2, then G4” behavior is therefore not testable.  
   **HOW TO VERIFY:** Define which estimator events qualify for lease renewal and the independent maximum age of each safety-relevant source. Replay minutes-long gaps in LEO observations while IMU propagation continues and verify the precise revoke time against the intended DR authority policy.

8. **Major — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:238`  
   **Claim:** An “innovation-consistency / NEES-style” online monitor provides a covariance-realism check that does not itself trust covariance.  
   **Finding:** NEES explicitly normalizes state error using the reported covariance and normally requires truth; it is not an online covariance-independent detector aboard a GNSS-denied vessel. Innovation consistency also uses predicted innovation covariance. The proposed mitigation is technically misstated and does not independently close optimistic-covariance faults.  
   **HOW TO VERIFY:** Specify the exact statistic, inputs, reference truth or redundancy, thresholds, false-alarm probability, and independence assumptions. Demonstrate detection using injected Jacobian, missing-propagation, and process-noise faults in GNSS-withheld replay. Rename offline NEES evidence separately from any online monitor.

9. **Major — high confidence**  
   **File:** `docs/design/SAFETY_CASE.md:210`  
   **Claim:** The physical override and kill-cord form the final backstop when the human does not respond.  
   **Finding:** The kill-cord only acts if its lanyard is physically pulled. It does nothing for a distracted or incapacitated helm who remains in position, which the document acknowledges at line 243. Thus the chain is real for displacement/overboard scenarios but not for the full mandated “human does not respond” case. The safety case labels that residual accepted without an acceptance authority, criterion, or required operational mitigation.  
   **HOW TO VERIFY:** Perform a hazard analysis for at least distracted, unconscious-at-helm, overboard, disconnected-cord, and failed-manual-override cases. Identify who formally accepts each residual. Require and test a second helm, dead-man device, or equivalent if unattended continued propulsion is outside the trial risk envelope.

10. **Major — medium confidence**  
    **File:** `docs/design/SAFETY_CASE.md:65`  
    **Claim:** The four-condition conjunction is “precisely and testably” specified.  
    **Finding:** Several predicates remain non-executable: protection-limit numbers are absent; “frequency-reference calibration … unexpired” has no baseline expiry rule; ephemeris validity is later assigned to G3 although G3’s predicate does not clearly include it; the stabilization dwell and all relevant timings are unset. Marking them `[UNVERIFIED]` is honest but does not satisfy the brief’s requirement for a precise, testable authority contract before steering.  
    **HOW TO VERIFY:** Produce a requirements table containing units, numerical thresholds, comparison operators, age limits, evaluation rate, missing-data behavior, hysteresis, dwell, and test IDs for every G1–G4 subpredicate. Trace every item to baseline authority or explicitly mark it as a proposed baseline change.

11. **Major — high confidence**  
    **File:** `docs/design/SAFETY_CASE.md:241`  
    **Claim:** During MAVLink link loss, the companion “detects link loss and treats it as revocation.”  
    **Finding:** Detection requirements are absent: no heartbeat/ACK source, deadline, directionality, or proof that a one-way companion→ArduPilot failure is observable by the companion. More importantly, internal revocation cannot alter the autopilot through the failed link. This mitigation therefore does not control the stated hazard.  
    **HOW TO VERIFY:** Inject independent TX, RX, full-duplex, intermittent, and corrupt-frame faults. Establish detection latency for each and show the mechanism that changes actuator authority despite the failed channel.

12. **Major — medium confidence**  
    **File:** `docs/design/SAFETY_CASE.md:242`  
    **Claim:** A companion software fault or spoofed `GPS_INPUT` is mitigated by “the same integrity monitor + watchdog.”  
    **Finding:** The integrity monitor, authority supervisor, watchdog, and publisher are all within the same companion fault domain. A process corruption capable of producing plausible false `GPS_INPUT` can also falsify integrity status or renew the lease. The hazard row recognizes the correlated residual but nevertheless describes the non-independent controls as mitigation. Physical kill-cord does not detect plausible wrong steering.  
    **HOW TO VERIFY:** Define fault-containment boundaries and inject faults after the integrity decision, in the publisher, supervisor state, and outgoing MAVLink serialization. Demonstrate an independent consumer or monitor rejects unauthorized/wrong output, or explicitly classify the residual as uncontrolled and obtain risk acceptance.

13. **Minor — high confidence**  
    **File:** `docs/design/SAFETY_CASE.md:194`  
    **Claim:** The caution band supplies hysteresis.  
    **Finding:** S1 returns to S0 when the metric “recovers past the caution threshold,” apparently using the same threshold for entry and exit. That is not hysteresis and permits S0/S1 alarm flapping, although latched S2 prevents steering-authority flapping after revocation.  
    **HOW TO VERIFY:** Define distinct caution-entry and caution-clear thresholds with ordering and dwell, then run noisy boundary traces through the state machine.

14. **Minor — high confidence**  
    **File:** `.orchestration/reports/U-S1.md:32`  
    **Claim:** `t_lease < 4000 ms` means the companion “always revokes before ArduPilot could fire its own failsafe.”  
    **Finding:** The report repeats the invalid guarantee and omits the critical distinction between internal companion state and a revocation observable by ArduPilot.  
    **HOW TO VERIFY:** Update the report after resolving findings 1–3 and cite the SITL/HIL evidence demonstrating the actual responder ordering.

Coverage result: all four brief deliverables and all seven mandated hazard categories are present. The failure is in correctness, testability, and the claimed independence/effectiveness of the backstop chain—not omission or style.

**FAIL**
