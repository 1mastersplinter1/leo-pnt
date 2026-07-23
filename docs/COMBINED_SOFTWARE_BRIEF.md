# Software implementation brief — sub-$1k core upgrades

**Status:** PROPOSED brief. **Date:** 2026-07-23. **Scope:** the four free (software/firmware)
upgrades from `docs/COMBINED_RESEARCH.md` §11. Device-agnostic — none of these depend on
which SDR feeds the samples. Each is a discrete, testable unit in the existing
`.orchestration` dual-review discipline.

State layout reference (`crates/pnt-estimator/src/lib.rs`): 9-state core
`[POS 0..3, VEL 3..6, HEADING 6, CLOCK_BIAS 7, CLOCK_DRIFT 8]`, plus dynamically augmented
per-satellite transmit-bias states and per-receiver-clock slots. Existing update methods:
`update_doppler`, `update_doppler_for_receiver`, `update_heading`, `update_speed_through_water`,
`update_msl_altitude`, `update_gnss`.

Ordering: **U1 → U2 → U4 → U3** (band-aware and current are pure filter changes and land first;
factor-graph is the largest; carrier-phase is gated on the OCXO hardware arriving).

---

## U1 — Band-aware fusion weighting (the Black Sea survival change)

**Goal:** the executive must not treat "LEO Doppler" as one trust category. Each Doppler
observation carries its **band** (VHF / L / Ku), and the measurement variance / gating is
scaled by a per-band, adaptively-updated jamming-susceptibility factor. Under interference,
Ku (Starlink/OneWeb) is down-weighted or gated out while VHF (Orbcomm) and L (Iridium) carry.

