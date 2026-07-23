# Authority-Contract Numeric-Freeze Proposal

Status: **subordinate to [`DESIGN_BASELINE.md`](DESIGN_BASELINE.md) and
[`SAFETY_CASE.md`](SAFETY_CASE.md)**
Unit: U-N1 · Contract read: CONTRACTS v5.1 (`AuthorityParams`) · 2026-07-23

> **This document proposes values. It does not freeze them and it grants no authority.**
> Every numeric here is a **proposal, [UNVERIFIED until validated]**. Per `SAFETY_CASE.md`
> §1 (the D17 fail-closed gate) an authority-contract parameter that is not *frozen and
> verified* is a **closed gate**, not a permissive default. Freezing any value in this
> document requires (a) the evidence named in that parameter's *validation plan* and (b) a
> signed decision in `DECISIONS.md`. Until both exist for a given parameter, the supervisor's
> corresponding `Option<f64>` should remain `None` and steering authority cannot be granted.
> Where the honest state of a value is "engineering estimate with weak evidentiary support",
> this document says so rather than dressing it as derived.

This proposal is the response to `SAFETY_CASE.md` §5's open numeric register and to review
finding **F9** (U-T1), which routed the tracker detection-threshold false-alarm analysis here
(`DECISIONS.md` D36). It turns each `[UNVERIFIED]` authority-contract parameter into a
proposed value with an explicit derivation chain, a two-sided sensitivity note, and a
validation plan. Coverage is **the whole safety-case register, not only the `AuthorityParams`
fields**: it includes the six protection limits, `t_lease`/`t_dr`/`t_eph`, the caution
band/revoke/dwells/`T_ack`, **and the per-source freshness deadlines** (§2.4) — the last of
which are named in the fail-closed register but are **absent from `AuthorityParams` and its
`is_complete()` check**, an enforcement gap this proposal fills with values and registers for
the contracts owner (§5.4) rather than absorbing silently. Nothing below revises the baseline
or the safety case; where a value would *require* a baseline/contracts change to be meaningful,
that is registered in §5, not assumed.

---

## 0. Method, and what the supervisor actually tests

### 0.1 The quantities the shipped supervisor compares (`crates/pnt-integrity`)

The proposal is bound to what the built supervisor (unit U-A1, CONTRACTS v5.1) actually
evaluates, not to an idealised metric:

- The horizontal-position protection limit is tested against `metric =
  solution.state.horizontal_accuracy_m()` — the epoch **2-D DRMS one-sigma** horizontal
  accuracy `sqrt(P_ENU[E,E] + P_ENU[N,N])` (CONTRACTS v3). **Units: metres.**
- The horizontal-velocity limit is tested against `speed_accuracy_mps()` — the same 2-D DRMS
  convention on ECEF velocity covariance. **Units: metres/second.**
- The heading limit is tested against the heading covariance one-sigma. **Units: radians.**
- **`revoke_threshold`, `caution_enter`, `caution_clear` are compared against the *same*
  `horizontal_accuracy_m()` metric** (`lib.rs` §metric). They are therefore **horizontal-
  accuracy values in metres**, ordered `caution_clear < caution_enter < revoke_threshold`
  (`SAFETY_CASE.md` §3.2), and they are **single scalars, not per-profile** — a limitation
  with a real consequence, registered in §5.1.
- Timers (`t_lease`, `t_dr`, `t_eph`, `dwell_clear`, `dwell_rearm`, `T_ack`) are all in
  **seconds** (CONTRACTS v5).

### 0.2 Coverage factors (acceptance profile → per-epoch protection limit)

`SAFETY_CASE.md` §0 requires per-epoch protection limits **no looser than the active
acceptance profile implies**, and the baseline's acceptance numbers are *error* limits
(actual error vs truth, at a percentile whose exact confidence definition is itself
`[UNVERIFIED]`, baseline §Acceptance profiles). The protection limit, by contrast, bounds the
*reported one-sigma DRMS*. Converting one to the other needs a coverage factor. This proposal
uses a **two-sided margin factor `k = 2`** on horizontal position: a protection limit
expressed as one-sigma DRMS, tested at `≤ PL`, keeps the **2-DRMS** bound (≈ 95–98 % of
horizontal error for a roughly circular 2-D Gaussian) inside the acceptance number. For a
circular 2-D Gaussian the 95 % radius is `1.73 × DRMS`, so `k = 2` is deliberately
conservative (it targets ≈ 98 % containment), which supplies the "no looser than implied"
margin automatically. Velocity and heading use the mapping stated in their sections. **The
factor `k = 2` is itself a proposal**: it stands in for the unfrozen acceptance-percentile
definition and must be reconciled once that definition is frozen (baseline obligation).

---

## 1. Protection limits (G2)

The six protection limits are the per-epoch heart of G2. All are one-sigma covariance-derived
ceilings; each profile is selected by `gnss_authority` (aided under `production`, denied under
`recorded_only`/`off`, `SAFETY_CASE.md` §0).

### 1.1 Horizontal position — `aided = 12 m`, `denied = 250 m`

**Derivation.** The D56-amended baseline retains aided ≤ 25 m and sets denied horizontal error
at **≤ 500 m p50 AND ≤ 750 m p95 over a ≥100 km constant-heading-dominated passage**.
With the existing proposed `k = 2` on the one-sigma DRMS metric:
`PL_aided = 25 / 2 = 12.5 → 12 m`; the p50-referenced denied proposal is
`PL_denied = 500 / 2 = 250 m`. The p95 acceptance separately maps to `750 / 2 = 375 m`,
the worst-case-derived ceiling informing future tuning rather than the proposed normal
per-epoch gate. The tighter 250 m typical-performance reference is proposed here. Exact trial
confidence/segment definitions and every PL mapping, including `k = 2`, remain **PROPOSED,
NOT FROZEN, [UNVERIFIED]**. The aided limit is unchanged because relaxing it could hide
failure-mode-2 (whether GNSS actually helps).

