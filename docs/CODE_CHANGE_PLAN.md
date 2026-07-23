# Code change plan — sub-$1k core upgrades (for external review)

**Status:** REVISED after Grok plan-review (2026-07-23). **Date:** 2026-07-23.
**Companion:** `COMBINED_SOFTWARE_BRIEF.md` (rationale), `COMBINED_RESEARCH.md` §11–12 (context).
This document is the *implementable* plan: exact crates, types, functions, and tests, grounded
in the current code. Units ordered **U1 → U2 → U4 → U3** (U4 split into U4a–U4e).

## Review-driven changes (Grok, 2026-07-23) — folded in below

Four blockers were raised and are now resolved in the plan:
- **B1 (was C1):** U2 as first written was rank-deficient — a scalar STW `hypot(v_g − c)`
  pins a 2-vector current to a *circle*, not a point. **Fix:** add the heading + zero-sideslip
  coupling (`v_water ≈ STW · û_heading` ⇒ `c = v_g − v_water`) so current is a *vector*
  observation, not a magnitude. (Fallback: a 1-state along-track current.)
- **B2 (was C2/OQ4):** U4's "smoother re-seeds EKF" **double-counts information** as written.
  **Fix:** exclusive measurement ownership — measurements go *only* into the smoother; the EKF
  does IMU-predict + accepts a bounded reseed (or an RTS-replace of the window). The reseed
  **is** the autopilot surface and gets fail-closed guards.
- **B3 (was C3):** current is *permanent* physics, so `augment_state` (built for pass-scoped
  nuisances) is the wrong mechanism — it re-enters the index-shift bug class on every retire.
  **Fix:** current lives in the **fixed core** (`CORE_DIM 9 → 11`, states 9–10), never shifts.
- **B4 (was H4):** U4 is too large to review as one unit → **split into U4a–U4e**.

**Confirmation-pass additions (Grok round 2, 2026-07-23) — new blockers folded in:**
- **N1 (U2 checklist gap):** extending `CORE_DIM` 9→11 also breaks the predictor's fixed
  `[f64; 9]` Jacobian (`pnt-predictor` `geometric_range_rate_linearisation`) and
  `FilterState::CORE_DIMENSION = 9` (`pnt-types`). Both are on the U2 checklist now; the
  Doppler Jacobian is **zero-padded on indices 9–10** (Doppler does not observe current).
- **N4 (U4 blocker):** the "kill-switch to EKF-only" is incoherent with exclusive measurement
  ownership — an EKF with no measurements is predict-only. The kill-switch must **re-enable
  measurement updates on the EKF** (a dual measurement path), stated in U4d.
- **N5 (U4 blocker):** the fixed-lag smoother state is at `t−L`; reseeding a live EKF with a
  lagged state is unsafe. Reseed must **propagate-to-now or RTS-replace the live window**
  before writing the EKF — stated in U4d/U4e.
- **N3/N9 (U2):** current init covariance must be an explicit prior (default `I` gives σ=1 m/s,
  not "frozen"); and the random walk is a **Q** (process-noise) term, not an **F**
  (transition-matrix) term — wording corrected.
- **Vector-STW R caveat:** the cross-track component of `v_water ≈ STW·û_heading` is a
  *model* pseudo-measurement, not a sensor reading — its R must **not** be set equal to the
  STW magnitude variance, or it injects fake cross-track information.

## Current-code facts this plan builds on (verified)

- Estimator (`crates/pnt-estimator/src/lib.rs`): `FilterStub`, 9-state core
  `[POS 0..3, VEL 3..6, HEADING 6, CLOCK_BIAS 7, CLOCK_DRIFT 8]` (`CORE_DIM = 9`).
- Dynamic state augmentation exists: `augment_state(value, variance) -> index`,
  `remove_state(index)` (with index-shift bookkeeping in `retire_satellite_bias`).
  Nuisance satellite-bias and per-receiver-clock slots already use it.
- Doppler update: `update_doppler(&DopplerRangeRateUpdate)` builds a scalar `H` from
  `core_jacobian` + nuisance; `variance_mps2` and `chi_square_threshold` are per-call.
- Speed model: `speed_model(&x)` returns predicted horizontal speed = `hypot(enu_E, enu_N)`
  of the **ground** velocity, with a numeric Jacobian over POS/VEL.
