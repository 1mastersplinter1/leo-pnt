# Safety Case

Status: **subordinate to [`DESIGN_BASELINE.md`](DESIGN_BASELINE.md)**
Contract: v1 (2026-07-22)

`DESIGN_BASELINE.md` is the single normative design document. This safety case implements
and is bounded by it. Where the two differ, the baseline governs until the baseline is
explicitly revised. This document adds no new sensor, estimator or acceptance requirement;
it specifies **how steering authority is granted, revoked and backstopped**, deliverable 6
of `docs/HANDOFF_PROMPT_BLADERF.md`, and it draws every degradation trigger from the
baseline's degradation contract.

Numbers the baseline and handoff do not supply — per-epoch protection limits, alarm and
timer values, ArduPilot failsafe parameters, the MAVLink revocation command set — are marked
**[UNVERIFIED]** and must be frozen before steering trials. Several safety claims in earlier
drafts overreached by conflating *companion-internal state* with *behaviour observable to
ArduPilot*; this revision states each boundary explicitly. Where the honest conclusion is
that a hazard is **not controlled by present design**, this document says so and registers it
(§5) rather than asserting a mitigation.

---

## 0. Terms, and the four things this case keeps separate

- **Acceptance profile** (baseline §Acceptance profiles): system-level trial statistics
  (e.g. ≤ 25 m aided, ≤ 200 m denied horizontal position) computed offline over declared test
  segments against the truth journal. They grade the *campaign*. The baseline states they
  **do not replace per-epoch integrity and authority checks**.
- **Protection limit**: the per-epoch bound the authority supervisor tests each solution
  against, derived by the solution integrity monitor (architecture module 10) from the filter
  covariance and integrity state, *independently of ArduPilot*. The baseline supplies no
  protection-limit numbers; they are **[UNVERIFIED]** and shall be no looser than the active
  acceptance profile implies.
- **Active profile**: which limit set applies, selected by `gnss_authority` — **aided** under
  `production`, **denied** under `recorded_only` and `off`. Never changes code path.
- **Fault domain**: the set of components that fail together under a single process-, board-
  or link-level fault (§3.0). Distinguishing fault domains is essential and was the central
  error of the previous draft: modules that share a fault domain cannot mitigate one another
  against faults of that domain.

Two roles:

- **Authority** is the companion's assertion of autonomous steering — whether the fused
  solution is permitted to steer via the MAVLink path. It is a *lease*, continuously
  re-earned, never a latch.
- **The helm** is the qualified person aboard with a physical, controller-independent manual
  override (baseline). The helm is the primary safety actor; software is subordinate to it at
  all times.

**Vessel-speed caveat.** The baseline vessel is a **displacement-hull** vessel in the Danish
straits (working assumption); handoff failure mode 9 frames the hazard as a *manned fast
boat*. The revocation *principle* (revocation is never itself a manoeuvre) holds at any speed.
The *timer* budgets are sized to the vessel's manoeuvre/collision time budget, which shrinks
with speed; all timer values here are conservative displacement-hull estimates, **[UNVERIFIED]**,
to be re-derived if the hull planes or slams.

---

## 1. What grants steering authority

Steering authority is granted **only while the full conjunction G1–G4 holds** (logical AND;
any false term drops authority — §2). The companion never self-grants; grant is opt-in by the
helm (G1) and continuously re-earned by the solution (G2–G4).

**Fail-closed gate on unverified parameters (per D17).** Steering authority shall **never** be
granted while **any** parameter of the authority contract — the protection limits, timers
(`t_lease`, `t_dr`), per-source freshness limits, dwells (`dwell_clear`, `dwell_rearm`) and
thresholds (`caution_enter`, `caution_clear`, `revoke_threshold`, `T_ack`, `t_eph`) — remains
**[UNVERIFIED]**. This turns the open numeric register (§5) from a list of deferred tuning
values into a hard precondition: an unfrozen parameter is not a permissive default but a closed
gate, so no on-water steering can occur until every such parameter has been frozen and verified.
This gate is in addition to, and independent of, the sea-trial gates in §2.1 and §3.3.

| # | Grant condition | Testable predicate | Evaluated in |
|---|---|---|---|
| G1 | **Human arm action** | The helm has raised an explicit arm command and not withdrawn it. Default at process start is **disarmed**. | Companion supervisor (module 11) — see arm-input gap below |
| G2 | **Solution integrity within the active profile's protection limits** | Horizontal-position, horizontal-velocity and heading protection limits are each within the active-profile set, per epoch on the true (unclamped) covariance; **and** every observation contributing to the solution passed its integrity gate, including the SupGP **ephemeris-age** gate; **and** the age of the last absolute (LEO/aided-GNSS) position-constraining observation is within the DR-authority timeout. | Integrity monitor (module 10) + observation gate (module 8) → supervisor |
| G3 | **Calibration/extrinsics validity** | Every measurement feeding the current solution carries a calibration ID that resolves and matches the surveyed antenna-phase-centre / IMU / vessel-reference extrinsics; none missing or mismatched. (Calibration/extrinsics only — ephemeris age now lives in G2.) | Bus ingress + integrity monitor |
| G4 | **Lease liveness** | `monotonic_now < lease_deadline` on the clock service's monotonic domain — i.e. a fresh solution frame has renewed the lease within `t_lease`. G4 tests only *that a fresh, in-integrity frame is still being produced*; it does not re-test G2/G3 content. | Supervisor monotonic watchdog |

