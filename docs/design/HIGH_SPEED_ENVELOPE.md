# High-Speed & Extended-Passage Envelope Analysis

Status: **subordinate to [`DESIGN_BASELINE.md`](DESIGN_BASELINE.md) (normative), and to
[`SAFETY_CASE.md`](SAFETY_CASE.md) and [`PARAMS_PROPOSAL.md`](PARAMS_PROPOSAL.md)**
Unit: U-H1 · implements `DECISIONS.md` **D46** (20 kn) + **D47** (30 kn exploratory)
Original: 2026-07-23 · **Revised 2026-07-23 (fix round)** after dual review FAIL
(`.orchestration/reports/U-H1-review-sol.md`, `U-H1-triage.md`) and incorporation of the sourced
research `docs/research/R5-highspeed-dynamics.md`.

`DESIGN_BASELINE.md` is the single normative design document. This analysis is subordinate to it
and to the safety case and params proposal it extends. It **adds no normative requirement** and
**grants no authority**: it re-derives the frequency-reference, timer, estimator and passage
analyses for the **planing / semi-planing regime up to 20 kn (10.29 m/s)**, an **exploratory 30 kn
(15.43 m/s)** tier (§6), and **≥ 500 km / ≥ 24 h** denied passages, and it emits **amendment
candidates** (§5) routed to the owners of the baseline, safety case, params proposal, BOM and
architecture/contracts. Every value it introduces is a proposal; each is scoped as
**evidence-supported** (traceable to R5's cited sources or a repo study) or **judgment** (engineering
inference, marked `[UNVERIFIED]`). Where 20 kn operation is not supportable on present evidence, this
document says so plainly (§5).

**What the fix round changed (read first).** The first draft rested several load-bearing conclusions
on *assumed* constants that R5's sourced values contradict. The corrections move conclusions in both
directions and are flagged inline as **[SOFTENED]** or **[HARDENED]**:

- The vibration-rectification "shock isolation MANDATORY for integrity" conclusion **[SOFTENED]** —
  it rested on an assumed `a_rms = 3 g`; R5's measured RMS is **0.44 g**, at which the rectified bias
  is ~48× *under* the denied PL. Isolation is now **recommended-and-difficult**, driven by the
  *linear* g-sensitivity's availability cost, not a proven integrity breach.
- The slam model **[corrected]** — R5 gives impact durations of **100–450 ms** (not the assumed
  5–50 ms), so slam energy is **low-frequency (< ~10 Hz)**: harder to isolate (below light-isolator
  resonances), and the low `f` gives *less* `1/(πfτ)` suppression, not more. Net: a *harder* isolation
  problem than the first draft implied.
- The tracker "no block length survives / structural loss of lock" **[SOFTENED]** — the study
  explicitly calls its block sweep "not a closed-form limit" on a ±4.08 kHz fixture band; the stacked
  rate is a conservative **screening bound**, not a cap.
- The isolation-dB figure **[corrected]** from 10–15 dB to **5–7 dB** (bias ∝ `a²`).
- Timer `1/v` rescaling **[replaced]** by an explicit response/distance budget; the safe-speed cap is
  **not located between 20 and 30 kn** — on R5's 3–5 s human-response floor it could be **14–23 kn**.
- Heading/position limits **[relabelled]** from acceptance envelopes to per-epoch authority PLs.
- The good-fix timeline **[reconciled]** with `t_dr`: authority still expires at `t_dr`, not at the
  heading-cross-track time.

The baseline's displacement-hull assumption remains **load-bearing** and **is not relaxed** here.

---

## 0. Scope, régime definition, and R5 reconciliation

- **Speed envelope:** denied navigation up to **20 kn = 10.289 m/s** (`v20`); exploratory **30 kn =
  15.433 m/s** (`v30`, §6); displacement reference **7 kn = 3.601 m/s** (`v7`). `v20/v7 = 2.86`,
  `v30/v7 = 4.29`, `v30/v20 = 1.50`.
- **Régime:** displacement → semi-planing → planing across roughly `Fn∇ ≈ 1.0–1.2` (hull-dependent,
  **judgment**); for a small craft, order **8–12 kn**. Below it the baseline's displacement analysis
  stands; above it this document governs the re-derivation. Exact transition speed is a hull property,
  `[UNVERIFIED]`.
- **Master caveat.** R5 supplies a general HSC acceleration/vibration environment but its
  **strongest full-scale dataset is the 82 ft MK V SOC at Hs ≈ 0.9 m — not a 6–12 m RIB at a stated
  20–30 kn**, and R5 found **no** open campaign tabulating peak/RMS/A₁/₁₀ vs both speed *and* Hs for
  small planing craft, **no** small-craft crash-stop distances, and **no** planing-spray satcom trial
  (R5 §1.3, §5.1, §6.2 "dead ends"). So the binding gating evidence remains a **measured slam /
  vibration / trim / manoeuvre environment on the actual selected hull at the actual speeds and sea
  states.** Until it exists, the honest posture is *fail-closed to the displacement envelope*.

### 0.1 R5 reconciliation table (value used · R5 anchor · why · effect on verdict)