**Where:**
- `pnt-types`: add a `Band { Vhf, L, Ku }` field to the Doppler measurement envelope
  (the tracker already knows the carrier; propagate it, don't re-derive).
- `pnt-estimator` / executive: before `update_doppler`, scale `variance_mps2` by a
  `band_trust[band]` factor and/or tighten `chi_square_threshold`. Do **not** hardcode a
  theater; drive `band_trust` from a live interference estimate (U1b).
- **U1b (interference estimate):** a simple per-band noise-floor / C-N0 monitor. Source can be
  the tracker's own `best_quality` distribution over recent blocks, and (when present) an
  external jam-monitor RX (the $30 RTL-SDR). When a band's floor rises, its trust drops
  smoothly (not a hard cliff).

**Tests:**
- Injected Ku jamming (raised noise floor) → Ku observations down-weighted, VHF/L unchanged;
  position solution degrades gracefully rather than diverging.
- No interference → all bands weighted per their honest measurement variance (no regression
  against current behaviour).
- Property test: `band_trust` monotonic in estimated interference; bounded in (0, 1].

**Why it's theater-agnostic:** the filter adapts to the *measured* RF environment, so the
same code is correct in the Baltic (Ku usually fine) and the Black Sea (Ku jammed alongside
GPS). This is the single most valuable software change and has no hardware dependency.

---

## U2 — Current (SOG≠STW) estimation state

**Goal:** stop treating unmodelled ocean current as process noise. It is a persistent,
non-zero-mean velocity bias (0.3–1 m/s → 300–900 m over a 15–20 min pass) and is the
**dominant within-pass error** — not observable from Doppler alone, so it needs its own
slowly-varying state plus an independent information source (speed-through-water).

**Where:**
- `pnt-estimator`: add a 2-component horizontal **current-velocity bias** state (ENU E,N)
  appended to the core (or as a fixed augmentation), with a slow random-walk process model
  (small Q). Relationship: `velocity_over_ground = water_velocity + current`.
- `update_speed_through_water` (exists, currently ties speed to ground velocity at
  `crates/pnt-estimator/src/lib.rs:256`): re-derive so STW constrains
  `velocity_over_ground − current` (speed *through water*), making the current state
  observable from the STW ↔ Doppler-SOG discrepancy.
- The Doppler update already constrains speed *over ground*; the two together separate
  current from vessel motion.

**Tests:**
- Synthetic mission with a known injected current: current state converges to it; position
  RMS over a pass drops from the current-dominated hundreds-of-metres to the CRLB floor.
- Zero current → current state stays ~0, no regression.
- Observability test: with STW absent, current is (correctly) unobservable and the filter
  inflates its covariance rather than inventing a value.

**Dependency:** needs a speed-through-water input (the ~$100–300 speed log, or an existing
one). Software lands first; the sensor wiring is trivial once present.

---

## U3 — Carrier-phase tracking (the ~10× position gain) — HARDWARE-GATED

**Goal:** track the residual carrier **phase** of PSS/SSS (Starlink) / SS (OneWeb) after
Doppler wipe-off, not just the FFT peak. Phase is a position-strong observable
(published 7.7 m Starlink carrier-phase result) vs the velocity-strong Doppler peak.

**Where:**
- `pnt-tracker`: after the existing correlation-peak + phase-refine step, add a phase-lock
  loop (PLL/FLL-assisted-PLL) that maintains carrier-phase continuity across blocks, with a
  **cycle-slip detector**. Emit an accumulated-phase / integrated-Doppler observable
  alongside the existing `correlation_peak_hz`.
- `pnt-predictor`: add the carrier-phase (integrated range) prediction + Jacobian (analogous
  to the existing range-rate linearisation).
- `pnt-estimator`: a new `update_carrier_phase` scalar update consuming the accumulated phase,
  with an integer-ambiguity / bias state per pass (float ambiguity is sufficient initially;
  no need for full integer resolution at 200 m-class targets).

**HARD dependency — external reference:** carrier-phase requires the SDR locked to a stable
external reference (the OCXO). This unit is **gated on the OCXO arriving and on the static
real-RF capture confirming the OCXO holds phase lock over the coherent integration window
(seconds).** Do not merge U3 as "validated" on synthetic IQ alone — synthetic IQ cannot test
oscillator phase stability, which is the whole risk. See `COMBINED_RESEARCH.md` §11 step 5.

**Tests:**
- Synthetic: accumulated phase reconstructs injected range change; cycle-slip detector fires
  on an injected slip; estimator ambiguity state absorbs the constant offset.
- **Real-RF gate (the decisive one):** on a static capture with the OCXO, phase lock holds
  long enough for a usable carrier-phase observable. If it does not, U3 is deferred and the
  CSAC (`COMBINED_RESEARCH.md` §11) becomes the justified spend to break the $1k cap.

---

## U4 — Factor-graph / robust-gate estimator upgrade

**Goal:** replace the brittle single-epoch chi-square accept/reject gate with (a) a robust
cost (Huber/DCS) and (b) a sliding-window smoother that can carry the **E/W (LOP-mirror)
ambiguity** as parallel hypotheses instead of collapsing to one Gaussian mode and
occasionally diverging. Also fixes the **stationary→moving prior-handoff bug**.

**Where:**
- New crate or module (e.g. `pnt-smoother`) implementing a fixed-lag sliding-window graph
  (GTSAM-style; a Rust factor-graph or a hand-rolled sparse least-squares over the window).
  Keep the EKF as the real-time output layer, re-seeded from the smoother each window
  ("smoother-refines-filter").
- Doppler / carrier-phase / STW / heading / IMU factors over a 60–180 s window; per-pass
  nuisance nodes as today; **robust (Huber) cost** on the Doppler residual replacing the
  hard gate.
- **Prior-handoff fix (the bug from D39/D43 lineage):** inject a completed stationary batch
  fix into the moving graph as a **soft `PriorFactor` whose covariance is the batch solver's
  own posterior inflated by elapsed-time process growth** — never a tight/arbitrary reset.
  Marginalise (don't hard-cut) the stationary window's factors across the transition.
- **ZUPT:** while a motion classifier reports stationary, add a zero-velocity factor.

**Tests:**
- Two-hypothesis case: a fixture with a genuine E/W ambiguity → the smoother keeps both
  branches until a second pass's geometry collapses it; a plain EKF gate picks wrong and
  diverges on the same input (regression discriminator).
- Prior-handoff: a stationary fix handed into a moving filter must be *competed with*, not
  overwritten — verify the moving solution can move away from a stale prior (the exact bug).
- Robust cost: an injected outlier Doppler observation is down-weighted, not hard-rejected
  and not fully trusted.

---

## Cross-cutting

- Every unit keeps the repo's `[UNVERIFIED]` discipline and dual adversarial review before
  merge; U3's real-RF gate is itself a review criterion.
- No unit weakens the safety case: all of this is estimation/measurement; steering authority
  stays moving-only and fail-closed.
- U1 and U2 are the highest value-per-effort and have **zero hardware dependency** — start
  there today; U3 waits for the OCXO; U4 is the largest and can proceed in parallel with the
  hardware wait.