**Sensitivity.** 2× too tight (aided 6 m): authority is revoked while the solution is still
inside operational grade — availability collapses and honest degradation reads as failure. 2×
too loose (aided 24 m): 2-DRMS = 48 m, roughly double the 25 m acceptance — per-epoch
integrity would no longer *imply* the campaign statistic, violating the `SAFETY_CASE.md` §0
"no looser than implied" rule. For denied operation, 125 m is 2× tighter than the proposed
250 m and reduces availability; 500 m is 2× looser and exceeds the D56-derived 375 m
worst-case ceiling.

**Validation plan.** Reproduce the controlled U-MS1.1 multi-satellite N=8 replay
(N=8 mean 116 m (p50 <= mean under right skew) / p95 554 m), then extend it with independent-truth withheld replay and
NEES/coverage scoring: confirm epochs passing `HPL ≤ PL` contain horizontal error within the
amended 500 m p50 / 750 m p95 passage acceptance at the frozen confidence. Stratify by
geometry, sea state, manoeuvre/converged leg, and time since last absolute fix. The replay
makes the amended denied acceptance evidence-MET where the old target was not reliably
deliverable; it does not verify `k`, the 250 m per-epoch PL, or real-signal performance.
Freeze only after real-signal coverage validation. Check aided separately in `production`.

### 1.2 Horizontal velocity — `aided = 0.014 m/s`, `denied = 0.028 m/s`

**Derivation.** Acceptance profile is **per-axis** velocity error (aided ≤ 0.02, denied ≤ 0.04
m/s). The metric is 2-D DRMS velocity `= √2 · σ_axis` for equal axes. Requiring per-axis 95 %
error `1.96 σ_axis ≤ limit` gives `PL_vel = (√2 / 1.96) · limit = 0.721 · limit`: aided
`0.721 × 0.02 = 0.0144 → 0.014 m/s`; denied `0.721 × 0.04 = 0.0288 → 0.028 m/s`.

**Anisotropy limitation (explicit).** This mapping assumes **equal per-axis variances and
Gaussian per-axis velocity errors**. Neither is guaranteed: the supervisor tests the *scalar
2-D DRMS* (trace of the rotated horizontal velocity covariance), which is blind to axis ratio —
a covariance with one large and one small axis can pass a DRMS gate while its large axis exceeds
the per-axis limit. Under LEO SoOP the velocity information is geometry-dependent (range rate
along instantaneous line-of-sight), so anisotropic and cross-correlated horizontal velocity
covariance is expected during a pass, not an edge case. Consequently `0.721 · limit` is a
faithful per-axis guarantee only under near-isotropic covariance; the general case needs either
a **per-axis (not DRMS) velocity gate** or a **frozen covariance-shape rule**. This is a
*shape* assumption, not merely the unfrozen-percentile coupling of §5.2, and is registered
alongside it in §5.2.

**Evidence chain to the tracker (U-T1).** LEO Doppler is a range-rate observation; single-
observation range-rate sigma follows from the tracker's measured frequency-error scale. U-T1
reports a conservative ≈ 4 Hz residual-frequency bound (⅛ of the 32 Hz fixture coarse bin) at
moderate/high C/N₀. Range rate `= −Δf · c / f_carrier`, so per-observation sigma is ≈ **0.11
m/s** at Ku (~11.3 GHz) and ≈ **0.74 m/s** at Iridium L-band (~1.62 GHz). Reaching a fused
0.014–0.028 m/s DRMS therefore demands tens (Ku) to hundreds (L-band) of well-conditioned
observations — exactly the baseline's 10–20 min constant-heading convergence legs and the
denied "after an uninterrupted 20-minute leg" clause. **The 4 Hz scale is a fixture number
(Fs = 8192, 32 Hz grid); production bin sizing differs**, so these are order-of-magnitude
scales, not frozen sigmas. The aided 0.014 m/s target is *demanding* and its achievability is
weak-evidence pending replay (§ summary).

**Sensitivity.** 2× tight (aided 0.007 m/s): likely unreachable pre-convergence → chronic
velocity-driven revocation. 2× loose (aided 0.028 m/s): per-axis 95 % error ≈ 0.039 m/s,
approaching the *denied* limit inside the aided profile.

**Validation plan.** Withheld replay velocity error vs truth (per-axis) scored against the
0.02/0.04 acceptance; per-constellation link-budget + real-capture discriminator sigma to
replace the 4 Hz fixture scale; convergence-time study confirming the fused sigma is reached
within a leg.

### 1.3 Heading — `aided = 0.01745 rad (1.0°)`, `denied = 0.04363 rad (2.5°)`

**Derivation.** Acceptance profile heading error: aided ≤ 2°, denied ≤ 5° (both `[UNVERIFIED]`
in the baseline — "the handoff supplies no validated heading limit"). Treating the acceptance
number as a ≈ 2-sigma bound, the one-sigma protection limit is half: aided 1.0° = 0.01745 rad,
denied 2.5° = 0.04363 rad. This preserves the baseline's stated purpose for the 2°/5° gates —
bounding transverse velocity error from heading-rotated speed-log measurements — while
allowing degraded magnetic operation.

**Sensitivity.** 2× tight (aided 0.5°): a single-magnetometer or degraded-heading condition
revokes immediately, discarding otherwise usable operation. 2× loose (aided 2°): the one-sigma
limit equals the *acceptance* number, so ~5 % of epochs exceed it — per-epoch integrity no
longer implies the campaign heading statistic; transverse velocity error from heading-rotated
speed log roughly doubles.

**Validation plan.** Replay heading error vs truth; dual-magnetometer-agreement and IMU-turn-
consistency characterisation; explicit test of the both-magnetometers-lost degradation row
(`SAFETY_CASE.md` §2.3) to confirm the heading PL revokes as intended.

---

## 2. Timers

### 2.1 `t_lease` — `1.0 s` (lease liveness, G4)

