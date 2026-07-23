# Combined research plan — stationary + moving LEO-Doppler navigation

**Status:** PROPOSED — plan only, no code changes yet.
**Date:** 2026-07-23.
**Supersedes for planning purposes:** the split between `leo-nav` (stationary Transit-style
design, 2026-07-21) and `leo-pnt` (moving-vessel real-time build). This document merges the
two research lines into one project whose receiver is **sometimes stationary and sometimes
moving**, and states how the combined system is structured.

This plan folds in the independent three-model review (Fable / Grok / Codex, 2026-07-23) of
the `leo-pnt` concept and tech, and the earlier `leo-nav` feasibility design and its
9-agent research round. Where the two research levels disagree, the disagreement is stated
explicitly rather than smoothed over.

---

## 1. Why combine

The two projects are the **same physics in two regimes**, not two ideas:

| | `leo-nav` (level up) | `leo-pnt` (this repo) |
|---|---|---|
| Receiver | **stationary** rooftop/fixed | **moving** manned boat |
| Solver | Transit/Argos batch LSQ, state `[lat, lon, f_offset]`, altitude fixed | 9-state error-state EKF (pos/vel/heading/clock) real-time |
| Accuracy frame | ~20 m 2D position (matches published static results) | ≤200 m position / 0.04 m/s velocity, denied-mode, under way |
| Clock | GPSDO (LBE-1421) — clock problem *removed* | **free-running** Rb/OCXO — clock estimated as a state |
| SDR | USRP B206mini-i (56 MHz BW, ~27 dB Starlink processing gain) | bladeRF 2.0 micro (2.5–5 MHz) |
| Signal order | **OneWeb first** (gentler DSP), then Starlink | Starlink PSS/SSS first; OneWeb behind a survey gate |
| Validation | real data first (SupGP + Iridium already downloaded) | synthetic-IQ only, then real |
| Built? | design + a 229-line `ephem-fetch` CLI + downloaded ephemeris; **Python science core never built** | 12 review-hardened crates, end-to-end synthetic demo, SITL |

A real deployment is not one or the other. A boat at anchor, a survey buoy, a vehicle
parked, or any initial-alignment phase is the **stationary** regime — which is exactly the
regime with all the published 2–30 m Doppler-only results. Under way is the **moving**
regime `leo-pnt` was built for. The combined project keeps both and selects at run time.

The complementarity is concrete: `leo-nav` has the one real asset `leo-pnt` lacks — a
working ephemeris-ingestion CLI and **actually-downloaded SupGP/Iridium data** — while
`leo-pnt` has everything `leo-nav` only designed (tracker, EKF, predictor, replay, safety).

---

## 2. Unified architecture: two solvers, one shared core

The measurement physics, the reference orbit, and the RF front end are identical across
regimes. Only the estimator differs. So the architecture is a **shared core feeding two
interchangeable solvers**, chosen by run mode.

```
                 ┌───────────────────────────── shared core ─────────────────────────────┐
   RF IQ ──▶ pnt-tracker ──correlation_peak_hz──▶                                          │
                                                  ├─▶ measurement bus (pnt-types)          │
   ephem-fetch ─▶ archived SupGP/TLE ─▶ pnt-ephemeris ─▶ pnt-predictor.predict()           │
                 (SGP4, TEME→ECEF, age gate)      (range-rate + Jacobian)                  │
                 └─────────────────────────────────────┬─────────────────────────────────┘
                                                        │  same range-rate + H for both
                        ┌───────────────────────────────┴───────────────────────────────┐
              stationary │                                                     moving      │
                        ▼                                                                 ▼
          BATCH LSQ (new; leo-nav design)                          9-state EKF (existing; leo-pnt)
          state [lat, lon, f_offset(+drift)]                       pos/vel/heading/clock + TX nuisance
          altitude fixed (DEM/MSL)                                 real-time recursive, NIS gate
          full-pass curve fit, E/W ambiguity                       fail-closed authority supervisor
          resolved by 2nd ground track                             (moving is the only steering regime)
```

Verified reuse boundaries (APIs already present in this repo):

- **`pnt-predictor::predict()`** returns range-rate + `geometric_range_rate_linearisation`
  (Jacobian). A batch LSQ consumes the same prediction the EKF's `update_doppler` does — no
  new physics needed for the stationary solver.
