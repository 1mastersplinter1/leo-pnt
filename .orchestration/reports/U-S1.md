# U-S1 report — safety case (fix round 1, after dual-seat FAIL)

Contract: v1 (2026-07-22)

Deliverable: `docs/design/SAFETY_CASE.md` (handoff deliverable 6). Documentation-only unit;
no code, no git commit, no web research. Files owned/written: `docs/design/SAFETY_CASE.md`,
`.orchestration/reports/U-S1.md`. Reviews addressed: `U-S1-review-sol.md` (codex deep seat,
findings 1–14) and the coordinator-relayed Sonnet completeness findings B1–D2. Decisions
consulted: D10, D12, D13, D14.

## The one error that drove most blockers

Findings 1, 2, 3, 11, 14 are the same mistake: the draft conflated **companion-internal
state** (lease/watchdog) with **behaviour observable to ArduPilot**. Fix: §2.1 now splits
revocation into **Case A** (companion alive/link up → active MAVLink mode-set to a helm mode,
then cease nav injection, then alarm; ordering to be SITL/HIL-verified) and **Case B**
(companion dead / link lost → the internal watchdog cannot reach ArduPilot; the real
responders are ArduPilot's pre-configured non-manoeuvre GPS timeout + the physical helm/kill-
cord). The "companion always revokes before ArduPilot's failsafe" guarantee is withdrawn;
`t_lease < GPS_TIMEOUT_MS` is now scoped to Case A only. §3.1 narrows the watchdog guarantee
to the companion-alive fault class. This propagated into H4/H5/H6 and the degradation table.

## Codex (Sol) findings 1–14

| # | Finding | Disposition |
|---|---|---|
| 1 | Internal watchdog can't act after whole-process stall | **Fixed** — §2.1 Case B, §3.1 "limits", degradation "Companion process stalls" row now = Case B (ArduPilot timeout + physical layer, not internal watchdog). |
| 2 | `t_lease < GPS_TIMEOUT_MS` gives no guarantee across a dead link | **Fixed** — claim withdrawn; scoped to Case A in §2.1 and §3.1; H5 rewritten. |
| 3 | No defined mechanism for "stop steering + hand to helm" | **Fixed** — §2.1 Case A defines the MAVLink mode-set path (candidate command/mode + RC arbitration, all [UNVERIFIED] SITL/HIL), explicitly states `GPS_INPUT` is not a control interface, and marks "hand to helm" unproven until the SITL/HIL ordering test passes (§5). |
| 4 | S2 entered on normal startup/intended disarm (G1 default-false) | **Fixed** — distinct **DISARMED** state added; §2.2 transition table makes helm-disarm a quiet handoff (no alarm); S2/S3 reserved for fault revocation. |
| 5 | Trigger classes omit G1 withdrawal | **Fixed** — full transition table (§2.2) covers every condition true→false / false→true with one authority + annunciation outcome each; intended-disarm vs fault-revocation distinguished. |
| 6 | G4 renewal circular (G4 prerequisite for its own renewal) | **Fixed** — §1.1 defines `R := seq-advanced ∧ G2 ∧ G3` (no lease-validity dependency) and `G4 := now < lease_deadline`; initial acquisition/recovery/boundary defined. |
| 7 | "Fresh solution" undefined vs DR fill; revoke time untestable | **Fixed** — §1.1: every 5 Hz frame incl. DR fill renews G4; authority in all-LEO-loss ends via **G2** (`t_dr` or PL breach). Three-timer table separates `t_lease`, `t_dr`, per-source freshness. |
| 8 | NEES mischaracterised as online covariance-independent monitor | **Fixed** — H2 rewritten: NEES-vs-truth is **offline** (replay); online is **NIS** (covariance-coupled, partial) plus **cross-sensor redundancy residuals** (covariance-independent, limited); prior claim explicitly withdrawn; residual "not closed online". |
| 9 | Kill-cord doesn't cover incapacitated-at-helm; residual "accepted" without authority | **Fixed** — §3.3 sub-case coverage table; unconscious-at-helm marked **UNCONTROLLED**, not accepted (no risk-acceptance authority here); requires operational control (second crew / helm dead-man) with trial-authority acceptance; trial envelope should bar unattended propulsion until then. |
| 10 | Predicates non-executable | **Fixed** — §1.2 sub-predicate requirements table (metric/unit, operator, eval rate, missing-data behaviour, test ID, trace); numbers remain [UNVERIFIED] but structure is testable and each item traced to baseline authority or marked "proposed". |
| 11 | Link-loss "detects and treats as revocation" can't alter autopilot over failed link | **Fixed** — §2.1 Case B: one-way TX failure not observable without return heartbeat; detection serves alarm/log only; actuator control is via ArduPilot timeout + physical layer. H5 rewritten. |
| 12 | Shared companion fault domain cited as independent mitigation | **Fixed** — new §3.0 fault-domain analysis; H6(a) mitigation-by-in-companion-modules withdrawn; correlated process-level fault classified **uncontrolled** pending an out-of-process monitor (registered §5). |
| 13 | Single-threshold "hysteresis" | **Fixed** — §3.2 distinct `caution_enter`/`caution_clear` + `dwell_clear`, ordering stated; replaces vague "margin shrinking". |
| 14 | Report repeats invalid guarantee | **Fixed** — this report; guarantee withdrawn and scoped (see header + finding 2). |