**Derivation.** `t_lease` bounds the age of the last *produced* solution frame (companion/
estimator liveness), and `SAFETY_CASE.md` §1.1/§3.1 requires `t_lease < GPS_TIMEOUT_MS`
(4000 ms). Frames renew at the DR-fill cadence (every accepted IMU propagation, ~10 ms at the
100 Hz nominal, CONTRACTS v5.1) with the nav solution nominal at 5 Hz (200 ms). D17a
(`tools/sitl/evidence/D17a.md`) measures native ArduPilot fix-type degradation at ~1.0 s and
EKF-failsafe HOLD at ~5.07 s. Proposed **1.0 s**. Margin basis, stated against the correct
cadence: the *implemented* renewal opportunity is the **DR-fill epoch emitted on every accepted
IMU propagation — nominally ~10 ms at the 100 Hz IMU rate (CONTRACTS v5.1)**, so 1.0 s is
**~100×** the renewal cadence; the 5 Hz nominal navigation *publication* period (200 ms) is the
coarser cadence and 1.0 s is 5× that. Either way, 1.0 s comfortably absorbs scheduler/GC jitter
and sparse-frame intervals. It is 4× below `GPS_TIMEOUT_MS` (4 s) and ~5× below the ~5.07 s
native failsafe — so in **Case A** (companion alive) the supervisor self-revokes well before
ArduPilot's own reaction. Note the guarantee is Case-A-only: `t_lease`
does nothing in Case B (companion death/link loss), where only the physical layer is credited
(`SAFETY_CASE.md` §2.1).

**Sensitivity.** 2× tight (0.5 s): nuisance lease expiry on a normal scheduling stall or GC
pause → spurious revocations. 2× loose (2.0 s): still < 4 s, but the Case-A self-revoke lead
over the ~5.07 s native failsafe shrinks from ~4 s to ~3 s, and a frozen-but-alive estimator
(H4) keeps asserting authority up to 2 s longer.

**Validation plan.** SITL/HIL timing of the Case-A hand-to-helm path at pinned Rover-4.6.3
(measured companion-commanded HOLD latency is ~51 ms, D17a) to confirm the self-revoke
completes with margin; replay of estimator-stall (H4) injections to confirm the lease expires
on a frozen sequence; on-target scheduler-jitter measurement to confirm 1.0 s clears the
worst-case frame gap.

### 2.2 `t_dr` — `120 s` (dead-reckoning authority, G2d)

**Derivation.** `t_dr` bounds the age of the last *absolute* (LEO/aided) position-constraining
observation and revokes authority even if the covariance still looks in-limit (H1 backstop,
independent of the possibly-optimistic covariance of H2). During all-LEO-loss, DR frames keep
renewing the lease (G4) and authority ends on G2 — the growing PL **or** `t_dr`, whichever
first (`SAFETY_CASE.md` §1.1/§2.3). Proposed **120 s**: long enough to ride through realistic
multi-satellite handover gaps (tens of seconds of no valid correlation), short enough that an
optimistic covariance cannot let a badly-drifted DR solution retain authority. At a
conservative post-loss residual velocity-error scale of ~0.05 m/s, 120 s of pure DR is ~6 m of
velocity-driven drift — well inside both the proposed 250 m scalar revoke ceiling and the
proposed 250 m denied PL — so under a *correct* covariance the
PL check remains the primary bound and `t_dr` is the backstop against an *incorrect* one.
**This is the weakest-evidence timer in the set**: no replayed LEO-gap statistics exist yet, so
120 s is a displacement-hull engineering estimate.

**Sensitivity.** 2× tight (60 s): may nuisance-revoke during normal sparse-fix gaps in weak
geometry, costing availability. 2× loose (240 s): admits ~12 m+ of unmonitored velocity-driven
drift and, more importantly, doubles the window in which an optimistic covariance (H2) can hold
authority on a stale solution — the exact hazard `t_dr` exists to bound.

**Validation plan.** GNSS-withheld replay of minutes-long LEO gaps (the test `SAFETY_CASE.md`
§1.1 explicitly names): measure real revisit-gap statistics for the operating area/constellation
mix, and the DR position-error growth vs truth across a gap, then set `t_dr` to the largest gap
the drift budget tolerates inside the denied PL. Re-derive for any non-displacement hull.

### 2.3 `t_eph` — `108000 s` (30 h ephemeris-age backstop, G2e)

**D59 reconciliation.** Proposed **108000 s (30 h)**, equal to the graduated-aging hard
ceiling. G2e is a freshness backstop for truly ancient ephemeris, not the accuracy governor.
G2p is the continuous accuracy governor: after the separate
`t_fresh = 21600 s (6 h)` inflation boundary, age-derived measurement-noise inflation widens
the protection limit and self-revokes on accuracy grounds when appropriate. Thus the 6 h
fresh-window still controls nominal versus inflated weighting, but the former 6 h authority
cliff is **superseded**. A hard G2e cliff at 6 h would defeat D45's passage requirement while
duplicating the continuously conservative G2p control.

**Sensitivity.** A tighter G2e ceiling reduces exposure to an invalid aging model but can
prematurely end a 100 km/long-passage mission even while G2p remains honest. A looser ceiling
admits ephemerides beyond the evidence envelope and increases reliance on extrapolated
inflation. The selected 30 h aligns the two finite backstops: observations are hard-rejected
and authority freshness fails beyond the same ceiling.

**Validation plan.** On real SupGP/real-signal captures, propagate ephemerides against
precise/near-real-time references across 0–30 h and replay the resulting range-rate errors
through the production inflation, EKF, and G2p gate. Demonstrate that the age-inflated PL
self-revokes before unsafe error, verify G2e stays open through exactly 30 h and closes above
it, and confirm pre-departure caching supports the intended passage. Synthetic epoch shifting
is availability evidence only and cannot freeze this value.

### 2.4 Per-source freshness deadlines — and a fail-closed enforcement gap