| Envelope quantity | Value used here | R5 sourced anchor | Reconciliation | Verdict effect |
|---|---|---|---|---|
| Sustained planing trim | **2–4°** | 2–4° efficient (Savitsky/Ghassemi; 2.24° @ 35 kn); >6° "uneconomical" (VERIFIED) | Adopt R5; drop the first draft's 3–6°. | cot-benefit *larger* (29–57×); mounting rule strengthened |
| Continuous vertical RMS accel | **0.44 g** (small-RIB upside to ~1 g as judgment) | 0.44 g RMS MK V head seas @ Hs 0.9 m (VERIFIED point) | Adopt R5's measured RMS; the first draft's 3 g was a **peak/A₁-class** number misused as RMS. | rectification bias collapses to ~48× under PL → isolation no longer *mandatory-for-integrity* |
| Slam peak (crew/coxswain station) | **~3–9 g** (8.62 g worst measured) | MK V peak 8.62 g head; A₁/₁₀ 2.7–3.2 g "max safe" (VERIFIED) | Adopt R5; drop the first draft's unsourced "bow 10–20+ g" (R5's bow-sea column is *lower*, 3.94 g). Small-RIB bow could exceed 9 g but that is **judgment**. | stacking/IMU concerns rescaled to sourced g |
| Slam duration | **100–450 ms** | NSWCCD-80-TR-2014/026 (VERIFIED) | Adopt R5; drop 5–50 ms. Energy is now **< ~10 Hz**. | isolation *harder* (low-fn, large stroke); suppression weaker |
| Oscillator linear g-sensitivity `Γ` | **~1×10⁻⁹/g** (good SC-cut OCXO) | Wenzel tip-over ~1e-9/g; NIST 1e-8…1e-10/g (VERIFIED) | Adopt R5 as the primary, sourced mechanism. | linear term becomes the lead concern |
| Oscillator quadratic `Γ₂` | **~1×10⁻¹¹/g² (judgment only)** | **Not in R5**; FE-5680A datasheet has *no* g-sensitivity row (VERIFIED absence) | Keep as an explicitly unsourced hypothesis; do **not** hang an integrity verdict on it. | rectification demoted to "measure it" |
| Vib phase-noise model | `L(f_v)=20log₁₀(Γ·a·f₀/2f_v)` | Wenzel/Filler/NIST (VERIFIED formula) | Adopt; this is the sourced mechanism replacing the g² story. | availability/phase-noise framing |
| IMU (VN-100) behaviour | rigid-mount preferred; **~4.5 g RMS random saturates**; VRE unspecified | VN-100 datasheet (VERIFIED) | Adopt R5's *sourced* saturation limit in place of the first draft's assumed "±16 g FS bow saturation". | IMU concern re-anchored; co-isolation now *in tension* with vendor guidance |
| Human alarm-response floor | **3–5 s** | R5 §5.2 (detection+recognition+motor, mixed VERIFIED/ASSUMED) | Adopt as a floor; do not scale below it. | 20 kn already at floor; cap not located at 20–30 kn |
| Small-craft stopping/turning | order boat-lengths (no metric table) | R5 §5.1 dead end (IMO ship rules don't apply to RIBs) | Keep order-of-magnitude only; require craft trial. | collision budget stays craft-specific `[UNVERIFIED]` |
| Translational Doppler @30 kn | **583 Hz Ku / 83 Hz L** (`v/c·f`) | R5 §6.3 says "~1.5 Hz Ku" — **arithmetic slip** (`5.1e-8×12e9 = 610 Hz`) | Reject R5's number, keep R5's *qualitative* point (attitude jitter/multipath/spray dominate over translational Doppler). Per D5, non-Grok arithmetic confirmed. | RF-availability rows added to §5 |

---

## 1. Frequency reference at planing speeds

### 1.1 The handoff analysis, restated

The handoff (lines ~119–124), for a displacement hull: the reference's enemy is **sustained mean
tilt, not wave motion**; **oscillatory FM** is suppressed as **`1/(π·f·τ)`**; a **sustained heel is a
step** the linear (bias + drift) clock model cannot absorb; free mitigation is to **mount the
resultant g-sensitivity vector `Γ` vertical** ("~11×"). D46 orders the planing re-derivation.

### 1.2 The clock → velocity identity (worst-case screening bound)

A fractional-frequency excursion `δ = Δf/f` appears on every carrier as `Δf_app = δ·f_carrier`;
converting to range rate the carrier cancels:

> **`Δv = δ · c`** — carrier- and vessel-speed-independent, at the **raw single-observation** level.

| `δ` | `Δv = δ·c` | vs denied velocity PL (0.028 m/s) |
|---:|---:|---|
| `1×10⁻⁹` | 0.300 m/s | 11× over |
| `1×10⁻¹⁰` | 0.030 m/s | at limit |
| `1×10⁻¹¹` | 0.003 m/s | 10× margin |

**Scoping (per review finding 14).** `Δv = δ·c` is a **worst-case screening bound on a single raw
range-rate observation**, not automatically a horizontal-velocity solution bias of the same
magnitude. The receiver clock bias/drift are *estimated common states*; how much of `δ` leaks into
solution velocity depends on satellite geometry, the clock-state process model, observation timing and
the excursion's bandwidth relative to the drift random-walk. Band-independence further assumes the
same fractional error reaches each chain coherently (a shared reference — true here, but separate
front-end electronics can add frequency-dependent terms). Repo evidence corroborates the raw mapping:
the tracker study measured clock bias `= f_carrier·ε` directly (`tracker/STUDY.md §4`: ε=1e-9 → 11.6 Hz
at Ku). **Evidence-supported:** the raw identity and its speed-independence. **Judgment:** the
solution-domain leakage, which needs a multi-satellite estimator injection study (routed, §5).

### 1.3 Sustained planing trim — vertical mounting survives and strengthens

At R5's sourced planing trim of **2–4°**, with `Γ` vertical, a sustained tilt `θ` changes the along-`Γ`
gravity projection only at **second order**, `δ_trim = |Γ|·g·(1−cosθ)`; the benefit vs horizontal
mounting is `cot(θ/2)`:

| tilt `θ` | benefit `cot(θ/2)` | `δ_trim` (`|Γ|=1e-9/g`) → `Δv` |
|---:|---:|---:|
| 2° (R5 optimum) | **57×** | 6.1×10⁻¹³ → 1.8×10⁻⁴ m/s |
| 4° (R5 efficient) | **29×** | 2.4×10⁻¹² → 7.3×10⁻⁴ m/s |
| 10° (displacement heel) | 11.4× | — |

**Evidence-supported:** the `cot(θ/2)` identity, and that at R5's 2–4° trim the benefit (29–57×) is
*larger* than at a 10° displacement heel, so the rule is kept and strengthened; the residual sustained-
trim excursion is sub-mm/s. **Correction (finding 13):** `cot(10°/2)=11.4` shows the handoff's "~11×"
is *consistent with* ~10° heel; it does **not prove** the handoff assumed that angle. **Conclusion:
sustained trim is not the frequency-reference threat.** The impulsive slam is (below).

### 1.4 Slamming — the linear g-sensitivity is the sourced concern; rectification is unproven

R5 gives sourced slam statistics (MK V, Hs ≈ 0.9 m, coxswain station): continuous **RMS 0.44 g**,
A₁/₃ 2.9 g, A₁/₁₀ 4.3 g, **peak 8.62 g** (head seas), **impact duration 100–450 ms** → dominant content
**below ~10 Hz**. A small RIB at 20–30 kn in the Danish straits may differ (bow, chop, loading); those
deltas are **judgment**, `[UNVERIFIED]` pending hull measurement.

**(a) Linear g-sensitivity — sourced, and the lead concern.** With `Γ ≈ 1×10⁻⁹/g` (R5), a slam of peak
`a` gives instantaneous `δ = Γ·a` → instantaneous `Δv = c·Γ·a`. Because R5's durations put the content
at **1–5 Hz**, the `1/(π·f·τ)` suppression is *weaker* than the first draft assumed (low `f` = less
suppression). Per-slam residual in a τ = 1 s integrated Doppler:

| slam peak | instantaneous `Δv` | residual @1 Hz | @2 Hz | @5 Hz |
|---:|---:|---:|---:|---:|
| 2.9 g (A₁/₃) | 0.87 m/s | 0.28 m/s | 0.14 m/s | 0.055 m/s |
| 4.3 g (A₁/₁₀) | 1.29 m/s | 0.41 m/s | 0.21 m/s | 0.082 m/s |
| 8.62 g (peak) | 2.58 m/s | 0.82 m/s | 0.41 m/s | 0.16 m/s |

These per-slam excursions are large relative to the velocity PL, but they are **transient and
zero-mean**: they inflate innovations and are chi-square-rejected (`SAFETY_CASE.md §2.3`), and they
raise the reference's close-in phase noise (R5's sourced `L(f_v)=20log₁₀(Γ·a·f₀/2f_v)`), degrading
tracker C/N₀ and risking cycle slips. **This is an availability / integrity-via-innovation concern —
epochs lost per slam — not a proven sustained bias.** Note the caveat (finding 3): `1/(π·f·τ)` is a
*sustained-sinusoid* model; a 100–450 ms half-sine needs direct convolution through the actual
correlator/discriminator/clock-estimator impulse responses (routed, §5). The numbers above are
screening estimates.