**Ephemeris-age placement (fixes B1/C1).** Ephemeris age is an *observation-integrity* input,
so it is tested inside G2 (via the observation gate), not in G3. G3 is calibration/extrinsics
only. The degradation mapping (§2.3) and hazard table use this placement consistently.

**Calibration validity window (fixes C2/D1).** The baseline and handoff define no *time-based
expiry* for the pre-deployment frequency-reference/extrinsics calibration. G3 therefore tests
only presence and identity match, not age. Any calibration-validity *window* would be a new
parameter and a **proposed baseline change**, registered in §5; the word "unexpired" from the
prior draft is dropped as unsupported.

**G1 arm-input architecture gap (fixes B3; per D13).** `ARCHITECTURE.md`'s measurement bus
defines no arm/disarm input or message, so G1 is **not yet realisable** against the approved
architecture. Per decision **D13**, an arm-command bus message plus a helm arm input is a
contracts requirement routed to U-C1 (contracts v3, executive routing). Until that lands, G1
is a specified-but-unwired predicate; this document does not edit `ARCHITECTURE.md` (not
owned). Registered in §5.

### 1.1 Lease renewal, defined non-circularly (fixes findings 6, 7)

The previous draft made G4 both a precondition for and a consequence of renewal. Corrected:

- **Renewal event** `R := (solution_sequence advanced since last renewal) ∧ G2 ∧ G3`.
  `R` is an event predicate over a *newly produced* solution frame; it does **not** depend on
  the current lease being valid.
- On `R`: `lease_deadline := monotonic_now + t_lease`.
- **G4** `:= monotonic_now < lease_deadline` (pure comparison; deadline arrival = expiry).
- **Initial acquisition / recovery:** at a G1 rising edge, the first fresh frame satisfying
  `G2 ∧ G3` fires `R` and makes the lease live. An expired lease can therefore always be
  re-acquired via a fresh in-integrity frame plus re-arm; expiry is never permanent.

**Which frames renew (fixes finding 7).** The baseline permits 5 Hz dead-reckoned fill between
sparse absolute fixes and makes the DR timeout govern *authority*. Accordingly, **every valid
5 Hz solution frame — including DR-propagated fill — renews the lease provided it passes G2**.
Lease liveness (G4) therefore proves only that the companion is *still producing integrity-
passing output*; it is **not** the mechanism that ends dead-reckoning. Three independent timers
do distinct jobs, and must not be conflated:

| Timer | Measures age of | Bounds | Effect on expiry | Provisional value |
|---|---|---|---|---|
| `t_lease` (G4) | last produced solution frame | companion/estimator liveness | lease expiry → revoke | few×200 ms, `< GPS_TIMEOUT_MS` — [UNVERIFIED] |
| `t_dr` (in G2) | last absolute position-constraining observation | dead-reckoning authority | revoke even if covariance still looks in-limit | [UNVERIFIED] |
| per-source freshness | last IMU / magnetometer / speed-log / ephemeris sample | per-sensor staleness feeding integrity | feeds G2/G3 | [UNVERIFIED] |

During all-LEO-loss, DR frames keep renewing the lease (G4 stays satisfied), and authority is
ended by **G2** — either the growing protection limit breaches, or `t_dr` elapses — whichever
comes first. This makes the "revoke time" testable against a replayed minutes-long LEO gap.

### 1.2 Sub-predicate requirements (fixes finding 10)

Structure is now executable and testable even where numbers remain deferred. Missing-data
behaviour is fail-safe (absence ⇒ condition false) throughout.

| ID | Sub-predicate | Metric / unit | Test | Eval rate | Missing-data | Trace / authority |
|---|---|---|---|---|---|---|
| G1 | armed | arm latch (bool) | `armed == true` | on-change + per cycle | ⇒ disarmed | brief; **arm message = proposed contracts v3 (D13)** |
| G2p | horiz position PL | HPL (m) | `HPL ≤ PL_pos[profile]` **[UNVERIFIED]** | per epoch (5 Hz) | ⇒ false | baseline acceptance profiles; per-epoch number **proposed** |
| G2v | horiz velocity PL | VPL (m/s) | `VPL ≤ PL_vel[profile]` **[UNVERIFIED]** | per epoch | ⇒ false | as above |
| G2h | heading PL | (deg) | `HdgPL ≤ PL_hdg[profile]` **[UNVERIFIED]** | per epoch | ⇒ false | baseline 2°/5° (both **[UNVERIFIED]**) |
| G2e | ephemeris age | max SV age (s) | `≤ t_eph` (provisional 6 h) | per observation | over-age ⇒ SV excluded | baseline SupGP 6 h gate **[UNVERIFIED]** |
| G2d | DR-authority age | age of last absolute fix (s) | `≤ t_dr` **[UNVERIFIED]** | per epoch | ⇒ false | baseline "DR timeout governs authority only"; failure mode 3 |
| G3 | calibration match | ID resolves & matches (bool) | all contributing meas. match | per epoch | ⇒ false (integrity fault) | baseline extrinsics rule |
| G4 | lease live | `now − last_R` (s) | `< t_lease` **[UNVERIFIED]** | continuous | ⇒ expiry | this doc (**proposed**); baseline motivates continuous output via `GPS_TIMEOUT_MS` |

