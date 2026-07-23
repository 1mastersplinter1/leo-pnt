# Coherent DF Array on Unknown Terrestrial Emitters — Feasibility Analysis

Status: **subordinate to [`DESIGN_BASELINE.md`](DESIGN_BASELINE.md) (normative)**, and informed by
[`HIGH_SPEED_ENVELOPE.md`](HIGH_SPEED_ENVELOPE.md), [`ARCHITECTURE.md`](ARCHITECTURE.md) and
[`SAFETY_CASE.md`](SAFETY_CASE.md).
Unit: U-X1 (Opus seat, deep engineering analysis) · 2026-07-23 · requirement `DECISIONS.md` **D48**
Owns: this file and `.orchestration/reports/U-X1.md` only. No code, no normative authority.

`DESIGN_BASELINE.md` is the single normative design document. This analysis is subordinate to it. It
**adds no requirement and grants no authority**: it evaluates a proposed 4–5 element coherent
direction-finding (DF) array as a navigation aid and emits one verdict plus routing candidates. Its
input research `docs/research/R6-unknown-emitter-array.md` was produced on Grok; per **D5** its numbers
are treated as **[UNVERIFIED-grok]** and every load-bearing figure below is either re-derived here or
flagged. Values this document introduces are scoped **evidence-supported** (a repo artifact, a standard
identity, or a calculation shown inline) or **judgment** (`[UNVERIFIED]`).

---

## 0. The question, and the honest headline

D48 asks whether a 4–5 element coherent array doing bearings-only DF on **unknown land-based emitters**
fixes the two failures the high-speed studies exposed:

- **(a) Position** — D51/D52: on the single-satellite synthetic fixture the real EKF is bounded but
  only to **tens of km**, because single-satellite range-rate geometry is near-unobservable for
  position; the 100–200 m denied class needs multi-satellite geometry over 10–20 min legs.
- **(b) Heading at speed** — D50/U-H1: after each manoeuvre the heading solution drifts, and a **5°
  error breaches the 200 m position PL in ~3.7 min at 20 kn** (heading-rotated speed-log cross-track).

**Headline verdict (expanded in §5).** The array's *unknown-emitter* framing — the thing D48 literally
names — is the **weakest** version of the idea. It is *observable* — a fixed emitter's range **is**
recoverable from a metrically-known track because the **speed log supplies the scale anchor** (§1.3), and
a manoeuvre strengthens it further — but the information is **weak, slow, and geometry-dependent**, so it
does not deliver a *fast* absolute fix of either problem, and it is weakest at exactly the high-speed,
post-manoeuvre moment U-H1 flagged. The **strong** version is a different problem — **known-beacon DF
against a surveyed coastal-emitter database** — which *can* give a fast, **genuinely absolute** heading
(and, with ≥3 beacons, a position) resection in seconds; that heading measurement is **not** capped by
the existing heading solution (§1.2, §2.2). But it needs a beacon database, needs field DF accuracy of a
few degrees (not guaranteed on a planing hull), and its position output is only **hundreds of metres** at
coastal ranges. The verdict below is therefore a **risk / evidence / prioritisation** judgment — field DF
accuracy is unverified, multipath/association/integration are hard, coastal-range position exceeds the PL,
and a lower-cost hardware-free alternative already exists — **not** a claim of observability impossibility.
Against the **already-routed multi-satellite LEO fixture (D52, U-MS1)** — pure software, no new hardware,
directly targets problem (a) — the array is a **research spur, not a near-term aid**. Gate any hardware
behind a cheap synthetic proof-of-concept.

---

## 1. The observability case

### 1.1 What a bearing measurement actually is

A DF array measures the **angle of arrival (AoA) in the array/body frame**, `α` — the angle of the
emitter off the bow. To use it for navigation you relate it to geographic quantities through

```
β  = atan2(E_emitter − E_own, N_emitter − N_own)     (geographic bearing to emitter)
α  = β − ψ                                            (body AoA = geo bearing − heading ψ)
```

Three quantities are entangled in one scalar measurement: **own-position** `(E_own,N_own)`, **heading**
`ψ`, and the **emitter position** `(E_emitter,N_emitter)`. Which of them a bearing observes depends
entirely on which of the others you already know. This coupling is the whole story.