**(b) Vibration rectification — legitimate mechanism, unsourced magnitude, and NOT at the PL on
sourced RMS. [SOFTENED]** A quadratic sensitivity `Γ₂` rectifies zero-mean vibration into a *sustained*
DC offset `δ_DC = Γ₂·⟨a²⟩` (un-suppressed by `1/(πfτ)`). But **R5 supplies no oscillator `Γ₂`**, and the
FE-5680A datasheet has no g-sensitivity row at all (R5 §3.1, VERIFIED absence). Using the sourced
**RMS 0.44 g** and a *hypothetical* `Γ₂ = 1×10⁻¹¹/g²`:

`δ_DC = 1.9×10⁻¹²` → **`Δv = 5.8×10⁻⁴ m/s` — ~48× *under* the 0.028 m/s denied PL.**

The first draft's "0.027 m/s, at the PL, isolation MANDATORY" used `a_rms = 3 g`, which is a **peak/
A₁-class statistic, not continuous RMS** (review finding 1 / triage F1, confirmed). At the sourced RMS
the rectified bias is negligible; it reaches the PL only near `a_rms ≈ 3 g` **RMS**, which R5 does not
support for this environment. **Corrected conclusion:** rectification is a *hypothesis to measure*, not
a proven integrity breach; it becomes material only if a small-RIB `a_rms` is many-fold R5's 0.44 g or
`Γ₂` is far worse than assumed. Both require a shaker test of the selected part over the measured hull
PSD.

### 1.5 Is shock isolation required? — recommended, and hard [SOFTENED + HARDENED]

**Downgraded from REQUIRED to STRONGLY RECOMMENDED** on present evidence, and the *driver* changes:
not the (unproven) rectified DC bias, but the **linear-term phase noise / cycle-slip availability cost**
(§1.4a, sourced) plus **bounding the unquantified rectification** (§1.4b). This is judgment informed by
sourced mechanisms, not a derived integrity requirement.

**And isolation is harder than the first draft claimed.** R5 (§3.4, VERIFIED): the 100–450 ms slams put
energy **below ~10 Hz**, *below* many light-isolator resonances, so effective isolation needs a **low
natural frequency (`fn` well below the slam content) and large stroke** — NSWCCD explicitly warns
compact shock mounts for electronics are difficult for long-duration wave slam. A ~10–15 Hz corner (first
draft) would sit *above* much of the slam energy and help little. Revised target: **`fn` low enough to
attenuate the 1–10 Hz band with adequate stroke, without a resonance on the 0.5–3 Hz encounter band** —
a genuine design tension, `[UNVERIFIED]`, resolvable only against the measured spectrum.

**Architecture caveat (finding 4 / triage F9). [HARDENED]** The first draft's shared reference+IMU
isolated plate **conflicts with R5's VN-100 guidance**: VectorNav prefers a **rigid** IMU mount, warns
soft isolation degrades AHRS via relative motion and filter lag, and notes ~**4.5 g RMS random can
saturate** the accelerometers (R5 §4.2, VERIFIED). So the correct split is likely **isolate the
oscillator; keep the IMU rigidly mounted (or use a low-VRE IMU that does not need isolation)** — not one
soft plate for both. Any isolated frame also moves the reference's `Γ` orientation and lever arms under
thrust/trim, requiring re-survey (baseline extrinsics rule). BOM row M-1 is split accordingly (§5.2).

---

## 2. Speed-dependent safety timers

### 2.1 Collision budget, and why `1/v` rescaling is not a derivation [SOFTENED]

`SAFETY_CASE.md §0` requires the timer budgets to be re-derived if the hull planes. The first draft
rescaled them as `1/v`; **that is valid only if the displacement values encode a fixed distance
budget, and they do not** — `PARAMS_PROPOSAL.md §3.5/§2.2` calls `T_ack = 10 s` and `t_dr = 120 s`
weakly-evidenced human-factors/engineering estimates, not distance budgets (review finding 8). The
honest method is an **explicit budget**, not a rescaling:

`available response time = f(closing speed, detection latency, alarm latency, human response, helm/
control transition, craft stopping/turning distance)`.

Distances (order-of-magnitude; R5 §5 confirms no published small-craft crash-stop table exists):

| Quantity | 7 kn | 20 kn | 30 kn |
|---|---:|---:|---:|
| Distance per second | 3.6 m | 10.3 m | 15.4 m |
| Head-on closing (vs like vessel) | 7.2 m/s | 20.6 m/s | 30.9 m/s |
| Distance in 5 s | 18 m | 51 m | 77 m |
| ArduPilot EKF-failsafe HOLD onset (~5.07 s, `D17a`) | 18 m | 52 m | 78 m |

**Human-response floor (R5 §5.2, sourced-but-mixed): 3–5 s** for detection + recognition + motor
response. This bounds `T_ack` from below regardless of speed.

### 2.2 The safe-speed cap is not located at 20–30 kn [HARDENED]

If — *and only if* — the displacement `T_ack = 10 s` at 7 kn encodes a fixed distance `D = 36 m`, then
`T_ack(v) = D/v` must stay `≥` the human floor, giving a cap `v ≤ D/floor`:

| Human floor | Implied `T_ack` at 20 kn | Implied cap speed |
|---:|---:|---:|
| 3 s | 3.5 s (already ≤ floor edge) | **≈ 23 kn** |
| 5 s | 3.5 s (**below floor**) | **≈ 14 kn** |

**What is supported (evidence + this budget):** (i) at 20 kn a distance-preserving `T_ack ≈ 3.5 s` is
**already at or below the 3–5 s human floor**, so the collision-timer margin at 20 kn is thin and needs
an explicit budget, not a rescaling; (ii) at 30 kn the implied `2.3 s` is **below any credible floor**;
(iii) **the cap could lie anywhere ~14–23 kn** depending on the validated floor and the (unmeasured)
craft stopping/closing budget — it is **not** established to sit specifically between 20 and 30 kn. The
supported conclusion is therefore *"do not grant denied autonomous authority at 30 kn, and derive the
cap from an explicit budget — it may be below 20 kn,"* not a specific 20-vs-30 boundary.

### 2.3 Proposed timer handling (two classes, fail-closed) — planing values are budget-derived, not rescaled

Retain the two-class structure (the régime change is physical, and discrete classes are gate-checkable),
with the **planing-class collision timers explicitly marked "derive from budget," not rescaled**:

| Parameter | Class A (displacement) | Class B (planing) | Basis / status |
|---|---:|---:|---|
| `t_lease` | 1.0 s | **1.0 s** (hold) | liveness, not distance; unchanged |
| IMU/mag/speed-log freshness | 0.10/0.50/1.00 s | **unchanged** | sensor physics |
| `T_ack` | 10 s | **derive from §2.1 budget; ≥ 3–5 s floor** | **not** a `1/v` rescale; `[UNVERIFIED]` |
| `t_dr` | 120 s | **derive from budget + gap replay** | bounds collision exposure *and* observation staleness (finding 8); `[UNVERIFIED]` |
| `dwell_clear` | 5 s | **5 s** (hold — justified §2.4) | recovery-only |
| `dwell_rearm` | 10 s | **10 s** (hold — justified §2.4) | re-grant gate |
| caution band | denied 75/60 m | per-profile + speed lead-distance (routed) | `PARAMS §5.1`; `[UNVERIFIED]` |

### 2.4 Discharging D46's dwell instruction (finding 9) — analysed, not asserted

D46 ordered re-derivation of "`T_ack`, dwells" against the collision budget. Tracing each dwell through
the `SAFETY_CASE.md §2.2` state machine shows **why** the dwells hold while `T_ack`/`t_dr` scale:

- **`dwell_clear`** gates only **CAUTION→NOMINAL** (a *recovery* transition). It **cannot delay a
  revocation** — any fault edge (`G2↓`/`G3↓`/`G4↯`) transitions to WARNING regardless of dwell. A
  recovery-only delay does not consume collision budget → **non-safety-critical for collision; hold.**
- **`dwell_rearm`** gates only **LATCHED-SAFE→NOMINAL re-grant**, and a *longer* dwell is *more*
  conservative (harder to re-arm into a hazard). It cannot delay revocation and cannot permit premature
  authority → **non-safety-critical for collision; hold (longer is safer).**

This is the analysis D46 required, and it *confirms* holding the dwells — but now on a traced argument
(revocation-independence), not a bare "human-factors" label.

---

## 3. Estimator envelope at 20 kn

### 3.1 Vessel velocity in Doppler — and a corrected conditioning claim [corrected]

| Quantity | 7 kn | 20 kn | Satellite (550 km) |
|---|---:|---:|---:|
| Vessel Doppler at Ku (11.325 GHz) | 136 Hz | **389 Hz** | max ±264 kHz |
| Vessel Doppler at L-band (1.616 GHz) | 19 Hz | **55 Hz** | max ±38 kHz |

(These confirm R5's *qualitative* §6.3 point — translational Doppler is a small perturbation — while
correcting R5's arithmetic slip: 30 kn Ku is 583 Hz, not "1.5 Hz"; see §0.1.)

**Correction (finding 15).** The first draft claimed the larger vessel Doppler at speed makes the
velocity solution "better-conditioned." **Withdrawn.** The Doppler sensitivity to receiver velocity is
the line-of-sight Jacobian (a unit vector), **independent of the receiver's actual speed**; a larger
state value produces a larger measured offset but **not more Fisher information** at fixed geometry and
measurement noise. Speed does not improve velocity observability; it is neutral. What matters at speed
is the *excitation* (slam) and the *manoeuvre-reset* geometry (§4), not conditioning.

### 3.2 Tracker drift tolerance — a screening bound, not a cap [SOFTENED]

The tracker study (`tracker/STUDY.md §3`) all-detects a Doppler ramp to **4000 Hz/s at the 256-sample
block and 8000 Hz/s at the 128-sample block**, but states explicitly these are the **"largest
all-detected coarse grid points … not a closed-form limit,"** non-monotone, on a **±4.08 kHz fixture
acquisition band** (production needs ephemeris wipe-off / a wider architecture). So neither number is a
hard cap (review finding 6, confirmed against the study text).

Worst-case satellite drift is **3718 Hz/s** (Ku-high, 550 km). Vessel heave adds `a·u_LOS·f/c`; at
overhead the vertical LOS aligns with vertical heave *and* with maximum satellite drift, so summing the
two maxima is a legitimate **conservative screening bound** (not a routine prediction — the slam is
transient, the study injected constant ramps; finding 7). Using **R5-sourced** heave peaks:

| Heave peak (sourced) | Heave rate (Ku) | + satellite 3718 | vs tested grid points |
|---:|---:|---:|---|
| 4.3 g (A₁/₁₀) | 1592 Hz/s | **5310 Hz/s** | exceeds 256-block (4000); within 128-block (8000) |
| 8.62 g (peak) | 3191 Hz/s | **6909 Hz/s** | within 128-block (8000) |

**Corrected reading:** at 20 kn with sourced g-levels the combined rate exceeds the 256-block grid point
but sits **inside** the 128-block one, so **shorter blocks plausibly cope** — the opposite of the first
draft's "no block survives." The residual risk is real but is *"severe slams may exceed the tested
envelope and cause loss of lock; verify with finer ramp grids, production sequences and time-aligned
6-DOF replay,"* not *"structural / unavoidable."* IMU heave-rate feed-forward (predict the platform rate
so the correlator spans only the un-aided residual) is a **recommended** mitigation, routed (§5).

### 3.3 Process-noise envelope — direction and magnitude are ungrounded [SOFTENED]