Hysteresis/dwell for the caution band (`dwell_clear`) and the re-arm dwell (`dwell_rearm`) are
defined in §2.2/§3.2; both are **[UNVERIFIED]**. Items traced
to "proposed" require a baseline/contracts change and cannot be treated as baseline-authorised
until made so.

### 1.3 Why the authority gate cannot live in ArduPilot

- **ArduPilot's covariance is censored.** It clamps reported horizontal accuracy at **100 m**;
  its EKF variance cannot distinguish a 5 m solution from a 5 km divergence — the exact
  discrimination G2 requires. The unclamped protection limit exists only upstream.
- **ArduPilot cannot see the LEO integrity state** (calibration-ID match, ephemeris age,
  per-pass transmit-bias health, cross-sensor residuals) that G2/G3 test.
- **ArduPilot cannot see companion liveness** (G4): a stalled companion looks, from
  ArduPilot's side, like a GPS gone quiet.
- **`gnss_authority` / profile selection are companion concepts.**

ArduPilot is therefore a downstream actuator that is **never relied on to judge navigation
integrity**. The *design intent* is that its GPS/EKF failsafe actions be configured so that
loss/degradation of `GPS_INPUT` **cannot itself command an autonomous manoeuvre**
(RTL/Loiter/disarm) — because that manoeuvre is the hazard (§2). Whether the pinned firmware
actually behaves this way on `GPS_INPUT` silence is **[UNVERIFIED]** and is precisely what the
U-M1 SITL characterisation (per D17) must establish, pinned to a firmware commit + artifact
checksum; until it does, this is a stated objective, not an evidenced property (§2.1 Case B).

---

## 2. What revokes steering authority

### 2.1 Revocation semantics, and the actual signal path to ArduPilot (fixes findings 2, 3, 11, 14)

Revocation is never a manoeuvre:

> **Revocation = stop steering + alarm + hand to helm.**

Software shall **never** autonomously select RTL, Loiter or disarm (failure mode 9; baseline
final degradation row). But "stop steering + hand to helm" is only a real safety action if
the mechanism that delivers it to ArduPilot is defined. It is **not** delivered by `GPS_INPUT`
— that message is navigation data, not a control-authority interface. Two physically distinct
cases exist, and the previous draft wrongly treated them as one:

**Case A — companion alive, link up** (integrity/limit breach, lease expiry while the process
still runs, or a helm-commanded disarm). The companion performs an *active* revocation:

1. It commands ArduPilot into a helm/manual control mode over MAVLink (candidate
   `MAV_CMD_DO_SET_MODE` / `SET_MODE` to `MANUAL` or an equivalent helm-controlled mode).
2. It then ceases / invalidates the `GPS_INPUT` nav injection, consistent with that mode.
3. It raises the alarm (§3.2).

Every element of this recipe remains a **candidate**, each marked **[UNVERIFIED]** and each a
SITL/HIL deliverable at the pinned firmware: (i) the exact mode-set **command** and target
**mode enum** `[UNVERIFIED]`; (ii) the **RC-arbitration parameters** that let physical helm input
dominate `[UNVERIFIED]`; (iii) the **command acknowledgement / failure behaviour** — what the
companion observes on `COMMAND_ACK`, and its fallback if the mode-set is rejected, times out or
is lost `[UNVERIFIED]`; (iv) the **actuator transition** across the mode change — that servo/throttle
outputs move continuously with no step `[UNVERIFIED]`. The ordering to be **verified in SITL/HIL**:
the mode transition to manual precedes or is simultaneous with cessation of nav injection, so
that no GPS-timeout failsafe is provoked and **no commanded turn, throttle change, loiter, RTL,
disarm, or actuator discontinuity occurs**; physical helm input dominates immediately.

**Sea-trial gate (per D17).** Until the full Case-A hand-to-helm recipe above — command, mode,
RC arbitration, command-ack/failure handling and actuator transition — is **demonstrated in
ArduPilot SITL at the pinned firmware**, the "hand to helm" action is **unproven**: **no
on-water steering authority shall be granted until the Case-A handoff recipe is demonstrated in
SITL at the pinned firmware.** Registered in §5.

**Case B — companion process dead, or MAVLink link lost** (whole-process stall, thread death,
cable/connector fault). The companion can send nothing; **its internal watchdog/lease cannot
reach ArduPilot.** ArduPilot observes only the *absence* of `GPS_INPUT`. **What ArduPilot Rover
actually does on `GPS_INPUT` silence was [UNVERIFIED] at first writing — now SITL-characterised; see the 2026-07-23 amendment below.** `GPS_TIMEOUT_MS` (4000 ms) is a GPS-backend *data*
timeout, not a demonstrated dedicated control-authority timeout; there is no present evidence
that Rover treats `GPS_INPUT` silence as a bounded non-manoeuvre action, and the resulting
mode/actuation/timing depend on flight mode, EKF source/validity and configuration rather than
on this backend timeout alone. The earlier claim that Case B is bounded by a "~4 s configured
non-manoeuvre timeout" is therefore **withdrawn**. Until U-M1 characterises the behaviour in
SITL, **Case B's residual is classified `uncontrolled-pending-evidence`** — the same honest
treatment as H6/H7.