### 1.2 Known emitter — the strong case (a resection, seconds)

If the emitter location is **known** (a surveyed tower / AIS base station / DAB site — public data):

- **Heading from one known beacon, given own-position.** With own-position from the LEO solution, `β` is
  computable, so `ψ = β − α` is a **direct scalar heading observation** each epoch. Its error is the AoA
  error *one-for-one* (`σ_ψ = σ_α`, plus a small `σ_pos/R` term). This is a heading source **immune to
  magnetic deviation and local anomaly** — the one axis LEO Doppler observes weakly and which the
  baseline currently trusts to magnetometers. **Crucially, this is a genuine *absolute* heading
  measurement and is NOT circularly capped by the existing heading solution:** the only inputs are the
  measured body-AoA `α` and the computed geographic bearing `β` (from known own-position + known emitter
  position); the prior heading estimate enters `ψ = β − α` **nowhere**. (A coarse prior heading is needed
  only to *associate* the AoA peak with the right catalogued beacon — a data-association step, not a
  metric cap on the resulting measurement.) This is the fundamental difference from the unknown-emitter
  case in §1.3/§2.2, where the geographic interpretation of a bearing **does** depend on heading. **This
  is the single most attractive property of the whole proposal**, and it is a *known-beacon* property,
  not an unknown-emitter one.
- **Position + heading from ≥3 known beacons (standalone resection).** Three body-AoAs give three
  equations in three unknowns `(E_own, N_own, ψ)` → an instantaneous fix of **both** position and
  heading, in seconds, with no LEO and no magnetometer. This is the classical three-bearing radio fix
  generalised to also yield attitude. Time-to-fix is seconds, not the 10–20 min LOS-evolution leg LEO
  needs — so it *would* directly attack D51's slow-position problem **if** a beacon database is allowed.
- **Accuracy ceiling** (§2): position error ≈ `R · σ_α / GDOP` — **hundreds of metres** at realistic
  10–30 km coastal ranges even at 1–3° DF; heading error = `σ_α` ≈ few degrees.

So the known-beacon case is genuinely useful — a fast, magnetically-independent heading/position
resection — but it is bounded to few-degree heading and hundreds-of-metre position, and it presupposes a
surveyed emitter database. **The brief's "unknown emitter" is a strictly weaker regime; the rest of §1
is about how much weaker.**

### 1.3 Unknown emitter — bearings-only SLAM (the D48 case, much weaker)

With emitter locations **unknown**, you are jointly estimating own-trajectory **and** the emitter map
from bearings only. This is bearings-only SLAM, adjacent to the classical Nardone–Aidala bearings-only
target-motion-analysis (BO-TMA) problem. It is **observable** given the sensors already aboard; the
issue is *how weak and slow* the information is, not impossibility. Four facts govern it:

1. **A fixed emitter's range IS observable on a straight, metrically-known track — weakly.** This is the
   correction to my first draft (and to a loose reading of R6 §3.1). The Nardone–Aidala "manoeuvre
   required" result is for a *moving* target whose own velocity is unknown, or when the observer's own
   scale is unknown. Here the emitter is **fixed** and own-velocity is **known from the speed log**, so a
   straight constant-velocity leg triangulates range as the bearing sweeps (best near beam passage, poor
   for near-radial or distant emitters). So range is **observable, not unobservable** — but the
   information is weak, concentrated near closest-point-of-approach, and slow to accumulate. A manoeuvre
   *strengthens* it (adds cross-range baseline and breaks near-radial degeneracy); it is an enhancer,
   not a precondition.
2. **The scale anchor is already aboard.** Metric scale comes from **known baseline length** — the
   **speed log + heading** (distance travelled) or the **LEO velocity** solution. This is a *strength*,
   not a blocker: the vessel already carries the speed log, so unknown-emitter DF is not scale-starved.
   The honest framing is that it is a **fusion partner**, not a standalone navigator — it uses the
   sensors it is meant to help, which caps its *independence*, not its observability.