The estimator study (`estimator/STUDY.md`, `D43`) found `Qa ≈ 4×10⁻⁴ (m/s²)²` optimal **for a near-truth
IMU**, and warned the sign could flip at sea. Planing certainly changes the environment, but the first
draft's "`Q` likely 10²–10⁴× larger" **overstates what is known** (finding 17): the IMU *measures* the
bulk acceleration, so unmodelled `Q` is set by IMU error residual — output filtering, clipping/
saturation, timing, aliasing, coning/sculling — **not by raw slam magnitude**, and neither the direction
nor the magnitude is established. **Corrected:** the `D43` low-`Q` tuning **must not be frozen for
planing**; the planing `Q` must be derived from the *residual acceleration spectrum of the selected IMU
on measured planing data*, then confirmed by NEES/NIS — no numeric multiplier is asserted here.

### 3.4 IMU requirements delta — re-anchored to R5's sourced VN-100 data [corrected]

R5 §4.2 (VERIFIED, VN-100 datasheet): powered shock survival **500 g**, sine vibration **6 g** operating,
**random vibration ~4.5 g RMS can saturate the accelerometers → filter collapse**, **rigid mount
preferred**, isolation can degrade AHRS, and **VRE is unspecified**. Re-anchored delta:

| Spec line | Why it matters (planing) | Status vs R5 |
|---|---|---|
| **Random-vib saturation (~4.5 g RMS)** | R5's *sourced* failure mode: broadband slam vibration approaching 4.5 g RMS saturates accels and collapses the AHRS filter → propagation loss. Replaces the first draft's unsourced "±16 g FS bow saturation." | **sourced concern**; hull `a_rms` vs 4.5 g must be measured |
| **VRE (µg/g²)** | DC accel bias under vibration → velocity drift; **unspecified on the VN-100 datasheet** → must be vendor-obtained or measured. | R5: unspecified `[UNVERIFIED]` |
| **Gyro g-sensitivity (°/s/g)** | Accel-dependent gyro bias under slam → heading drift (the handoff's weakest link). MEMS weakness; datasheet-bound or measure. | `[UNVERIFIED]` |
| **Mounting** | R5: **rigid preferred**; soft-isolating the IMU degrades AHRS. Contradicts a shared soft plate (§1.5). | R5 VERIFIED |
| **Output rate / anti-alias** | Slam content < ~10 Hz (R5) is largely *below* Nyquist at 100 Hz, so aliasing is **less** severe than the first draft's 10–100 Hz assumption implied — but bandwidth still matters for the fast content and for capturing the slam shape. | re-scoped |

**Verdict on the IMU [SOFTENED from "insufficient" to "conditionally survivable, unproven"]:** the
VN-100 class **survives the shock** (500 g) and moderate sine vibration (6 g); the binding question is
whether the hull's **broadband RMS** approaches the sourced ~4.5 g saturation limit and what its
**VRE/gyro-g** are — all `[UNVERIFIED]` pending the hull PSD and vendor data. Rigid mounting is
indicated. A lower-VRE industrial/tactical MEMS (R5 lists ADIS/SBG classes) is the fallback if VRE/gyro-g
prove inadequate. This is a BOM *decision*, not a proven inadequacy.

### 3.5 Heading dynamics [relabelled to authority PL]

Fast-turn heading lag: at a turn rate `ω` with magnetometer interval `Δt`, the gyro must bridge `ω·Δt`
of heading. Using **PARAMS per-epoch authority PLs** (not acceptance envelopes): the **denied heading PL
is 2.5°**, aided **1.0°** (`PARAMS §1.3`). At an *assumed* `ω = 30°/s` and `Δt = 100 ms` (both
`[UNVERIFIED]`, finding 11) the ~3° gyro-bridge lag exceeds the 1.0° aided PL and approaches the 2.5°
denied PL → **authority is expected to drop during aggressive turns**, consistent with "manoeuvres reset
convergence." Also: magnetometer propulsion-current deviation is larger at planing throttle (calibrate at
planing power). Registered as a degradation nuance (§5), tune `[UNVERIFIED]`.

---

## 4. Passage math at 20 kn

### 4.1 Convergence-distance table

Position is observable only over 10–20 min constant-heading legs (baseline); distance `= v·T`:

| Leg | 7 kn | 20 kn | 30 kn |
|---:|---:|---:|---:|
| 10 min | 2.16 km | 6.17 km | 9.26 km |
| 20 min | 4.32 km | **12.35 km** | 18.52 km |