*Amendment (2026-07-23, per D24): U-M1's SITL characterisation at pinned Rover-4.6.3 now
exists — see `tools/sitl/evidence/D17a.md`. Measured: fix-type degradation at ~1.0 s of
`GPS_INPUT` silence; EKF failsafe (`FS_EKF_ACTION=1`) commanded armed HOLD (throttle zero,
neutral steering, no manoeuvre) at ~5.07 s; a companion-commanded HOLD executes in ~51 ms.
This upgrades Case B from `uncontrolled-pending-evidence` to `SITL-characterised`: at this
firmware and configuration a bounded non-manoeuvre response exists, via the EKF failsafe
rather than `GPS_TIMEOUT_MS` as such. Scope limits: SITL-only (sea-trial confirmation
pending), measured in GUIDED mode with default failsafe configuration; other modes and
configurations remain [UNVERIFIED]. The pre-trial checklist retains confirmation of this
behaviour on the actual vessel installation. Pending that on-vessel confirmation, the
conservatively credited responder remains the **physical helm / kill-cord** — the same honest
status as H6(a) and H7.* A one-way companion→ArduPilot TX failure is **not observable to the companion**
without a return heartbeat; the companion judges link health via the ArduPilot→companion MAVLink
heartbeat with its own deadline, but even on detection it cannot correct ArduPilot over the
failed channel — detection serves alarm and logging, not actuator control.