**Why this section exists (fixes the review HIGH finding).** The `SAFETY_CASE.md` fail-closed
register (§1.1 timer table, §1.2 sub-predicates, §5) names **per-source freshness deadlines**
— "last IMU / magnetometer / speed-log / ephemeris sample" — as authority-contract parameters
feeding G2/G3, and §1's fail-closed gate covers "per-source freshness limits" by name. **These
deadlines are absent from `AuthorityParams` (CONTRACTS v5) and from `AuthorityParams::is_complete()`.**
The consequence is a real enforcement gap: **a fully-populated `AuthorityParams` satisfies
`is_complete()` while the per-source freshness limits named in the safety-case register remain
unfrozen** — so the code-level fail-closed gate does *not* cover the whole safety-case register.
This section proposes values *and* registers the gap (§5.4) as a contracts-owner action item;
it is **not** silently absorbed into the appendix as though the fields existed.

The per-source deadlines are derived from the rate contract (`DESIGN_BASELINE.md`/CONTRACTS v2
Rate contract) as a small multiple of each source's nominal period, sized to the fault each
staleness represents:

| Source | Nominal rate | Proposed deadline | Derivation / fault represented |
|---|---|---|---|
| IMU | 100 Hz (10 ms) | **0.10 s** | ~10 nominal periods. IMU drives *every* propagation; staleness makes time-propagation untrustworthy → the baseline's "IMU stream stale ⇒ revoke" row. Tightest deadline; must be ≪ `t_lease` (1.0 s) since IMU staleness is a more immediate fault than mere output silence. |
| Magnetometer (each) | 10 Hz (100 ms) | **0.50 s** | ~5 nominal periods, per magnetometer. Loss of one → continue on the other (baseline); loss of both → heading PL (G2h) revokes. Looser than IMU because heading degradation is caught by G2h. |
| Speed log | 5 Hz (200 ms) | **1.00 s** | ~5 nominal periods. Speed-log loss removes only current/ground-speed separation (baseline); G2 catches downstream effects. Least safety-critical of the three marine feeds. |
| Ephemeris | event / cache | **= `t_eph` (108000 s)** | The ephemeris "sample" is the SupGP record. G2e uses `t_eph` as the 30 h ancient-ephemeris backstop (§2.3); the separate 6 h inflation fresh-window only ends nominal weighting. |

**Sensitivity.** IMU 2× tight (0.05 s): nuisance revoke on scheduler jitter/USB hiccup. IMU 2×
loose (0.20 s): up to 0.2 s of propagation on a stale IMU before revoke — the exact
untrustworthy-propagation hazard the deadline exists to bound. Magnetometer/speed-log looser
deadlines trade heading/current availability against staleness; their downstream effects are
also caught by G2, so they are secondary to IMU.

**Validation plan.** On-target bus-jitter and gap measurement per source (the deadline must
clear the worst-case healthy inter-sample gap with margin); withheld replay of single-source
dropout injections confirming revoke fires at the deadline and the estimator keeps running
(baseline: staleness stops authority, never the estimator); IMU-stale injection tied to the
`SAFETY_CASE.md` §2.3 degradation row.

**Registered gap (see §5.4).** Until CONTRACTS adds these fields to `AuthorityParams` and
`is_complete()`, freezing them cannot be enforced by the code-level fail-closed gate; they must
be enforced upstream (executive/integrity ingress) and their freeze tracked separately.

---

## 3. Caution band, revoke backstop, dwells, and ACK timeout

The metric for the caution/revoke scalars is the horizontal-accuracy metre value (§0.1);
`dwell_clear`, `dwell_rearm`, `T_ack` are seconds. Ordering constraint (`SAFETY_CASE.md`
§3.2): `caution_clear (60) < caution_enter (75) < revoke_threshold (250)`.
Per D59, the revoke scalar now matches the proposed 250 m denied PL. The 60/75 m caution
values remain conservative proposals pending re-derivation for useful helm lead time.

### 3.1 `revoke_threshold` — `250 m`

**D59 reconciliation after D56/D58.** A single, profile-independent hard horizontal-accuracy ceiling
at which the metric forces WARNING (`metric < revoke_threshold` is a G2 conjunct in `lib.rs`).
The proposed value moves from the former 100 m denied PL to **250 m**, matching the D56/D58
denied PL (`500 m p50 / k=2`). This resolves the retained/overrides note: the scalar no longer
makes the D56 relaxation inert by revoking at 100 m before the profile check.
In aided operation the tighter 12 m profile PL still trips first. It is a backstop against
a **finite-but-dangerously-loose profile limit**, not against an *absent* one: an absent
(`None`) profile limit already fails closed independently, because `AuthorityParams::is_complete()`
is itself a G2 conjunct (`lib.rs`) and rejects any missing limit regardless of `revoke_threshold`.
The scalar's value is a second, profile-independent ceiling under a mis-set-but-present limit.

**Sensitivity.** Tightening toward 100 m restores the superseded availability override and
defeats the settled D56 target. Loosening above 250 m makes the independent backstop weaker
than the intended denied PL and could mask a mis-set-but-present profile limit. The 250 m
choice and its caution-band/hazard evidence remain `[UNVERIFIED]`.

**Validation plan.** Use the §1.1 multi-satellite and real-signal withheld-replay coverage
study to demonstrate the 250 m profile PL and scalar trip consistently, with boundary tests
for strict scalar semantics. Re-derive `caution_enter`/`caution_clear` and verify useful helm
lead time before freezing any scalar.

### 3.2 `caution_enter` — `75 m` · `caution_clear` — `60 m`

**Derivation.** The caution band summons the helm *before* revocation. Referenced to the
**denied** envelope (where the ladder is most safety-relevant). The 75/60 m values predate
D59 and are retained as conservative proposals, not as percentages of the new 250 m revoke:
pre-alert on ≥ 75 m, clear on ≤ 60 m sustained for `dwell_clear`. The 15 m entry/clear
separation is real hysteresis against metric noise near the threshold (H3).
**Consequence of single-scalar thresholds:** in *aided* operation the metric
sits far inside 12 m, so this band never fires — aided mode has no caution pre-alert. This is
the profile-independence limitation registered in §5.1; the values are chosen to make the
ladder meaningful in the mode where it matters most.