3. **Absolute orientation of the map resolves through motion, not instantly.** Bearings give *relative*
   shape at a single epoch (rotating heading + all emitter azimuths together leaves every bearing
   unchanged), so absolute heading is not recoverable from one snapshot of un-converged emitters; it
   becomes observable as the track evolves and the map de-rotates against the known velocity direction.
   Concretely: **a bearing to an un-converged emitter carries little absolute heading information until
   the emitter is localised.** The clean, instantaneous heading win of §1.2 is a *known-beacon*
   property; the unknown case earns heading only slowly, after convergence.
4. **Convergence is slow.** Diverse geometry (course changes and/or beam passages, not just distance) at
   7–30 kn for kilometre-scale ranges over minutes (R6 §3.2, judgment; consistent with BO Fisher-
   information scaling `∝ 1/σ_θ²` and baseline/range ratio). During that window the map is poorly known
   and the bearings contribute little absolute-position information.

**Does unknown-emitter DF fix problem (a), position?** *Yes in principle, but only slowly.* The geometry
is observable (§1.3.1–2), and once emitters converge to landmarks their bearings give continuous
cross-range position without a fresh 10–20 min LEO leg — a genuinely complementary second geometry
channel. The limitation is timing, not observability: convergence takes minutes of geometry and the
absolute-position value arrives *through* the speed-log/LEO scale anchor. It is a real aid but **not** a
fast absolute fix.

**Does it fix problem (b), heading at speed?** *Weakly, and with the wrong timing.* Absolute heading from
un-converged emitters is slow to earn (§1.3.3) and strengthens with the same manoeuvres that *reset* LEO
convergence (D50) and are limited at planing speed. So the unknown-emitter mode offers the least heading
help precisely at the post-manoeuvre, high-speed moment U-H1 identified as the breach. The **fast,
absolute** heading reset the heading problem actually needs is available in the **known-beacon mode** of
§1.2 — which is a genuine, non-circular absolute-heading measurement.

**Net reading.** Unknown-emitter bearings-only SLAM is an *observable but slow-burn, fusion-dependent*
aid that adds an independent geometry channel but neither a fast position fix nor a fast heading reset.
The useful, fast capability is the known-beacon resection — a different, database-dependent proposition,
and the one worth studying.

---

## 2. DF accuracy reality check (re-derived, not taken from R6)

### 2.1 The instrumental (thermal) floor — I confirm R6's 1–2° ideal

For a two-element interferometer of baseline `b`, the measured phase is `φ = (2π b/λ) sinθ`, phase-noise
`σ_φ ≈ 1/√(2·SNR·N_snap)` (rad), so

```
σ_θ  ≈  1 / [ √(2·SNR·N_snap) · (2π · b/λ) · cosθ ]      (rad, high-SNR CRB, boresight-ish)
```

A filled N-element array roughly adds a `√N` effective-SNR factor and a geometry constant. Plugging the
candidate bands for a **~1.5 m** mast array at **20 dB SNR (=100), single snapshot**:

| Band | f | λ | D/λ | phase slope 2π·D/λ | σ_θ (thermal floor) |
|---|---|---|---|---|---|
| FM | 100 MHz | 3.0 m | 0.50 | 3.14 | ~1.3° |
| DAB III | 200 MHz | 1.5 m | 1.0 | 6.28 | **~0.65°** |
| Cellular low | 800 MHz | 0.375 m | 4.0 | 25.1 | ~0.16° (but see ambiguity) |

So R6's "instrumental CI/MUSIC ~1–2° RMS ideal" is **confirmed by first-principles CRB** at DAB with a
1λ aperture — this is a **thermal-noise-only, perfectly-calibrated-manifold** floor and it improves with
snapshots and SNR. R6 is internally consistent here.

### 2.2 Three reasons the floor is not the field number

1. **Multipath bias is not in the CRB.** Over water the two-ray (direct + specular sea-surface)
   channel distorts the array manifold and adds an AoA **bias** the CRB never sees; SFN bands
   (DAB/DVB-T2, multiple transmitters on one frequency) can make MUSIC lock to a **composite virtual
   direction**. R6's "field marine 3–15°+", and its 8–20° FM / 10–30° planing budgets, are the honest
   numbers and I adopt them as **judgment**. This is the dominant error term, not thermal noise.