A 20-min convergence leg at 20 kn needs ~12.4 km of straight searoom (D46's "~12 km"). Feasible in open
water, not in narrow traffic-dense passages — a *position*-observability constraint; velocity converges
in seconds regardless.

### 4.2 The 24 h / 500 km scenario vs the U-P1 30 h ephemeris ceiling

U-P1 (in progress): graduated age handling, fresh < 6 h, additive inflation `6 h < a ≤ 30 h`, hard
ceiling **30 h (108 000 s)**, from the SGP4/SupGP fit **0.94 km @ 6 h, 2.6 km @ 24 h** (→ `σ_add(24 h)
≈ 18.42 m/s`, U-P1-internal, inherited, `[UNVERIFIED]`). Ephemeris cached at departure:

| Passage | Duration | Age at arrival | Margin to 30 h |
|---|---:|---:|---:|
| 500 km @ 20 kn | 13.5 h | 13.5 h | 16.5 h — comfortable |
| 24 h denied (≈ 889 km @ 20 kn) | 24 h | 24 h | **6 h — tight** |

**Finding (evidence-supported by U-P1 pending its real-SupGP validation):** the binding case is ≥ 24 h
denied, which sits at the 24 h datum with **6 h to the ceiling — entirely consumed by any pre-departure
cache age.** Requires a **cache-at-departure freshness gate** (age counted from cache time). All U-P1
aging numbers are synthetic-only (`D43`/`D45`), `[UNVERIFIED]`.

### 4.3 DR-bridge error — authority-PL time [relabelled], and the good-fix reconciliation

Cross-track from a heading error `θ_h` over a leg is `≈ θ_h·v·t`. Using **per-epoch authority PLs**
(`PARAMS`: denied 100 m / 2.5°, aided 12 m / 1.0°) — not the acceptance envelopes the first draft
mis-labelled (finding 10):

| To reach... | authority PL / error | 7 kn | 20 kn | 30 kn |
|---|---:|---:|---:|---:|
| denied 100 m PL | 2.5° | 637 s (10.6 min) | **223 s (3.7 min)** | 149 s (2.5 min) |
| aided 12 m PL | 1.0° | 191 s | **67 s** | 45 s |

(The times are numerically ≈ the first draft's, because halving both numerator and angle cancels — a
coincidence, now correctly attributed to the *authority* PLs.)

**Good-fix reconciliation (finding 11) [corrected].** The D47 scenario enters from a good GPS fix, so at
loss the position is **aided-grade (~5–25 m)** and velocity is well-initialised. But authority is
governed by **`t_dr` — the age of the last *absolute position-constraining* observation** — and a LEO
Doppler epoch is a *range-rate* (velocity) observation. **Whether a LEO observation resets `t_dr` is an
open definitional question** (also flagged by review finding 21) routed to architecture/contracts (§5).
Under the strict reading (LEO velocity does **not** reset `t_dr`), authority **expires at `t_dr`
(~28–120 s depending on class), not at the 2.5–3.7 min heading-cross-track time**. So the good fix buys
**accuracy** (start converged, the full PL budget as drift headroom) and removes the cold-convergence
transient — but it does **not** extend authority past `t_dr`. The first draft's "good fix buys ~2.5 min
of authority" is **withdrawn**; the correct statement is *"the good fix buys start-accuracy and headroom;
authority still cycles at `t_dr` unless LEO observations are credited as position-constraining."*

### 4.4 Manoeuvre resets

Every course change resets position convergence (baseline). At 20 kn the denied 100 m PL is reachable
from heading error alone in ~3.7 min, and a re-convergence needs ~12.4 km of straight leg. So in
frequently-manoeuvring water **continuous denied position authority at 20 kn is not achievable**; options
(unchanged): long planned straight legs, a tighter absolute-heading source (non-magnetic compass rises in
value), or a denied speed cap. Registered as a baseline-scope decision (§5), not resolved here.

---

## 5. Consequence register

Each item routed and status-scoped as **evidence-supported** or **judgment**; all `[UNVERIFIED]` remain
fail-closed per `SAFETY_CASE.md §1`.

### 5.1 Baseline candidates (`DESIGN_BASELINE.md`)

| # | Candidate | Status |
|---|---|---|
| B-1 | Admit a planing régime with the fail-closed class selector (§2.3). | judgment |
| B-2 | Require **oscillator** shock isolation (recommended, hard — §1.5); **not** a shared reference+IMU soft plate. | judgment (softened from "required") |
| B-3 | Amend rate contract for IMU bandwidth/anti-alias as needed for planing (re-scoped — slam is < ~10 Hz, so aliasing is less severe than first thought). | judgment |
| B-4 | Decide denied-speed support: continuous vs long-leg-only vs **cap (which may be < 20 kn, §2.2)**. | **decision required** |
| B-5 | Cache-at-departure ephemeris freshness gate for ≥ 24 h (§4.2). | evidence-supported (U-P1 pending) |
| B-6 | Record that vertical `Γ`-mounting is kept and stronger at 2–4° trim (29–57×). | evidence-supported |

### 5.2 BOM deltas (`BOM.md`)

| # | Delta | Status |
|---|---|---|
| M-1a | **Isolate the oscillator** (low-`fn`, large stroke for < 10 Hz slam energy; align `Γ` to the best isolator axis) — new line, absent today. | judgment; hard per R5 |
| M-1b | **Keep the IMU rigidly mounted** (R5/VN-100 vendor guidance) or select a low-VRE IMU that needs no isolation — do **not** co-isolate with the reference. | evidence-supported (R5) |
| M-2 | Verify hull broadband **`a_rms` vs the sourced ~4.5 g RMS VN-100 saturation limit**; verify **VRE** and **gyro g-sensitivity** (VN-100 VRE unspecified). | evidence-supported (R5) |
| M-3 | Obtain/measure oscillator **`Γ` and `Γ₂`** (FE-5680A has no g-sens datasheet row; shaker-test the selected part). | evidence-supported (R5 absence) |
| M-4 | Non-magnetic absolute heading (solar/sky-polarisation) rises to strongly-indicated at speed. | judgment |

### 5.3 Safety-case deltas (`SAFETY_CASE.md`)

| # | Delta | Status |
|---|---|---|
| S-1 | Planing timer class **derived from an explicit response/distance budget** (§2.1), not a `1/v` rescale; the cap may be < 20 kn. | judgment |
| S-2 | Discharge D46's dwell order with the §2.4 revocation-independence analysis (dwells held, now justified). | evidence-supported (state-machine trace) |
| S-3 | Authority expected to drop during aggressive turns (§3.5) — annunciate. | judgment |
| S-4 | Frequency-reference row: the sourced concern is **linear-g phase-noise availability + cycle slips** (§1.4a); rectification (H8) is a *measure-it* hypothesis, not a proven bias. | evidence-supported (R5) |

### 5.4 Params deltas (`PARAMS_PROPOSAL.md`)

| # | Delta | Status |
|---|---|---|
| P-1 | Add a Class-B timer column whose `T_ack`/`t_dr` are budget-derived (≥ 3–5 s floor), not rescaled. | judgment |
| P-2 | Planing `Q` derived from the selected IMU's residual-acceleration spectrum on measured planing data (no asserted multiplier). | judgment |
| P-3 | Planing tracker block length + IMU heave-rate feed-forward as steering-relevant `[UNVERIFIED]` (screening bound, §3.2). | judgment |

### 5.5 Architecture / contracts (newly routed, per review finding 21)

| # | Item | Owner |
|---|---|---|
| A-1 | **Definition of which LEO observation resets `t_dr`** (position-constraining vs velocity-only) — governs the good-fix scenario (§4.3). | contracts |
| A-2 | Speed-class selector: source, uncertainty, hysteresis, fail-closed behaviour (§2.3). | architecture/contracts |
| A-3 | IMU heave-rate feed-forward interface (timing, covariance) for the tracker (§3.2). | architecture |
| A-4 | Isolated-frame dynamic extrinsics (reference `Γ`/lever-arm under isolation motion, §1.5). | architecture |
| A-5 | Clock transient/leakage monitor interface (§1.2 solution-domain leakage). | architecture |

### 5.6 High-speed effects the first draft omitted (review finding 20 / triage F15) [added]

R5 §6 and operational experience flag high-speed concerns beyond Doppler dynamics; each is a register
row or an explicit scope boundary, `[UNVERIFIED]`:

| # | Effect | Route |
|---|---|---|
| E-1 | **Antenna attitude / servo-rate limits / slam-synchronised L-band & Ku fades** (R5 §6.1) — motion-correlated observation outages directly interact with convergence, `t_dr` and the good-fix scenario. | BOM/tracker/safety |
| E-2 | **Spray / washdown / rain fade** on L/Ku at planing speed (R5 §6.2 — no primary trial; risk row). | BOM/baseline |
| E-3 | **Sea-surface multipath at planing speed** — tracker integrity, not only Doppler rate. | tracker |
| E-4 | **Power / EMI at planing throttle** on the reference and bladeRF (only mag deviation was noted before). | BOM |
| E-5 | **Crew human factors beyond `T_ack`** — R5's A₁/₁₀ "extremely uncomfortable"; ability to take the helm after a slam train degrades. | safety |
| E-6 | **Sea-state (Hs) coupling** — every slam number is Hs-conditional; class-rule `n_cg` scales with V *and* Hs (R5 §1.2). | analysis/baseline |
| E-7 | **Speed-log ventilation/cavitation on plane** — freshness/validity of speed-through-water at planing (affects current separation). | BOM/baseline |

### 5.7 Plainly: what 20 kn does NOT support, and the clearing evidence (finding 12 corrected)

The first draft's "all six clear through one measurement" is **false** — the hull slam/vibration/trim
measurement is *necessary* for several rows but *sufficient* for none; each needs its own
component/integration/human/replay evidence:

| Not supportable today | Necessary evidence (multiple, not one) |
|---|---|
| Reference under planing (availability) | hull PSD **and** measured oscillator `Γ`,`Γ₂` **and** isolator transfer function installed **and** correlator-convolution study |
| IMU adequacy | hull `a_rms` vs 4.5 g **and** VN VRE/gyro-g data **and** planing replay |
| Timers / speed cap | craft crash-stop/turn trial **and** validated human-response floor **and** an explicit budget |
| Continuous denied position at 20 kn | tighter heading-source characterisation **or** an operational long-leg/cap decision |
| Planing `Q` / velocity tuning | selected-IMU residual spectrum on measured planing data **and** NEES/NIS |
| Tracker under slam | finer ramp grids + production sequences + time-aligned 6-DOF replay |
| RF availability (E-1..E-3) | antenna-pattern/attitude/spray characterisation at speed |

**Until this evidence exists, fail closed to the displacement envelope; treat 20 kn as aided-only or
unproven** — consistent with `SAFETY_CASE.md §1`'s fail-closed gate.

---

## 6. Exploratory tier — 30 kn (per D47)

D47 adds **30 kn = 15.433 m/s** as an **EXPLORATORY** tier (not supported), scenario of record: a
**100 km denied passage entered from a good GPS fix then loss, up to 30 kn** (~1.8 h). All §6 numbers
inherit §0's caveat and are *weaker* than the 20 kn ones — R5 gives no small-craft 30 kn dataset, and
the slam-scaling exponent is unsourced.

### 6.1 Derivation chains at 30 kn (corrected)

| Quantity | 20 kn | 30 kn |
|---|---:|---:|
| Vessel Doppler at Ku | 389 Hz | **583 Hz** (R5 §6.3's "1.5 Hz" is an arithmetic slip, §0.1) |
| Distance per second | 10.3 m | **15.4 m** |

**Trim / mounting:** at R5's 2–4° the `cot(θ/2)` benefit (29–57×) is undiminished; mounting is not the
30 kn problem.

**Slam scaling [SOFTENED to a stated hypothesis].** Impact acceleration is often modelled `v^1.5`–`v^2`
for fixed sea state (**judgment, not in R5**; finding 18). Anchoring on R5's sourced 20 kn-class peak
(8.62 g) scaled by 1.84–2.25× → **~16–19 g peak** at 30 kn — a hypothesis requiring hull measurement,
`[UNVERIFIED]`.

**Vibration rectification [SOFTENED — the first draft's "3–5× over PL" is withdrawn].** With R5's
*sourced* RMS 0.44 g scaled by the same (unsourced) factor → `a_rms ≈ 0.8–1.0 g`; with the hypothetical
`Γ₂ = 1e-11/g²`: `Δv_DC ≈ 0.002–0.003 m/s` — still **~10× under** the denied PL. The first draft's
0.09–0.14 m/s (3.3–4.9× over) came from mis-using 3 g as RMS *and* squaring an unsourced peak-scaling
exponent (compounding two unsupported steps, finding 18). **On sourced RMS, 30 kn rectification is not at
the PL either.** The 30 kn reference concern is the same as 20 kn — *linear-g phase noise / cycle slips
and harder isolation* — scaled up, not a proven integrity breach.

**Isolation dB [corrected].** Because bias ∝ `⟨a²⟩`, reducing it by factor `R` needs acceleration
transmissibility `1/√R`, i.e. **amplitude attenuation `10·log₁₀(R)`** — for `R = 3.3–4.9` that is
**~5–7 dB**, not the first draft's 10–15 dB (finding 5 / triage F4). (And since rectification is no
longer shown to be over the PL, even this is a contingency figure.)

**Heave-Doppler stacking [SOFTENED to a screening bound].** With R5-sourced peaks scaled to ~16–19 g,
heave rate ≈ 5900–7200 Hz/s; + 3718 satellite ≈ **9600–10900 Hz/s**, which **exceeds the largest tested
grid point (8000 Hz/s at block-128)**. Correct statement: *"the conservative screening bound exceeds any
tested all-detect configuration, so severe-slam loss of lock at 30 kn is a real risk requiring IMU
heave-rate aiding and replay validation"* — **not** "no block survives / structural / unavoidable" (the
study is explicitly not a closed-form cap; finding 6). At 30 kn IMU heave-rate feed-forward moves from
recommended to likely-necessary.

**Collision / cap [HARDENED to honest indeterminacy].** Scaled distance-preserving `T_ack ≈ 2.3 s` is
**below any credible 3–5 s human floor** → **do not grant 30 kn denied autonomous authority.** But the
cap is **not located at 20–30 kn**: on a 5 s floor it is ~14 kn; on 3 s, ~23 kn (§2.2). The supported
claim is "no 30 kn denied authority on current evidence," not "Class-B/20 kn territory is proven safe."

**Heading-vs-authority-PL time.** denied 100 m / 2.5°: **149 s (2.5 min)** at 30 kn (vs 223 s at 20 kn)
— the same geometry tightened 1.5×.

**Convergence / good-fix.** A 10–20 min leg at 30 kn = **9.3–18.5 km = 9–19 % of the 100 km passage**
(vs 6–12 % at 20 kn). The good-fix start removes cold convergence and gives aided-grade start accuracy —
but per §4.3 authority still expires at `t_dr`, not at the 2.5 min heading time.

### 6.2 The 100-km-after-good-fix scenario, end to end

Passage: **100 km @ 30 kn = 1.80 h**; @ 20 kn = 2.70 h. Phase timeline (open-water; **judgment**, classes
`[UNVERIFIED]` pending replay):

| Phase | Trigger | Position class @ 20 kn | Position class @ 30 kn | Authority-limiting factor |
|---|---|---|---|---|
| P0 good fix | pre-loss | aided ~5–25 m | aided ~5–25 m | — |
| P1 DR bridge | fix loss → first re-acq | starts ~5–25 m, decays | decays 1.5× faster | **`t_dr` (§4.3), not heading time** |
| P2 convergence leg | straight leg | re-bounds to denied ~100 m | same, if track survives slams (§6.1) | needs 12–18 km straight |
| P3 cruise | steady legs | denied ~100 m on legs | denied; more of passage converging | manoeuvre resets |

**Position class is the same at 20 and 30 kn** (aided at loss → denied cruise). 30 kn changes *timing and
feasibility*: 1.5× faster decay, 19 % vs 12 % of the passage per convergence leg, PL reached 1.5× sooner
after resets, and (judgment) harder tracker slam survival and reference availability.

**Honest verdict on class vs margins.** For the **estimator/passage** layer, 30 kn **tightens margins**
(same classes, 1.5×), materially helped by the good-fix start. For the **reference/tracker** layer it is
**not the class change the first draft claimed** — on sourced RMS the rectification stays under the PL at
both speeds; the real, sourced degradation (linear-g availability, harder isolation, slam loss-of-lock
risk) worsens with speed but is an **availability** problem, not a proven integrity breach. For the
**safety** layer, 30 kn is over the human-response floor → **denied cap**, but the cap is not shown to sit
specifically at 20–30 kn.

### 6.3 30 kn consequence-register additions (beyond §5)

| # | 30 kn delta | Status |
|---|---|---|
| H-1 | Isolation must attenuate ~**5–7 dB** more than the 20 kn contingency (corrected from 10–15 dB) — and only if measurement shows rectification approaches the PL. | judgment (contingent) |
| H-2 | Oscillator `Γ₂` **measured** matters more as `a_rms` rises, but even scaled it is ~10× under PL on sourced RMS — measure to confirm, don't assume breach. | evidence-supported (R5 absence) |
| H-3 | IMU heave-rate tracker aiding moves from recommended to **likely necessary** (screening bound exceeds tested envelope). | judgment |
| H-4 | Hull `a_rms` vs the **4.5 g RMS VN saturation** limit is the binding IMU question at 30 kn (scaled 0.8–1 g RMS is still under it, but bow/severe seas unknown). | evidence-supported (R5) |
| H-5 | **Denied-speed cap** (may be < 20 kn); no third autonomous-denied class. | judgment |
| H-6 | Exploratory-tier trial gate: no 30 kn denied trial until 20 kn is cleared and H-1..H-5 + §5.6 are evidenced. | judgment |

The good-fix scenario adds **no** new requirement — it is *more* forgiving than a cold start and is the
right framing for any 30 kn work.

---

## Verdict — 20 kn

**20 kn denied operation is NOT supportable on present evidence — but the reason is narrower and better-
sourced than the first draft claimed.** With R5's sourced values the frequency-reference case
**softens**: vertical `Γ`-mounting is kept and stronger at 2–4° trim (29–57×), and the vibration-
rectification "integrity breach / mandatory isolation" conclusion is **withdrawn** — at R5's measured
0.44 g RMS the rectified bias is ~48× under the denied PL, and its magnitude rested on an unsourced `Γ₂`
and a peak-as-RMS error. The real, sourced reference concern is the **linear g-sensitivity** (~1e-9/g)
driving per-slam velocity transients (0.1–0.8 m/s, innovation-gated) and close-in phase noise → a
**tracker-availability / cycle-slip** problem, for which shock isolation is **recommended but hard**
(R5's 100–450 ms slams put energy below ~10 Hz, needing low-`fn`/large-stroke mounts, and vendor guidance
prefers a *rigid* IMU — so isolate the oscillator, not a shared plate). What genuinely blocks 20 kn
denied today: (1) an **unmeasured hull** slam/vibration/trim/manoeuvre environment (R5 has no small-craft
20–30 kn dataset); (2) an **IMU** whose broadband `a_rms` vs the sourced ~4.5 g RMS saturation limit and
whose VRE/gyro-g are unknown; (3) **collision timers** that a distance-preserving rescale would push to
~3.5 s at 20 kn — already at/below R5's 3–5 s human floor — so the safe-speed cap must be derived from an
explicit budget and **may be below 20 kn**; (4) **continuous denied position authority** that heading
error breaches within ~3.7 min of every manoeuvre reset; and (5) a **planing `Q`/velocity retune** that
cannot inherit the near-truth-IMU synthetic result. None of these clears through a single measurement —
each needs its own component, integration, human-factors and replay evidence (§5.7), plus the newly
added RF-availability, spray, power, crew and sea-state rows (§5.6). Until that evidence exists, fail
closed to the displacement envelope and treat 20 kn as aided-only or unproven.

## Verdict — 30 kn exploratory tier

**Do not grant 30 kn denied autonomous authority on current evidence; 30 kn is aided/manual-only and
exploratory.** Corrected against R5, 30 kn does **not** change the estimator/passage conclusion *class*
(aided-at-loss → denied cruise, margins 1.5× tighter, helped by the good-fix start which removes cold
convergence), and — contrary to the first draft — it is **not** a hardware *class change* either: on
sourced RMS the vibration-rectification bias stays ~10× under the PL at 30 kn as at 20 kn, the required
extra isolation is ~5–7 dB (not 10–15 dB), and the tracker "no block survives" claim is withdrawn as the
study is explicitly not a closed-form cap — the honest 30 kn tracker statement is that a *conservative
screening bound* (~9600–10900 Hz/s) exceeds any *tested* configuration, so severe-slam loss of lock is a
real risk needing IMU heave-rate aiding and replay, not a structural certainty. The one firmly supported
30 kn conclusion is on the **human-response floor**: a distance-preserving `T_ack ≈ 2.3 s` is below R5's
3–5 s floor, so denied autonomous authority at 30 kn is not defensible — but the analysis **does not
locate the cap at 20–30 kn**; on a 5 s floor it could be ~14 kn. 30 kn remains scoping analysis, gated
behind full clearance of the 20 kn envelope plus a measured slam spectrum and requirements H-1..H-6.