**Sensitivity.** Entry too near the 250 m revoke provides too little helm lead time; entry too
low causes nuisance pre-alerts. Entry too near clear (narrow band) causes alarm chatter; clear
too low leaves caution annunciation after genuine recovery.

**Validation plan.** Replay the degrading-geometry / LEO-loss trajectory and tune the band so
the helm gets a stable, useful lead time before revoke without chatter; **strongly recommend
adopting the §5.1 per-profile caution band** so aided mode regains a pre-alert.

### 3.3 `dwell_clear` — `5 s`

**Derivation.** Sustained-recovery dwell before CAUTION→NOMINAL clears (on top of the 60/75 m
hysteresis). ~5 s ≈ 25 solution frames at 5 Hz — enough to confirm the metric has genuinely
settled below `caution_clear`, short enough not to strand a stale soft-alert. Human-factors /
anti-flap reasoning; no trial evidence yet.

**Sensitivity.** 2× tight (2.5 s): metric noise can re-trip caution → soft-alert flapping. 2×
loose (10 s): recovered solution still annunciates caution, eroding helm trust in the alert.

**Validation plan.** Replay metric-noise near the clear threshold; sea-trial helm feedback on
alert stability.

### 3.4 `dwell_rearm` — `10 s`

**Derivation.** Minimum dwell after a latched revocation (state L) before a helm re-arm can
re-grant (L→N guard, `SAFETY_CASE.md` §2.2). Longer than `dwell_clear` because re-granting
authority is higher-stakes than clearing a soft alert: it forces the fault condition to have
genuinely cleared and a deliberate, considered helm re-arm, preventing arm/fault/arm cycling
(H3). Proposed **10 s**.

**Sensitivity.** 2× tight (5 s): rapid re-arm after a transient fault risks authority cycling.
2× loose (20 s): operationally frustrating recovery after a genuinely transient condition,
tempting the helm to steer manually / distrust the system.

**Validation plan.** Fault-injection replay of transient-then-recover sequences; sea-trial helm
workflow feedback. (Note the open `latched_since_ns`-keying test item from `DECISIONS.md` D33
interacts with this dwell — fresh-from-DISARMED arms.)

### 3.5 `T_ack` — `10 s`

**Derivation.** Time in WARNING with no helm ACK before escalation to ESCALATED/max alarm
(`SAFETY_CASE.md` §2.2/§3.2). Because the terminal state is **non-kinetic** (software never
manoeuvres), `T_ack` sizes *attention-summoning*, not collision avoidance: a helm should
acknowledge a loud alarm within a few seconds; ~10 s escalates promptly without escalating on a
helm who is mid-action. It is nonetheless bounded by the vessel manoeuvre/collision time budget
(displacement-hull assumption); a planing/fast boat shrinks it (`SAFETY_CASE.md` vessel-speed
caveat).

**Sensitivity.** 2× tight (5 s): premature escalation → alarm fatigue / desensitisation. 2×
loose (20 s): slow escalation eats into the manoeuvre budget on a fast boat.

**Validation plan.** Human-factors alarm-response measurement in the trial environment; re-derive
against the actual hull's manoeuvre/collision time budget before trials.

---

## 4. Tracker detection-threshold false-alarm analysis (U-T1 finding F9)

Finding F9 (`DECISIONS.md` D36) routes the `pnt-tracker` default detection threshold of **32**
(15.1 dB) here for a probability-of-false-alarm (PFA) analysis. The tracker's per-block quality
statistic is `Q = peak_delay_bin_power / mean_of_the_other_delay_bins_power`, maximised over
the frequency-hypothesis grid (`crates/pnt-tracker/src/lib.rs`). The fixture geometry is
**256 delay bins × 256 frequency bins** (Fs = 8192, N = 256, 32 Hz coarse grid). Measured
noise-only quality statistics over 4000 pure-noise blocks: **median 11.5, p99 15.7, max 20.0**
(zero false alarms at threshold 32), from `.orchestration/reports/U-T1-review-opus-measurements.md`.

> **Provenance caveat (carried from the artifact).** The 4000-block quantiles are **review
> measurements produced by the U-T1 deep-review seat's own probes** (pnt-tracker commit
> 54005dd, probes run in-worktree then removed), **not shipped-test evidence and not
> reproducible from the committed suite** — the committed noise test covers only **24 blocks**
> (U-T1 report). They are archived as citable evidence but carry no reproducible seed/command.
> Everything below that leans on these quantiles inherits that reproducibility gap; see the §4.4
> assumptions.

### 4.1 Analytic model

**Statistic → Fisher's g.** Within one frequency hypothesis, the N_d = 256 circular-correlation
delay-bin powers are (under the assumptions below) i.i.d. exponential (complex-Gaussian
magnitude-squared). Let `S` be their sum and `P_max` the peak. The code's floor is the mean of
the **other** N_d−1 bins, so

```
Q = P_max / [ (S − P_max) / (N_d − 1) ] = (N_d − 1) · g / (1 − g),   where g = P_max / S.
```

`g` is exactly **Fisher's g-statistic** (largest ordinate / sum) for the max of N_d i.i.d.
exponentials, whose exact exceedance is

```
P(g > x) = Σ_{j=1}^{⌊1/x⌋} (−1)^{j−1} · C(N_d, j) · (1 − j·x)^{N_d − 1}.
```

Inverting the map, threshold `Q` corresponds to `g_thr = Q / (Q + N_d − 1)`. For the reported
quality the tracker also **maximises over the N_f = 256 frequency rows**, so the per-block PFA
is bounded by a union over rows:

```
P_block(Q) ≤ N_f · P(g > g_thr) ≈ N_f · N_d · (1 − g_thr)^{N_d − 1}
           = N_cells · (1 − g_thr)^{N_d − 1},   N_cells = 65536.
```