2. **The high-aperture bands buy ambiguity, not just accuracy.** R6's aperture table calls cellular
   "excellent aperture" without flagging that a fixed 5-whip UCA of 1.5 m diameter has adjacent-element
   spacing ≈ `1.5·sin36° ≈ 0.88 m` = **2.35 λ at 800 MHz** — well past λ/2, so the manifold is
   spatially aliased and correlative interferometry must resolve grating-lobe ambiguities from a known
   manifold. High `D/λ` sharpens the peak *and* multiplies the ambiguities; robustness drops. **R6
   under-weights this** — flagged in the report.
3. **Body-frame → geographic needs attitude — but this cost applies to the UNKNOWN case only.** The AoA
   is measured in the hull frame. When a bearing is used *as a geographic bearing to constrain
   own-position* (the unknown-emitter / SLAM interpretation, and any position resection), the geographic
   bearing carries the **heading/attitude error directly** (`σ_geo² = σ_α² + σ_ψ²`): a 5° attitude error
   injects 5° of bearing error, so in that mode the position-constraint quality is capped by the attitude
   solution. **This does NOT apply to the known-beacon *heading* measurement of §1.2.** There the flow is
   reversed — `ψ = β − α` uses the *computed* geographic bearing `β` and the *measured* body-AoA `α` to
   **produce** an absolute heading; the prior attitude estimate does not enter, so there is no circular
   cap (only a coarse prior is needed to associate the peak with the catalogued beacon). So: the
   attitude coupling is a real limit on **DF-for-position**, and is **not** a limit on **known-beacon
   DF-for-heading** — the two must be kept separate (see §3).

### 2.3 The load-bearing number: bearing error → position metres

Cross-range position error from one bearing at range `R` is `σ_x ≈ R · σ_θ`:

| σ_θ | at 5 km | at 10 km | at 20 km | at 30 km |
|---|---|---|---|---|
| 1° (ideal, calm) | 87 m | 175 m | 349 m | 524 m |
| 3° (good field) | 262 m | 524 m | 1047 m | 1571 m |
| 5° (typical field) | 436 m | 873 m | 1745 m | 2618 m |
| 10° (planing/multipath) | 873 m | 1745 m | 3491 m | 5236 m |

A **single** bearing gives cross-range only (nothing down-range); a position **fix** intersects ≥2
bearings and the along-baseline error inflates by `1/sin(Δaz)` (GDOP). Read against the **200 m denied
position PL**: even the *optimistic* 1° at 10 km (175 m) barely clears it before GDOP inflation, and the
*realistic* 3–5° field accuracy at coastal ranges gives **500 m–1.7 km** — i.e. a bearing fix on this
hull is a **coarse** position aid, not an operational-grade one. R6's cited community "tens of metres"
geolocation is **multi-point mobile triangulation over a long track on strong land signals**, not a
single-epoch marine fix; R6 tags it anecdotal and I **reject** its use as an achievable navigation
accuracy.

### 2.4 Multipath over water — verdict

Open sea removes urban clutter but keeps **specular sea-surface reflection, coastal ducting, and
superstructure shadowing**; wave motion makes the multipath **time-varying** at exactly the encounter
frequencies that matter at speed. Bearings must be **quality-weighted** (MUSIC peak sharpness,
eigenvalue spread, residual) before any EKF update, and outlier-rejected hard. This matches R6 §2.2 and
the baseline's existing "sea-surface multipath biases a tracker → correlation-quality-driven rejection"
degradation row.

---

## 3. Hardware reality

### 3.1 It is separate hardware from the bladeRF

A 5-channel **coherent** front end is **not** the 2-channel bladeRF the baseline already carries. R6's
pick is **KrakenSDR** (RTL2832U-class, 5 coherent RX on one LO with switched noise-source auto-cal,
24–1766 MHz, ~2.56 MHz IBW, ~USD 749 + ~USD 199 antenna kit). Key hardware facts (R6 `[V]`, plausible,
not independently re-priced here):

- **Channel count is the point.** The bladeRF's 2 coherent RX give an *interferometer* baseline (one
  ambiguous phase), not an N=4–5 MUSIC/CI array. There is no cheap way to make one bladeRF an N=5 array;
  multi-bladeRF phase coherence needs external LO/1-PPS distribution and is a known-hard integration.
  So the array is **additive hardware**, roughly **+USD 950** plus a fabricated/mag-mount 5-element mast
  array plus a dedicated SBC (Pi-4/5) running the `krakensdr_doa` DSP.