- **`pnt-tracker::process_block()`** emits `Detection { correlation_peak_hz, delay_samples,
  … }` agnostic to which solver ingests it.
- **`pnt-ephemeris`** (SGP4, age gate) feeds both; **`ephem-fetch`** (moved in from
  `leo-nav`) is the ingestion front end that keeps the archive fresh.

The **batch LSQ is the only genuinely new estimator component.** Everything else is reuse.

### Run-mode selection

A stationary/moving decision drives solver selection and is itself an observable, not a
config toggle:

- **Detector:** speed-log + IMU (and, when moving, the EKF's own velocity state) classify
  the current epoch as stationary or moving with hysteresis.
- **Stationary → batch LSQ** over the accumulated pass(es). Static receiver means captures
  need not be simultaneous — sessions hours apart combine in one solve with a shared
  position and per-chain `f_offset` (leo-nav §4). This is also the **calibration regime**:
  a known-stationary session is where `f_offset`/drift and lever arms are best pinned.
- **Moving → 9-state EKF** as built, with the fail-closed authority supervisor. Steering
  authority is a **moving-only** concern; the batch solver never touches authority.
- **Handoff:** a completed stationary batch fix (position + clock estimate + covariance)
  is the natural **initialization prior** for the moving EKF — which directly attacks the
  reviewers' finding that most of the current synthetic headline gain is the *prior*, not
  Doppler. A real, Doppler-derived stationary prior replaces the disclosed synthetic one.

---

## 3. What the three-model review says, mapped onto the combined plan

All three reviewers assessed `leo-pnt` without seeing `leo-nav`. Several of their
recommendations are, in effect, "do what `leo-nav` already planned." The combined project
resolves these by construction.

| Review finding (Fable / Grok / Codex) | Combined-plan resolution |
|---|---|
| Concept physically sound; velocity-first framing correct | Kept. Batch (stationary) is the classic position-from-curve regime; EKF (moving) is velocity-first. Both correct, now both present. |
| "Not a position instrument" slightly overstated — position weakly observable via LOS geometry | The stationary batch solver **is** the position-from-Doppler-curve solver; it makes that weak observability the primary output when the platform allows it. |
| **Free-running clock is the sharpest threat** (ε=1e-9 → 11.6 Hz @ Ku → 0.3 m/s > velocity gate); Fable adds g-sensitivity | Kept as the top open risk. Stationary regime is where clock `f_offset`/drift is **most observable** (no velocity aliasing), so the batch solver becomes the clock-characterization tool. Reconcile with `leo-nav`'s GPSDO option (§5). |
| Highest-value next experiment (unanimous): **static, truth-surveyed, real-RF Ku capture — no boat** | This is literally `leo-nav`'s Stage 2 Rung A1, feeding the **combined** stationary solver. Already designed, BOM verified, ephemeris downloaded. |
| Maritime antenna/link budget is the top under-weighted risk (Fable) | Carried in as a paper deliverable (§5). `leo-nav`'s 56 MHz USRP + ~27 dB processing-gain analysis is the starting point, not the 2.5–5 MHz bladeRF. |
| "No public structure" is outdated | Already refuted by this repo's R4. No change. |
| Doppler-degrades-velocity open item | Moving-regime EKF item; fix the replay prior-init defect (D43) first. A real stationary prior (above) reduces reliance on the synthetic one. |
| Research/claim hygiene is exceptional | Preserve it. The combined doc set keeps `[UNVERIFIED]` discipline, Wilson CIs, artifact diagnosis. |

---

## 4. Reconciled decisions where the two levels disagreed

1. **Clock: free-running vs disciplined.** `leo-pnt` D42 chose a free-running Rb (owned
   equipment, no service fees). `leo-nav` removed the clock problem with a €185 GPSDO.
   The reviewers show the free-running premise re-creates the hardest problem they found.
   **Combined stance:** keep free-running as the *goal*, but (a) use the stationary batch
   regime to characterize the real reference's ADEV **and g-sensitivity on a motion table**
   before committing, and (b) treat `leo-nav`'s GPSDO/holdover chain as a documented
   fallback if the 0.04 m/s velocity gate proves unreachable free-running. This is a
   go/no-go on the premise, decided with measured numbers, not an assumption.

2. **Signal order: OneWeb-first vs Starlink-first.** `leo-nav` leads with OneWeb (gentler
   DSP); `leo-pnt` leads with Starlink and gates OneWeb behind a survey. Reviewers favor
   the gentler DSP first. **Combined stance:** for the *first real capture* (falsification),
   use the gentlest available real signal — Iridium bursts (mature `gr-iridium` tooling,
   ephemeris already downloaded) or OneWeb beacon — to isolate estimator correctness from
   Starlink-specific DSP risk. Starlink PSS/SSS remains the priority *production* target.

3. **Bandwidth: 56 MHz USRP vs 2.5–5 MHz bladeRF.** `leo-nav`'s USRP choice directly
   addresses the link-budget risk. **Combined stance:** resolve in the antenna/link-budget
   paper (§5) with the tracker's own accuracy-vs-C/N0 curve; do not assume the 2.5–5 MHz
   contract is sufficient for *accuracy* (as opposed to detection) until measured.

---

## 5. Staged next steps (no hardware spend before Stage 0)

**Stage 0 — mechanical merge (structural, low risk).** Absorb `leo-nav`'s real asset into
this workspace: move `ephem-fetch` in as a crate, move `data/` in, wire `pnt-ephemeris` to
read the archived SupGP/TLE, `cargo test` green. Archive `leo-nav` read-only. *No estimator
work.* Deliverable: one workspace, real ephemeris ingestion available to the stack.

**Stage 1 — paper falsification (cheapest, no hardware).** Port `leo-nav`'s CRLB/observability
simulation intent into a `pnt-studies` module: real Danish-straits pass schedules from the
downloaded ephemeris, Rb/OCXO drift + g-sensitivity, heading/current/speed-log error, for
**both** regimes (stationary batch and moving EKF). If neither regime meets its target
except under ideal clock/current assumptions, down-scope before RF spend (Codex).

**Stage 2 — the batch solver (new shared-core component).** Implement the stationary
Transit-style batch LSQ against `pnt-predictor`/`pnt-tracker`/`pnt-ephemeris`, with the E/W
ambiguity resolution and per-chain `f_offset`. Validate on synthetic IQ through the existing
harness, dual-reviewed to the same bar as the rest of the repo.

**Stage 3 — first real-RF capture (the unanimous #1).** Static, truth-surveyed capture
(gentlest real signal per §4.2), score measured Doppler against SGP4/SupGP prediction, and
feed it through the **batch** solver to a surveyed mark. Pass bar (Grok): residual bulk
within a few kHz, σ_f ≤ ~10 Hz after ~1 s averaging. This simultaneously validates real
C/N0, the wipe-off budget, and the clock coupling — *no boat, no EKF retune*.

**Stage 4 — antenna/link-budget paper + clock characterization** (Fable's top risk;
§4.1/§4.3). Close EIRP → G/T on a rocking mount → C/N0 → variance → estimator weight, and
measure the real reference's ADEV/g-sensitivity. Decide the free-running go/no-go.

**Stage 5 — moving regime.** Only after Stages 1–4: fix the EKF Doppler prior-init defect
(D43), initialize the moving EKF from a real stationary batch fix, then constant-heading
harbour legs, GNSS-truth journal, multi-hour denied replay. Steering authority (real
supervisor, never `IntegrityStub`) gates powered trials as already specified.

---

## 6. What carries over unchanged

- All of `leo-pnt`'s review/orchestration discipline (`.orchestration/`, `[UNVERIFIED]`
  markers, dual adversarial review, fail-closed authority) applies to every new stage.
- The document hierarchy (DESIGN_BASELINE normative, etc.) absorbs `leo-nav`'s design as an
  informative research input under `docs/research/`, not a normative override.
- The safety case is untouched: steering authority remains moving-regime-only and
  fail-closed; the stationary batch solver produces coordinates, never manoeuvres.

---

# Part II — Validation, upgrade menu, and two-theater analysis

**Added:** 2026-07-23. **Source:** ten independent research agents (six with live
2024–2026 literature search) plus the three external model reviews (Fable / Grok / Codex).
This part is the evidence base and prioritised recommendations behind Part I.

## 7. Concept-validation verdict

**Position navigation: VALIDATED and precedented — but the free-running-clock premise is on
the research frontier.**

- A ship off Greenland (Kassas/MILCOM) went from **>1 km dead-reckoning to 27 m** fusing
  Starlink+OneWeb Doppler. Ground vehicles reach **4–10 m**. The 200 m position target is
  *generous* against this record.
- **The load-bearing caveat:** essentially every published real-RF LEO-Doppler result ran on
  a **GPS-disciplined oscillator (GPSDO)**. The EKFs *model* clock states, but the *physical*
  reference feeding the ADC was GPS-timed. There is **zero published real-RF, moving-platform
  result with a validated free-running clock.** The "no GPS time, anywhere" premise is
  *outside* the demonstrated envelope, not merely behind it — this is the project's genuine
  research contribution and its central risk.

**Velocity target (0.04 m/s): OVER-CLAIMED by ~55×.** Unsupported by any real-signal result
at any clock discipline; the repo's own synthetic pipeline misses it 55× (Doppler currently
*degrades* velocity, D39). It reads as autopilot requirements-flowdown, not evidence-derived.
**Action:** velocity should come primarily from speed-log/IMU with Doppler *bounding drift*,
or be relaxed.