## Sonnet completeness findings (coordinator-relayed)

| # | Finding | Disposition |
|---|---|---|
| B1/C1 | G3 label covers ephemeris age but its definition is calibration-only | **Fixed** — ephemeris-age moved into **G2** (observation integrity, sub-predicate G2e); G3 is calibration/extrinsics only; degradation "Ephemeris" row retriggered to G2 consistently. |
| B3 | G1 arm has no bus message; per D13 an arm-command message is a contracts requirement | **Fixed** — §1 G1 note + §5 register the D13 dependency (U-C1 contracts v3); `ARCHITECTURE.md` not edited (not owned). |
| C2/D1 | "unexpired" freq-ref calibration — no expiry concept in baseline | **Fixed** — word "unexpired" dropped; G3 tests presence + identity match only; a calibration validity window recorded in §5 as a **proposed baseline change**, not assumed. |
| B4 | Add D10 Orbcomm-fusion caveat | **Fixed** — §2.3 Orbcomm caveat: observations excluded from fusion until a second receiver-clock/nuisance state exists; also in §5. |
| C3 | Define or de-scope "margin shrinking" | **Fixed** — replaced by the concrete `caution_enter` crossing in §3.2. |
| B2 | "Calibration ID missing/mismatched" row is not one of the 11 literal baseline degradation rows (it derives from extrinsics prose); breaks the 1:1 mapping | **Fixed** (round 2, on receipt of the Sonnet file) — §2.3: the 11 literal baseline rows are listed 1:1 in baseline order (ending with "Companion process stalls"); the calibration row is moved below them and labelled an *additional row* citing its source (baseline §Estimator and degradation contract extrinsics rule). |
| B5 | §2.3 rows drop qualifying clauses from baseline text (one-mag, both-mags, IMU-stale) | **Fixed** (round 2) — restored: one-mag row now includes "IMU turn dynamics and any selected non-magnetic heading sensor"; both-mags row includes "short-term attitude propagation and any selected non-magnetic heading measurement"; IMU-stale row includes "journalling and recovery may continue". |
| D2 | "stabilisation dwell" not tagged [UNVERIFIED] at first use | **Fixed** (round 2) — the dwell terms (`dwell_clear`, `dwell_rearm`) are now tagged **[UNVERIFIED]** at their first mention in §1.2, ahead of the later §2.2/§3.2/§5 uses. |

## Rejected / partial

- Nothing rejected. All 22 findings fixed. B2/B5/D2 were briefly "not actionable" in round 1
  (no Sonnet file on disk); the coordinator supplied `U-S1-review-sonnet.md` and all three are
  now applied in round 2.

## Key design changes this round

- Revocation is now a defined, two-case MAVLink action, not an assertion; Case B honestly
  hands the backstop to ArduPilot's bounded non-manoeuvre timeout + the physical layer.
- Fault-domain boundaries (§3.0) make independence claims honest; two residuals (H6(a)
  companion process-level fault; H7 unconscious-at-helm) are now labelled **uncontrolled** and
  routed to an out-of-process monitor and an operational control respectively — not papered
  over as mitigated.
- Authority is a clean state machine (DISARMED + S0–S4) with a non-circular lease predicate,
  three separated timers, real caution-band hysteresis, and a structurally testable
  sub-predicate table.

## Open uncertainties (full list in §5 register)

Per-epoch protection-limit numbers; the timer set; the Case-A MAVLink revocation
command/mode/ordering; the Case-B ArduPilot non-manoeuvre failsafe params; MAVLink signing;
the out-of-process independent monitor (H6a); the content-liveness check (H4); the operational
human-response control (H7); the G1 arm bus message (D13); the Orbcomm fusion gate (D10).

## Evidence / process

- Read in full: `U-S1-review-sol.md` (14 findings) and `DECISIONS.md` (D10/D12/D13/D14);
  re-read `DESIGN_BASELINE.md`, `ARCHITECTURE.md`, handoff failure modes 3/5/9 for the
  degradation contract, DR-timeout rule and the never-auto-manoeuvre rule.
- Confirmed no `U-S1-review-sonnet.md` file exists; addressed the coordinator-relayed Sonnet
  findings that carried substance.
- Documentation-only unit: no executable test suite. No git commit (D3, brief). `ARCHITECTURE.md`
  deliberately not edited (not owned; the arm-message gap is registered, per D13).