- Measurement bus (`crates/pnt-types/src/lib.rs`): `Constellation {Starlink, Iridium, OneWeb,
  Orbcomm}` and `TrackerDoppler { constellation, correlation_peak_hz, nominal_carrier_hz }`
  already flow on the bus. **Band is derivable from `constellation` — no new bus field needed.**
- Tracker (`crates/pnt-tracker/src/lib.rs`): `Detection { correlation_peak_hz, delay_samples,
  … }`; `process_block` returns `TrackOutcome`. Phase-refine step exists; no PLL / accumulated
  phase yet.

---

## U1 — Band-aware fusion weighting  (no hardware dep; smallest; first)

**Insight:** the bus already carries `Constellation`. Add a band mapping + an adaptive
per-band trust factor applied to the Doppler measurement variance in the executive, *before*
`update_doppler`. No estimator-internal change, no new bus field.

**Changes:**
1. `pnt-types`: add `pub enum Band { Vhf, L, Ku }` and `Constellation::band(&self) -> Band`
   (Starlink/OneWeb→Ku, Iridium→L, Orbcomm→Vhf). Pure, `#[must_use]`, unit-tested.
2. New small module (executive-side, e.g. `fusion-executive` or a `pnt-integrity`-adjacent
   helper) `BandTrust`:
   - holds `trust: [f64; 3]` in `(0,1]`, one per band, default `1.0`;
   - `observe(band, interference_level)` lowers that band's trust smoothly (e.g.
     `trust = 1/(1+k·interference)`), clamped, with hysteresis so it can't chatter;
   - `scale_variance(band, base_variance) -> f64` returns `base_variance / trust[band]`
     (lower trust → larger effective variance → down-weighted; never NaN/inf).
3. Executive: where it currently forms a `DopplerRangeRateUpdate`, scale `variance_mps2`
   by `BandTrust::scale_variance(band, …)`. **Inflate R only — do NOT also tighten
   `chi_square_threshold`** (B/H2: inflating R shrinks NIS, so tightening the gate too would
   double-penalize and could drop good VHF/L while Ku is merely deweighted).
4. **U1b interference source (defined statistic, not raw quality):** feed `observe()` with a
   *defined* per-band statistic — residual/noise-floor inflation **vs. a clear-sky baseline**,
   with a minimum sample count. `best_quality` alone confounds range/elevation/antenna pattern
   with jamming, so it is not used raw. **Unknown / insufficient samples ⇒ trust = 1.0**
   (never "unknown ⇒ kill Ku"). A jam-monitor RX improves the estimate but is not required.

**Tests:**
- `Constellation::band` mapping table.
- `BandTrust`: monotonic in interference, bounded `(0,1]` (trust floor > 0 → no div-by-zero),
  hysteresis (no chatter), `scale_variance` finite for all inputs; unknown-input ⇒ trust 1.
- Executive integration: injected Ku interference → Ku Doppler down-weighted, VHF/L unchanged;
  degrades gracefully vs. diverging. No-interference regression: identical to today.

**Risk/notes:** trust floored above 0. Down-weighting only; a **hard gate for a
confirmed-jammed band lives in the integrity supervisor/executive, not here** (OQ3 ruling) —
U1's default is a soft floor plus an integrity alert, never a silent `trust→ε`.

---

## U2 — Current (SOG≠STW) estimation state  (needs speed log + heading; software first)

**Insight (revised per B1/B3):** current is a **vector** (2-state ENU) and **permanent
physics**, so it goes in the **fixed core** (`CORE_DIM 9 → 11`, states 9–10), and it is made
observable by a **vector** water-velocity relation, not a scalar speed magnitude.

**Observability model (B1 — the fix):** a scalar `STW = hypot(v_g − c)` only pins `c` to a
circle. Use the heading + zero-sideslip assumption instead:
`v_water ≈ STW · û_heading`, so `c = v_g − v_water` — a **2-component** constraint that,
combined with the Doppler-constrained ground velocity, makes ENU current observable. Sideslip
is a small modelled/[UNVERIFIED] residual. **Fallback if zero-sideslip is too strong at sea:
a 1-state along-track current** (rank-safe by construction). Decide with the synthetic study;
the plan implements the vector form and keeps the 1-state form as the documented fallback.