## 8. Adjudicated error budget (what actually limits accuracy)

| Regime | Dominant error | Magnitude | Fix |
|---|---|---|---|
| **Within a pass** | Unmodeled **ocean current** (SOG≠STW) | 0.3–1 m/s → **300–900 m** over 15–20 min | speed-through-water sensor + explicit current state |
| **Between passes (DR coast)** | **Clock drift** (free-running oscillator) | sets inter-pass growth | CSAC (~$2–6k) cuts ~100×; OCXO is the cheap partial |
| **Real-RF, untested by any synthetic study** | **LNB LO drift** + **antenna-pointing loss on a rolling hull** | tens of kHz / lock loss per roll | 2nd referenced LNB or cal tone; sea-state pointing sweep |

Key nuances: the free-running clock **self-averages within a pass** (~100+ wave cycles →
sub-metre); it dominates the *inter-pass coast*, not the fix. True CRLB floor is **~50–150 m**
for a good pass, so 200 m is reachable *only if current is estimated*, not absorbed as
process noise.

## 9. Upgrade menu — by role and regime

The system is **velocity-strong**; it needs **position-strong** and **absolute** complements.

**Tier 1 — highest leverage (adopt):**
- **Differential two-receiver** (shore reference cancels common-mode SV clock + ephemeris) →
  **~9.5 m 3D RMSE** (Kassas). Biggest accuracy win; coastal, needs a reference + link.