**Consequently the corrected guarantee is:** *for faults in which the companion keeps
executing, the live supervisor revokes within `t_lease` (Case A).* For companion death or link
loss (Case B) there is **no in-process responder**, and the autopilot-side response to
`GPS_INPUT` silence is `[UNVERIFIED]` (per D17 / U-M1); that residual is
`uncontrolled-pending-evidence`, backed only by the physical layer, until the SITL
characterisation proves whether and when any manoeuvre occurs. The claim "the companion always
revokes before ArduPilot's failsafe" is withdrawn; `t_lease < GPS_TIMEOUT_MS` matters only in
Case A, where it bounds how long a degraded-but-alive companion may keep asserting before
self-revoking (the `GPS_TIMEOUT_MS` value here is used only to size `t_lease` against the data
timeout, not as evidence of ArduPilot's control action).

### 2.2 Authority state machine — total state/event matrix (fixes findings 4, 5; NEW state-machine finding)

Intended disarm and startup-unarmed are **not** faults and must not alarm. A distinct
**DISARMED** state is introduced; the S2/S3 alarm ladder is reserved for loss of a
*previously granted* lease due to a hazardous condition.

**States (6):** **DISARMED (D)** · **NOMINAL / S0 (N)** · **CAUTION / S1 (C)** ·
**WARNING / S2 (W)** · **ESCALATED / S3 (E)** · **LATCHED-SAFE / S4 (L)**. Process start with
G1 false enters **D**.

**Authority + annunciation are a function of the destination state** (so the matrix below need
only give the next state):

| State | Authority | Annunciation |
|---|---|---|
| D | none | quiet status ("disarmed") |
| N | **granted** | quiet ("armed") |
| C | **granted** | soft pre-alert |
| W | **revoked, latched** | loud alarm, demand ack |
| E | **revoked, latched** | max continuous alarm |
| L | **revoked, latched** | steady fault, awaits re-arm |

**Events (columns).** `G1↓` helm disarm / arm withdrawn · `G1↑` helm arm-or-re-arm rising edge ·
`G2↓`/`G2↑` protection-limit/integrity breach and recovery · `G3↓`/`G3↑` calibration fault and
recovery · `G4↯` lease-deadline expiry (no renewing frame within `t_lease`) · `Tack↯` `T_ack`
elapses with no ACK · `ACK` helm acknowledges · `ce` metric ≥ `caution_enter` · `cc` metric ≤
`caution_clear` sustained `dwell_clear`. (`ce`/`cc` are the caution-band crossings *within* the
G2-true region; `G2↓` here means the revoke-threshold crossing / integrity-false, `G2↑` its
recovery.)

Cell legend: a letter = transition to that state; **`·`** = self-loop (no change to state,
authority or annunciation); **`N?`** = **guarded grant** — becomes N iff the guard holds this
tick, else `·` with a "cannot (re)arm" status. Arm guard (from D): `R` fires, i.e. a fresh
frame with `G2∧G3`. Re-arm guard (from L): `R` **and** `dwell_rearm` elapsed since latching.

| State ↓ / Event → | G1↓ | G1↑ | G2↓ | G2↑ | G3↓ | G3↑ | G4↯ | Tack↯ | ACK | ce | cc |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **D** | · | **N?** | · | · | · | · | · | · | · | · | · |
| **N** | **D** | · | **W** | · | **W** | · | **W** | · | · | **C** | · |
| **C** | **D** | · | **W** | · | **W** | · | **W** | · | · | · | **N** |
| **W** | · | · | · | · | · | · | · | **E** | **L** | · | · |
| **E** | · | · | · | · | · | · | · | · | **L** | · | · |
| **L** | · | **N?** | · | · | · | · | · | · | · | · | · |

Reading of the fail-safe self-loops: in **D**, `G2/G3/G4` edges have no lease to revoke (they
only update the predicate consulted at the next arm attempt), and `G4↯` cannot fire because no
lease runs while disarmed. In **W/E**, `G1↑` is *ignored* (re-arm is honoured only from L/D
after `dwell_rearm`); `G1↓` only records the arm latch false for the eventual re-arm gate;
**recovery edges `G2↑`/`G3↑` do not auto-clear a latched fault** — exit is via `ACK` only. In
**L**, recovery alone (`G2↑`/`G3↑`) never re-grants; a re-arm edge (`G1↑`) does, under guard.

**Simultaneous-event precedence (one tick, highest wins), with a default rule.** When more than
one event is present in the same evaluation tick, apply the single highest-priority event whose
cell is non-`·` in the current state; ties within a tier resolve to the same destination so
order among them only sets the logged cause:

1. **Fault edges** `G3↓`, `G2↓`, `G4↯` (all → **W** from N/C). A fault present this tick beats
   an intended disarm — so a concurrent fault still **alarms** rather than being masked as a
   quiet disarm.
2. **`G1↓`** intended disarm (→ **D** from N/C).
3. **`Tack↯`** escalation (→ **E** in W).
4. **`ACK`** (→ **L** in W/E).
5. **`G1↑`** arm/re-arm (guarded; honoured only from D/L).
6. **Recovery / band edges** `G2↑`, `G3↑`, `ce`, `cc` (lowest).

**Default rule:** if no listed event with a non-`·` cell is present, the state self-loops
(authority and annunciation unchanged). Combined with the destination-function table above and
the fail-safe invariant — *authority is granted only in N/C and only while `G1∧G2∧G3∧G4` all
hold; the falling (false) of any grant term maps to a revoking transition or to a non-granting
self-loop, never to a grant* — **every (state, event) cell resolves to exactly one successor**,
with one authority result and one annunciation. This discharges the "every Boolean transition
has exactly one outcome" obligation (finding 5) for all six states, including during
WARNING/ESCALATED/LATCHED-SAFE and DISARMED.

Note the distinction the previous draft missed: **G1 falling by helm command is an intended,
quiet disarm** (→ D), while **G2/G3 false or G4 expiry are fault revocations that alarm** (→ W).
"Any false term drops authority" still holds; only the *annunciation* differs by cause.
Whole-process death (Case B) cannot self-annunciate — it presents to the helm via the absence of
the healthy indication plus any independent annunciator, and to ArduPilot via the *absence* of
`GPS_INPUT`, whose effect is `[UNVERIFIED]` (per D17 / U-M1; §2.1), not via a proven timeout
action.

### 2.3 Degradation-row mapping (baseline degradation contract)

"Continue" = estimator keeps running (baseline: authority expiry never stops the estimator);
authority persists only while G1–G4 hold.

| Baseline degradation row | Supervisor response | Trigger |
|---|---|---|
| GNSS absent or forbidden | Normal denied operation; **denied** profile active; GNSS cannot re-enter fusion (bus ingress). | none (mode) |
| One LEO constellation / RF front end lost | Continue if protection limits met; degrade honestly. | G2 if breached |
| **All LEO observations lost** | DR frames keep renewing G4; authority ends on G2 — protection-limit breach **or** `t_dr` elapsed, whichever first. | G2 (`t_dr` or PL) |
| One magnetometer lost/rejected | Continue on the other calibrated magnetometer, IMU turn dynamics and any selected non-magnetic heading sensor; inflate heading uncertainty. | G2 (heading) if breached |
| **Both magnetometers lost/rejected** | Continue short-term attitude propagation and any selected non-magnetic heading measurement; not passage-capable by inertial integration alone; revoke when heading PL exceeded. | G2 (heading) |
| Speed log lost | Continue; current separation unavailable. | G2 if breached |
| **IMU stream stale** | Propagation untrustworthy → revoke; estimator stays running, fabricates no propagation samples, and journalling and recovery may continue. | G2 (per-source freshness) |
| Frequency reference degrades / sustained heel | Reject biased Doppler per integrity; revoke when limits exceeded. | G2 |
| Sea-surface multipath biases a tracker | Reject/inflate on correlation quality + residuals; persistent correlated residuals revoke. | G2 |
| Ephemeris missing or too old | Do not form/accept affected Doppler (never bypass age gate); revoke only if the surviving solution breaches limits. | **G2 (ephemeris-age gate)** |
| **Companion faults, by class** *(aligns with the amended baseline row, per D17(b))* | **Estimator-only stall / internal fault, supervisor alive:** the live supervisor's monotonic watchdog expires the lease and stops steering (Case A). **Whole-process death / companion–autopilot link loss / board or power loss:** no in-process responder exists; the autopilot-side response to `GPS_INPUT` silence is **[UNVERIFIED]** (SITL, per D17 / U-M1), so this residual is `uncontrolled-pending-evidence`, backed only by the physical helm/kill-cord (Case B, §2.1). In every class, software never selects RTL/Loiter/disarm. | Case A (watchdog) / Case B (`[UNVERIFIED]`) |
| Calibration ID missing/mismatched *(additional row — not one of the 11 literal baseline rows; sourced from the extrinsics rule, `DESIGN_BASELINE.md` §Estimator and degradation contract, "missing or mismatched calibration is an integrity fault that forbids steering authority")* | Authority-blocking integrity fault. | G3 |

**Orbcomm fusion caveat (fixes B4; per D10).** Orbcomm arrives on an independent, non-coherent
receiver introducing a **second, unmodelled receiver clock**. Per decision **D10**, Orbcomm
observations shall **not enter fusion** until a second receiver-clock state or per-receiver
nuisance term exists; until then they cannot corrupt the steering solution or its protection
limits. This is a safety-relevant integrity constraint on which observations G2 may rest.

The DR timeout `t_dr` **governs authority only** (baseline; failure mode 3) — it never stops
propagation, observation processing, journalling, or the 10–20 min convergence.

---

## 3. The backstop when the human does not respond

### 3.0 Fault domains and independence (fixes finding 12; coordinator point ii)

Independence claims are only valid across fault-domain boundaries. The domains are:

- **Companion process / board (one domain).** Clock service, estimator, integrity monitor
  (module 10), authority supervisor + watchdog (module 11) and MAVLink publisher (module 12)
  all execute here. A **process-level fault** (memory corruption, scheduler stall, a fault
  that simultaneously produces a plausible-wrong solution *and* a passing integrity verdict)
  takes them down **together**. Within this domain, separate modules with design-diverse code
  contain **design faults** of one another (e.g. a finite-difference-checked integrity monitor
  catches a wrong-Jacobian estimator bug), but they contain **no** process-level fault.
- **ArduPilot (separate compute).** Independent of companion process faults, but blind to
  navigation-integrity correctness (censored 100 m view) — it can react to *absence* of input,
  not to plausible-wrong input.
- **Physical helm override + kill-cord (no compute).** Independent of all software; but they
  act on gross vessel behaviour and human presence, not on nav correctness (§3.3).
- **The human.** The only element that can judge that steering "looks wrong". Single point.

The consequence, stated plainly: **no in-companion module is an independent mitigation of a
companion process-level fault.** Where the earlier draft cited "the same integrity monitor +
watchdog" as mitigation for a companion producing plausible-wrong output (H6), that is
withdrawn. That residual is **uncontrolled by present design** and requires an **out-of-
companion-process independent monitor** which does not yet exist; registered in §5.

### 3.1 Layer 1 — supervisor monotonic watchdog (bounded scope)

The lease (§1.1) on the clock service's monotonic domain is renewed only by a fresh,
integrity-passing frame. Its **genuine, defensible job**: while the companion process is still
executing, stop it from continuing to assert steering on a **frozen or stale solution** (H4) —
a frozen estimator does not advance the solution sequence, so `R` does not fire, the lease
expires, and the *live* supervisor executes a Case-A revocation.

Its **limits**, now stated (correcting the previous over-claim): the watchdog **cannot** act
once its own process has died or the link is lost (Case B) — it is inside the failed domain.
"Authority cannot outlive the solution" therefore holds *for the companion-alive fault class
only*; for companion death/link loss there is **no in-process responder** and ArduPilot's
response to `GPS_INPUT` silence is **[UNVERIFIED]** (SITL, per D17 / U-M1), so that residual is
`uncontrolled-pending-evidence` and the physical layer is the only present backstop — not the
internal watchdog and not an assumed timeout action. Independence properties that do hold: the
watchdog runs on the monotonic clock (a device/UTC reset or spoofed label cannot extend it),
and its expiry cannot stop the estimator (baseline).

`t_lease` shall be `< GPS_TIMEOUT_MS` (4000 ms) so that in Case A the companion self-revokes
before its output would age out at ArduPilot; here `GPS_TIMEOUT_MS` is used only as the
GPS-backend data-timeout value to size `t_lease`, not as evidence of any ArduPilot control
action on silence (that action is `[UNVERIFIED]`, §2.1). Provisional few×200 ms — **[UNVERIFIED]**.

### 3.2 Layer 2 — un-acknowledged-alarm escalation ladder, with real hysteresis (fixes finding 13)

The ladder exists to *summon the helm*, not to replace the helm; because software never
manoeuvres, **the terminal state is not kinetic**. States and transitions are in §2.2. The
caution band uses **distinct entry and clear thresholds with a dwell** (single-threshold logic
from the prior draft was not hysteresis):

- ordering, for an uncertainty metric (larger = worse): `caution_clear < caution_enter < revoke_threshold`;
- **CAUTION entry:** metric `≥ caution_enter` (this concrete crossing replaces the prior vague
  "margin shrinking" — fixes C3);
- **WARNING:** metric `≥ revoke_threshold`;
- **CAUTION→NOMINAL clear:** metric `≤ caution_clear` sustained for `dwell_clear`;
- all of `caution_enter`, `caution_clear`, `dwell_clear`, `revoke_threshold`, `T_ack`,
  `dwell_rearm` are **[UNVERIFIED]**, frozen before trials against the manoeuvre budget.

Latched revocation requiring explicit re-arm (§2.2) prevents steering-authority flapping;
the entry/clear band with dwell prevents S0/S1 *alarm* flapping. **Terminal state = ESCALATED
+ max alarm**, indefinitely, with no kinetic action; if the helm never responds the vessel
holds its last physical control state (least-surprising) until the helm or the physical layer
(§3.3) intervenes.

### 3.3 Layer 3 — physical, controller-independent override (final layer)

Independent of companion, ArduPilot and MAVLink:

- **Helm manual override** — steering/throttle mechanically dominates the autopilot at any
  time (baseline precondition). This is how "hand to helm" becomes real motion control, and is
  the only control effective in Case B when software cannot signal.
- **Helm kill-cord** — a physical lanyard that cuts propulsion (failure mode 9). It acts only
  when *pulled* — i.e. when the helmsman is **displaced** (overboard / falls from station).

**Human-response failure, by sub-case (fixes finding 9 / H7).** The mandated "human does not
respond" case is not uniform; coverage differs:

| Sub-case | Covered by | Status |
|---|---|---|
| Distracted but present | escalation ladder summons attention | controlled if helm eventually responds |
| Overboard | kill-cord (lanyard pulls) | controlled |
| Displaced from station | kill-cord | controlled |
| **Unconscious at the helm, cord still clipped** | nothing in present design | **UNCONTROLLED** |
| Kill-cord not clipped | pre-trial checklist (procedural) | controlled only if checklist enforced |
| Manual override mis-wired/failed | pre-trial test (procedural) | controlled only if tested |

The unconscious-at-helm residual — propulsion continues at last physical state until fuel
exhaustion or third-party intervention — is **not controlled by software and not accepted
here**: this document has no risk-acceptance authority. It requires an **operational control**
(a second qualified crew member, or a helm dead-man / vigilance device that cuts propulsion on
loss of periodic helm input) whose specification and formal risk acceptance belong to the
trial authority. Registered in §5; **the trial risk envelope should not permit unattended
continued propulsion until this control is in place.**

Pre-trial checklist (operational): kill-cord clipped; manual override tested; ArduPilot
response to `GPS_INPUT` silence characterised and proven non-manoeuvring in SITL (Case B, per
D17 / U-M1); Case-A hand-to-helm recipe demonstrated in SITL/HIL on pinned firmware; all
authority-contract parameters frozen and verified (fail-closed gate, §1).

---

## 4. Hazard table

hazard → cause → mitigation → residual. Residuals marked **[UNVERIFIED]** are bounded only
after GNSS-withheld replay and/or SITL/HIL evidence.

| # | Hazard | Cause | Mitigation | Residual risk |
|---|---|---|---|---|
| **H1** | **Stale-solution steering** | Sparse absolute fixes; DR fill; covariance growth. | Covariance time-propagated from IMU every accepted sample (guards failure mode 5); G2 per-epoch PL; **`t_dr` DR-authority timeout** revokes on last-absolute-fix age even if covariance looks in-limit; G4 requires a live producer. | If the covariance model is itself optimistic (H2), a fresh-looking frame under-reports staleness; `t_dr` bounds this independently of covariance. Magnitude **[UNVERIFIED]** until withheld-replay validation. |
| **H2** | **Optimistic covariance (wrong-Jacobian class)** — the gate consumes the very covariance that is wrong. | Wrong Jacobian (silent; filter still emits a covariance), missing propagation, mis-modelled process noise. | **Offline, pre-trial:** finite-difference Jacobian check; **NEES against truth in aided/withheld replay** (needs truth ⇒ offline only). **Online:** normalized-innovation-squared (**NIS**) consistency — flags innovations too large for the claimed innovation covariance, but is *covariance-coupled*, not covariance-independent; **cross-sensor redundancy residuals** (speed-log vs LEO ground speed, dual-magnetometer agreement, IMU-predicted vs measured) which *are* covariance-independent but limited in coverage; conservative process noise. | A slow bias that shrinks state **and** innovation covariance consistently evades NIS and lies below the redundancy floor; **not closed online**. Bounded only by conservative limits, offline gates and the human. **[UNVERIFIED]**. (Prior "NEES online, covariance-independent" claim withdrawn as technically wrong.) |
| **H3** | **Authority flapping** | Metric noise near threshold. | Distinct `caution_enter`/`caution_clear` + `dwell_clear` (real hysteresis); latched revocation + explicit re-arm with `dwell_rearm`. | Mis-tuned band flaps or lags; values **[UNVERIFIED]**, frozen before trials. |
| **H4** | **Silent estimator halt vs authority-alive inversion** | Estimator stalls/freezes while publisher stays alive. | `R` fires only on an *advancing* solution sequence that passes integrity; a frozen output does not advance the sequence ⇒ lease expires ⇒ **live** supervisor revokes (Case A). Watchdog on independent monotonic clock; its expiry cannot stop the estimator (baseline). | A bug re-emitting *stale-but-sequence-advancing* frames defeats sequence-freshness; needs a **content/innovation-liveness** check (registered §5). Whole-process death is Case B, not H4. **[UNVERIFIED]**. |
| **H5** | **MAVLink link loss mid-authority** | Cable/USB/serial/connector fault. | This is **Case B**: internal revocation cannot reach ArduPilot. ArduPilot's response to `GPS_INPUT` silence is **SITL-characterised (D24 amendment, §2.1)**: EKF-failsafe HOLD (no manoeuvre) at ~5.07 s at pinned Rover-4.6.3, per `tools/sitl/evidence/D17a.md`; SITL-only, on-vessel confirmation pending **[UNVERIFIED]**. Present backstop is helm override/kill-cord (link-independent). Companion detects link loss via ArduPilot→companion heartbeat deadline → alarm/log only. | **SITL-characterised, on-vessel-unconfirmed**: SITL shows a non-manoeuvre HOLD at ~5.07 s (D17a); until confirmed on the vessel installation, only the physical layer is credited. The prior "bounded ~4 s non-manoeuvre timeout" and "`t_lease` guarantees the companion revokes first" claims are withdrawn. **[UNVERIFIED]/uncontrolled — registered §5.** |
| **H6** | **`GPS_INPUT` spoof / companion producing plausible-wrong output** | (a) companion process-level fault; (b) external MAVLink injection. | (b) dedicated point-to-point companion↔autopilot link (not networked); companion sole authorised source (`GPS1_TYPE=14`); enable MAVLink signing (**[UNVERIFIED]** support). LEO-SoOP removes the *RF* GNSS-spoof surface; the MAVLink surface remains. (a) **No in-companion module is independent of a process-level fault (§3.0)** — the integrity monitor/watchdog can be falsified by the same fault; this is **not** cited as mitigation. | (a) **Uncontrolled by present design**: requires an out-of-companion-process independent monitor (does not exist). Physical kill-cord cannot detect plausible-wrong steering. **[UNVERIFIED]/uncontrolled — registered §5.** |
| **H7** | **Human-override failure** | Distracted / unconscious-at-helm / overboard / cord unclipped / override failed. | Per-sub-case coverage table (§3.3): ladder + kill-cord cover distracted/overboard/displaced; checklist covers clip/override. Software never auto-manoeuvres, so an unresponsive human triggers no surprise manoeuvre. | **Unconscious-at-helm with cord clipped is UNCONTROLLED** by software; needs an operational control (second crew / helm dead-man) with formal risk acceptance by the trial authority. Not accepted here. **[UNVERIFIED]/uncontrolled — registered §5.** |
| **H8** | **Frequency-reference/heel bias or total heading loss steering** | Reference degrades or sustained heel steps the drift model; both magnetometers lost. | Reject biased Doppler per integrity; G2 revokes on limit breach; heading PL revokes on dual-magnetometer loss; mount g-sensitivity vector vertical (baseline). | Slow in-model bias below the innovation-detection floor. **[UNVERIFIED]**; bounded by conservative limits and the human. |

---

## 5. Assumptions and the [UNVERIFIED] / uncontrolled register

Safety-relevant items not yet frozen, and honest control status. **Every authority-contract
parameter marked [UNVERIFIED] below is fail-closed per §1: while it remains unverified, steering
authority cannot be granted.**

- **Per-epoch protection-limit numbers** (position/velocity/heading, aided and denied) — the
  baseline gives only campaign-grading acceptance statistics; per-epoch gates are a proposed
  addition.
- **Timers:** `t_lease` (`< 4000 ms`), `t_dr` (DR-authority), per-source freshness deadlines,
  `t_eph` (provisional 6 h), `T_ack`, `caution_enter`/`caution_clear`/`dwell_clear`,
  `dwell_rearm`.
- **G1 arm input:** no arm/disarm message exists in `ARCHITECTURE.md`; an arm-command bus
  message + helm arm input is a contracts requirement per **D13** (U-C1, contracts v3). G1 is
  specified-but-unwired until then.
- **Calibration validity window:** baseline/handoff define no time-based calibration expiry;
  any such window is a **proposed baseline change**, not assumed here (G3 tests presence +
  identity match only).
- **MAVLink revocation command set / hand-to-helm recipe (Case A):** exact mode-set command,
  target helm/manual mode, RC-arbitration parameters, command-acknowledgement/failure handling,
  the actuator transition (no servo/throttle step), and the verified ordering (mode-to-manual
  before nav-injection cessation, no actuator discontinuity) — all **[UNVERIFIED]**, a SITL/HIL
  deliverable on pinned firmware. **Sea-trial gate (D17): no on-water steering authority until
  the Case-A handoff recipe is demonstrated in SITL at the pinned firmware.** Until then, "hand
  to helm" is unproven.
- **ArduPilot response to `GPS_INPUT` silence (Case B) — [UNVERIFIED], `uncontrolled-pending-evidence`:**
  the mode/actuation/timing Rover exhibits when `GPS_INPUT` stops is not evidenced
  (`GPS_TIMEOUT_MS` is a GPS-backend *data* timeout, not a demonstrated control-authority action;
  behaviour depends on mode, EKF source/validity and configuration). **U-M1** has characterised (SITL, 2026-07-23, `tools/sitl/evidence/D17a.md`; on-vessel confirmation outstanding) — originally: shall characterise
  it in SITL at the pinned firmware — in every intended autonomous mode, stopping only
  `GPS_INPUT` and recording GPS backend status, EKF source/validity, mode, navigation demand,
  servo outputs and RC override from last input through ≥ 10 s — and must prove `GPS_INPUT`
  loss/degradation cannot command RTL/Loiter/disarm (against pinned commit + artifact checksum).
  Until confirmed on-vessel, Case B remains conservatively backed only by
  the physical layer; the earlier "bounded ~4 s non-manoeuvre timeout" framing is withdrawn.
- **MAVLink signing** availability on the pinned firmware.
- **Independent (out-of-companion-process) monitor** for H6(a): does not exist. The
  correlated companion process-level fault — plausible-wrong `GPS_INPUT` with a falsified
  integrity verdict — is **uncontrolled** pending such a monitor; no in-companion module is
  independent of it.
- **Content/innovation-liveness check** to fully close H4 against stale-but-advancing frames.
- **Operational human-response control** for the unconscious-at-helm residual (H7): a second
  qualified crew member or a helm dead-man / vigilance device, with formal risk acceptance by
  the trial authority. The trial risk envelope should not permit unattended continued
  propulsion until this is in place.
- **Orbcomm fusion (D10):** Orbcomm observations must not enter fusion until a second
  receiver-clock/per-receiver nuisance state exists.
- **Timer budgets assume a displacement hull;** a planing/slamming hull invalidates them.

Assumptions carried from the baseline: manned displacement-hull vessel, Danish straits
(working assumption), qualified helm aboard with physical controller-independent override,
`gnss_authority` enforced at bus ingress, and the MAVLink `GPS_INPUT` (`GPS1_TYPE=14`)
interface. Implementation field encodings, ArduPilot parameter/mode names and bus message
schemas are subordinate v2/v3 contracts and code/config work; this document does not edit
`ARCHITECTURE.md`.