**Changes:**
1. Estimator: extend the **fixed core** to 11 states — `CURRENT_E = 9`, `CURRENT_N = 10`,
   `CORE_DIM = 11`. Current **never shifts** — entirely outside the
   `augment_state`/`remove_state` index-shift class (B3). Update: the covariance **init**
   (default `I` gives σ=1 m/s — set an explicit current prior instead, N3), the process-noise
   injection in `propagate` (current is a **random walk → added in Q, `F` current rows stay
   identity**, N9; new `ProcessNoise::current_random_walk_variance`, default small e.g. `1e-8`,
   **≪ velocity Q** so absent STW can't corrupt `v`), and every `CORE_DIM`-sized copy/slice.
2. **N1 checklist — the `CORE_DIM 9→11` blast radius (must all change together):**
   - `pnt-predictor` `geometric_range_rate_linearisation` returns `[f64; 9]`; extend to
     `[f64; 11]` **zero-padded on 9–10** (Doppler does not observe current).
   - `pnt-types` `FilterState::CORE_DIMENSION = 9` → 11, plus current mean + covariance-slice
     fields; the executive JSON serialization (`fusion-executive`) and `pnt-studies`/mission
     consumers updated in lockstep. Accuracy helpers keyed on POS(0)/VEL(3) stay valid as long
     as that layout is preserved.
3. STW update (B1): model water-relative velocity as a **vector** residual
   `(v_g − c) − STW·û_heading`; `H` nonzero on VEL, CURRENT, and HEADING only (**no clock
   coupling** — keep the clock alias out). This needs a **2-row (or two sequential scalar)**
   update — only `scalar_update` exists today, so add the multi-row form. **R caveat:** the
   cross-track row is a *model* pseudo-measurement (zero-sideslip assumption), **not** a sensor
   value — its variance must be set to the sideslip-model uncertainty, **not** the STW
   magnitude variance, or it injects fake cross-track information. Prefer a softer cross-track R
   (or the 1-state along-track fallback) when sideslip is expected.
4. `FilterState`: export current (E,N) + covariance slice so executive/integrity can see it.
   Regression (M3): audit every consumer that read `speed_model` as *ground* speed.
5. **No STW available:** freeze current (Q≈0) or hold the current-prior so an unobserved
   current does not random-walk into `v` (M1).

**Tests:**
- Synthetic mission w/ known injected current + heading: current (E,N) converges to truth;
  per-pass position RMS drops from current-dominated hundreds-of-m toward the CRLB floor.
- **Circle-degeneracy regression:** confirm the *scalar* model is rank-deficient (documents
  why the vector model is required) and the vector model resolves a unique current.
- Zero current → current ≈ 0; regression vs. today within tolerance (with the ground-speed
  call-site audit done).
- Observability: no STW ⇒ current covariance inflates / stays frozen, estimate does not
  fabricate a value and does **not** leak into `v` (assert velocity unaffected).
- Fixed-core sanity: current indices are constant (9,10) under any augment/retire sequence —
  the index-shift risk is eliminated by construction, and a test asserts current H columns are
  correct after arbitrary satellite augment/retire.

**Risk/notes:** putting current in the core removes the index-shift footgun entirely (B3) —
this is the main reason to prefer it over dynamic augmentation. The remaining risk is the
observability model; the circle-degeneracy test is the guard.

---

## U4 — Robust estimation + smoother  (split into U4a–U4e per B4)

Too large to review or safety-validate as one unit. Five slices, each independently reviewable;
the **information rule** and **reseed safety gate** are hard prerequisites before any reseed.

| Slice | Deliverable | Notes |
|---|---|---|
| **U4a** | Huber/DCS robust cost on the EKF Doppler residual (no graph) | pure `FilterStub` change; immediate value; replaces brittle hard gate. **Handoff (N):** when U4d takes measurement ownership, this robust cost must **migrate into the smoother** (and remain on the kill-switch EKF path) — not left orphaned |
| **U4b** | Stationary→moving **soft-prior** handoff (D39/D43 bug) | batch posterior covariance inflated by elapsed-time growth; never a tight reset; marginalise, don't hard-cut |
| **U4c** | **Maritime ZUPT** (H5) — **hard prereq: U2** | zero **water-relative** velocity, or "moored: current free, SOG≈current" — **not** `v_g = 0` (that fights the U2 current state), so U2 must land first |
| **U4d** | Fixed-lag smoother crate `pnt-smoother` + **exclusive information rule** + **kill-switch dual path (N4)** | measurements enter the **smoother only** (EKF measurements re-enabled only in kill-switch mode); hand-rolled sparse Gauss-Newton over `nalgebra`; information rule in the crate root, not just README |
| **U4e** | Dual E/W hypotheses + collapse → **lag-aligned reseed gate (N5)** | reseed only post-collapse; smoothed state propagated-to-now / RTS-replaced before writing the EKF; bounded/guarded |

**Information accounting (B2 — the fix):** the smoother-reseeds-EKF design double-counts as
originally written (same measurements into both, then reseed reinjects information → optimistic
P). Resolution: **exclusive measurement ownership** — scalar Doppler/STW/heading updates go
into the **smoother only**; the EKF does IMU-predict + accepts the bounded reseed. (RTS-replace
of the window is the alternative; a full information-filter fusion only if simultaneous EKF
updates *and* smoother fusion are ever needed — they are not for this scope.) "Clean-data
agreement" is **not** accepted as proof of correct accounting.

**Reseed safety gate (M3 / safety-trap fixes) — the reseed IS the autopilot surface:**
- **Lag alignment (N5, blocker):** the smoother estimate is at `t−L`. Before writing the EKF,
  **propagate the smoothed state to now** (or RTS-replace the live window) — never reseed a
  live EKF with a lagged state.
- bound the reseed step `Δx` (reject jumps beyond a limit → hold last EKF state);
- **optimistic-P rejection (N-caveat):** reject a reseed whose covariance is not PSD **or is
  smaller than the EKF's in Löwner order** (`P_reseed ⪰̸ P_ekf`), i.e. an eigenvalue floor vs
  process noise integrated over the lag — not just a scalar "covariance floor";
- **fail-closed:** if the smoother diverges or the reseed is rejected, hold the last EKF state;
  the integrity supervisor must not report `steering_authorised` on smoother-shrunk P;
- dual E/W branches **never reseed until collapsed** to a single hypothesis;
- **kill-switch to EKF-only (N4, blocker):** EKF-only mode must **re-enable measurement updates
  on the EKF** (a dual measurement path: measurements → smoother in normal mode, → EKF in
  kill-switch mode). An EKF with neither is predict-only and unsafe. The smoother is an optional
  refinement, never a dependency.

**Tests:**
- U4a: injected outlier Doppler down-weighted (not hard-rejected, not fully trusted).
- U4b: a stationary fix handed to the moving filter is *competed with*, not overwritten — the
  solution can leave a stale prior (reproduces + fixes the D39/D43 bug).
- U4c: at anchor in a current, ZUPT does not pin a wrong SOG; current state stays free.
- U4d: **information rule** — measurements enter only the smoother; a test asserts the EKF
  posterior P is **not** optimistically shrunk by reseeding (the double-count guard).
- U4e: E/W ambiguity fixture → both branches held until a 2nd pass collapses it; plain EKF
  gate diverges on the same input; reseed only fires post-collapse; reseed-gate rejects an
  out-of-bound / non-PSD reseed and holds last EKF state.

**Risk/notes:** the reseed path is safety-critical (autopilot-facing via the EKF output).
U4d/U4e do not merge until the information rule and reseed gate are implemented and tested.

---

## U3 — Carrier-phase tracking  (HARDWARE-GATED on the OCXO)

**Insight:** track residual carrier phase of PSS/SSS after Doppler wipe-off → a
position-strong observable. Hard-gated: requires the bladeRF locked to the OCXO and a real-RF
capture proving phase lock holds over the coherent window.

**Changes:**
1. `pnt-tracker`: after correlation-peak + phase-refine, add a PLL (FLL-assisted) maintaining
   carrier-phase continuity across blocks + a **cycle-slip detector**. Emit
   accumulated-phase / integrated-Doppler alongside `correlation_peak_hz`.
2. `pnt-predictor`: add carrier-phase (integrated range) prediction + Jacobian, analogous to
   `geometric_range_rate_linearisation`.
3. `pnt-estimator`: new `update_carrier_phase` scalar update with a **float** ambiguity/bias
   state per pass (no integer resolution needed at 200 m-class targets) via `augment_state`.
4. `pnt-types`: extend `TrackerDoppler` (or a sibling `TrackerCarrierPhase` payload) to carry
   accumulated phase + a slip flag.

**Tests:**
- Synthetic: accumulated phase reconstructs injected range change; slip detector fires on an
  injected slip; ambiguity state absorbs the constant offset.
- **Real-RF gate (decisive, not synthetic):** on a static OCXO-locked capture, phase lock
  holds long enough for a usable observable. If not → U3 deferred, CSAC becomes the justified
  spend. This gate is a merge criterion, per `COMBINED_RESEARCH.md` §11 step 5.

**Risk/notes:** synthetic IQ *cannot* validate oscillator phase stability — the whole risk. U3
is not "done" on synthetic tests alone; the real-RF gate is mandatory.

---

## Cross-cutting

- Ordering rationale: U1/U2 are pure filter/bus changes with immediate value and no hardware
  dep; U4 is large and parallelisable during the OCXO wait; U3 is last (hardware-gated).
- Every unit: dual adversarial review + `[UNVERIFIED]` discipline before merge (existing
  `.orchestration` process). U3's real-RF gate and U2's index-shift property test are explicit
  review criteria.
- No unit weakens the safety case: all changes are estimation/measurement; steering authority
  stays moving-only and fail-closed; the smoother never becomes autopilot-facing (EKF remains
  the sole output).
- Disjoint ownership for parallel work: U1 (bus + executive), U2 (estimator), U4 (new crate),
  U3 (tracker+predictor+estimator, later). U2 and U3 both touch the estimator's augmentation —
  sequence them, don't parallelise those two.

## Open questions — RESOLVED by the Grok plan-review (2026-07-23)

1. **Current model** → 2-state ENU + slow random walk is fine **once observability is fixed**
   (vector water-velocity via heading/sideslip, or 1-state along-track fallback). Current atlas
   as a soft prior is a *later* addition, not MVP; per-water-mass is out of scope for the
   sub-$1k core. *(folded into U2)*
2. **Hand-rolled vs. factor-graph dep** → **hand-rolled sparse Gauss-Newton over `nalgebra`**,
   for a safety-reviewed minimal-dependency surface; a GTSAM-class dependency fails the
   minimal-deps story. Record the information rule in the crate root. *(folded into U4d)*
3. **Hard-gate policy** → lives in the **integrity supervisor / executive**, not the estimator.
   The estimator only ever sees R (and an optional gate threshold); the supervisor does
   jam-detect → hard band/constellation exclusion. Keeps the filter pure and the policy
   auditable. *(folded into U1)*
4. **Double-counting** → **not free as specified.** Use exclusive measurement ownership
   (measurements → smoother only) or RTS-replace; information-filter fusion only if simultaneous
   EKF updates *and* smoother fusion are needed (they are not). The consistency test alone is
   insufficient. *(folded into U4d)*

## Post-review status (two Grok passes)

**Round 1** raised B1–B4 (rank-deficient current, double-counting, dynamic-vs-fixed current
state, U4 too large) — all folded in. **Round 2 (confirmation pass)** verdicts:
- **B1 CONFIRMED-WITH-CAVEAT** — vector STW + fixed core is observable and rank-safe; residual
  heading/sideslip aliasing remains (bounded by the compass; 1-state fallback available); the
  cross-track pseudo-measurement R must be model-uncertainty, not STW variance.
- **B2 CONFIRMED-WITH-CAVEAT** — exclusive ownership eliminates double-counting; the reseed gate
  needed **lag alignment (N5)**, a **kill-switch dual path (N4)**, and a **Löwner-order
  optimistic-P test** — all now added.
- **B3 CONFIRMED** — fixed core eliminates the current index-shift class by construction.
- **B4 CONFIRMED-WITH-CAVEAT** — slicing sound; U4c depends on U2; U4a's robust cost must
  migrate into the smoother at U4d — both now noted.

**New items folded in:** N1 (the `CORE_DIM 9→11` blast radius — predictor `[f64;9]`,
`FilterState::CORE_DIMENSION`, executive JSON, studies), N3 (explicit current init prior),
N4 + N5 (the two new blockers above), N9 (RW is Q not F).

**Implement-readiness:** the plan is now consistent with the code and internally coherent.
Safe starting points with no remaining blockers: **U1** (band-aware, drop χ² tighten, defined
interference statistic) and **U4a** (Huber robust cost on the EKF). **U2** is ready once its N1
checklist (CORE_DIM blast radius) and the vector-R design are implemented as review criteria;
its circle-degeneracy test is mandatory. **U4d/U4e** do not merge until the information rule,
kill-switch dual path, and lag-aligned reseed gate are implemented and tested. U3 stays last,
hardware-gated.