- **Carrier-phase tracking** (track PSS/SSS residual phase, not just the FFT peak) →
  **7.7 m**, *firmware only, no new RF*. Needs local-oscillator phase stability.
- **CSAC** (chip-scale atomic clock, ~$2–6k) — cheapest high-leverage buy; kills the #1
  inter-pass error and enables carrier-phase.
- **Factor-graph sliding-window smoother** — resolves the E/W ambiguity a plain EKF
  collapses; also gives the correct fix for the prior-handoff bug (soft `PriorFactor` with
  real posterior covariance, not a tight reset).
- **DVL (Doppler velocity log)** — DR drift 1–10% → <0.3% of distance. Combined Tier-1
  budget lands at **~80–90 m moving**.

**Tier 2 — terrestrial position-strong (coastal only; all land-sited → one-sided seaward DOP):**
- **DAB TDOA** — SFN-synchronised → TDOA without per-tower clock estimation; ~tens of m.
  **Baltic/NW-Europe only** (absent around the Black Sea).
- **MF R-Mode beacons** (~285–315 kHz) — **~12 m @95% daytime**, ground-wave over sea.
  Baltic testbed exists; night skywave degrades 2–5×.
- **DVB-T** — cm–m static but SFN **path-ambiguous** at sea; use the coarse LEO fix as the
  disambiguation prior.