(At the operating threshold **Q = 32** the leading Fisher term dominates the next by ~12.7
orders of magnitude, so the single-term approximation is exact for the PFA figure. This is
**not** true across the lower validation thresholds — near the median `Q = 12.06` the second
term is ~5e-4 of the first (≈3.3 orders), and near p99 ~5.2 orders — so the validation-table
computations below retain the full Fisher sum, and the "single leading term" simplification is
scoped to `Q = 32` only.)

### 4.2 Model validation against the measured noise statistics

Inverting `P_block` for the quantile thresholds and comparing to the empirical 4000-block
statistics (independent-cell prediction vs observed):

| Quantile | P_block | Q predicted (i.i.d. cells) | Q observed |
|---|---:|---:|---:|
| median | 0.5 | 12.06 | **11.5** |
| p99 | 0.01 | 16.19 | **15.7** |
| max (≈1/4000) | 2.5e-4 | 20.14 | **20.0** |

The model reproduces all three empirical quantiles to within ~0.5 in Q. The observed values sit
**slightly below** the independent-cell prediction, and the gap narrows toward the tail. This is
the signature of **mild positive inter-cell dependence**: positive correlation among the
searched cells **reduces the effective number of independent trials**, so the independent-cell
union calculation predicts a *larger* maximum-tail than reality — i.e. for the row/search
dependence alone, the independent-cell PFA is an **over**-estimate (**conservative**), not
optimistic. **Correction of the previous draft:** the earlier claim that the lower empirical
quantiles made the model "slightly optimistic" and therefore warranted inflation was
**direction-inverted and is withdrawn**. The lower empirical quantiles cannot, on their own,
justify inflating the PFA. The legitimate reasons to carry margin above the analytic estimate
are the **separate** modelling-limit caveats in §4.4 (A3-class effects: coloured or
non-Gaussian noise, interference, normalisation mismatch, non-stationarity on real
captures — reference-sidelobe dependence alone is conservative, per A1), which act on the
*marginal distribution* and can make real-capture tails **heavier** in a way this fixture
cannot bound —
distinct from, and not inferable from, the conservative row-dependence effect.

### 4.3 PFA at threshold 32, and the false-alarm rate

At Q = 32: `g_thr = 32/287 = 0.1115`, giving

```
P_block(32) ≈ 65536 · (255/287)^255 = 5.30e-9   (ANALYTIC-MODEL estimate only).
```

**This 5.3e-9 is the analytic-model figure, not an empirically validated PFA.** 4000 noise
blocks (and they are review probes, not shipped tests — provenance caveat above) can bound a
probability of order ~1e-3, not ~5e-9; they can show threshold 32 sat ~12 above the observed
noise-only max of 20.0 in that run, but they **cannot validate a false-alarm probability near
5e-9 nor justify any specific numeric bracket** (e.g. the earlier "order 1e-8" bracket is
**withdrawn** — the A3-class effects of §4.4 (interference, coloured/non-Gaussian noise, non-stationarity) admit an *unquantified*
heavier real-capture tail that could lie well above 1e-8). As an illustrative *model* rate: the
fixture block is 256/8192 = 31.25 ms (~32 blocks/s per tracked signal), so the model per-tracker
FA rate is ≈ 32 × 5.3e-9 ≈ 1.7e-7 s⁻¹ → ~68 model-days between false alarms per tracker — a
figure of the *analytic model at the fixture geometry only*, superseded by real-capture
measurement (§4.5).

### 4.4 Assumptions and caveats (including cell correlation)

- **A1 — i.i.d. exponential delay bins.** Exact only for white input noise and an ideal
  delta-autocorrelation reference. **Real Starlink PSS/SSS, Iridium, Orbcomm sequences have
  non-zero autocorrelation sidelobes** → off-peak bins become correlated and the search
  structure no longer matches the independent-cell model. Under the stated white
  complex-Gaussian noise model this dependence alone reduces the effective independent-cell
  count (a conservative direction, as in A2) and does **not** by itself make each normalised
  bin's marginal heavier than exponential (amended 2026-07-23 per verify NEW-1; the prior
  heavier-tail claim here is withdrawn as unsupported). Genuinely heavier marginals require
  separate evidence — coloured or non-Gaussian noise, interference, normalisation mismatch,
  or non-stationarity on real captures — which is exactly A3's territory and remains the
  reason the synthetic-fixture PFA cannot be trusted as the production PFA.
  (No real sequences are shipped — U-T1 `[UNVERIFIED]`.)