- **It does not naturally share the FE-5680A reference.** KrakenSDR distributes its *own* onboard clock
  to all five tuners; feeding the baseline's 10 MHz rubidium in would be a hardware modification, not a
  supported input. So it is a **parallel RF chain with its own clock discipline**, decoupled from the
  coherent bladeRF/Ku/L chain — new USB, new SBC, new processing pipeline.
- **Band coverage** tops out at 1.766 GHz (misses mid-band 5G) and its 2.56 MHz IBW cannot span an 8 MHz
  DVB-T2 channel — fine for DF on a *slice* of DAB/FM/cellular, but not a wideband capability.

### 3.2 The array at planing speed inherits every U-H1 concern, amplified

- **Manifold vs mast flex/vibration.** DF assumes a **fixed, calibrated element geometry**. Planing
  vibration and mast flex move the element phase centres → the calibrated manifold is wrong → bearing
  wander (R6's 10–30°/outlier regime). This is the U-H1 mechanical-stability concern applied to five
  antennas instead of one oscillator.
- **The attitude coupling (from §2.2.3) bites the position mode at speed — but not the known-beacon
  heading mode.** When a bearing is used to constrain *position* (unknown-emitter SLAM, or any
  geographic-bearing fix), it needs a good high-rate attitude solution to turn body AoA into a geographic
  bearing; but heading/attitude is the *weak* axis at speed (D50), so at the post-manoeuvre moment U-H1
  flags, attitude error is largest **and** the bearing-to-geographic conversion is worst — the two
  failures correlate. **The known-beacon *heading* measurement (§1.2) is exempt** — it produces heading
  rather than consuming it — which is exactly why that mode, not the position mode, is the capability
  worth studying.
- **Spray, superstructure shadowing, EMI** at planing degrade SNR and add manifold error exactly when
  the geometry rate (R6's claimed benefit of speed) is highest. R6's "net positive for observability if
  phase stability is controlled" carries a large *iff* that the same U-H1 evidence says is unmeasured.

Net hardware verdict: **feasible and cheap to buy, non-trivial to integrate, and at its worst in the
exact regime it is meant to help.**

---

## 4. Integration path into the existing EKF

### 4.1 Measurement model and Jacobian

A bearing update is a scalar innovation on `z = atan2(E_e − E_own, N_e − N_own) − ψ + b_array`, with `R`
adaptive from DF quality metrics. Its Jacobian couples **own-position** (states `POS`, indices 0–2),
**heading** (`HEADING`, index 6), and — in the unknown case — the **augmented emitter-location states**.
This is a well-behaved EKF update *once the states exist and are observable*; the shape mirrors the
existing scalar `update_heading` / `update_doppler` paths in `pnt-estimator`.

### 4.2 The per-SV-nuisance machinery is the precedent — but the lifecycle is inverted

`pnt-estimator` already grows and shrinks the state vector at runtime: `augment_state` /
`remove_state` resize `x` and the covariance, and `augment_satellite_bias` / `retire_satellite_bias`
manage per-satellite-per-pass scalar bias slots with index-shifting on retirement (D22/U-F1). That is
exactly the *mechanism* emitter states would reuse. But three semantic differences make it more than a
copy:

1. **2 states, not a scalar, with a pathological initial covariance.** Each emitter is `(E,N)` — two
   correlated states — and at first sighting the **range is near-unobservable**, so the honest initial
   covariance is a hugely elongated ellipse along the line of sight. EKF linearisation handles that
   badly (the classic BO-SLAM initialisation failure); a robust build needs an **inverse-range /
   bearing-range parametrisation or a delayed-initialisation / pseudo-linear front end**, none of which
   the scalar-bias precedent provides. This is a real implementation risk, not a wiring exercise.
2. **Persist-and-associate, not create-and-retire.** Per-SV biases are *retired at pass end*; emitter
   landmarks must **persist across the whole passage** (that is their entire value as a map) and be
   **re-associated** epoch-to-epoch by frequency + track ID. **Data association** — which bearing
   belongs to which emitter, with SFN and mobile-AIS confusers — is a **new subsystem with no precedent
   in-tree**.
3. **Observability-gated augmentation.** The baseline explicitly *forbids decorative unobservable
   states* and requires a measurement path per state. A fresh emitter's range has a measurement path
   (bearings) but is only *conditionally and weakly* observable — recoverable on a straight,
   metrically-known track via the speed-log scale anchor (§1.3), with a manoeuvre strengthening the
   geometry rather than being required. So augmentation must be
   **gated on realised observability** (e.g. accumulated bearing spread / effective range
   information), or the
   filter carries states it cannot yet constrain — precisely what the baseline discipline warns
   against.

### 4.3 Bus and message types — yes, new ones

The measurement bus and adapter pattern accommodate this cleanly but need additions:
a **`BearingObservation`** envelope (body-frame AoA, quality metrics: MUSIC peak sharpness, eigenvalue
spread, SNR, association key = frequency/track ID, and the **array phase-centre calibration ID + the
attitude epoch** used, per the baseline's extrinsics rule); a new **DF sensor adapter** wrapping the
Kraken DoA DSP (a signal tracker in module-5 terms); and journal records for it. This is consistent with
the existing typed-bus architecture — bounded new work, not an architectural break. The fusion executive
gains a bearing-update dispatch and an emitter-state manager; the authority supervisor needs a policy for
how much a coarse bearing fix may be trusted for steering (likely: aiding-only, never sole authority).

---

## 5. Verdict, routing, and the direct comparison

### 5.1 Tier: research spur, not a near-term aid — a risk/evidence/prioritisation call

To be explicit about the basis: the array is *not* ruled out on an observability impossibility — the
unknown-emitter geometry **is** observable given the speed-log scale anchor (§1.3). It is deprioritised on
four concrete, honest grounds:

- **Evidence gap.** Field DF accuracy on a vibrating, spray-exposed planing mast is **unverified** — the
  whole value hinges on whether σ_θ is a few degrees (useful) or 10°+ (not), and no marine at-speed
  measurement exists (§2.2, §3.2).
- **Implementation risk.** Multipath/SFN bias, data association, and the BO-SLAM state-initialisation
  hazard (§4.2) are genuinely hard, none with an in-tree precedent.
- **Accuracy ceiling.** Even in the favourable known-beacon mode, coastal-range position is
  **hundreds of metres to >1 km** — above the 200 m denied PL (§2.3).
- **A cheaper alternative already exists.** The multi-satellite LEO fixture (§5.2) targets the
  *demonstrated* position gap with no new hardware.

The one genuinely attractive capability — a **magnetically-independent absolute heading (and ≥3-beacon
position) resection** — is a **known-beacon** capability (non-circular, §1.2) that still needs a surveyed
emitter database and few-degree field DF. It is worth *studying*, but it is not on the critical path and
should not gate hardware money before a synthetic study says it clears the gates.

### 5.2 Direct comparison with the multi-satellite LEO fixture (D52 / U-MS1)

| Axis | Multi-satellite LEO fixture (U-MS1, already REQUIRED) | Coherent DF array |
|---|---|---|
| Targets | Problem (a) position — the *demonstrated* D51/D52 gap | (a) slowly + weakly; (b) only via known-beacon mode |
| Hardware | **None** — uses Starlink/Iridium/Orbcomm already in baseline | **+~USD 950** + mast array + SBC + DSP stack |
| Software | Add satellites to the existing EKF fixture | New states (BO-SLAM init risk), data association, bus messages, DF adapter |
| Fast absolute fix? | Yes — multi-sat geometry is the literature basis for 100–200 m | No fast fix (unknown); fast only with beacon DB (known) |
| Heading help | Indirect | Only in known-beacon mode, capped at DF accuracy (few °) |
| High-speed behaviour | Neutral (geometry, not mechanics) | Worst exactly post-manoeuvre at planing |
| Risk / cost | Low; engineering days | Higher; weeks + on-water DF calibration + unproven field DF |
| Status | **Routed REQUIRED (D52)** | This analysis |

The comparison is lopsided: **U-MS1 is strictly the higher-priority, lower-risk path** and it addresses
the same position-unobservability problem with hardware already in the baseline. **Do U-MS1 first.** The
array does not substitute for it and should not compete for its slot.

### 5.3 Minimal synthetic proof-of-concept (route as a candidate unit)

Before any purchase, the array question is settlable **in pure simulation** on the existing
`pnt-estimator`. Proposed candidate unit **U-EA1** (Opus/Sol, **LOW** priority, sequenced **after
U-MS1**):

- **Add** a `BearingUpdate` path to `pnt-estimator` (Jacobian coupling `POS`, `HEADING`, and optional
  augmented emitter `(E,N)` states via the existing `augment_state` machinery), with finite-difference
  Jacobian checks per the baseline verification rule.
- **Fixture**: a Danish-strait emitter layout (a handful of towers at 5–30 km), a 20 kn track with an
  explicit manoeuvre schedule, synthetic bearings injected at **σ_θ ∈ {1, 3, 5, 10}°** *plus* a separate
  **body-frame attitude error** injection (to expose the §2.2.3 circularity).
- **Two modes**: known-emitter (states fixed) and unknown-emitter (observability-gated augmentation +
  data association stub).
- **Metrics**: heading error vs the **2°/5°** gates; cross-track position vs **200 m**; unknown-emitter
  time-to-converge and its sensitivity to the manoeuvre schedule and to σ_θ; degradation under attitude
  error. **Kill criterion**: if the *known-beacon* mode cannot clear the 2° heading / 200 m gates under
  realistic 3–5° DF + attitude error over representative geometry, the *unknown* mode certainly cannot,
  and hardware is not justified.

This study is hardware-free, reuses the shipped EKF, and would settle D48 for a few engineering days
against a ~USD 950 + weeks-of-integration commitment.

### 5.4 Consequence routing (candidates only — this document edits nothing)

- **PLAN**: candidate unit **U-EA1** (synthetic bearings-only PoC), LOW, after U-MS1.
- **BOM**: no purchase now; KrakenSDR-class array is a *conditional* line item gated on U-EA1 passing its
  kill criterion. `[UNVERIFIED]`.
- **ARCHITECTURE/CONTRACTS**: if U-EA1 passes — new `BearingObservation` bus message + DF sensor adapter
  + emitter-state manager + authority policy for coarse bearing fixes (aiding-only).
- **DESIGN_BASELINE**: unchanged; interferometric/array use remains explicitly an "open research option,
  not baseline functionality" (baseline §"Vessel equipment"), which this analysis is consistent with.

---

## 6. Verdict (one paragraph)

A 4–5 element coherent DF array is cheap to buy and cleanly, if non-trivially, integrable into the
existing typed-bus EKF via its per-SV state-augmentation precedent, and the capability D48 names —
bearings-only SLAM on unknown terrestrial emitters — is **observable**, not impossible: a fixed emitter's
range is recoverable from a metrically-known track because the speed log supplies the scale anchor, and a
manoeuvre strengthens it. The problem is that this information is **weak, slow, and geometry-dependent**,
so it delivers neither a *fast* absolute position fix nor a *fast* heading reset, and it is weakest at
exactly the post-manoeuvre planing moment U-H1 identified. The one genuinely valuable, *fast* property is
a **known-beacon** absolute **heading (and ≥3-beacon position) resection** — a genuine absolute-heading
measurement, **not** circularly capped by the existing heading solution — but it needs a surveyed emitter
database, needs field DF of a few degrees that a vibrating, spray-blinded planing mast is unverified to
deliver (thermal floor ~1° at DAB, but field 3–15°+ with a one-for-one attitude penalty when a bearing
constrains *position*), and yields only **hundreds of metres** of position at coastal ranges. So the
verdict is a **risk/evidence/prioritisation** judgment — unverified field DF, hard
multipath/association/initialisation, coastal-range position above the PL, and a cheaper hardware-free
alternative — **not** an observability veto: against the already-required, hardware-free **multi-satellite
LEO fixture (U-MS1)**, which targets the demonstrated position gap directly, the array is a **research
spur, not a near-term aid**. Pursue it only as a hardware-free synthetic proof-of-concept (**U-EA1**,
sequenced after U-MS1) whose kill criterion is whether even the *known-beacon* mode can clear the 2°
heading / 200 m position gates under realistic DF and attitude error, and buy no hardware until it does.