**Tier 3 — absolute / un-spoofable:**
- **SWIR celestial** (not thermal — stars aren't warm blackbody targets). Cloud-limited to
  **~21–33%** availability at 55°N; IR does *not* penetrate thick maritime cloud. Role:
  **anti-spoof integrity cross-check + stationary-only fix**, not a primary source. A
  **Sun/Moon single-LOP tracker (~$0.5–3k)** is the better ROI than a full star camera.

**Dead ends for NW Europe:** generic AM broadcast (transmitters gone), FM (nearshore only),
eLoran (Baltic coverage not yet built).

## 10. Two theaters — Baltic vs Black Sea

Designing for both forces the system to become **theater-adaptive, not theater-specific** —
a stronger architecture for something whose premise is "operate wherever GNSS fails."

| Dimension | Baltic (~55°N) | Black Sea (~41–47°N) |
|---|---|---|
| GNSS denial | Intermittent (2,500+ events/2024) | **Structural — near-daily, multi-day blackouts, L1/L2/L5** |
| Motivation strength | Weak (GPS mostly works) | **Strong (GPS genuinely fails for days)** |
| LEO geometry (Starlink 53°) | Good | Equal or better (mid-lat density peak) |
| **Ku survives jamming?** | Generally yes | **No — Krasukha-4 jams 10.9–14 GHz, ~300 km radius** |
| Iridium L-band? | Probably | Uncertain (proximity to GPS L1 noise floor) |
| VHF (Orbcomm) / terrestrial? | Yes | **Yes — outside all documented EW bands** |
| DAB | Dense | **Effectively absent on the coast** |
| AM/MW ground-wave | Dead | **Live — Turkey TRT 300 kW; higher salinity → better range** |
| Celestial clear-sky | ~21–33% | Materially higher (sunnier) |
| Testability | Yes (peacetime) | **No — war zone, mines, EW → design-for only** |

**Single most important cross-theater implication:** do **not** treat "LEO-Doppler" as one
resilience category. In a contested theater **Ku (Starlink/OneWeb) is a combat-proven jamming
target** — the very band the Baltic design leans on — while **VHF/Orbcomm and terrestrial
bands survive**. The fusion filter needs **band-specific, adaptive jamming-susceptibility
weighting**, and the terrestrial stack a **theater-selectable profile** (DAB-primary for NW
Europe, AM/MW-ground-wave-primary for the Black Sea). This retroactively strengthens D42
(owned passive hardware) and D10 (Orbcomm caution): VHF/Orbcomm being the jam-survivor makes
the passive-owned-hardware premise more defensible, not less.

## 11. Cost-constrained recommendations

The two dominant error terms (current, clock) and the ~10× position gain (carrier-phase) are
mostly **software/firmware plus one clock buy**, so meaningful progress is cheap.

**Free tier (software/firmware, both theaters, do first):**
1. **Band-aware fusion weighting** (Ku down-weights under interference; VHF/Orbcomm carries)
   — the key Black Sea survival change, theater-agnostic by construction.
2. **Current (SOG≠STW) estimation state** — attacks the #1 within-pass error (needs a
   speed-through-water reading).
3. **Carrier-phase tracking** (firmware) — ~10× position gain, no new RF.
4. **Factor-graph / robust-gate estimator** — E/W ambiguity + prior-handoff fix.

**Sub-$1k build (theater-agnostic core — recommended entry point):**
- **OCXO with external-reference output (~$150–500)** — CSAC substitute; 10–100× better
  short-term than the bladeRF stock TCXO. Carrier-phase becomes *experiment-gated* (validate
  phase lock on real IQ) rather than assumed; inter-pass coast leans harder on IMU + the
  current state. An OCXO is also the *more honest* test of the owned free-running premise
  than a CSAC.
- **Speed log / STW sensor (~$100–300 if not aboard)** — feeds the current state.
- **RTL-SDR (~$30)** — a VHF/L-band **jamming monitor** driving the band-aware weighting
  (theater-agnostic use, not a terrestrial position aid).
- **Total ~$300–800.** Accuracy: within-pass fix is current-fixed (software) and gets the
  carrier-phase gain; inter-pass is looser than a CSAC build. Retires the free-running-clock
  question partially and honestly.

**< $8k build (add one item to the above):**
- **CSAC (~$2–6k)** — the single hardware buy that moves a dominant error term (inter-pass
  clock) *and* unlocks carrier-phase reliably. Everything else stays software/firmware.

**< $30k build:** add a **DVL (~$10–20k)** → the full ~80–90 m-moving budget.

**Deferred at every tier under a cost cap:** differential 2nd receiver (theater-specific),
DAB/MF/AM terrestrial (theater-specific), SWIR celestial + gimbal ($30–60k, cloud-limited),
wideband USRP swap.

**Sequence (cheapest-first, each gates the next):**
1. Confirm a passive logging-only trial is permissible (regulatory) — before any spend.
2. Ship the free software/firmware wins.
3. Sub-$1k hardware: OCXO + speed log + jam-monitor RTL-SDR.
4. Add carrier-phase (firmware).
5. **Static real-RF capture on the existing bladeRF** — the single decisive experiment:
   validates real C/N0, wipe-off, LNB-LO drift, and **whether the OCXO holds phase lock long
   enough for carrier-phase** (which tells you if the CSAC is ever worth breaking the cap).