- **A2 — row union bound (conservative).** The N_f frequency rows are **positively correlated**
  (same IQ block; the 32 Hz grid equals the FFT bin spacing Fs/N, so a row shift is a circular
  re-index), so the union bound over-counts and `P_block` is an **upper** bound; positive
  dependence reduces the effective independent-trial count. The empirical quantiles sitting
  *below* the independent-cell prediction are consistent with this conservative direction. This
  effect **cannot** justify inflating the PFA — inflation is warranted only by A3-class evidence (§4.2; A1's dependence effect is itself conservative).
- **A3 — stationary white Gaussian noise.** Real captures carry coloured noise, narrowband
  interference lines, ADC quantization, dropouts and sea-surface multipath. The per-row CFAR
  normalisation self-adapts to broadband level changes but **not** to a narrowband interferer
  that mimics a correlation peak → real tails fatten.
- **A4 — fixture geometry ≠ production.** Fs = 8192, N = 256, 32 Hz grid, 256×256 cells are the
  **test fixture**. Production is 2.5–5 MHz bandwidth with real-length sequences; both N_cells and
  the block duration change, so **both the PFA and the block rate must be recomputed for
  production geometry**. Threshold 32 is validated *for the fixture*, not frozen for production.
- **A5 — evidence provenance and reproducibility.** The 4000-block quantiles are the U-T1 deep-
  review seat's **in-worktree probes** (commit 54005dd, probes removed after the run),
  **irreproducible from the committed suite** (24-block noise test only), with no recorded
  seed/command. They corroborate the analytic model's *shape* but are not a reproducible PFA
  measurement; §4.5's validation plan therefore requires a fresh, recorded, real-capture
  characterisation before any production freeze.

### 4.5 Proposed threshold policy

1. **Keep the peak-to-mean-of-others ratio (CFAR) statistic.** Its scale-invariance to input
   noise power is a genuine robustness property for a maritime RF environment with a changing
   noise floor — prefer it over any absolute-power threshold.
2. **Set the threshold from a target per-block PFA via the Fisher-g model scaled to the actual
   production search volume**, not a magic constant: solve `N_cells^prod · (1 − g)^{N_d^prod − 1}
   = PFA_target` with `g = Q/(Q + N_d^prod − 1)`. This makes the threshold grow ~logarithmically
   with search volume (Bonferroni/CFAR-consistent) as N and the Doppler span change.
3. **Choose `PFA_target` from a false-observation budget, not from minimising PFA.** The tracker
   is the **first** gate; a spurious detection still faces the EKF per-component 1-DOF chi-square
   NIS gate, the 5° elevation mask, the ephemeris-age gate, the pass-nuisance state and cross-
   sensor residuals. A per-block PFA of ~1e-7 to ~1e-8 in production geometry (≈ <1 false
   detection per hours-to-days per tracker) is a reasonable starting budget, leaving the EKF gate
   as the second line. (At the fixture, ~1e-7 corresponds to Q ≈ 29; 32 gives ~5e-9.)
4. **The missed-detection cost sets the *upper* bound on the threshold.** Too high → fewer
   accepted Doppler observations → weaker geometry → larger protection limits → authority harder
   to sustain and convergence slower. The threshold is a **detection/false-alarm trade-off to be
   set jointly with the per-constellation link budget** (C/N₀), which is currently `[UNVERIFIED]`
   (no per-constellation C/N₀ measurements). Do **not** maximise for low PFA in isolation.
5. **Validate on real noise-only and weak-signal captures**, using this Fisher-g model only as
   the analytic scaffold. Deploy a threshold set from measured real-capture noise statistics with
   explicit margin above the measured noise-only max (the fixture shows 32 ≈ 12 above max 20).

**Fixture recommendation:** retain the explicit default **32** (analytic-model PFA ~5e-9;
observed ~12 above the noise-only max in the review run) as a conservative provisional. **Production threshold: `[UNVERIFIED]`**, to be
re-derived per §4.5(2)–(5) once real sequences, production geometry and per-constellation link
budgets exist. The tracker threshold is **not** an `AuthorityParams` field and is not in the
appendix; it is nonetheless a steering-relevant `[UNVERIFIED]` value with its own freeze path.

---

## 5. Open design issues registered by this proposal

### 5.1 Single-scalar caution/revoke thresholds vs per-profile protection limits

`AuthorityParams` carries `ProtectionLimits` **per profile** (aided/denied, about 21× apart on
position) but `caution_enter`/`caution_clear`/`revoke_threshold` are **single scalars** compared
against the same metre metric. With the aided PL at 12 m and the caution band referenced to the
250 m denied scalar ceiling, **aided mode has no caution pre-alert** — the profile PL revokes before
the caution metric is ever reached. This proposal makes the band meaningful in the mode where it
matters most (denied/degrading) and accepts the aided-mode gap. **Recommended contracts change:
make the caution band (and revoke backstop) per-profile**, matching `ProtectionLimits`. Routed
to the contracts owner as a proposed v6 refinement; not assumed here.

### 5.2 Coverage-factor / acceptance-percentile / covariance-shape coupling

The `k = 2` position factor and the velocity/heading mappings all stand in for the baseline's
**unfrozen acceptance-percentile/confidence definition**. When that definition is frozen (baseline
obligation), every §1 limit must be re-derived consistently — a single coupled freeze, not
independent. Additionally, the velocity mapping assumes **isotropic, Gaussian per-axis
covariance** (§1.2 anisotropy limitation): a scalar DRMS gate does not guarantee a per-axis
bound under the anisotropic covariance LEO geometry produces. **Recommended: freeze a covariance-
shape rule (or adopt per-axis velocity/heading gates) alongside the percentile.** Routed to the
contracts/baseline owner with §5.1.

### 5.3 Values with the weakest evidentiary support

`t_dr` (no LEO-gap replay statistics), the aided velocity PL 0.014 m/s (achievability unproven;
4 Hz tracker scale is fixture-only), and the human-factors dwells/`T_ack` (no trial evidence)
are the least-anchored proposals. See the summary paragraph.

### 5.4 `AuthorityParams`/CONTRACTS lacks per-source freshness fields — enforcement gap

The `SAFETY_CASE.md` §1/§5 fail-closed register includes **per-source freshness deadlines**
(IMU/magnetometer/speed-log/ephemeris), but `AuthorityParams` (CONTRACTS v5) and its
`is_complete()` completeness check do **not** carry them. **A fully-populated `AuthorityParams`
therefore passes `is_complete()` while these register items remain unfrozen** — the code-level
fail-closed gate does not cover the entire safety-case register. Proposed values are in §2.4.
**Contracts-owner action item (proposed v6):** add IMU/magnetometer/speed-log freshness fields
(seconds) to `AuthorityParams` and to `is_complete()`, and either add an ephemeris-freshness
field or record that ephemeris freshness is governed by `t_eph`. Until then these deadlines must
be enforced upstream (executive/integrity ingress) and their freeze tracked outside
`is_complete()`. Routed to the contracts owner; not assumed resolved here.

### 5.5 Coupled denied-authority rulings after D56 — resolved by D59

D59 resolves the two design-intent choices without freezing either:

1. **G2p accuracy governor:** denied horizontal PL and `revoke_threshold` are both proposed
   at 250 m. The former 100 m scalar override is superseded.
2. **G2e freshness backstop:** `t_eph_s` is proposed at the graduated 30 h ceiling. The
   separate 6 h inflation fresh-window remains the point where nominal weighting ends; it is
   not an authority cliff.

This makes D56 and D45 self-consistent, but real-signal validation of the PL mapping,
age-inflation behavior, 30 h ceiling, and alert lead time remains required. Every value remains
`[UNVERIFIED]`, PROPOSED-NOT-FROZEN, and fail closed until validated and frozen.

---

## 6. Machine-readable appendix — `AuthorityParams`

The block below **mirrors the flat `AuthorityParams` struct** (`crates/pnt-integrity/src/lib.rs`):
the nine scalar fields are **top-level** (before any table header, as TOML requires) with their
exact Rust names, and `aided`/`denied` are `[aided]`/`[denied]` tables mirroring the nested
`ProtectionLimits`. It deserializes field-for-field into a serde mirror of the struct — there is
no intermediate `[timers]`/`[thresholds]` grouping (a prior draft's non-deserializable
organisational nesting is removed).

```toml
# ============================================================================
#  AuthorityParams — PROPOSED, NOT FROZEN
# ============================================================================
#  Every value below is a PROPOSAL, [UNVERIFIED until validated].
#  This block grants NO authority. Per SAFETY_CASE.md §1 (D17 fail-closed
#  gate), an unfrozen authority-contract parameter is a CLOSED GATE: the
#  supervisor cannot grant steering authority while any field is unverified.
#  Freezing any value requires (a) the evidence in that parameter's §1-§3
#  validation plan AND (b) a signed DECISIONS.md line. Until then the
#  corresponding supervisor Option<f64> should remain None.
#
#  Shape mirrors the flat AuthorityParams struct: scalar fields top-level with
#  exact Rust names; aided/denied as ProtectionLimits sub-tables.
#  Units: position/velocity SI (m, m/s); heading rad; caution/revoke metric in
#  m (horizontal_accuracy_m); all *_s timers in seconds.
#  NOTE: per-source freshness deadlines (§2.4) are NOT fields of this struct
#  (register gap §5.4) and appear in a SEPARATE block below.
# ============================================================================

# --- Scalar fields (top-level; exact AuthorityParams field names) ----------
t_lease_s        = 1.0       # §2.1  < GPS_TIMEOUT_MS (4 s)              [UNVERIFIED]
t_dr_s           = 120.0     # §2.2  DR-authority backstop; weak evidence [UNVERIFIED]
t_eph_s          = 108000.0  # §2.3  30 h G2e freshness backstop         [UNVERIFIED]
dwell_clear_s    = 5.0       # §3.3  CAUTION->NOMINAL sustain dwell       [UNVERIFIED]
dwell_rearm_s    = 10.0      # §3.4  L->N re-arm dwell                    [UNVERIFIED]
caution_enter    = 75.0      # §3.2  metres (horizontal_accuracy_m)       [UNVERIFIED]
caution_clear    = 60.0      # §3.2  metres; clear < enter < revoke       [UNVERIFIED]
revoke_threshold = 250.0     # §3.1  D59; matches denied PL               [UNVERIFIED]
t_ack_s          = 10.0      # §3.5  WARNING->ESCALATED ack timeout        [UNVERIFIED]

# --- Protection limits: AIDED profile (gnss_authority = production) ---------
[aided]
horizontal_position_m   = 12.0     # §1.1  acceptance 25 m / k=2   [UNVERIFIED]
horizontal_velocity_mps = 0.014    # §1.2  per-axis 0.02 -> DRMS   [UNVERIFIED]
heading_rad             = 0.01745  # §1.3  1.0deg (acceptance 2/2) [UNVERIFIED]

# --- Protection limits: DENIED profile (recorded_only | off) ---------------
[denied]
horizontal_position_m   = 250.0    # §1.1  D56 p50 500 m / k=2     [UNVERIFIED]
horizontal_velocity_mps = 0.028    # §1.2  per-axis 0.04 -> DRMS   [UNVERIFIED]
heading_rad             = 0.04363  # §1.3  2.5deg (acceptance 5/2) [UNVERIFIED]
```

**Per-source freshness deadlines (§2.4) — NOT `AuthorityParams` fields (gap §5.4).** These are
*not* part of the struct above; `is_complete()` does not cover them. Shown separately as a
proposed structure pending the CONTRACTS v6 addition:

```toml
# ============================================================================
#  PROPOSED per-source freshness deadlines — NOT in AuthorityParams (§5.4)
#  Not covered by AuthorityParams::is_complete(); enforce upstream until
#  CONTRACTS v6 adds these fields. All seconds. PROPOSED, [UNVERIFIED].
# ============================================================================
[freshness_deadlines_s]
imu          = 0.10       # §2.4  ~10x the 100 Hz nominal period   [UNVERIFIED]
magnetometer = 0.50       # §2.4  ~5x the 10 Hz nominal, per unit  [UNVERIFIED]
speed_log    = 1.00       # §2.4  ~5x the 5 Hz nominal period      [UNVERIFIED]
# ephemeris freshness is governed by t_eph (108000 s); 6 h is inflation-only (§2.4)
```

```toml
# --- NOT an AuthorityParams field; recorded for traceability ---------------
# tracker_detection_threshold = 32   # §4  fixture-only; analytic-model PFA ~5e-9;
#                                    #     production value [UNVERIFIED]
```

---

## 7. Status

This document is **subordinate to `DESIGN_BASELINE.md` (normative) and `SAFETY_CASE.md`**; where
either governs, it governs. It adds no sensor or estimator requirement and does not revise the
safety case; it derives a proposed denied PL from the baseline's D56 acceptance amendment.
**Every value herein is a proposal, [UNVERIFIED until
validated]; nothing in this document grants steering authority.** Per `SAFETY_CASE.md` §1 the
authority-contract parameters remain a **fail-closed gate** until each is frozen by its named
validation evidence and a signed `DECISIONS.md` decision. This proposal's role is to make each
value *defensible and testable*, not to open the gate.