**Bottom line:** a boat *has* navigated on LEO Doppler to 27 m — the concept is real. The
frontier is the **free-running clock** (no real-RF precedent), best approached with a cheap
**OCXO + a current state**, not heroic oscillator work. The biggest accuracy upgrade is
**carrier-phase** (~10×, firmware). Terrestrial and celestial aids are **regime-bound** — DAB/MF
for the Baltic, AM ground-wave for the Black Sea, celestial as an un-spoofable integrity
check. And the **Black Sea is the use case that justifies the project** — GPS genuinely fails
there — but it is *design-for, test-elsewhere*, and it forces the one architectural change
that matters most: **make the fusion band-aware**, because where the system is needed most,
the Ku signals it leans on are jammed alongside GPS.

## 12. Hardware — confirmed front-end and reference

**Front-end decision: stay on the Nuand bladeRF 2.0 micro xA4** (~$540; already owned). A
head-to-head against the USRP B205mini/B206mini/B210 resolved it:

- **The bladeRF is 2×2 coherent (shared LO)** — required for the future 2-antenna /
  differential carrier-phase option. The **B205mini/B206mini are 1×1 and structurally cannot
  ever do it.** Matching the bladeRF's 2×2 coherence on a USRP means a **B210 at $2,387
  (4.4×)** for the same AD9361-class RFIC, same 56 MHz, same 12-bit ADC — no new capability.
- **External-reference input (the hard requirement for carrier-phase) is native and clean:**
  the bladeRF's ADF4002 PLL disciplines the onboard VCTCXO to any external ref (default
  10 MHz), AC-coupled and biased to accept **either 3.3V CMOS or a sine-wave OCXO directly —
  no board mod, no daughtercard.** (The B210 needs its optional GPSDO daughterboard *absent*
  to accept an external ref.)
- Both tune 70 MHz–6 GHz (cover Ku-IF ~1.5 GHz + Iridium L-band 1616–1626 MHz); **neither
  covers 137 MHz Orbcomm VHF** → VHF stays on the separate RTL-SDR path (Doppler-only, no
  coherence needed). Toolchain favours bladeRF for a Rust repo (libbladeRF C API FFI-binds
  cleanly; UHD is heavier C++).
- **No deal-breaker forces a USRP.** Reconsider a B210 only if a future need bladeRF cannot
  meet emerges (>122.88 MHz combined BW, or mandatory UHD-specific software). The **xA9
  ($860)** is the same RF with a bigger FPGA — only if PSS-correlation offload onto the FPGA
  is wanted later.

**Reference: a 10 MHz OCXO into the bladeRF external-clock input.** Recommended single buy is
a **Leo Bodnar LBE-1420 GPSDO/OCXO (~£120/$150)** — squarewave 10 MHz, AC-couple into the
bladeRF SMA. It does double duty: run **disciplined** for the first clean carrier-phase
capture (isolating "does the method work" from "does my clock hold"), then flip to
**holdover/free-running** to measure how the actual owned-hardware premise degrades. This
directly drives the U3 real-RF gate (`COMBINED_SOFTWARE_BRIEF.md`): if the OCXO holds phase
lock long enough for carrier-phase, the CSAC is never needed. Cheaper alternative: a bare
used OCXO module (~$30–120) — purest free-running test, needs a clean supply + warm-up.

**Confirmed sub-$1k bill:**

| Item | Cost | Note |
|---|---|---|
| bladeRF 2.0 micro xA4 | already owned | main Ku-IF/L-band coherent chain |
| Leo Bodnar LBE-1420 (10 MHz → bladeRF ext-ref) | ~£120 / ~$150 | disciplined + holdover; drives the U3 gate |
| Speed log / STW sensor | ~$100–300 | free if aboard; feeds U2 current state |
| RTL-SDR | ~$30 | VHF/Orbcomm + jamming monitor (drives U1) |
| SMA cable + antennas/bias-tee | ~$50–100 | ref→bladeRF, Ku LNB feed, VHF whip |
| **Total** | **~$330–580** | |

The software counterpart to this hardware is `docs/COMBINED_SOFTWARE_BRIEF.md` (units U1–U4).
