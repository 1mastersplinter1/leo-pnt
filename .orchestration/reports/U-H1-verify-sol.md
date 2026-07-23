Reading additional input from stdin...
OpenAI Codex v0.144.5
--------
workdir: /home/od/work/leo-pnt
model: gpt-5.6-sol
provider: openai
approval: on-request
sandbox: read-only
reasoning effort: none
reasoning summaries: none
session id: 019f8f5b-2d38-73f1-83eb-727c5bf77b33
--------
user
You FAILed docs/design/HIGH_SPEED_ENVELOPE.md; it has been rewritten against R5's sourced values (git diff shows the rework; your review at .orchestration/reports/U-H1-review-sol.md). Verify the corrections: (1) the assumed 3g-RMS premise is replaced by R5's sourced 0.44g continuous RMS with the rectified-bias budget recomputed correctly (recompute it: does it now sit ~48x under the denied PL, and is the conclusion correctly downgraded from 'mandatory isolation for integrity' to 'recommended for availability/cycle-slip'); (2) the 8000 Hz/s 'hard cap' is corrected to a screening bound consistent with what docs/studies/tracker actually measured; (3) the 1/v timer scaling and T_ack floors now trace to R5/HSC guidance; (4) every previously-assumed constant is now either R5-sourced (cite) or [UNVERIFIED]; (5) the 20 kn and 30 kn verdicts FOLLOW from the corrected derivations and are scoped evidence-vs-judgment (esp. the withdrawn 'cap specifically between 20-30 kn' and the corrected isolation-dB and rectification-multiplier figures); (6) stale first-draft numbers survive only inside explicit withdrawal context. Per-finding verdict on YOUR original findings + any NEW + final PASS or FAIL.
codex
I’ll compare the rewritten design, the original review, and the underlying R5/tracker sources, then independently recompute the key ratios and audit stale constants.
exec
/bin/bash -lc "sed -n '1,260p' .orchestration/reports/U-H1-review-sol.md && sed -n '1,320p' docs/design/HIGH_SPEED_ENVELOPE.md && git diff -- docs/design/HIGH_SPEED_ENVELOPE.md" in /home/od/work/leo-pnt
 succeeded in 0ms:
(Reviewer: Sol/codex deep seat. Verdict: FAIL.)

The arithmetic tables are mostly reproducible, but several load-bearing physical and safety conclusions do not follow from the cited evidence. The largest failures are the oscillator rectification argument, use of tracker measurements as hard limits, timer scaling below the cited human-response floor, and the good-fix phase timeline.

## Findings

1. **Critical · High confidence · §1.4–1.5, §5.5, Verdict — vibration-rectification reference budget is unsupported**

   **Claim:** Every real oscillator has a quadratic sensitivity `Γ₂`; taking `Γ₂ = 10⁻¹¹/g²` and `a_rms = 3 g` yields a sustained `0.027 m/s` bias, making shock isolation mandatory.

   **Finding:** The algebra `δ_DC = Γ₂⟨a²⟩` and `Δv = cδ` is correct for an established quadratic coefficient. Neither input is established here. R5 documents linear OCXO vibration sensitivity and phase-noise methodology but supplies no oscillator `Γ₂`, no evidence that `10⁻¹¹/g²` is representative, and no measured 3 g RMS environment. R5’s closest measured RMS point is 0.44 g, while roughly 3 g is an upper-event statistic, not RMS. At 0.44 g, the assumed model gives about `0.00058 m/s`, roughly 47× below the denied VPL.

   The mechanism is a legitimate hypothesis requiring measurement, but the numeric integrity breach and “shock isolation REQUIRED/MANDATORY” verdict do not follow. A low-`Γ₂` reference could pass without isolation; a poor one might not pass even with the proposed mount.

   **How to verify:** Shaker-test the selected oscillator over the measured hull PSD, fit signed linear and quadratic coefficients, and measure residual clock-frequency bias after estimator clock-state absorption. Compare the resulting solution-domain VPL contribution, not raw `cδ`, against the integrity allocation.

2. **Major · High confidence · §1.4 — R5 directly disagrees with the assumed slam duration and spectrum**

   **Claim:** Marine slams last 5–50 ms and have dominant content at 10–100 Hz.

   **Finding:** R5’s full-scale NSWCCD evidence gives rigid-body impact durations of **100–450 ms**. Its 20 g/23 ms value is a laboratory shock-qualification pulse derived by shock-response-spectrum equivalence, not the actual sea-slam duration. R5 explicitly warns that long slam pulses contain substantial energy below about 10 Hz and make compact isolation difficult.

   This disagreement materially affects:

   - the `1/(πfτ)` suppression;
   - the proposed 10–15 Hz isolator;
   - tracker transient duration;
   - IMU sampling/aliasing conclusions;
   - the assertion that the important energy lies above the isolator corner.

   **How to verify:** Replace the 5–50 ms estimate with a bounded spectrum derived from R5’s 100–450 ms half-sines, then repeat using measured acceleration histories from the selected hull.

3. **Major · High confidence · §1.4(a) — “sub-cm/s” suppression is numerically false and the sinusoidal averaging model is misapplied**

   **Claim:** A 10 g event at 50 Hz, initially equivalent to 3 m/s, is suppressed about 150× to sub-cm/s.

   **Finding:** `3/157 ≈ 0.019 m/s`, which is about **1.9 cm/s**, not sub-cm/s. At 10 Hz the same calculation gives about 9.5 cm/s. More importantly, `1/(πfτ)` applies to a sustained sinusoid with a defined averaging interval and phase assumptions; a finite half-sine slam or broadband transient needs direct integration through the actual Doppler estimator/window.

   **How to verify:** Convolve representative 100–450 ms acceleration pulses and measured vibration PSDs with the actual coherent correlator, Doppler discriminator, and clock estimator impulse responses.

4. **Major · High confidence · §1.5, M-1 — proposed isolation architecture conflicts with R5 guidance**

   **Claim:** The reference and IMU should share a 10–15 Hz isolated sub-plate.

   **Finding:** R5 reports VectorNav’s preference for rigid IMU mounting and warns that soft isolation can degrade AHRS performance through relative motion and filtering lag. It recommends isolating the vibration source or whole subsystem when isolation is necessary. A shared reference-plus-IMU plate is better than soft-mounting the IMU alone, but antennas, lever arms, magnetometers, speed log, and vessel frame remain outside that moving coordinate frame. The consequence register mentions extrinsics re-survey but does not bound dynamic plate motion or prove that this architecture preserves navigation observability.

   **How to verify:** Model and measure six-DOF isolator motion, cable shorting, lever-arm dynamics, resonance, attitude lag, and estimator performance. Compare rigid low-VRE IMU, reference-only isolation, and whole-navigation-subsystem isolation.

5. **Major · High confidence · §6.1, H-1, 30 kn verdict — required extra isolation attenuation is overstated by about 2× in dB**

   **Claim:** A rectified bias 3.3–4.9× over budget requires 10–15 dB more isolation.

   **Finding:** Because rectified bias is proportional to acceleration squared, reducing bias by factor `R` requires acceleration transmissibility `1/√R`. Expressed as the usual mechanical amplitude dB, this is:

   `20 log10(√R) = 10 log10(R)`

   For `R = 3.3–4.9`, the extra attenuation is approximately **5.2–6.9 dB**, not 10–15 dB. The document appears to apply `20 log10` directly to the bias ratio.

   **How to verify:** State separately acceleration-transmissibility dB and rectified-bias/power reduction, then propagate the selected isolator’s measured transfer function through `Γ₂∫PSD df`.

6. **Major · High confidence · §3.2, §6.1, H-3 — tracker measurements are treated as hard caps**

   **Claim:** Block 128 “caps at 8000 Hz/s”; above that no available block survives, so severe-slam loss of lock is essentially unavoidable.

   **Finding:** The tracker study explicitly says the block-length sweep reports the largest **coarse tested all-detected grid point**, is non-monotonic, and is “not a closed-form limit.” It does not establish 8000 Hz/s as a cap. The evidence is synthetic fixture behavior with a ±4.08 kHz acquisition band, while production requires ephemeris wipe-off and different bandwidth/sequences.

   “No tested configuration demonstrated all-detection above 8000 Hz/s” would follow. “No available block length survives” and “unavoidable” do not.

   **How to verify:** Sweep finer ramp grids beyond 8000 Hz/s for all block lengths, with production sequences, block timing, acquisition/wipe-off architecture, and transient rather than constant ramps.

7. **Major · High confidence · §3.2 and §6.1 — worst-case heave and satellite rates are stacked without LOS/time/sign qualification**

   **Claim:** Peak vertical acceleration converts wholly to LOS Doppler rate and adds to the maximum overhead satellite drift.

   **Finding:** The equation should use `a·u_LOS`, not acceleration magnitude. At overhead, vertical acceleration projects strongly but the precise geometry, slam timing, antenna motion, and sign matter. Peak slam acceleration is also transient, whereas the tracker study injected constant ramps over blocks. Adding both maxima is a valid conservative screening bound, but not a prediction of routine loss or “structural” failure.

   **How to verify:** Replay time-aligned six-DOF vessel acceleration through real satellite LOS geometries and the correlator, reporting probability/duration of bound exceedance rather than a sum of independent maxima.

8. **Major · High confidence · §2.2–2.4, §6.1 — timer scaling is not a complete safety derivation**

   **Claim:** `T_ack` and `t_dr` should scale exactly as `1/v`, producing 3.5 s at 20 kn and 2.3 s at 30 kn.

   **Finding:** `1/v` is correct only when preserving a fixed distance budget. Neither PARAMS nor the safety case establishes that the original 10 s and 120 s values encode a fixed distance. PARAMS calls both weakly evidenced and requires actual hull manoeuvre and human-response testing. R5 recommends at least 3–5 s for alarm detection, recognition, and motor response as an assumed design building block.

   Thus:

   - 3.5 s at 20 kn is already inside the cited 3–5 s floor range;
   - 2.3 s at 30 kn is below it;
   - the derived safe-speed cap could be below 20 kn depending on the validated response floor and collision distance;
   - `t_dr` also bounds estimator/observation staleness, not merely distance travelled.

   **How to verify:** Build an explicit budget: detection latency + alarm latency + measured human response + helm/control transition + craft stopping/turning distance + obstacle/closing-speed assumptions. Solve for permissible speed rather than rescaling provisional timers.

9. **Major · High confidence · §2.2–2.4 — D46’s dwell instruction is not discharged**

   **Claim:** `dwell_clear` and `dwell_rearm` have no speed coupling and remain unchanged.

   **Finding:** D46 expressly ordered re-derivation of “`T_ack`, dwells” against the collision-time budget. Holding the dwells may ultimately be correct, but the document only labels them human-factors quantities; it does not analyze whether caution-clear delay, latch duration, or re-arm timing affects authority exposure at speed.

   **How to verify:** Trace each dwell through the full authority state machine and show whether it can delay revocation, permit premature regrant, or only delay recovery. Only recovery-only dwells can be declared non-safety-critical with respect to collision distance.

10. **Major · High confidence · §4.4, §6.1 — acceptance limits are mislabeled as authority protection limits**

   **Claim:** 5° heading error reaches the “denied 200 m PL”; 2° reaches the “aided 25 m PL.”

   **Finding:** These are campaign acceptance limits. PARAMS’ proposed per-epoch authority limits are 2.5°/100 m denied and 1°/12 m aided. The denied time happens to be nearly identical because both numerator and heading angle were halved:

   - acceptance: `200/(sin 5°·v)`;
   - authority: `100/(sin 2.5°·v)`.

   That numerical coincidence does not justify calling 200 m and 5° protection limits. The aided authority calculation is likewise close but not identical.

   Independently re-derived 5°/200 m times are correct:

   - 7 kn: about **637 s = 10.6 min**;
   - 20 kn: about **223 s = 3.7 min**;
   - 30 kn: about **149 s = 2.5 min**.

   **How to verify:** Present separate acceptance-envelope and authority-gate tables using exact `sin θ`, then propagate covariance/protection limits rather than treating the maximum allowed heading error as a persistent deterministic bias.

11. **Major · High confidence · §6.1–6.2 — the good-fix timeline conflicts with `t_dr` and unsupported covariance behavior**

   **Claim:** A good fix buys about 2.5 minutes before first reacquisition; P1 runs until the first reacquisition, then a 10–20 minute LEO convergence phase begins.

   **Finding:** The proposed 30 kn `t_dr` is about **28 s**. If no new absolute position-constraining observation arrives, autonomous authority expires far before the claimed 2.5-minute bridge. LEO velocity observations do not automatically reset “age of last absolute position-constraining observation.”

   Also, LEO convergence does not wait for position error to reach 200 m; observations begin contributing immediately. Conversely, a good initial covariance does not prove that no position-observability transient exists. The claimed 2.5-minute benefit assumes a persistent worst-case 5° error, ignores the initial 5–25 m error when claiming the “full 200 m” headroom, and lacks a covariance propagation/replay result.

   **How to verify:** Run the D47 scenario through the actual estimator and supervisor, logging HPL, heading PL, last absolute-position-observation age, authority state, and contribution of LEO measurements from the first post-loss epoch.

12. **Major · High confidence · §5.5 — “all six clear through one gating measurement” is false**

   **Claim:** One measured hull slam/vibration/trim environment clears all six 20 kn blockers.

   **Finding:** That measurement is necessary but does not clear:

   - oscillator `Γ₂`;
   - isolator transfer function and installed behavior;
   - IMU VRE/g-sensitivity;
   - human alarm response;
   - craft-specific collision/manoeuvre budget;
   - real tracker captures;
   - real-IMU estimator tuning;
   - heading-source performance.

   The document’s own individual clearing-evidence lists contradict the one-measurement summary.

   **How to verify:** Convert the six items into an evidence matrix and require every row’s hull, component, integration, human-factors, and replay/sea-trial evidence independently.

13. **Moderate · High confidence · §1.3 and report — 11.4× recovery does not prove the handoff assumed 10°**

   **Claim:** `cot(10°/2)=11.4`, “confirming” the handoff assumed roughly 10° heel.

   **Finding:** The identity is correct for the document’s idealized comparison: one sensitivity vector exactly vertical versus exactly horizontal, with the tilt axis producing maximum first-order horizontal projection. It shows that 11× is consistent with 10°, not that the handoff actually assumed that angle. Other mounting geometry, vector components, or a different reference comparison can yield the same factor.

   **How to verify:** Recover the handoff’s original derivation or assumptions. Specify the full 3-D sensitivity vector and pitch/roll axes, including mounting and survey uncertainty.

14. **Moderate · High confidence · §1.2 — raw `Δv=cδ` identity is correct, but comparison with horizontal VPL overstates its solution consequence**

   **Claim:** A reference excursion directly creates an apparent vessel velocity bias `cδ`, independent of band and vessel speed.

   **Finding:** Carrier cancellation is correct for a single raw range-rate observation. It is not automatically a horizontal vessel-velocity bias of the same magnitude. Receiver clock drift is a common state; leakage depends on satellite geometry, clock-state process model, observation timing, and transient bandwidth. Band independence also assumes the same fractional error reaches each receiver chain coherently; separate front ends/references or frequency-dependent electronics can violate that premise.

   The speed-independence claim is valid at the raw measurement level, but the table’s direct comparison to horizontal VPL is only a worst-case screening bound.

   **How to verify:** Inject clock steps, ramps, sinusoidal modulation, and rectified offsets into the multi-satellite estimator and report solution-domain velocity/position bias and protection-limit response.

15. **Moderate · High confidence · §3.1 — higher vessel speed does not inherently improve velocity conditioning**

   **Claim:** The larger vessel Doppler at 20 kn makes the velocity solution better-conditioned.

   **Finding:** Doppler sensitivity to receiver velocity is the LOS Jacobian, essentially independent of the receiver’s actual speed. Increasing the state value increases the measured Doppler offset but does not increase Fisher information when measurement noise and geometry are unchanged. It may improve relative fractional resolution in some implementation, but that is not demonstrated.

   **How to verify:** Compare estimator information matrices or Monte Carlo velocity covariance at 7, 20, and 30 kn with identical satellite geometry and measurement noise.

16. **Moderate · High confidence · §1.3, §3.4, report — R5 values are not incorporated or explicitly reconciled**

   **Claim:** Planing trim is 3–6°, RMS vibration 0.5–2 g or 3 g for the binding case, and the VN-100 is assumed roughly ±16 g.

   **Finding:** R5 provides materially different or stronger evidence:

   - efficient running trim **2–4°**, with 2–3° common optimum;
   - measured RMS point **0.44 g**;
   - full-scale severe peaks around 6–9 g at the measured station;
   - VN-100 powered shock and sine-vibration survival;
   - random vibration near **4.5 g RMS may saturate/filter-collapse**;
   - rigid mounting preferred;
   - VRE unspecified.

   Estimates may still be retained for a smaller bow or harsher sea state, but the document must explicitly state why it departs from R5 and distinguish survival shock from measurement full scale and navigation performance.

   **How to verify:** Add an R5 reconciliation table with “R5 value / envelope value / reason for difference / effect on verdict.”

17. **Moderate · High confidence · §3.3, P-2 — proposed `Q` increase of 10²–10⁴ is ungrounded**

   **Claim:** Planing process noise is likely 100–10,000× the near-truth-IMU optimum.

   **Finding:** D43 establishes that the low-Q optimum may not transfer to a real IMU; it does not establish the direction or magnitude. Since measured acceleration is propagated, physical slam magnitude alone does not determine unmodelled process noise. Output filtering, clipping, timing error, aliasing, coning/sculling, and IMU model residual do.

   **How to verify:** Remove the numeric range or derive it from residual acceleration spectra of the selected IMU on measured planing data, followed by NEES/NIS consistency tests.

18. **Moderate · High confidence · §6.1 — `v³–v⁴` rectification scaling is conditional, not an established craft law**

   **Claim:** Slam acceleration scales as `v^1.5–v²`, so rectified bias scales as `v³–v⁴`.

   **Finding:** The exponent is explicitly an estimate and is not supplied by R5. Even at fixed nominal sea state, acceleration depends on wavelength, encounter geometry, throttle reduction, trim, deadrise, loading, and operator behavior. RMS acceleration need not scale like peak acceleration. Squaring an assumed peak-scaling exponent to predict RMS-square bias compounds two unsupported steps.

   **How to verify:** Fit RMS acceleration PSD versus speed and sea state from hull measurements. Apply the quadratic law only after confirming oscillator `Γ₂`.

19. **Moderate · High confidence · §6.1, H-5 — “cap below 30 kn” is directionally sensible but not derived**

   **Claim:** The 2.3 s scaled acknowledgment time supports a denied-speed cap below 30 kn, while the 20 kn class remains the leading supported extension.

   **Finding:** A cap recommendation is prudent, but the derivation does not locate it. If the credible minimum response is 5 s, direct scaling from the provisional 10 s at 7 kn gives about 14 kn; if 3 s is accepted, the corresponding point is about 23 kn. Craft stopping and closing-speed budgets can move it further. Therefore the analysis supports “do not grant 30 kn denied authority on current evidence,” not a specific implication that Class B territory is safe.

   **How to verify:** Derive the cap from the explicit response/manoeuvre budget described in Finding 8.

20. **Moderate · High confidence · §5 consequence register — RF/antenna availability consequences are missing**

   **Claim:** The register captures every item forced by 20 kn operation.

   **Finding:** R5 identifies attitude jitter, pointing-rate limits, spray, washdown, Ku fade, and synchronized motion outages as high-speed concerns. The envelope discusses tracker loss from Doppler dynamics but registers no antenna-pattern/placement, radome/spray, stabilization, multi-band diversity, or motion-correlated observation-availability requirement. Those outages directly interact with convergence, `t_dr`, and the good-fix scenario.

   **How to verify:** Add an RF availability analysis using antenna beam patterns, hull attitude histories, blockage, spray/rain testing, and constellation diversity; route resulting requirements to baseline, BOM, tracker, and safety case.

21. **Moderate · High confidence · §5.5 and §6.3 — routing is incomplete for estimator/supervisor contracts**

   **Claim:** Requirements are adequately routed to baseline, BOM, safety case, params, and tracker.

   **Finding:** Several consequences need architecture/contracts ownership, not only those owners:

   - speed-class selector source, uncertainty, hysteresis, and fail-closed behavior;
   - clock transient/leakage monitor interface;
   - IMU heave feed-forward timing and covariance;
   - isolated-frame dynamic extrinsics;
   - definition of which LEO observation resets `t_dr`;
   - good-fix phase/state-machine behavior.

   **How to verify:** Map every new predicate/state/input to its owning module and message schema, then add architecture/contracts rows.

22. **Minor · High confidence · §6.1 — textual internal-consistency error**

   **Claim:** “no small-craft rarely runs a full planing hull at 30 kn…”

   **Finding:** The double negative reverses or obscures the intended statement.

   **How to verify:** Rewrite as either “small craft rarely run…” or state the intended sea-state limitation precisely.

## Re-derived chains that do hold

- `sin θ/(1−cos θ) = cot(θ/2)` is correct under the stated idealized mounting geometry.
- `cot 5° = 11.43`, so a 10° tilt is consistent with an approximately 11× benefit.
- Raw clock-induced range-rate error `Δv = cδ` is carrier- and vessel-speed-independent.
- Given the assumed `Γ₂` and RMS acceleration, `0.027`, `0.091`, and `0.137 m/s` are arithmetically correct.
- Fixed-distance collision time scales as `1/v`; applying that rule gives 3.5 s at 20 kn and 2.33 s at 30 kn.
- The 5°/200 m heading times at 7/20/30 kn are approximately 10.6/3.7/2.5 minutes.
- The convergence-distance table is correct: 20 minutes at 20 kn is about 12.35 km; 10–20 minutes at 30 kn is about 9.26–18.52 km.
- 100 km at 30 kn is about 1.80 hours.
- 500 km at 20 kn is about 13.5 hours.

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

codex
The main corrections are present in the visible rewrite. I’m now checking the exact source language and sweeping the whole document for uncited/legacy numbers, since that is where a nominal correction can still fail.
exec
/bin/bash -lc "rg -n \"8000|3 g|3g|0\\.44|48|47|mandatory|isolation|10–15|10-15|5–50|5-50|10–100|10-100|20–30|20-30|10–15 dB|10-15 dB|3\\.3|4\\.9|100–10|v³|v\\"'^|UNVERIFIED|R5|T_ack|t_dr|20 kn|30 kn|WITHDRAW|first draft|assum" docs/design/HIGH_SPEED_ENVELOPE.md && rg -n "8000|largest.*coarse|closed-form|4000|128|256" docs/studies tracker docs -g '"'*.md' | head -160 && rg -n \"0\\.44|100.?450|3.?5 s|response|RMS|8\\.62|4\\.3\" . -g '*R5*' -g '*.md' | head -220" in /home/od/work/leo-pnt
 succeeded in 0ms:
5:Unit: U-H1 · implements `DECISIONS.md` **D46** (20 kn) + **D47** (30 kn exploratory)
8:research `docs/research/R5-highspeed-dynamics.md`.
13:analyses for the **planing / semi-planing regime up to 20 kn (10.29 m/s)**, an **exploratory 30 kn
17:**evidence-supported** (traceable to R5's cited sources or a repo study) or **judgment** (engineering
18:inference, marked `[UNVERIFIED]`). Where 20 kn operation is not supportable on present evidence, this
21:**What the fix round changed (read first).** The first draft rested several load-bearing conclusions
22:on *assumed* constants that R5's sourced values contradict. The corrections move conclusions in both
25:- The vibration-rectification "shock isolation MANDATORY for integrity" conclusion **[SOFTENED]** —
26:  it rested on an assumed `a_rms = 3 g`; R5's measured RMS is **0.44 g**, at which the rectified bias
27:  is ~48× *under* the denied PL. Isolation is now **recommended-and-difficult**, driven by the
29:- The slam model **[corrected]** — R5 gives impact durations of **100–450 ms** (not the assumed
30:  5–50 ms), so slam energy is **low-frequency (< ~10 Hz)**: harder to isolate (below light-isolator
31:  resonances), and the low `f` gives *less* `1/(πfτ)` suppression, not more. Net: a *harder* isolation
32:  problem than the first draft implied.
36:- The isolation-dB figure **[corrected]** from 10–15 dB to **5–7 dB** (bias ∝ `a²`).
38:  **not located between 20 and 30 kn** — on R5's 3–5 s human-response floor it could be **14–23 kn**.
40:- The good-fix timeline **[reconciled]** with `t_dr`: authority still expires at `t_dr`, not at the
43:The baseline's displacement-hull assumption remains **load-bearing** and **is not relaxed** here.
47:## 0. Scope, régime definition, and R5 reconciliation
49:- **Speed envelope:** denied navigation up to **20 kn = 10.289 m/s** (`v20`); exploratory **30 kn =
55:  `[UNVERIFIED]`.
56:- **Master caveat.** R5 supplies a general HSC acceleration/vibration environment but its
58:  20–30 kn**, and R5 found **no** open campaign tabulating peak/RMS/A₁/₁₀ vs both speed *and* Hs for
60:  (R5 §1.3, §5.1, §6.2 "dead ends"). So the binding gating evidence remains a **measured slam /
64:### 0.1 R5 reconciliation table (value used · R5 anchor · why · effect on verdict)
66:| Envelope quantity | Value used here | R5 sourced anchor | Reconciliation | Verdict effect |
68:| Sustained planing trim | **2–4°** | 2–4° efficient (Savitsky/Ghassemi; 2.24° @ 35 kn); >6° "uneconomical" (VERIFIED) | Adopt R5; drop the first draft's 3–6°. | cot-benefit *larger* (29–57×); mounting rule strengthened |
69:| Continuous vertical RMS accel | **0.44 g** (small-RIB upside to ~1 g as judgment) | 0.44 g RMS MK V head seas @ Hs 0.9 m (VERIFIED point) | Adopt R5's measured RMS; the first draft's 3 g was a **peak/A₁-class** number misused as RMS. | rectification bias collapses to ~48× under PL → isolation no longer *mandatory-for-integrity* |
70:| Slam peak (crew/coxswain station) | **~3–9 g** (8.62 g worst measured) | MK V peak 8.62 g head; A₁/₁₀ 2.7–3.2 g "max safe" (VERIFIED) | Adopt R5; drop the first draft's unsourced "bow 10–20+ g" (R5's bow-sea column is *lower*, 3.94 g). Small-RIB bow could exceed 9 g but that is **judgment**. | stacking/IMU concerns rescaled to sourced g |
71:| Slam duration | **100–450 ms** | NSWCCD-80-TR-2014/026 (VERIFIED) | Adopt R5; drop 5–50 ms. Energy is now **< ~10 Hz**. | isolation *harder* (low-fn, large stroke); suppression weaker |
72:| Oscillator linear g-sensitivity `Γ` | **~1×10⁻⁹/g** (good SC-cut OCXO) | Wenzel tip-over ~1e-9/g; NIST 1e-8…1e-10/g (VERIFIED) | Adopt R5 as the primary, sourced mechanism. | linear term becomes the lead concern |
73:| Oscillator quadratic `Γ₂` | **~1×10⁻¹¹/g² (judgment only)** | **Not in R5**; FE-5680A datasheet has *no* g-sensitivity row (VERIFIED absence) | Keep as an explicitly unsourced hypothesis; do **not** hang an integrity verdict on it. | rectification demoted to "measure it" |
75:| IMU (VN-100) behaviour | rigid-mount preferred; **~4.5 g RMS random saturates**; VRE unspecified | VN-100 datasheet (VERIFIED) | Adopt R5's *sourced* saturation limit in place of the first draft's assumed "±16 g FS bow saturation". | IMU concern re-anchored; co-isolation now *in tension* with vendor guidance |
76:| Human alarm-response floor | **3–5 s** | R5 §5.2 (detection+recognition+motor, mixed VERIFIED/ASSUMED) | Adopt as a floor; do not scale below it. | 20 kn already at floor; cap not located at 20–30 kn |
77:| Small-craft stopping/turning | order boat-lengths (no metric table) | R5 §5.1 dead end (IMO ship rules don't apply to RIBs) | Keep order-of-magnitude only; require craft trial. | collision budget stays craft-specific `[UNVERIFIED]` |
78:| Translational Doppler @30 kn | **583 Hz Ku / 83 Hz L** (`v/c·f`) | R5 §6.3 says "~1.5 Hz Ku" — **arithmetic slip** (`5.1e-8×12e9 = 610 Hz`) | Reject R5's number, keep R5's *qualitative* point (attitude jitter/multipath/spray dominate over translational Doppler). Per D5, non-Grok arithmetic confirmed. | RF-availability rows added to §5 |
108:the excursion's bandwidth relative to the drift random-walk. Band-independence further assumes the
117:At R5's sourced planing trim of **2–4°**, with `Γ` vertical, a sustained tilt `θ` changes the along-`Γ`
123:| 2° (R5 optimum) | **57×** | 6.1×10⁻¹³ → 1.8×10⁻⁴ m/s |
124:| 4° (R5 efficient) | **29×** | 2.4×10⁻¹² → 7.3×10⁻⁴ m/s |
127:**Evidence-supported:** the `cot(θ/2)` identity, and that at R5's 2–4° trim the benefit (29–57×) is
130:is *consistent with* ~10° heel; it does **not prove** the handoff assumed that angle. **Conclusion:
135:R5 gives sourced slam statistics (MK V, Hs ≈ 0.9 m, coxswain station): continuous **RMS 0.44 g**,
136:A₁/₃ 2.9 g, A₁/₁₀ 4.3 g, **peak 8.62 g** (head seas), **impact duration 100–450 ms** → dominant content
137:**below ~10 Hz**. A small RIB at 20–30 kn in the Danish straits may differ (bow, chop, loading); those
138:deltas are **judgment**, `[UNVERIFIED]` pending hull measurement.
140:**(a) Linear g-sensitivity — sourced, and the lead concern.** With `Γ ≈ 1×10⁻⁹/g` (R5), a slam of peak
141:`a` gives instantaneous `δ = Γ·a` → instantaneous `Δv = c·Γ·a`. Because R5's durations put the content
142:at **1–5 Hz**, the `1/(π·f·τ)` suppression is *weaker* than the first draft assumed (low `f` = less
148:| 4.3 g (A₁/₁₀) | 1.29 m/s | 0.41 m/s | 0.21 m/s | 0.082 m/s |
153:raise the reference's close-in phase noise (R5's sourced `L(f_v)=20log₁₀(Γ·a·f₀/2f_v)`), degrading
162:DC offset `δ_DC = Γ₂·⟨a²⟩` (un-suppressed by `1/(πfτ)`). But **R5 supplies no oscillator `Γ₂`**, and the
163:FE-5680A datasheet has no g-sensitivity row at all (R5 §3.1, VERIFIED absence). Using the sourced
164:**RMS 0.44 g** and a *hypothetical* `Γ₂ = 1×10⁻¹¹/g²`:
166:`δ_DC = 1.9×10⁻¹²` → **`Δv = 5.8×10⁻⁴ m/s` — ~48× *under* the 0.028 m/s denied PL.**
168:The first draft's "0.027 m/s, at the PL, isolation MANDATORY" used `a_rms = 3 g`, which is a **peak/
170:the rectified bias is negligible; it reaches the PL only near `a_rms ≈ 3 g` **RMS**, which R5 does not
172:a proven integrity breach; it becomes material only if a small-RIB `a_rms` is many-fold R5's 0.44 g or
173:`Γ₂` is far worse than assumed. Both require a shaker test of the selected part over the measured hull
176:### 1.5 Is shock isolation required? — recommended, and hard [SOFTENED + HARDENED]
183:**And isolation is harder than the first draft claimed.** R5 (§3.4, VERIFIED): the 100–450 ms slams put
184:energy **below ~10 Hz**, *below* many light-isolator resonances, so effective isolation needs a **low
186:compact shock mounts for electronics are difficult for long-duration wave slam. A ~10–15 Hz corner (first
189:a genuine design tension, `[UNVERIFIED]`, resolvable only against the measured spectrum.
191:**Architecture caveat (finding 4 / triage F9). [HARDENED]** The first draft's shared reference+IMU
192:isolated plate **conflicts with R5's VN-100 guidance**: VectorNav prefers a **rigid** IMU mount, warns
193:soft isolation degrades AHRS via relative motion and filter lag, and notes ~**4.5 g RMS random can
194:saturate** the accelerometers (R5 §4.2, VERIFIED). So the correct split is likely **isolate the
195:oscillator; keep the IMU rigidly mounted (or use a low-VRE IMU that does not need isolation)** — not one
205:`SAFETY_CASE.md §0` requires the timer budgets to be re-derived if the hull planes. The first draft
207:budget, and they do not** — `PARAMS_PROPOSAL.md §3.5/§2.2` calls `T_ack = 10 s` and `t_dr = 120 s`
214:Distances (order-of-magnitude; R5 §5 confirms no published small-craft crash-stop table exists):
216:| Quantity | 7 kn | 20 kn | 30 kn |
223:**Human-response floor (R5 §5.2, sourced-but-mixed): 3–5 s** for detection + recognition + motor
224:response. This bounds `T_ack` from below regardless of speed.
226:### 2.2 The safe-speed cap is not located at 20–30 kn [HARDENED]
228:If — *and only if* — the displacement `T_ack = 10 s` at 7 kn encodes a fixed distance `D = 36 m`, then
229:`T_ack(v) = D/v` must stay `≥` the human floor, giving a cap `v ≤ D/floor`:
231:| Human floor | Implied `T_ack` at 20 kn | Implied cap speed |
236:**What is supported (evidence + this budget):** (i) at 20 kn a distance-preserving `T_ack ≈ 3.5 s` is
237:**already at or below the 3–5 s human floor**, so the collision-timer margin at 20 kn is thin and needs
238:an explicit budget, not a rescaling; (ii) at 30 kn the implied `2.3 s` is **below any credible floor**;
240:craft stopping/closing budget — it is **not** established to sit specifically between 20 and 30 kn. The
241:supported conclusion is therefore *"do not grant denied autonomous authority at 30 kn, and derive the
242:cap from an explicit budget — it may be below 20 kn,"* not a specific 20-vs-30 boundary.
253:| `T_ack` | 10 s | **derive from §2.1 budget; ≥ 3–5 s floor** | **not** a `1/v` rescale; `[UNVERIFIED]` |
254:| `t_dr` | 120 s | **derive from budget + gap replay** | bounds collision exposure *and* observation staleness (finding 8); `[UNVERIFIED]` |
257:| caution band | denied 75/60 m | per-profile + speed lead-distance (routed) | `PARAMS §5.1`; `[UNVERIFIED]` |
261:D46 ordered re-derivation of "`T_ack`, dwells" against the collision budget. Tracing each dwell through
262:the `SAFETY_CASE.md §2.2` state machine shows **why** the dwells hold while `T_ack`/`t_dr` scale:
276:## 3. Estimator envelope at 20 kn
280:| Quantity | 7 kn | 20 kn | Satellite (550 km) |
285:(These confirm R5's *qualitative* §6.3 point — translational Doppler is a small perturbation — while
286:correcting R5's arithmetic slip: 30 kn Ku is 583 Hz, not "1.5 Hz"; see §0.1.)
288:**Correction (finding 15).** The first draft claimed the larger vessel Doppler at speed makes the
298:block and 8000 Hz/s at the 128-sample block**, but states explicitly these are the **"largest
306:transient, the study injected constant ramps; finding 7). Using **R5-sourced** heave peaks:
310:| 4.3 g (A₁/₁₀) | 1592 Hz/s | **5310 Hz/s** | exceeds 256-block (4000); within 128-block (8000) |
311:| 8.62 g (peak) | 3191 Hz/s | **6909 Hz/s** | within 128-block (8000) |
313:**Corrected reading:** at 20 kn with sourced g-levels the combined rate exceeds the 256-block grid point
320:### 3.3 Process-noise envelope — direction and magnitude are ungrounded [SOFTENED]
331:### 3.4 IMU requirements delta — re-anchored to R5's sourced VN-100 data [corrected]
333:R5 §4.2 (VERIFIED, VN-100 datasheet): powered shock survival **500 g**, sine vibration **6 g** operating,
335:preferred**, isolation can degrade AHRS, and **VRE is unspecified**. Re-anchored delta:
337:| Spec line | Why it matters (planing) | Status vs R5 |
339:| **Random-vib saturation (~4.5 g RMS)** | R5's *sourced* failure mode: broadband slam vibration approaching 4.5 g RMS saturates accels and collapses the AHRS filter → propagation loss. Replaces the first draft's unsourced "±16 g FS bow saturation." | **sourced concern**; hull `a_rms` vs 4.5 g must be measured |
340:| **VRE (µg/g²)** | DC accel bias under vibration → velocity drift; **unspecified on the VN-100 datasheet** → must be vendor-obtained or measured. | R5: unspecified `[UNVERIFIED]` |
341:| **Gyro g-sensitivity (°/s/g)** | Accel-dependent gyro bias under slam → heading drift (the handoff's weakest link). MEMS weakness; datasheet-bound or measure. | `[UNVERIFIED]` |
342:| **Mounting** | R5: **rigid preferred**; soft-isolating the IMU degrades AHRS. Contradicts a shared soft plate (§1.5). | R5 VERIFIED |
343:| **Output rate / anti-alias** | Slam content < ~10 Hz (R5) is largely *below* Nyquist at 100 Hz, so aliasing is **less** severe than the first draft's 10–100 Hz assumption implied — but bandwidth still matters for the fast content and for capturing the slam shape. | re-scoped |
348:**VRE/gyro-g** are — all `[UNVERIFIED]` pending the hull PSD and vendor data. Rigid mounting is
349:indicated. A lower-VRE industrial/tactical MEMS (R5 lists ADIS/SBG classes) is the fallback if VRE/gyro-g
356:is 2.5°**, aided **1.0°** (`PARAMS §1.3`). At an *assumed* `ω = 30°/s` and `Δt = 100 ms` (both
357:`[UNVERIFIED]`, finding 11) the ~3° gyro-bridge lag exceeds the 1.0° aided PL and approaches the 2.5°
360:planing power). Registered as a degradation nuance (§5), tune `[UNVERIFIED]`.
364:## 4. Passage math at 20 kn
370:| Leg | 7 kn | 20 kn | 30 kn |
375:A 20-min convergence leg at 20 kn needs ~12.4 km of straight searoom (D46's "~12 km"). Feasible in open
383:≈ 18.42 m/s`, U-P1-internal, inherited, `[UNVERIFIED]`). Ephemeris cached at departure:
387:| 500 km @ 20 kn | 13.5 h | 13.5 h | 16.5 h — comfortable |
388:| 24 h denied (≈ 889 km @ 20 kn) | 24 h | 24 h | **6 h — tight** |
393:aging numbers are synthetic-only (`D43`/`D45`), `[UNVERIFIED]`.
398:(`PARAMS`: denied 100 m / 2.5°, aided 12 m / 1.0°) — not the acceptance envelopes the first draft
401:| To reach... | authority PL / error | 7 kn | 20 kn | 30 kn |
406:(The times are numerically ≈ the first draft's, because halving both numerator and angle cancels — a
409:**Good-fix reconciliation (finding 11) [corrected].** The D47 scenario enters from a good GPS fix, so at
411:governed by **`t_dr` — the age of the last *absolute position-constraining* observation** — and a LEO
412:Doppler epoch is a *range-rate* (velocity) observation. **Whether a LEO observation resets `t_dr` is an
414:Under the strict reading (LEO velocity does **not** reset `t_dr`), authority **expires at `t_dr`
417:transient — but it does **not** extend authority past `t_dr`. The first draft's "good fix buys ~2.5 min
419:authority still cycles at `t_dr` unless LEO observations are credited as position-constraining."*
423:Every course change resets position convergence (baseline). At 20 kn the denied 100 m PL is reachable
425:frequently-manoeuvring water **continuous denied position authority at 20 kn is not achievable**; options
433:Each item routed and status-scoped as **evidence-supported** or **judgment**; all `[UNVERIFIED]` remain
441:| B-2 | Require **oscillator** shock isolation (recommended, hard — §1.5); **not** a shared reference+IMU soft plate. | judgment (softened from "required") |
443:| B-4 | Decide denied-speed support: continuous vs long-leg-only vs **cap (which may be < 20 kn, §2.2)**. | **decision required** |
451:| M-1a | **Isolate the oscillator** (low-`fn`, large stroke for < 10 Hz slam energy; align `Γ` to the best isolator axis) — new line, absent today. | judgment; hard per R5 |
452:| M-1b | **Keep the IMU rigidly mounted** (R5/VN-100 vendor guidance) or select a low-VRE IMU that needs no isolation — do **not** co-isolate with the reference. | evidence-supported (R5) |
453:| M-2 | Verify hull broadband **`a_rms` vs the sourced ~4.5 g RMS VN-100 saturation limit**; verify **VRE** and **gyro g-sensitivity** (VN-100 VRE unspecified). | evidence-supported (R5) |
454:| M-3 | Obtain/measure oscillator **`Γ` and `Γ₂`** (FE-5680A has no g-sens datasheet row; shaker-test the selected part). | evidence-supported (R5 absence) |
461:| S-1 | Planing timer class **derived from an explicit response/distance budget** (§2.1), not a `1/v` rescale; the cap may be < 20 kn. | judgment |
464:| S-4 | Frequency-reference row: the sourced concern is **linear-g phase-noise availability + cycle slips** (§1.4a); rectification (H8) is a *measure-it* hypothesis, not a proven bias. | evidence-supported (R5) |
470:| P-1 | Add a Class-B timer column whose `T_ack`/`t_dr` are budget-derived (≥ 3–5 s floor), not rescaled. | judgment |
472:| P-3 | Planing tracker block length + IMU heave-rate feed-forward as steering-relevant `[UNVERIFIED]` (screening bound, §3.2). | judgment |
478:| A-1 | **Definition of which LEO observation resets `t_dr`** (position-constraining vs velocity-only) — governs the good-fix scenario (§4.3). | contracts |
481:| A-4 | Isolated-frame dynamic extrinsics (reference `Γ`/lever-arm under isolation motion, §1.5). | architecture |
484:### 5.6 High-speed effects the first draft omitted (review finding 20 / triage F15) [added]
486:R5 §6 and operational experience flag high-speed concerns beyond Doppler dynamics; each is a register
487:row or an explicit scope boundary, `[UNVERIFIED]`:
491:| E-1 | **Antenna attitude / servo-rate limits / slam-synchronised L-band & Ku fades** (R5 §6.1) — motion-correlated observation outages directly interact with convergence, `t_dr` and the good-fix scenario. | BOM/tracker/safety |
492:| E-2 | **Spray / washdown / rain fade** on L/Ku at planing speed (R5 §6.2 — no primary trial; risk row). | BOM/baseline |
495:| E-5 | **Crew human factors beyond `T_ack`** — R5's A₁/₁₀ "extremely uncomfortable"; ability to take the helm after a slam train degrades. | safety |
496:| E-6 | **Sea-state (Hs) coupling** — every slam number is Hs-conditional; class-rule `n_cg` scales with V *and* Hs (R5 §1.2). | analysis/baseline |
499:### 5.7 Plainly: what 20 kn does NOT support, and the clearing evidence (finding 12 corrected)
501:The first draft's "all six clear through one measurement" is **false** — the hull slam/vibration/trim
510:| Continuous denied position at 20 kn | tighter heading-source characterisation **or** an operational long-leg/cap decision |
515:**Until this evidence exists, fail closed to the displacement envelope; treat 20 kn as aided-only or
520:## 6. Exploratory tier — 30 kn (per D47)
522:D47 adds **30 kn = 15.433 m/s** as an **EXPLORATORY** tier (not supported), scenario of record: a
523:**100 km denied passage entered from a good GPS fix then loss, up to 30 kn** (~1.8 h). All §6 numbers
524:inherit §0's caveat and are *weaker* than the 20 kn ones — R5 gives no small-craft 30 kn dataset, and
527:### 6.1 Derivation chains at 30 kn (corrected)
529:| Quantity | 20 kn | 30 kn |
531:| Vessel Doppler at Ku | 389 Hz | **583 Hz** (R5 §6.3's "1.5 Hz" is an arithmetic slip, §0.1) |
534:**Trim / mounting:** at R5's 2–4° the `cot(θ/2)` benefit (29–57×) is undiminished; mounting is not the
535:30 kn problem.
537:**Slam scaling [SOFTENED to a stated hypothesis].** Impact acceleration is often modelled `v^1.5`–`v^2`
538:for fixed sea state (**judgment, not in R5**; finding 18). Anchoring on R5's sourced 20 kn-class peak
539:(8.62 g) scaled by 1.84–2.25× → **~16–19 g peak** at 30 kn — a hypothesis requiring hull measurement,
540:`[UNVERIFIED]`.
542:**Vibration rectification [SOFTENED — the first draft's "3–5× over PL" is withdrawn].** With R5's
543:*sourced* RMS 0.44 g scaled by the same (unsourced) factor → `a_rms ≈ 0.8–1.0 g`; with the hypothetical
544:`Γ₂ = 1e-11/g²`: `Δv_DC ≈ 0.002–0.003 m/s` — still **~10× under** the denied PL. The first draft's
545:0.09–0.14 m/s (3.3–4.9× over) came from mis-using 3 g as RMS *and* squaring an unsourced peak-scaling
546:exponent (compounding two unsupported steps, finding 18). **On sourced RMS, 30 kn rectification is not at
547:the PL either.** The 30 kn reference concern is the same as 20 kn — *linear-g phase noise / cycle slips
548:and harder isolation* — scaled up, not a proven integrity breach.
551:transmissibility `1/√R`, i.e. **amplitude attenuation `10·log₁₀(R)`** — for `R = 3.3–4.9` that is
552:**~5–7 dB**, not the first draft's 10–15 dB (finding 5 / triage F4). (And since rectification is no
555:**Heave-Doppler stacking [SOFTENED to a screening bound].** With R5-sourced peaks scaled to ~16–19 g,
557:grid point (8000 Hz/s at block-128)**. Correct statement: *"the conservative screening bound exceeds any
558:tested all-detect configuration, so severe-slam loss of lock at 30 kn is a real risk requiring IMU
560:study is explicitly not a closed-form cap; finding 6). At 30 kn IMU heave-rate feed-forward moves from
563:**Collision / cap [HARDENED to honest indeterminacy].** Scaled distance-preserving `T_ack ≈ 2.3 s` is
564:**below any credible 3–5 s human floor** → **do not grant 30 kn denied autonomous authority.** But the
565:cap is **not located at 20–30 kn**: on a 5 s floor it is ~14 kn; on 3 s, ~23 kn (§2.2). The supported
566:claim is "no 30 kn denied authority on current evidence," not "Class-B/20 kn territory is proven safe."
568:**Heading-vs-authority-PL time.** denied 100 m / 2.5°: **149 s (2.5 min)** at 30 kn (vs 223 s at 20 kn)
571:**Convergence / good-fix.** A 10–20 min leg at 30 kn = **9.3–18.5 km = 9–19 % of the 100 km passage**
572:(vs 6–12 % at 20 kn). The good-fix start removes cold convergence and gives aided-grade start accuracy —
573:but per §4.3 authority still expires at `t_dr`, not at the 2.5 min heading time.
577:Passage: **100 km @ 30 kn = 1.80 h**; @ 20 kn = 2.70 h. Phase timeline (open-water; **judgment**, classes
578:`[UNVERIFIED]` pending replay):
580:| Phase | Trigger | Position class @ 20 kn | Position class @ 30 kn | Authority-limiting factor |
583:| P1 DR bridge | fix loss → first re-acq | starts ~5–25 m, decays | decays 1.5× faster | **`t_dr` (§4.3), not heading time** |
587:**Position class is the same at 20 and 30 kn** (aided at loss → denied cruise). 30 kn changes *timing and
591:**Honest verdict on class vs margins.** For the **estimator/passage** layer, 30 kn **tightens margins**
593:**not the class change the first draft claimed** — on sourced RMS the rectification stays under the PL at
594:both speeds; the real, sourced degradation (linear-g availability, harder isolation, slam loss-of-lock
596:**safety** layer, 30 kn is over the human-response floor → **denied cap**, but the cap is not shown to sit
597:specifically at 20–30 kn.
599:### 6.3 30 kn consequence-register additions (beyond §5)
601:| # | 30 kn delta | Status |
603:| H-1 | Isolation must attenuate ~**5–7 dB** more than the 20 kn contingency (corrected from 10–15 dB) — and only if measurement shows rectification approaches the PL. | judgment (contingent) |
604:| H-2 | Oscillator `Γ₂` **measured** matters more as `a_rms` rises, but even scaled it is ~10× under PL on sourced RMS — measure to confirm, don't assume breach. | evidence-supported (R5 absence) |
606:| H-4 | Hull `a_rms` vs the **4.5 g RMS VN saturation** limit is the binding IMU question at 30 kn (scaled 0.8–1 g RMS is still under it, but bow/severe seas unknown). | evidence-supported (R5) |
607:| H-5 | **Denied-speed cap** (may be < 20 kn); no third autonomous-denied class. | judgment |
608:| H-6 | Exploratory-tier trial gate: no 30 kn denied trial until 20 kn is cleared and H-1..H-5 + §5.6 are evidenced. | judgment |
611:right framing for any 30 kn work.
615:## Verdict — 20 kn
617:**20 kn denied operation is NOT supportable on present evidence — but the reason is narrower and better-
618:sourced than the first draft claimed.** With R5's sourced values the frequency-reference case
620:rectification "integrity breach / mandatory isolation" conclusion is **withdrawn** — at R5's measured
621:0.44 g RMS the rectified bias is ~48× under the denied PL, and its magnitude rested on an unsourced `Γ₂`
624:**tracker-availability / cycle-slip** problem, for which shock isolation is **recommended but hard**
625:(R5's 100–450 ms slams put energy below ~10 Hz, needing low-`fn`/large-stroke mounts, and vendor guidance
626:prefers a *rigid* IMU — so isolate the oscillator, not a shared plate). What genuinely blocks 20 kn
627:denied today: (1) an **unmeasured hull** slam/vibration/trim/manoeuvre environment (R5 has no small-craft
628:20–30 kn dataset); (2) an **IMU** whose broadband `a_rms` vs the sourced ~4.5 g RMS saturation limit and
630:~3.5 s at 20 kn — already at/below R5's 3–5 s human floor — so the safe-speed cap must be derived from an
631:explicit budget and **may be below 20 kn**; (4) **continuous denied position authority** that heading
636:closed to the displacement envelope and treat 20 kn as aided-only or unproven.
638:## Verdict — 30 kn exploratory tier
640:**Do not grant 30 kn denied autonomous authority on current evidence; 30 kn is aided/manual-only and
641:exploratory.** Corrected against R5, 30 kn does **not** change the estimator/passage conclusion *class*
643:convergence), and — contrary to the first draft — it is **not** a hardware *class change* either: on
644:sourced RMS the vibration-rectification bias stays ~10× under the PL at 30 kn as at 20 kn, the required
645:extra isolation is ~5–7 dB (not 10–15 dB), and the tracker "no block survives" claim is withdrawn as the
646:study is explicitly not a closed-form cap — the honest 30 kn tracker statement is that a *conservative
649:30 kn conclusion is on the **human-response floor**: a distance-preserving `T_ack ≈ 2.3 s` is below R5's
650:3–5 s floor, so denied autonomous authority at 30 kn is not defensible — but the analysis **does not
651:locate the cap at 20–30 kn**; on a 5 s floor it could be ~14 kn. 30 kn remains scoping analysis, gated
652:behind full clearance of the 20 kn envelope plus a measured slam spectrum and requirements H-1..H-6.
rg: tracker: No such file or directory (os error 2)
docs/studies/tracker/STUDY.md:3:Date: 2026-07-23. Harness: `pnt-studies` schema 1. Fixture: Fs = 8192 Hz, 256-sample
docs/studies/tracker/STUDY.md:4:PN/BPSK reference, 256 frequency rows at 32 Hz spacing, ±4080 Hz acquisition band,
docs/studies/tracker/STUDY.md:5:threshold Q = 32, and ±128 Hz tracking window. Full command:
docs/studies/tracker/STUDY.md:25:- The archived 4000-block probe is reproduced (median 11.50, p99 15.77). At one million
docs/studies/tracker/STUDY.md:30:- At this fixture, all 16 blocks track through 4000 Hz/s; only 8/16 survive at 8000 Hz/s.
docs/studies/tracker/STUDY.md:80:For one frequency row with N=256 i.i.d. exponential delay powers, let
docs/studies/tracker/STUDY.md:93:The reported prediction conservatively unions this over 256 frequency rows. Observations:
docs/studies/tracker/STUDY.md:143:The 256-sample fixture tracked all blocks at 4000 Hz/s and half at 8000 Hz/s. Block-length
docs/studies/tracker/STUDY.md:144:sweeps found largest all-detected grid points of 4000, 8000, 4000, and 2000 Hz/s for
docs/studies/tracker/STUDY.md:145:64, 128, 256, and 512 samples respectively; these non-monotone coarse-grid results show
docs/studies/tracker/STUDY.md:146:interaction among coherent smear, bin spacing, and extrapolation and are not a closed-form
docs/studies/tracker/STUDY.md:151:At 8000 Hz/s loss occurred before midpoint crossing and 9 wrong-lock blocks followed.
docs/design/SAFETY_CASE.md:212:actually does on `GPS_INPUT` silence was [UNVERIFIED] at first writing — now SITL-characterised; see the 2026-07-23 amendment below.** `GPS_TIMEOUT_MS` (4000 ms) is a GPS-backend *data*
docs/design/SAFETY_CASE.md:405:`t_lease` shall be `< GPS_TIMEOUT_MS` (4000 ms) so that in Case A the companion self-revokes
docs/design/SAFETY_CASE.md:495:- **Timers:** `t_lease` (`< 4000 ms`), `t_dr` (DR-authority), per-source freshness deadlines,
docs/HANDOFF_PROMPT_BLADERF.md:116:  4000 ms, so fixes arriving minutes apart cannot be published raw.
docs/design/HIGH_SPEED_ENVELOPE.md:34:  explicitly calls its block sweep "not a closed-form limit" on a ±4.08 kHz fixture band; the stacked
docs/design/HIGH_SPEED_ENVELOPE.md:297:The tracker study (`tracker/STUDY.md §3`) all-detects a Doppler ramp to **4000 Hz/s at the 256-sample
docs/design/HIGH_SPEED_ENVELOPE.md:298:block and 8000 Hz/s at the 128-sample block**, but states explicitly these are the **"largest
docs/design/HIGH_SPEED_ENVELOPE.md:299:all-detected coarse grid points … not a closed-form limit,"** non-monotone, on a **±4.08 kHz fixture
docs/design/HIGH_SPEED_ENVELOPE.md:310:| 4.3 g (A₁/₁₀) | 1592 Hz/s | **5310 Hz/s** | exceeds 256-block (4000); within 128-block (8000) |
docs/design/HIGH_SPEED_ENVELOPE.md:311:| 8.62 g (peak) | 3191 Hz/s | **6909 Hz/s** | within 128-block (8000) |
docs/design/HIGH_SPEED_ENVELOPE.md:313:**Corrected reading:** at 20 kn with sourced g-levels the combined rate exceeds the 256-block grid point
docs/design/HIGH_SPEED_ENVELOPE.md:314:but sits **inside** the 128-block one, so **shorter blocks plausibly cope** — the opposite of the first
docs/design/HIGH_SPEED_ENVELOPE.md:557:grid point (8000 Hz/s at block-128)**. Correct statement: *"the conservative screening bound exceeds any
docs/design/HIGH_SPEED_ENVELOPE.md:560:study is explicitly not a closed-form cap; finding 6). At 30 kn IMU heave-rate feed-forward moves from
docs/design/HIGH_SPEED_ENVELOPE.md:646:study is explicitly not a closed-form cap — the honest 30 kn tracker statement is that a *conservative
docs/design/ARCHITECTURE.md:178:| 5 | Signal trackers | `pnt-tracker` | Shipped (U-T1, D36): FFT correlation-peak Doppler over a frequency/delay grid with phase refinement, `NoDetection` below threshold. Synthetic-IQ validated over 2000-4000 Monte-Carlo seeds; real PSS/SSS/beacon sequences are `[UNVERIFIED]` (U-R4 research only, no real-signal decoder in-tree). OneWeb tracking remains gated on the un-run 24-hour occupancy survey. |
docs/design/PARAMS_PROPOSAL.md:170:(4000 ms). Frames renew at the DR-fill cadence (every accepted IMU propagation, ~10 ms at the
docs/design/PARAMS_PROPOSAL.md:376:**256 delay bins × 256 frequency bins** (Fs = 8192, N = 256, 32 Hz coarse grid). Measured
docs/design/PARAMS_PROPOSAL.md:377:noise-only quality statistics over 4000 pure-noise blocks: **median 11.5, p99 15.7, max 20.0**
docs/design/PARAMS_PROPOSAL.md:380:> **Provenance caveat (carried from the artifact).** The 4000-block quantiles are **review
docs/design/PARAMS_PROPOSAL.md:390:**Statistic → Fisher's g.** Within one frequency hypothesis, the N_d = 256 circular-correlation
docs/design/PARAMS_PROPOSAL.md:407:quality the tracker also **maximises over the N_f = 256 frequency rows**, so the per-block PFA
docs/design/PARAMS_PROPOSAL.md:424:Inverting `P_block` for the quantile thresholds and comparing to the empirical 4000-block
docs/design/PARAMS_PROPOSAL.md:431:| max (≈1/4000) | 2.5e-4 | 20.14 | **20.0** |
docs/design/PARAMS_PROPOSAL.md:458:**This 5.3e-9 is the analytic-model figure, not an empirically validated PFA.** 4000 noise
docs/design/PARAMS_PROPOSAL.md:465:fixture block is 256/8192 = 31.25 ms (~32 blocks/s per tracked signal), so the model per-tracker
docs/design/PARAMS_PROPOSAL.md:494:- **A4 — fixture geometry ≠ production.** Fs = 8192, N = 256, 32 Hz grid, 256×256 cells are the
docs/design/PARAMS_PROPOSAL.md:498:- **A5 — evidence provenance and reproducibility.** The 4000-block quantiles are the U-T1 deep-
docs/studies/tracker/STUDY.md:3:Date: 2026-07-23. Harness: `pnt-studies` schema 1. Fixture: Fs = 8192 Hz, 256-sample
docs/studies/tracker/STUDY.md:4:PN/BPSK reference, 256 frequency rows at 32 Hz spacing, ±4080 Hz acquisition band,
docs/studies/tracker/STUDY.md:5:threshold Q = 32, and ±128 Hz tracking window. Full command:
docs/studies/tracker/STUDY.md:25:- The archived 4000-block probe is reproduced (median 11.50, p99 15.77). At one million
docs/studies/tracker/STUDY.md:30:- At this fixture, all 16 blocks track through 4000 Hz/s; only 8/16 survive at 8000 Hz/s.
docs/studies/tracker/STUDY.md:80:For one frequency row with N=256 i.i.d. exponential delay powers, let
docs/studies/tracker/STUDY.md:93:The reported prediction conservatively unions this over 256 frequency rows. Observations:
docs/studies/tracker/STUDY.md:143:The 256-sample fixture tracked all blocks at 4000 Hz/s and half at 8000 Hz/s. Block-length
docs/studies/tracker/STUDY.md:144:sweeps found largest all-detected grid points of 4000, 8000, 4000, and 2000 Hz/s for
docs/studies/tracker/STUDY.md:145:64, 128, 256, and 512 samples respectively; these non-monotone coarse-grid results show
docs/studies/tracker/STUDY.md:146:interaction among coherent smear, bin spacing, and extrapolation and are not a closed-form
docs/studies/tracker/STUDY.md:151:At 8000 Hz/s loss occurred before midpoint crossing and 9 wrong-lock blocks followed.
docs/research/R4-signal-structures.md:46:- **Structure:** Cyclic prefix + **8 repetitions** of a length-\(N/8 = 128\) subsequence; **CP and first repetition polarity-inverted**.
docs/research/R4-signal-structures.md:47:- **Subsequence source:** Length-**127** maximal m-sequence from 7-stage Fibonacci LFSR with primitive polynomial **\(1 + D^3 + D^7\)**, initial state \((a_{-1},\ldots,a_{-7}) = (0,0,1,1,0,1,0)\). Append 0 → 128-bit hex:
docs/research/R4-signal-structures.md:61:- Successors (Neinavaie, Kassas “full OFDM beacon”) estimate **additional default/pilot structure** by capture when demand is low—**not fully closed-form published like PSS/SSS**.
docs/research/R4-signal-structures.md:273:| Constellation | Best published reference for correlation | Rate / period | Fully closed-form? | 2.5–5 MHz OK? | Always present? |
./docs/research/R4-signal-structures.md:128:| **Ring Alert (IRA)** | Beam schedule ~**90 ms** step across beams; **per-beam revisit ~4.32 s**; burst length often cited **~2.56 ms** (Orabi) or **7–20 ms** class (Decode Systems—treat exact length as **uncertain**) | Unencrypted broadcast used heavily in SoOP / spoofing-detection literature |
./docs/research/R4-signal-structures.md:243:### 4.3 Published SS hex (correlator seed) **[VERIFIED]**
./docs/research/R4-signal-structures.md:276:| **Iridium** | 12-sym BPSK UW + optional tone; or M-power Doppler | **Burst ~4.32 s/beam**; TDMA 90 ms lattice | **Partial** (UW in FOSS; tone length fuzzy) | **Yes** (channel ~35 kHz; band ~8.5 MHz total) | Simplex **periodic**, not continuous |
./docs/research/R5-highspeed-dynamics.md:23:| Peak (g) | **8.62** | 3.94 | 6.02 | 1.67 | 1.51 |
./docs/research/R5-highspeed-dynamics.md:24:| Average 1/10 highest (g) | **4.30** | 2.48 | 2.39 | 1.56 | 1.26 |
./docs/research/R5-highspeed-dynamics.md:27:| RMS (g) | **0.44** | 0.38 | 0.32 | 0.24 | 0.23 |
./docs/research/R5-highspeed-dynamics.md:39:- Duration matters for structural and equipment response (SRS / resonant systems).
./docs/research/R5-highspeed-dynamics.md:45:- Standardized stats: peaks, **A₁/₁₀**, **A₁/₃**, RMS, impact count / ICI, Ride Severity Index.  
./docs/research/R5-highspeed-dynamics.md:69:Rationale: long field pulses (~100–150+ ms, peak often <10 g) mapped via **shock response spectrum (SRS)** to shorter higher-amplitude lab pulses that commercial shock machines can produce. Margins: **1.2** (measurement/processing) × **1.5** (lab vs sea uncertainty).
./docs/research/R5-highspeed-dynamics.md:76:- Secondary reviews (e.g. seakeeping criteria papers citing 0.2 g RMS bridge vertical).  
./docs/research/R5-highspeed-dynamics.md:78:- Commonly cited **RMS vertical acceleration limit ~0.2 g** (personnel/ship seakeeping).  
./docs/research/R5-highspeed-dynamics.md:79:- Same NATO paper **explicitly questions** applying ship/passenger RMS and ISO 2631-style limits to **high-Froude planing HSC** injury risk; crest factors ≫ 3; impact statistics preferred.  
./docs/research/R5-highspeed-dynamics.md:80:- Alternate RMS comfort figure cited: **0.3 g rms** (Mandel 1979, via Peterson).  
./docs/research/R5-highspeed-dynamics.md:82:**ASSUMED (class rules structure, exact formula coefficients not re-derived here from paid ISO/ABS text):** Classification / ISO small-craft rules (ABS HSC, ISO 12215-5, DNV HSLC) use a **design vertical acceleration at LCG (n_cg)** that increases with **V** and **Hs (h₁/₃)** and depends on deadrise / length-beam factors — used for **bottom slamming pressure**, not as an at-sea measured RMS. Design n_cg for hard planing patrol craft is often **several g**, not 0.2 g RMS.
./docs/research/R5-highspeed-dynamics.md:88:| RMS vertical (coxswain-class station, head seas, Hs~0.9 m, larger HSC) | **0.44 g** (MK V table) | VERIFIED data point; smaller RIBs may differ |
./docs/research/R5-highspeed-dynamics.md:90:| Peak single slam (severe head seas, larger craft) | **~6–9 g** class (MK V peak 8.62 g) | VERIFIED for that craft/sea; can be higher at bow |
./docs/research/R5-highspeed-dynamics.md:91:| Impact duration (deep-V, 6–50 t class) | **100–450 ms** | VERIFIED NSWCCD |
./docs/research/R5-highspeed-dynamics.md:93:| Personnel RMS criterion (ships) | **0.2 g RMS** STANAG | VERIFIED as criterion text; **poor** for slam injury on planing HSC |
./docs/research/R5-highspeed-dynamics.md:97:**Gap:** No single open primary campaign was found that tabulates peak/RMS/A₁/₁₀ **explicitly vs both 20–30 kn and Hs for small RIB/patrol boats** in one matrix. Use NSWCCD methods + craft-specific trials for envelope replacement of remaining [UNVERIFIED] cells.
./docs/research/R5-highspeed-dynamics.md:145:**Not published on commercial FE-5680A sheet:** acceleration sensitivity Γ (fractional Δf per g), vibration PSD response, or shock survival.  
./docs/research/R5-highspeed-dynamics.md:191:**Marine slam note (VERIFIED context from §1):** Pulse durations **100–450 ms** imply significant energy **below ~10 Hz** — **below many light isolator resonances**, so isolation must be designed for **low-fn + large excursion**, not only high-frequency engine vibration. NSWCCD warns long-duration wave slam makes compact shock mounts for electronics **difficult** (large stroke needed).
./docs/research/R5-highspeed-dynamics.md:202:- **VRE** = accelerometer response to AC vibration that **rectifies to DC**, appearing as anomalous **bias/offset**.  
./docs/research/R5-highspeed-dynamics.md:230:| Saturation warning | Random vib **~4.5 g RMS** can **saturate accelerometers** → filter collapse |
./docs/research/R5-highspeed-dynamics.md:235:### 4.3 Anti-vibration mounting practice
./docs/research/R5-highspeed-dynamics.md:247:**ASSUMED workflow for envelope:** Use measured vertical accel time history (or half-sine train: A_peak, T=0.1–0.45 s, rate from wave encounter) as vibration input; apply manufacturer VRE model if available; else budget bias steps of order **mg-class** under multi-g RMS vib for industrial MEMS.
./docs/research/R5-highspeed-dynamics.md:287:- For **alarm acknowledgment timers**, a conservative design often uses **detection + recognition + motor response** ≥ **3–5 s** plus craft-specific stopping/turn distance — **ASSUMED** for UI policy unless project human-factors test exists.
./docs/research/R5-highspeed-dynamics.md:333:| Vertical RMS, moderate head seas, HSC | **~0.3–0.5 g** class (0.44 g MK V @ Hs~0.9 m) | VERIFIED point | Haupt MK V |
./docs/research/R5-highspeed-dynamics.md:336:| Slam duration | **100–450 ms** | VERIFIED | NSWCCD-80-TR-2014/026 |
./docs/research/R5-highspeed-dynamics.md:343:| IMU saturate | **~4.5 g RMS** random | VERIFIED | VN-100 datasheet |
./docs/research/R2-services-regs.md:249:### 4.3 Access method & terms
./docs/research/R6-unknown-emitter-array.md:15:| Usable DF accuracy on a small mast array (1–2 m) at VHF/UHF? | **Instrumental CI/MUSIC ~1–2° RMS ideal; field marine 3–15°+** depending multipath, SNR, aperture in wavelengths, platform motion. | Medium–High |
./docs/research/R6-unknown-emitter-array.md:36:| Published DF accuracy (manufacturer) | **No official RMS-degree specification** on main product pages | **[V]** | Product pages (absence of number is itself observed) |
./docs/research/R6-unknown-emitter-array.md:37:| Community localization claims | Forum anecdotes of **tens of metres** geo-location after multi-point mobile bearings (not single-fix bearing RMS) | **[A/V mixed]** | e.g. RadioReference thread citing “20–50 m” worst-case geo — treat as **anecdotal**, not metrology: http://forums.radioreference.com/threads/krakensdr-direction-finding-p25-is-it-possible.481595/ |
./docs/research/R6-unknown-emitter-array.md:74:| WiNRADiO WD-7200 class | 2 RX + commutated array | HF–VHF/UHF options | Professional | Quasi-coherent correlative interferometer | Spec: **typ. 2° RMS**, instrumental **<0.5°** (reflection-free) | **[V]** https://winradio.com/home/wd7200.htm |
./docs/research/R6-unknown-emitter-array.md:105:| WiNRADiO WD-7200 correlative | **Typ. 2° RMS**; instrumental **<0.5°** after cal | Reflection-free; specific antenna | **[V]** | https://winradio.com/home/wd7200.htm |
./docs/research/R6-unknown-emitter-array.md:107:| High-resolution interferometry analysis | **0.1° RMS** may need **~50 dB SNR** at λ/2 spacing — rarely available | Academic thesis result | **[V]** | https://open.metu.edu.tr/bitstream/handle/11511/112712/index.pdf |
./docs/research/R6-unknown-emitter-array.md:119:**Rule of thumb (engineering):** RMS bearing error scales roughly as  
./docs/research/R6-unknown-emitter-array.md:221:- TX powers: European DAB examples often **~kW RMS class** per site (much lower than peak FM in some comparisons). **[V]** industry comparison PDFs (GatesAir / ITU-D presentation classes).
./docs/research/R6-unknown-emitter-array.md:223:### 4.3 DVB-T2
./docs/research/R6-unknown-emitter-array.md:307:| **[V] VERIFIED** | Kraken price/freq/cal method; bladeRF/B210 specs/prices; R&S ~1° CI claim; WiNRADiO 2° RMS; BO-TMA observability theorems; DK DAB+ national mux structure; >590 DK FM transmitters claim on radiomap; AIS ~20 NM; LTE offshore range articles |
./docs/research/R1-bladerf-market.md:210:### 4.3 Python
./docs/design/DESIGN_BASELINE.md:134:| Loss or condition | What remains / required response |
./docs/design/DESIGN_BASELINE.md:146:| Companion faults, by class (amended 2026-07-22 per D17) | Estimator-only stall or internal fault with the supervisor alive: the supervisor's monotonic watchdog shall expire the authority lease and stop steering output. Whole-process death, companion–autopilot link loss, or board/power loss: no in-process responder exists; the autopilot-side response to `GPS_INPUT` silence is SITL-characterised (2026-07-23, `tools/sitl/evidence/D17a.md`: EKF-failsafe HOLD, no manoeuvre, at ~5.07 s at pinned Rover-4.6.3; SITL-only, on-vessel confirmation pending [UNVERIFIED]) — until on-vessel confirmation, only alarm hardware and the physical helm override are credited as backstops. In every class, software shall not autonomously select RTL, Loiter or disarm. |
./docs/design/SAFETY_CASE.md:226:firmware and configuration a bounded non-manoeuvre response exists, via the EKF failsafe
./docs/design/SAFETY_CASE.md:239:loss (Case B) there is **no in-process responder**, and the autopilot-side response to
./docs/design/SAFETY_CASE.md:335:| Baseline degradation row | Supervisor response | Trigger |
./docs/design/SAFETY_CASE.md:347:| **Companion faults, by class** *(aligns with the amended baseline row, per D17(b))* | **Estimator-only stall / internal fault, supervisor alive:** the live supervisor's monotonic watchdog expires the lease and stops steering (Case A). **Whole-process death / companion–autopilot link loss / board or power loss:** no in-process responder exists; the autopilot-side response to `GPS_INPUT` silence is **[UNVERIFIED]** (SITL, per D17 / U-M1), so this residual is `uncontrolled-pending-evidence`, backed only by the physical helm/kill-cord (Case B, §2.1). In every class, software never selects RTL/Loiter/disarm. | Case A (watchdog) / Case B (`[UNVERIFIED]`) |
./docs/design/SAFETY_CASE.md:399:response to `GPS_INPUT` silence is **[UNVERIFIED]** (SITL, per D17 / U-M1), so that residual is
./docs/design/SAFETY_CASE.md:441:**Human-response failure, by sub-case (fixes finding 9 / H7).** The mandated "human does not
./docs/design/SAFETY_CASE.md:462:response to `GPS_INPUT` silence characterised and proven non-manoeuvring in SITL (Case B, per
./docs/design/SAFETY_CASE.md:479:| **H5** | **MAVLink link loss mid-authority** | Cable/USB/serial/connector fault. | This is **Case B**: internal revocation cannot reach ArduPilot. ArduPilot's response to `GPS_INPUT` silence is **SITL-characterised (D24 amendment, §2.1)**: EKF-failsafe HOLD (no manoeuvre) at ~5.07 s at pinned Rover-4.6.3, per `tools/sitl/evidence/D17a.md`; SITL-only, on-vessel confirmation pending **[UNVERIFIED]**. Present backstop is helm override/kill-cord (link-independent). Companion detects link loss via ArduPilot→companion heartbeat deadline → alarm/log only. | **SITL-characterised, on-vessel-unconfirmed**: SITL shows a non-manoeuvre HOLD at ~5.07 s (D17a); until confirmed on the vessel installation, only the physical layer is credited. The prior "bounded ~4 s non-manoeuvre timeout" and "`t_lease` guarantees the companion revokes first" claims are withdrawn. **[UNVERIFIED]/uncontrolled — registered §5.** |
./docs/design/SAFETY_CASE.md:511:- **ArduPilot response to `GPS_INPUT` silence (Case B) — [UNVERIFIED], `uncontrolled-pending-evidence`:**
./docs/design/SAFETY_CASE.md:527:- **Operational human-response control** for the unconscious-at-helm residual (H7): a second
./docs/design/HIGH_SPEED_ENVELOPE.md:26:  it rested on an assumed `a_rms = 3 g`; R5's measured RMS is **0.44 g**, at which the rectified bias
./docs/design/HIGH_SPEED_ENVELOPE.md:29:- The slam model **[corrected]** — R5 gives impact durations of **100–450 ms** (not the assumed
./docs/design/HIGH_SPEED_ENVELOPE.md:37:- Timer `1/v` rescaling **[replaced]** by an explicit response/distance budget; the safe-speed cap is
./docs/design/HIGH_SPEED_ENVELOPE.md:38:  **not located between 20 and 30 kn** — on R5's 3–5 s human-response floor it could be **14–23 kn**.
./docs/design/HIGH_SPEED_ENVELOPE.md:58:  20–30 kn**, and R5 found **no** open campaign tabulating peak/RMS/A₁/₁₀ vs both speed *and* Hs for
./docs/design/HIGH_SPEED_ENVELOPE.md:69:| Continuous vertical RMS accel | **0.44 g** (small-RIB upside to ~1 g as judgment) | 0.44 g RMS MK V head seas @ Hs 0.9 m (VERIFIED point) | Adopt R5's measured RMS; the first draft's 3 g was a **peak/A₁-class** number misused as RMS. | rectification bias collapses to ~48× under PL → isolation no longer *mandatory-for-integrity* |
./docs/design/HIGH_SPEED_ENVELOPE.md:70:| Slam peak (crew/coxswain station) | **~3–9 g** (8.62 g worst measured) | MK V peak 8.62 g head; A₁/₁₀ 2.7–3.2 g "max safe" (VERIFIED) | Adopt R5; drop the first draft's unsourced "bow 10–20+ g" (R5's bow-sea column is *lower*, 3.94 g). Small-RIB bow could exceed 9 g but that is **judgment**. | stacking/IMU concerns rescaled to sourced g |
./docs/design/HIGH_SPEED_ENVELOPE.md:71:| Slam duration | **100–450 ms** | NSWCCD-80-TR-2014/026 (VERIFIED) | Adopt R5; drop 5–50 ms. Energy is now **< ~10 Hz**. | isolation *harder* (low-fn, large stroke); suppression weaker |
./docs/design/HIGH_SPEED_ENVELOPE.md:75:| IMU (VN-100) behaviour | rigid-mount preferred; **~4.5 g RMS random saturates**; VRE unspecified | VN-100 datasheet (VERIFIED) | Adopt R5's *sourced* saturation limit in place of the first draft's assumed "±16 g FS bow saturation". | IMU concern re-anchored; co-isolation now *in tension* with vendor guidance |
./docs/design/HIGH_SPEED_ENVELOPE.md:76:| Human alarm-response floor | **3–5 s** | R5 §5.2 (detection+recognition+motor, mixed VERIFIED/ASSUMED) | Adopt as a floor; do not scale below it. | 20 kn already at floor; cap not located at 20–30 kn |
./docs/design/HIGH_SPEED_ENVELOPE.md:135:R5 gives sourced slam statistics (MK V, Hs ≈ 0.9 m, coxswain station): continuous **RMS 0.44 g**,
./docs/design/HIGH_SPEED_ENVELOPE.md:136:A₁/₃ 2.9 g, A₁/₁₀ 4.3 g, **peak 8.62 g** (head seas), **impact duration 100–450 ms** → dominant content
./docs/design/HIGH_SPEED_ENVELOPE.md:148:| 4.3 g (A₁/₁₀) | 1.29 m/s | 0.41 m/s | 0.21 m/s | 0.082 m/s |
./docs/design/HIGH_SPEED_ENVELOPE.md:149:| 8.62 g (peak) | 2.58 m/s | 0.82 m/s | 0.41 m/s | 0.16 m/s |
./docs/design/HIGH_SPEED_ENVELOPE.md:156:*sustained-sinusoid* model; a 100–450 ms half-sine needs direct convolution through the actual
./docs/design/HIGH_SPEED_ENVELOPE.md:157:correlator/discriminator/clock-estimator impulse responses (routed, §5). The numbers above are
./docs/design/HIGH_SPEED_ENVELOPE.md:161:sourced RMS. [SOFTENED]** A quadratic sensitivity `Γ₂` rectifies zero-mean vibration into a *sustained*
./docs/design/HIGH_SPEED_ENVELOPE.md:164:**RMS 0.44 g** and a *hypothetical* `Γ₂ = 1×10⁻¹¹/g²`:
./docs/design/HIGH_SPEED_ENVELOPE.md:169:A₁-class statistic, not continuous RMS** (review finding 1 / triage F1, confirmed). At the sourced RMS
./docs/design/HIGH_SPEED_ENVELOPE.md:170:the rectified bias is negligible; it reaches the PL only near `a_rms ≈ 3 g` **RMS**, which R5 does not
./docs/design/HIGH_SPEED_ENVELOPE.md:172:a proven integrity breach; it becomes material only if a small-RIB `a_rms` is many-fold R5's 0.44 g or
./docs/design/HIGH_SPEED_ENVELOPE.md:183:**And isolation is harder than the first draft claimed.** R5 (§3.4, VERIFIED): the 100–450 ms slams put
./docs/design/HIGH_SPEED_ENVELOPE.md:193:soft isolation degrades AHRS via relative motion and filter lag, and notes ~**4.5 g RMS random can
./docs/design/HIGH_SPEED_ENVELOPE.md:211:`available response time = f(closing speed, detection latency, alarm latency, human response, helm/
./docs/design/HIGH_SPEED_ENVELOPE.md:223:**Human-response floor (R5 §5.2, sourced-but-mixed): 3–5 s** for detection + recognition + motor
./docs/design/HIGH_SPEED_ENVELOPE.md:224:response. This bounds `T_ack` from below regardless of speed.
./docs/design/HIGH_SPEED_ENVELOPE.md:233:| 3 s | 3.5 s (already ≤ floor edge) | **≈ 23 kn** |
./docs/design/HIGH_SPEED_ENVELOPE.md:234:| 5 s | 3.5 s (**below floor**) | **≈ 14 kn** |
./docs/design/HIGH_SPEED_ENVELOPE.md:236:**What is supported (evidence + this budget):** (i) at 20 kn a distance-preserving `T_ack ≈ 3.5 s` is
./docs/design/HIGH_SPEED_ENVELOPE.md:237:**already at or below the 3–5 s human floor**, so the collision-timer margin at 20 kn is thin and needs
./docs/design/HIGH_SPEED_ENVELOPE.md:253:| `T_ack` | 10 s | **derive from §2.1 budget; ≥ 3–5 s floor** | **not** a `1/v` rescale; `[UNVERIFIED]` |
./docs/design/HIGH_SPEED_ENVELOPE.md:310:| 4.3 g (A₁/₁₀) | 1592 Hz/s | **5310 Hz/s** | exceeds 256-block (4000); within 128-block (8000) |
./docs/design/HIGH_SPEED_ENVELOPE.md:311:| 8.62 g (peak) | 3191 Hz/s | **6909 Hz/s** | within 128-block (8000) |
./docs/design/HIGH_SPEED_ENVELOPE.md:334:**random vibration ~4.5 g RMS can saturate the accelerometers → filter collapse**, **rigid mount
./docs/design/HIGH_SPEED_ENVELOPE.md:339:| **Random-vib saturation (~4.5 g RMS)** | R5's *sourced* failure mode: broadband slam vibration approaching 4.5 g RMS saturates accels and collapses the AHRS filter → propagation loss. Replaces the first draft's unsourced "±16 g FS bow saturation." | **sourced concern**; hull `a_rms` vs 4.5 g must be measured |
./docs/design/HIGH_SPEED_ENVELOPE.md:347:whether the hull's **broadband RMS** approaches the sourced ~4.5 g saturation limit and what its
./docs/design/HIGH_SPEED_ENVELOPE.md:373:| 20 min | 4.32 km | **12.35 km** | 18.52 km |
./docs/design/HIGH_SPEED_ENVELOPE.md:395:### 4.3 DR-bridge error — authority-PL time [relabelled], and the good-fix reconciliation
./docs/design/HIGH_SPEED_ENVELOPE.md:453:| M-2 | Verify hull broadband **`a_rms` vs the sourced ~4.5 g RMS VN-100 saturation limit**; verify **VRE** and **gyro g-sensitivity** (VN-100 VRE unspecified). | evidence-supported (R5) |
./docs/design/HIGH_SPEED_ENVELOPE.md:461:| S-1 | Planing timer class **derived from an explicit response/distance budget** (§2.1), not a `1/v` rescale; the cap may be < 20 kn. | judgment |
./docs/design/HIGH_SPEED_ENVELOPE.md:470:| P-1 | Add a Class-B timer column whose `T_ack`/`t_dr` are budget-derived (≥ 3–5 s floor), not rescaled. | judgment |
./docs/design/HIGH_SPEED_ENVELOPE.md:478:| A-1 | **Definition of which LEO observation resets `t_dr`** (position-constraining vs velocity-only) — governs the good-fix scenario (§4.3). | contracts |
./docs/design/HIGH_SPEED_ENVELOPE.md:509:| Timers / speed cap | craft crash-stop/turn trial **and** validated human-response floor **and** an explicit budget |
./docs/design/HIGH_SPEED_ENVELOPE.md:539:(8.62 g) scaled by 1.84–2.25× → **~16–19 g peak** at 30 kn — a hypothesis requiring hull measurement,
./docs/design/HIGH_SPEED_ENVELOPE.md:543:*sourced* RMS 0.44 g scaled by the same (unsourced) factor → `a_rms ≈ 0.8–1.0 g`; with the hypothetical
./docs/design/HIGH_SPEED_ENVELOPE.md:545:0.09–0.14 m/s (3.3–4.9× over) came from mis-using 3 g as RMS *and* squaring an unsourced peak-scaling
./docs/design/HIGH_SPEED_ENVELOPE.md:546:exponent (compounding two unsupported steps, finding 18). **On sourced RMS, 30 kn rectification is not at
./docs/design/HIGH_SPEED_ENVELOPE.md:564:**below any credible 3–5 s human floor** → **do not grant 30 kn denied autonomous authority.** But the
./docs/design/HIGH_SPEED_ENVELOPE.md:573:but per §4.3 authority still expires at `t_dr`, not at the 2.5 min heading time.
./docs/design/HIGH_SPEED_ENVELOPE.md:583:| P1 DR bridge | fix loss → first re-acq | starts ~5–25 m, decays | decays 1.5× faster | **`t_dr` (§4.3), not heading time** |
./docs/design/HIGH_SPEED_ENVELOPE.md:593:**not the class change the first draft claimed** — on sourced RMS the rectification stays under the PL at
./docs/design/HIGH_SPEED_ENVELOPE.md:596:**safety** layer, 30 kn is over the human-response floor → **denied cap**, but the cap is not shown to sit
./docs/design/HIGH_SPEED_ENVELOPE.md:604:| H-2 | Oscillator `Γ₂` **measured** matters more as `a_rms` rises, but even scaled it is ~10× under PL on sourced RMS — measure to confirm, don't assume breach. | evidence-supported (R5 absence) |
./docs/design/HIGH_SPEED_ENVELOPE.md:606:| H-4 | Hull `a_rms` vs the **4.5 g RMS VN saturation** limit is the binding IMU question at 30 kn (scaled 0.8–1 g RMS is still under it, but bow/severe seas unknown). | evidence-supported (R5) |
./docs/design/HIGH_SPEED_ENVELOPE.md:621:0.44 g RMS the rectified bias is ~48× under the denied PL, and its magnitude rested on an unsourced `Γ₂`
./docs/design/HIGH_SPEED_ENVELOPE.md:622:and a peak-as-RMS error. The real, sourced reference concern is the **linear g-sensitivity** (~1e-9/g)
./docs/design/HIGH_SPEED_ENVELOPE.md:625:(R5's 100–450 ms slams put energy below ~10 Hz, needing low-`fn`/large-stroke mounts, and vendor guidance
./docs/design/HIGH_SPEED_ENVELOPE.md:628:20–30 kn dataset); (2) an **IMU** whose broadband `a_rms` vs the sourced ~4.5 g RMS saturation limit and
./docs/design/HIGH_SPEED_ENVELOPE.md:630:~3.5 s at 20 kn — already at/below R5's 3–5 s human floor — so the safe-speed cap must be derived from an
./docs/design/HIGH_SPEED_ENVELOPE.md:644:sourced RMS the vibration-rectification bias stays ~10× under the PL at 30 kn as at 20 kn, the required
./docs/design/HIGH_SPEED_ENVELOPE.md:649:30 kn conclusion is on the **human-response floor**: a distance-preserving `T_ack ≈ 2.3 s` is below R5's
./docs/design/HIGH_SPEED_ENVELOPE.md:650:3–5 s floor, so denied autonomous authority at 30 kn is not defensible — but the analysis **does not
./docs/design/PARAMS_PROPOSAL.md:17:This proposal is the response to `SAFETY_CASE.md` §5's open numeric register and to review
./docs/design/PARAMS_PROPOSAL.md:40:  solution.state.horizontal_accuracy_m()` — the epoch **2-D DRMS one-sigma** horizontal
./docs/design/PARAMS_PROPOSAL.md:42:- The horizontal-velocity limit is tested against `speed_accuracy_mps()` — the same 2-D DRMS
./docs/design/PARAMS_PROPOSAL.md:59:*reported one-sigma DRMS*. Converting one to the other needs a coverage factor. This proposal
./docs/design/PARAMS_PROPOSAL.md:61:expressed as one-sigma DRMS, tested at `≤ PL`, keeps the **2-DRMS** bound (≈ 95–98 % of
./docs/design/PARAMS_PROPOSAL.md:63:circular 2-D Gaussian the 95 % radius is `1.73 × DRMS`, so `k = 2` is deliberately
./docs/design/PARAMS_PROPOSAL.md:80:(both **estimate**-tagged in the baseline). With `k = 2` on the one-sigma DRMS metric:
./docs/design/PARAMS_PROPOSAL.md:81:`PL_aided = 25 / 2 = 12.5 → 12 m` (2-DRMS = 24 m ≤ 25 m); `PL_denied = 200 / 2 = 100 m`
./docs/design/PARAMS_PROPOSAL.md:82:(2-DRMS = 200 m = acceptance). The denied limit is set numerically equal to
./docs/design/PARAMS_PROPOSAL.md:92:too loose (aided 24 m): 2-DRMS = 48 m, roughly double the 25 m acceptance — per-epoch
./docs/design/PARAMS_PROPOSAL.md:94:"no looser than implied" rule. Denied 200 m PL (2× loose) would put 2-DRMS at 400 m, twice the
./docs/design/PARAMS_PROPOSAL.md:106:m/s). The metric is 2-D DRMS velocity `= √2 · σ_axis` for equal axes. Requiring per-axis 95 %
./docs/design/PARAMS_PROPOSAL.md:112:2-D DRMS* (trace of the rotated horizontal velocity covariance), which is blind to axis ratio —
./docs/design/PARAMS_PROPOSAL.md:113:a covariance with one large and one small axis can pass a DRMS gate while its large axis exceeds
./docs/design/PARAMS_PROPOSAL.md:118:a **per-axis (not DRMS) velocity gate** or a **frozen covariance-shape rule**. This is a
./docs/design/PARAMS_PROPOSAL.md:127:0.014–0.028 m/s DRMS therefore demands tens (Ku) to hundreds (L-band) of well-conditioned
./docs/design/PARAMS_PROPOSAL.md:365:**Validation plan.** Human-factors alarm-response measurement in the trial environment; re-derive
./docs/design/PARAMS_PROPOSAL.md:450:### 4.3 PFA at threshold 32, and the false-alarm rate
./docs/design/PARAMS_PROPOSAL.md:556:covariance** (§1.2 anisotropy limitation): a scalar DRMS gate does not guarantee a per-axis
./docs/design/PARAMS_PROPOSAL.md:625:horizontal_velocity_mps = 0.014    # §1.2  per-axis 0.02 -> DRMS   [UNVERIFIED]
./docs/design/PARAMS_PROPOSAL.md:631:horizontal_velocity_mps = 0.028    # §1.2  per-axis 0.04 -> DRMS   [UNVERIFIED]
./crates/pnt-mission/README.md:20:attributes the prior's and Doppler's RMS contributions, so neither can masquerade as the
./crates/pnt-mission/README.md:23:synthetic geometry/tuning, Doppler assimilation improves position RMS given the prior but
./crates/pnt-mission/README.md:24:**degrades speed RMS against the same-initialization baseline** (mechanism [UNVERIFIED],
./docs/studies/tracker/STUDY.md:45:  For Q<180, a log-log fit gives `ln(var_Hz2) = 27.222 - 4.279 ln(Q)`, RMS log residual
./docs/studies/tracker/STUDY.md:60:| 34 | 0.988 | 0.974–0.994 | 4.32 | 195.44 | 660.73 | 47.85 |
./docs/studies/tracker/STUDY.md:62:| 46 | 1.000 | 0.992–1.000 | -0.44 | 11.93 | 32.54 | 160.91 |
./docs/studies/tracker/STUDY.md:201:with RMS residual 0.571 in natural-log variance. Per-bin fitted values and residuals are in
./docs/studies/estimator/STUDY.md:11:Prior-only velocity RMS: 0.2506 m/s; baseline Doppler: 0.3753 m/s (along LOS 0.3075, across LOS 0.2153). Two mechanisms are evidenced. The current replay prior path is structurally confounded: variance 1 against initial variance 1 gives gain 0.5 and retains an analytically computed 3189068 m radial ECEF error. After removing that confound, Doppler raises velocity RMS from 0.2506 to 0.3753 m/s entirely in the LOS component (across-LOS changes from 0.2243 to 0.2153 m/s). Q=4e-4 empirically minimizes velocity RMS at 0.2365 m/s as an interior optimum; the reviewer's independent extended sweep confirms 0.2659/0.2616/0.2398/0.2365/0.3084/0.3753/0.6643 m/s across Q=4e-7..0.4. This Doppler-degrades-velocity result is contingent on the near-truth IMU (propagation error about 5e-4 m/s^2): at sea with realistic IMU error the sign could flip and the low-Q fix could be wrong.
./docs/studies/estimator/STUDY.md:13:| Treatment | velocity RMS (m/s) | along-LOS RMS | across-LOS RMS | horizontal position RMS (m) |
./docs/studies/estimator/STUDY.md:33:The stub reproduces only the relative 20-minute emergence: Doppler is worse at 2 min (5.74 vs 0.81 m RMS) but better at 30 min (149.59 vs 199.81 m). Absolute RMS does not converge; it grows throughout. The 20-minute crossover is fragile: it rests on means from only 6 seeds. Turn reset observed: false, but the maneuver-reset question is unfalsifiable by construction here because the turn enters through the near-truth IMU; this is a harness limitation, not evidence about the real filter. The stub also has no heading-to-velocity coupling or manoeuvre covariance reset.
./docs/studies/estimator/STUDY.md:35:| duration (min) | prior-only RMS (m) | prior+Doppler RMS (m) |
./docs/studies/estimator/STUDY.md:49:Threshold 9 first rejects at least 95% at 1 h; all tested nonzero staleness (>=1 h) is rejected. Epoch shifting aliases orbital phase, producing non-monotonic innovation RMS, and innovations are roughly 3000-5000 times the threshold-9 gate. The missing HPH' term makes this rejection result an upper bound. This deliberately phase-shifted fixture does not support the 6 h choice or validate a real SupGP age-error curve.
./docs/studies/estimator/STUDY.md:51:| epoch offset | innovation mean (m/s) | innovation RMS (m/s) | threshold-9 rejection |
./README.md:73:| denied-mode run (same journal) | horiz. position RMS | horiz. speed RMS |
./README.md:247:- On-vessel confirmation of the SITL-only D17a characterisation of ArduPilot's response to

exec
/bin/bash -lc "sed -n '1,240p' docs/research/R5-highspeed-dynamics.md; sed -n '290,680p' docs/design/HIGH_SPEED_ENVELOPE.md; sed -n '250,340p' docs/studies/tracker/STUDY.md; git diff --check -- docs/design/HIGH_SPEED_ENVELOPE.md" in /home/od/work/leo-pnt
 succeeded in 0ms:
# U-R5 — High-Speed Marine Dynamics: Published Values for Planing Envelope Analysis
**Contract:** v5.1 research brief · **Access date for all live sources:** 2026-07-23  
**Purpose:** Replace [UNVERIFIED] engineering estimates for planing dynamics at ~20–30 kn with published primary/secondary sources.  
**Claim labels:** **VERIFIED** = content read from the cited primary document or manufacturer datasheet; **ASSUMED** = engineering inference, secondary summary, or gap fill not present as a measured value in a primary source.

---

## 1. Planing-hull vertical accelerations and slam statistics (20–30 kn, moderate seas)

### 1.1 Full-scale measurement campaigns (primary)

**VERIFIED — MK V Special Operations Craft (NSWCCD case study)**  
Source: Haupt, K., *High-Speed Craft Motions: a Case Study*, NSWC Carderock / Combatant Craft; PDF  
URL: https://www.isthq.com/wp-content/uploads/2023/05/NSWC_High_Speed_Watercraft_Motions.pdf  

- Craft: 82 ft aluminum monohull planing SOC; top speed ~50 kn.  
- Sea: significant wave height **Hs ≈ 3.0–3.1 ft (~0.9 m)** (Datawell Waverider + NOAA CMAN).  
- Trigger: events > **0.5 g for ≥ 50 ms**; sample rate **512 Hz**, 200 Hz anti-alias.  
- **Vertical acceleration at coxswain’s station** (table in source):

| Statistic | Head | Bow | Beam | Quartering | Following |
|-----------|------|-----|------|------------|-----------|
| Peak (g) | **8.62** | 3.94 | 6.02 | 1.67 | 1.51 |
| Average 1/10 highest (g) | **4.30** | 2.48 | 2.39 | 1.56 | 1.26 |
| Average 1/3 highest (g) | **2.90** | 1.89 | 1.65 | 1.12 | 0.96 |
| Mean (g) | 1.67 | 1.24 | 1.07 | 0.71 | 0.64 |
| RMS (g) | **0.44** | 0.38 | 0.32 | 0.24 | 0.23 |

- Severity ranking by heading: head > bow > beam > quartering > following.  
- Vertical >> longitudinal or transverse for structure/equipment/crew.  
- Note: exact craft speed for the tabulated run is **not stated** in the extracted text; craft is a high-speed planing combatant (design envelope far above 20–30 kn). **Do not treat the table as a 25 kn / small-RIB result.**

**VERIFIED — Wave impact duration (NSWCCD multi-craft database)**  
Source: Riley, Haupt, Murphy, *An Investigation of Wave Impact Duration in High-Speed Planing Craft in Rough Water*, NSWCCD-80-TR-2014/026, Apr 2014  
URL: https://apps.dtic.mil/sti/tr/pdf/ADA616198.pdf  

- Abstract/findings: impact durations **100 ms to 450 ms** for craft mass **~14,000–105,000 lb**, deep-V deadrise **18°–22°**.  
- Pulse shape for severe rigid-body impacts often approximated as **half-sine**.  
- Duration matters for structural and equipment response (SRS / resonant systems).

**VERIFIED — Measurement/analysis methods and half-sine model (NSWCCD guide)**  
Source: Riley et al., *A Guide for Measuring, Analyzing, and Evaluating Accelerations Recorded During Seakeeping Trials of High-Speed Craft*, NSWCCD-80-TR-2016/003, Jan 2016  
URL: https://apps.dtic.mil/sti/tr/pdf/AD1021121.pdf  

- Standardized stats: peaks, **A₁/₁₀**, **A₁/₃**, RMS, impact count / ICI, Ride Severity Index.  
- Rigid-body pulse modeled as half-sine; ΔV relation (US customary in report):  
  **ΔV (ft/s) ≈ 64.4 · A_max(g) · T(s)** for half-sine of peak A and duration T.  
- Filtering: rigid-body content typically emphasized with low-pass (report discusses Fourier content and cutoff selection; companion literature uses ~10 Hz for event find / ~80 Hz for peak amplitude in NSWC-PC algorithms).

**VERIFIED — Coxswain “maximum safe speed” linked to A₁/₁₀**  
Source: Riley et al., *Standardized Laboratory Test Requirements for Hardening Equipment…*, NSWCCD-80-TR-2017/002, Feb 2017 (Appendix material citing multi-craft analyses)  
URL: https://apps.dtic.mil/sti/pdfs/AD1032710.pdf  

- Across **>20 craft** / sea states: max safe speeds chosen by experienced coxswains corresponded to **A₁/₁₀ ≈ 2.7–3.2 g**.  
- Earlier anecdotal naval-crew descriptor: **A₁/₁₀ ≈ 3 g** “extremely uncomfortable.”  
- Equipment baseline: **A₁/₁₀ = 4.0 g + 20% margin → 4.8 g** as max severity reference for equipment test development.

**VERIFIED — Equipment laboratory shock derived from HSC wave slam**  
Same NSWCCD-80-TR-2017/002:  

| Test | Pulse | Application |
|------|-------|-------------|
| Single severe (×3 each axis) | **20 g, 23 ms half-sine** | General hard-mounted equip any craft/location |
| Repeated low severity | **5 g, 23 ms**, **800 pulses** @ 1 s | Simulates 15–20 min rough transit |
| Optional known Z-up | 10 g (X,Y), 20 g (Z), 23 ms | Fixed orientation |
| Limited (craft-size) | 10 or 15 g, 23 ms | Fragile/high-value limited install |
| Vibration | MIL-STD-810G Method 514.7, vertical PSD Fig. 514.7C-4, 1 h/axis | Broadband vibration |

Rationale: long field pulses (~100–150+ ms, peak often <10 g) mapped via **shock response spectrum (SRS)** to shorter higher-amplitude lab pulses that commercial shock machines can produce. Margins: **1.2** (measurement/processing) × **1.5** (lab vs sea uncertainty).

### 1.2 Design / seakeeping standards

**VERIFIED — STANAG 4154 vertical acceleration criterion (cited in open literature)**  
Sources citing STANAG 4154:  
- Peterson et al., *Shock Mitigation for the Human on High Speed Craft*, RTO-MP-AVT-110, 2004 — https://publications.sto.nato.int/publications/STO%20Meeting%20Proceedings/RTO-MP-AVT-110/MP-AVT-110-31.pdf  
- Secondary reviews (e.g. seakeeping criteria papers citing 0.2 g RMS bridge vertical).  

- Commonly cited **RMS vertical acceleration limit ~0.2 g** (personnel/ship seakeeping).  
- Same NATO paper **explicitly questions** applying ship/passenger RMS and ISO 2631-style limits to **high-Froude planing HSC** injury risk; crest factors ≫ 3; impact statistics preferred.  
- Alternate RMS comfort figure cited: **0.3 g rms** (Mandel 1979, via Peterson).  

**ASSUMED (class rules structure, exact formula coefficients not re-derived here from paid ISO/ABS text):** Classification / ISO small-craft rules (ABS HSC, ISO 12215-5, DNV HSLC) use a **design vertical acceleration at LCG (n_cg)** that increases with **V** and **Hs (h₁/₃)** and depends on deadrise / length-beam factors — used for **bottom slamming pressure**, not as an at-sea measured RMS. Design n_cg for hard planing patrol craft is often **several g**, not 0.2 g RMS.

### 1.3 Practical envelope for 20–30 kn / moderate Hs (synthesis)

| Quantity | Published anchor | Use for 20–30 kn moderate seas |
|----------|------------------|--------------------------------|
| RMS vertical (coxswain-class station, head seas, Hs~0.9 m, larger HSC) | **0.44 g** (MK V table) | VERIFIED data point; smaller RIBs may differ |
| A₁/₁₀ “uncomfortable / max safe” | **2.7–3.2 g** | VERIFIED multi-craft coxswain correlation |
| Peak single slam (severe head seas, larger craft) | **~6–9 g** class (MK V peak 8.62 g) | VERIFIED for that craft/sea; can be higher at bow |
| Impact duration (deep-V, 6–50 t class) | **100–450 ms** | VERIFIED NSWCCD |
| Design / equip shock qualification | **20 g @ 23 ms** + **5 g × 800** | VERIFIED NSWCCD procurement practice |
| Personnel RMS criterion (ships) | **0.2 g RMS** STANAG | VERIFIED as criterion text; **poor** for slam injury on planing HSC |

**Occurrence rates:** Head-sea impact events collected at **hundreds per 5–10 min** on MK V (target ≥200 events/heading). Exact rate scales with **wave encounter frequency** ≈ V_rel / λ_enc — **ASSUMED** scaling if used without wave spectrum measurement.

**Gap:** No single open primary campaign was found that tabulates peak/RMS/A₁/₁₀ **explicitly vs both 20–30 kn and Hs for small RIB/patrol boats** in one matrix. Use NSWCCD methods + craft-specific trials for envelope replacement of remaining [UNVERIFIED] cells.

---

## 2. Sustained trim angles for planing hulls vs speed

**VERIFIED — Optimum running trim (Savitsky-based literature)**  
- Ghassemi et al., *Minimization of Resistance of the Planing Boat by Trim-tab*, Int. J. Physics 7(1), 2019  
  URL: https://pubs.sciepub.com/ijp/7/1/4/  
  - Optimum trim for min resistance often **~2–3°** across planing speeds studied.  
  - Example: **2.24° at 35 kn**; polynomial of optimum τ vs volume Froude number given.  
- Classic industry / Savitsky community consensus (secondary but long-standing): resistance minimum often near **3–4°** bow-up hull trim of the running surface (ContinuousWave forum synthesis of Savitsky; RIB.net practice notes).  

**VERIFIED — Operational guidance (popular technical, not class rule)**  
- Boote Magazin (EN): fast planing craft “best and most safely” with trim **2–4°**; **>6°** classed uneconomical.  
  URL: https://www.boote-magazin.de/en/boat-knowledge/beginner/boat-trim-the-3-most-important-motorboat-types-and-their-trim-angle/

**Typical ranges for envelope work (split):**

| Regime | Dynamic trim (bow up) | Status |
|--------|----------------------|--------|
| Efficient planing cruise | **2–4°** | VERIFIED multi-source (Savitsky optimization + practice) |
| Optimum min-drag (many studies) | **~2–3°** | VERIFIED (Ghassemi/Savitsky method papers) |
| Hump / transition | Higher, often **4–6°+** | ASSUMED range (varies strongly with LCG, loading, tabs) |
| Over-trimmed | **>6°** high drag | VERIFIED as “uneconomical” guidance |
| Porpoising risk | High trim + high Fn | VERIFIED as stability concern in planing literature (DTIC porpoising studies exist) |

**Note:** Running trim is relative to **static zero** or **buttock reference**; report which datum is used. Trim tabs / interceptors routinely pull trim down for efficiency and ride.

---

## 3. Crystal / rubidium oscillators under vibration and shock

### 3.1 FE-5680A-class Rb

**VERIFIED — FE-5680A commercial datasheet (no g-sensitivity row)**  
URL: https://www.miedema.dyndns.org/co/2019/rb/3rb/FE-5680A-Rubidium-datasheet.pdf  

| Parameter | Published value |
|-----------|-----------------|
| Frequency | 10 MHz typical (factory 1 Hz–20 MHz) |
| Allan (τ) | **1.4×10⁻¹¹ / √τ** (datasheet) |
| Drift | **2×10⁻¹¹ / day**; **2×10⁻⁹ / year** class (options vary) |
| f vs T | **±3×10⁻¹⁰** (−5 to +50 °C) typical option |
| Phase noise @10 MHz | **−100 dBc/Hz @10 Hz; −125 @100 Hz; −145 @1 kHz** |
| Power / size | ~11 W SS @25 °C; ~25×88×125 mm; 434 g |
| MIL environment option | Option 22 “MIL environment (foamed)” listed |

**Not published on commercial FE-5680A sheet:** acceleration sensitivity Γ (fractional Δf per g), vibration PSD response, or shock survival.  

**ASSUMED (physics of Rb standards):** Physics package resonance is atomic; **VCXO + RF chain** still dominate vibration coupling. NIST notes atom-based elements can approach low g-sensitivity but **electronics volume** remains vulnerable; suppression often still required to approach **~10⁻¹⁰ / g** class (Hati/Nelson/Howe NIST chapter).

### 3.2 Quality OCXO g-sensitivity

**VERIFIED — Typical SC-cut / quality crystal**  
- Wenzel Associates, *Vibration-Induced Phase Noise*:  
  URL: https://wenzel.com/library/time-frequency-articles/vibration-induced-phase-noise/  
  - Tip-over test: typical SC-cut **10 MHz** shifts ~0.02 Hz over 2 g → **Γ ≈ 1×10⁻⁹ / g**.  
- NIST / Filler framework (Hati et al.): typical oscillator acceleration sensitivity **~10⁻⁸ to 10⁻¹⁰ / g**.  
  URL: https://tf.nist.gov/general/pdf/2328.pdf  
- Industry design target often quoted: **Γ = 1×10⁻⁹ / g** for good SC-cut OCXOs; precision low-g units and multi-crystal compensation lower.

### 3.3 Vibration-induced phase noise formula (VERIFIED)

For sinusoidal vibration (small modulation index), single-sideband:

**L(f_v) = 20 log₁₀[ (Γ · a · f₀) / (2 · f_v) ]** [dBc]

where Γ = acceleration sensitivity [1/g], a = peak acceleration [g], f₀ = carrier [Hz], f_v = vibration frequency [Hz].  

For random vibration, use a → √(2 · G(f)) with G = acceleration PSD [g²/Hz] (Wenzel / Filler / NIST conventions).  

**Example (VERIFIED method, illustrative numbers):** Γ=1e-9/g, f₀=10 MHz, a=1 g peak sine, f_v=100 Hz → L ≈ 20 log(1e-9·1·1e7 / 200) = 20 log(0.05) ≈ **−26 dBc** discrete sideband (very large vs quiet OCXO floors). Marine slam spectra (broadband + high peak g) therefore dominate close-in phase noise unless isolation is used.

### 3.4 COTS isolation for precision oscillators (marine/vehicle)

**VERIFIED practices / products:**

| Approach | Notes | EU availability / price |
|----------|-------|-------------------------|
| **Wenzel vibration-isolated OCXO** product line | Factory isolated OCXOs for harsh vib; custom 10–25 MHz class | US maker; EU via distributors; **quote-only** (typically mid–high $100s–$1000s+ depending on grade) — https://wenzel.com/product/crystal-oscillators/vibration-isolated/oven-controlled-ocxos/ |
| **Elastomer / urethane shock mounts** | Small omni mounts inside package; natural freq often tens–hundreds Hz | Worldwide (Amazon, RS, etc.); **€1–€20**/mount typical |
| **Wire-rope isolators** (e.g. Aeroflex / Hutchinson circular arch style, cited by Wenzel) | Better temp stability than soft rubber; high damping ζ~0.2+ | EU industrial suppliers; **€20–€200+**/mount depending on size |
| **Mass-loaded plate + soft mounts** | Add brass ballast to push **fn ≲ 50–100 Hz** | DIY/industrial; cost dominated by machining |
| **Foam wrap in can** | Simple; often **high** fn, low damping | Cheap; performance limited |

**VERIFIED isolation physics (Wenzel):**  
- Isolation only above system **natural frequency fn**; **amplification at resonance**.  
- Modest OCXO isolation often **fn < 200 Hz**; **fn < 100 Hz** needs extra mass.  
- Orientation: align crystal **Γ vector** with least vibration or best isolator axis.  
- Flexible cable service loops required or isolation is shorted.

**ASSUMED price EU (2026, order-of-magnitude):** quality OCXO €50–€500; vibration-isolated OCXO module €300–€3000+; wire-rope set for small chassis €50–€400; full instrumented isolator tray higher.

**Marine slam note (VERIFIED context from §1):** Pulse durations **100–450 ms** imply significant energy **below ~10 Hz** — **below many light isolator resonances**, so isolation must be designed for **low-fn + large excursion**, not only high-frequency engine vibration. NSWCCD warns long-duration wave slam makes compact shock mounts for electronics **difficult** (large stroke needed).

---

## 4. MEMS IMU in high-vibration marine use

### 4.1 Vibration rectification error (VRE)

**VERIFIED definition (Analog Devices technical article):**  
URL: https://www.analog.com/en/resources/technical-articles/vibration-rectification-in-mems-accelerometers.html  

- **VRE** = accelerometer response to AC vibration that **rectifies to DC**, appearing as anomalous **bias/offset**.  
- Critical for tilt/attitude and navigation under broadband vibration.  
- Many mid-tier datasheets **omit VRE**; better industrial/tactical parts publish VRE (units e.g. **µg/g²** or bias vs g² PSD).  
- Gyro **g² / vibration rectification** also exists (bias shift under vib).

**ASSUMED class comparison (industry practice, not a single standard table):**

| Class | Examples | VRE / vib behavior | Marine high-dynamics |
|-------|----------|--------------------|----------------------|
| Consumer / toy | Phone IMUs | Poor, often unpublished | Unsuitable alone |
| Automotive | AEC-Q100 MEMS | Better robustness, VRE mixed | Common in boats as cheap AHRS core |
| Industrial / tactical MEMS | ADIS16xxx, better ADXL35x, SBG Ellipse, etc. | Often low VRE specified or marketed | Preferred for planing craft |
| VN-100 class | VectorNav VN-100 | Shock/vib tested; **no VRE number on datasheet** | Widely used; see below |
| FOG / RLG / high-end | Fiber / ring laser | Low VRE, high cost | Patrol / naval grade |

### 4.2 VN-100 class (VERIFIED datasheet)

URL: https://metromatics.com.au/wp-content/uploads/2025/12/VN100CR-Datasheet-v7.0-DS100-CR-70-R1.pdf  

| Spec | Value |
|------|-------|
| Gyro in-run bias | **5–7 °/h typ.** |
| Accel in-run bias | **< 0.04 mg** |
| Pitch/roll (static/dynamic) | **0.5° / ~1°** class |
| Powered shock | **500 g** without significant bias/scale change (SMD core) |
| Unpowered shock | up to **10,000 g** (reported) |
| Sine vibration | **10 Hz–2 kHz @ 6 g** operated successfully |
| Mounting advice | **Rigid mount preferred**; isolation hard to get right and can **degrade** AHRS |
| Saturation warning | Random vib **~4.5 g RMS** can **saturate accelerometers** → filter collapse |
| Features | VPE disturbance rejection; hard/soft iron; heave estimate |

**No published VRE (µg/g²) on VN-100 datasheet** — treat as **unspecified** for envelope analysis.

### 4.3 Anti-vibration mounting practice

**VERIFIED (VectorNav + SBG industry guidance):**  
- Prefer **rigid** mechanical bond to structure for AHRS/INS so vib is measured (and filtered) consistently.  
- If isolation used: isolate **source** or whole subsystem; avoid soft-mounting IMU alone (relative motion, filter lag, double-integration error).  
- NSWCCD: equipment on isolators needs **different** shock qualification than hard-mount 20 g / 23 ms rules.  
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

exec
/bin/bash -lc "sed -n '240,520p' .orchestration/reports/U-H1-review-sol.md; sed -n '320,395p' docs/design/HIGH_SPEED_ENVELOPE.md; rg -n \"cap|hard cap|unavoidable|structural|MANDATORY|REQUIRED|10–15 dB|0\\.09|0\\.14|0\\.027|3–6|10–20\\+|±16|5–50|10–100|100–10,000|100–10000\" docs/design/HIGH_SPEED_ENVELOPE.md" in /home/od/work/leo-pnt
 succeeded in 0ms:

22. **Minor · High confidence · §6.1 — textual internal-consistency error**

   **Claim:** “no small-craft rarely runs a full planing hull at 30 kn…”

   **Finding:** The double negative reverses or obscures the intended statement.

   **How to verify:** Rewrite as either “small craft rarely run…” or state the intended sea-state limitation precisely.

## Re-derived chains that do hold

- `sin θ/(1−cos θ) = cot(θ/2)` is correct under the stated idealized mounting geometry.
- `cot 5° = 11.43`, so a 10° tilt is consistent with an approximately 11× benefit.
- Raw clock-induced range-rate error `Δv = cδ` is carrier- and vessel-speed-independent.
- Given the assumed `Γ₂` and RMS acceleration, `0.027`, `0.091`, and `0.137 m/s` are arithmetically correct.
- Fixed-distance collision time scales as `1/v`; applying that rule gives 3.5 s at 20 kn and 2.33 s at 30 kn.
- The 5°/200 m heading times at 7/20/30 kn are approximately 10.6/3.7/2.5 minutes.
- The convergence-distance table is correct: 20 minutes at 20 kn is about 12.35 km; 10–20 minutes at 30 kn is about 9.26–18.52 km.
- 100 km at 30 kn is about 1.80 hours.
- 500 km at 20 kn is about 13.5 hours.

## Verdict assessment

The narrow conclusion “20 kn denied operation is not supportable today” follows from the fail-closed status of the timer set, uncharacterized hull environment, unverified IMU/reference behavior, tracker production gaps, and heading/position limitations. It does **not** require the speculative `Γ₂ = 10⁻¹¹/g²` integrity breach, and that mechanism should not be presented as the central proven reason.

Likewise, “do not support 30 kn denied authority today” follows. The stronger claims that 30 kn necessarily requires 10–15 dB additional isolation, that no tracker block can survive, that loss of lock is unavoidable, and that the cap boundary lies specifically between 20 and 30 kn do not follow.

The report `.orchestration/reports/U-H1.md` faithfully repeats the document’s calculations, but it also repeats its unsupported 3 g RMS/`Γ₂` premise, 5–50 ms slam model, hard 8000 Hz/s tracker cap, timer scaling, and good-fix interpretation. It therefore does not provide an independent reconciliation.

FAIL
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
25:- The vibration-rectification "shock isolation MANDATORY for integrity" conclusion **[SOFTENED]** —
30:  5–50 ms), so slam energy is **low-frequency (< ~10 Hz)**: harder to isolate (below light-isolator
33:- The tracker "no block length survives / structural loss of lock" **[SOFTENED]** — the study
35:  rate is a conservative **screening bound**, not a cap.
36:- The isolation-dB figure **[corrected]** from 10–15 dB to **5–7 dB** (bias ∝ `a²`).
37:- Timer `1/v` rescaling **[replaced]** by an explicit response/distance budget; the safe-speed cap is
68:| Sustained planing trim | **2–4°** | 2–4° efficient (Savitsky/Ghassemi; 2.24° @ 35 kn); >6° "uneconomical" (VERIFIED) | Adopt R5; drop the first draft's 3–6°. | cot-benefit *larger* (29–57×); mounting rule strengthened |
70:| Slam peak (crew/coxswain station) | **~3–9 g** (8.62 g worst measured) | MK V peak 8.62 g head; A₁/₁₀ 2.7–3.2 g "max safe" (VERIFIED) | Adopt R5; drop the first draft's unsourced "bow 10–20+ g" (R5's bow-sea column is *lower*, 3.94 g). Small-RIB bow could exceed 9 g but that is **judgment**. | stacking/IMU concerns rescaled to sourced g |
71:| Slam duration | **100–450 ms** | NSWCCD-80-TR-2014/026 (VERIFIED) | Adopt R5; drop 5–50 ms. Energy is now **< ~10 Hz**. | isolation *harder* (low-fn, large stroke); suppression weaker |
75:| IMU (VN-100) behaviour | rigid-mount preferred; **~4.5 g RMS random saturates**; VRE unspecified | VN-100 datasheet (VERIFIED) | Adopt R5's *sourced* saturation limit in place of the first draft's assumed "±16 g FS bow saturation". | IMU concern re-anchored; co-isolation now *in tension* with vendor guidance |
76:| Human alarm-response floor | **3–5 s** | R5 §5.2 (detection+recognition+motor, mixed VERIFIED/ASSUMED) | Adopt as a floor; do not scale below it. | 20 kn already at floor; cap not located at 20–30 kn |
147:| 2.9 g (A₁/₃) | 0.87 m/s | 0.28 m/s | 0.14 m/s | 0.055 m/s |
168:The first draft's "0.027 m/s, at the PL, isolation MANDATORY" used `a_rms = 3 g`, which is a **peak/
178:**Downgraded from REQUIRED to STRONGLY RECOMMENDED** on present evidence, and the *driver* changes:
226:### 2.2 The safe-speed cap is not located at 20–30 kn [HARDENED]
229:`T_ack(v) = D/v` must stay `≥` the human floor, giving a cap `v ≤ D/floor`:
231:| Human floor | Implied `T_ack` at 20 kn | Implied cap speed |
239:(iii) **the cap could lie anywhere ~14–23 kn** depending on the validated floor and the (unmeasured)
242:cap from an explicit budget — it may be below 20 kn,"* not a specific 20-vs-30 boundary.
295:### 3.2 Tracker drift tolerance — a screening bound, not a cap [SOFTENED]
301:hard cap (review finding 6, confirmed against the study text).
317:6-DOF replay,"* not *"structural / unavoidable."* IMU heave-rate feed-forward (predict the platform rate
339:| **Random-vib saturation (~4.5 g RMS)** | R5's *sourced* failure mode: broadband slam vibration approaching 4.5 g RMS saturates accels and collapses the AHRS filter → propagation loss. Replaces the first draft's unsourced "±16 g FS bow saturation." | **sourced concern**; hull `a_rms` vs 4.5 g must be measured |
343:| **Output rate / anti-alias** | Slam content < ~10 Hz (R5) is largely *below* Nyquist at 100 Hz, so aliasing is **less** severe than the first draft's 10–100 Hz assumption implied — but bandwidth still matters for the fast content and for capturing the slam shape. | re-scoped |
427:value), or a denied speed cap. Registered as a baseline-scope decision (§5), not resolved here.
443:| B-4 | Decide denied-speed support: continuous vs long-leg-only vs **cap (which may be < 20 kn, §2.2)**. | **decision required** |
461:| S-1 | Planing timer class **derived from an explicit response/distance budget** (§2.1), not a `1/v` rescale; the cap may be < 20 kn. | judgment |
509:| Timers / speed cap | craft crash-stop/turn trial **and** validated human-response floor **and** an explicit budget |
510:| Continuous denied position at 20 kn | tighter heading-source characterisation **or** an operational long-leg/cap decision |
545:0.09–0.14 m/s (3.3–4.9× over) came from mis-using 3 g as RMS *and* squaring an unsourced peak-scaling
552:**~5–7 dB**, not the first draft's 10–15 dB (finding 5 / triage F4). (And since rectification is no
559:heave-rate aiding and replay validation"* — **not** "no block survives / structural / unavoidable" (the
560:study is explicitly not a closed-form cap; finding 6). At 30 kn IMU heave-rate feed-forward moves from
563:**Collision / cap [HARDENED to honest indeterminacy].** Scaled distance-preserving `T_ack ≈ 2.3 s` is
565:cap is **not located at 20–30 kn**: on a 5 s floor it is ~14 kn; on 3 s, ~23 kn (§2.2). The supported
596:**safety** layer, 30 kn is over the human-response floor → **denied cap**, but the cap is not shown to sit
603:| H-1 | Isolation must attenuate ~**5–7 dB** more than the 20 kn contingency (corrected from 10–15 dB) — and only if measurement shows rectification approaches the PL. | judgment (contingent) |
607:| H-5 | **Denied-speed cap** (may be < 20 kn); no third autonomous-denied class. | judgment |
630:~3.5 s at 20 kn — already at/below R5's 3–5 s human floor — so the safe-speed cap must be derived from an
645:extra isolation is ~5–7 dB (not 10–15 dB), and the tracker "no block survives" claim is withdrawn as the
646:study is explicitly not a closed-form cap — the honest 30 kn tracker statement is that a *conservative
648:real risk needing IMU heave-rate aiding and replay, not a structural certainty. The one firmly supported
651:locate the cap at 20–30 kn**; on a 5 s floor it could be ~14 kn. 30 kn remains scoping analysis, gated

codex
Final verdict: PASS

Independent recomputation:

`δDC = Γ₂·aRMS² = 1×10⁻¹¹·0.44² = 1.936×10⁻¹²`

`Δv = c·δDC = 2.9979×10⁸·1.936×10⁻¹² = 5.804×10⁻⁴ m/s`

`0.028 / 5.804×10⁻⁴ = 48.24`

Thus the rewritten value is correctly about 48× below the denied velocity PL.

## Original findings

1. PASS — 3 g RMS is withdrawn; 0.44 g RMS is used; the 48.2× margin is correct. `Γ₂` remains explicitly hypothetical. Isolation is downgraded from integrity-mandatory to availability/cycle-slip recommended.

2. PASS — 5–50 ms/10–100 Hz survives only as withdrawn first-draft context. The active model uses R5’s 100–450 ms impacts and low-frequency content.

3. PASS — the false “sub-cm/s” result is removed. The sinusoidal suppression calculation is explicitly characterized as screening, with half-sine/convolution validation required.

4. PASS — shared reference/IMU isolation is withdrawn. The rewrite traces rigid IMU mounting to R5/VN-100 guidance and recommends oscillator-only isolation as judgment.

5. PASS — attenuation is corrected to `10 log10(R)`, giving 5.2–6.9 dB for `R=3.3–4.9`, rounded correctly to 5–7 dB. It is explicitly contingent, not a current requirement.

6. PASS — 8000 Hz/s is now the largest tested all-detected coarse grid point, not a cap. Claims of unavoidable/structural tracker failure are withdrawn.

7. PASS — heave uses `a·uLOS`; maximum stacking is scoped as a conservative screening bound rather than a routine prediction.

8. PASS — `1/v` is retained only as a conditional fixed-distance illustration. Operational `T_ack` and `t_dr` are now to be derived from an explicit response/manoeuvre budget, with the R5/HSC 3–5 s response floor.

9. PASS — both dwells are traced through the state machine. Their retention is justified because they affect recovery/regrant, not revocation latency.

10. PASS — heading calculations now use the authority limits: denied 100 m/2.5° and aided 12 m/1°. Acceptance-limit terminology is withdrawn.

11. PASS — the good-fix timeline is reconciled with `t_dr`. The claimed 2.5-minute authority bridge is explicitly withdrawn, and the definition of a position-constraining observation is routed as open.

12. PASS — the “one measurement clears all six” claim is explicitly withdrawn and replaced with a multi-row evidence matrix.

13. PASS — 11.4× is described as consistent with, not proof of, an assumed 10° heel.

14. PASS — `Δv=cδ` is correctly scoped to a raw single-observation screening bound; estimator/clock-state leakage is left for injection testing.

15. PASS — the claim that speed improves Doppler conditioning is explicitly withdrawn.

16. PASS — R5’s trim, RMS, peak, duration, VN-100, mounting, and response values are reconciled in a dedicated table. Departures are labeled judgment or `[UNVERIFIED]`.

17. PASS — the asserted `Q` multiplier is withdrawn. Planing `Q` is deferred to measured residual spectra and NEES/NIS validation.

18. PASS — `v^1.5–v²` and the resulting scaling are explicitly identified as unsourced judgment and `[UNVERIFIED]`, not an established craft law.

19. PASS — the claim that the cap lies specifically between 20 and 30 kn is withdrawn. The document correctly says the cap may be below 20 kn and requires a craft-specific budget.

20. PASS — antenna motion, spray, multipath, RF availability, power/EMI, crew effects, and sea-state coupling are added and routed.

21. PASS — architecture/contract ownership is now explicit for `t_dr` reset semantics, class selection, heave aiding, dynamic extrinsics, and clock-transient monitoring.

22. PASS — the double-negative textual defect is gone.

## Requested verdict checks

- 20 kn: PASS. The verdict remains “not presently supportable,” but now follows from missing hull/component/human/replay evidence, timer uncertainty, IMU characterization, position observability, and tuning—not a fabricated rectification integrity breach.
- 30 kn: PASS. It remains exploratory/manual-or-aided only. The document distinguishes sourced facts from the explicitly hypothetical speed extrapolation and withdraws the alleged hardware-class change and specifically located 20–30 kn cap.
- Legacy numbers: PASS. Values such as 3 g RMS, 5–50 ms, 10–15 dB, 0.09–0.14 m/s, ±16 g, and the hard 8000 Hz/s interpretation occur only in explicit correction/withdrawal context.
- New findings: none material. The conditional 14–23 kn illustration must not be mistaken for a validated cap, but the document repeatedly states that limitation.

PASS.
tokens used
48,511
Final verdict: PASS

Independent recomputation:

`δDC = Γ₂·aRMS² = 1×10⁻¹¹·0.44² = 1.936×10⁻¹²`

`Δv = c·δDC = 2.9979×10⁸·1.936×10⁻¹² = 5.804×10⁻⁴ m/s`

`0.028 / 5.804×10⁻⁴ = 48.24`

Thus the rewritten value is correctly about 48× below the denied velocity PL.

## Original findings

1. PASS — 3 g RMS is withdrawn; 0.44 g RMS is used; the 48.2× margin is correct. `Γ₂` remains explicitly hypothetical. Isolation is downgraded from integrity-mandatory to availability/cycle-slip recommended.

2. PASS — 5–50 ms/10–100 Hz survives only as withdrawn first-draft context. The active model uses R5’s 100–450 ms impacts and low-frequency content.

3. PASS — the false “sub-cm/s” result is removed. The sinusoidal suppression calculation is explicitly characterized as screening, with half-sine/convolution validation required.

4. PASS — shared reference/IMU isolation is withdrawn. The rewrite traces rigid IMU mounting to R5/VN-100 guidance and recommends oscillator-only isolation as judgment.

5. PASS — attenuation is corrected to `10 log10(R)`, giving 5.2–6.9 dB for `R=3.3–4.9`, rounded correctly to 5–7 dB. It is explicitly contingent, not a current requirement.

6. PASS — 8000 Hz/s is now the largest tested all-detected coarse grid point, not a cap. Claims of unavoidable/structural tracker failure are withdrawn.

7. PASS — heave uses `a·uLOS`; maximum stacking is scoped as a conservative screening bound rather than a routine prediction.

8. PASS — `1/v` is retained only as a conditional fixed-distance illustration. Operational `T_ack` and `t_dr` are now to be derived from an explicit response/manoeuvre budget, with the R5/HSC 3–5 s response floor.

9. PASS — both dwells are traced through the state machine. Their retention is justified because they affect recovery/regrant, not revocation latency.

10. PASS — heading calculations now use the authority limits: denied 100 m/2.5° and aided 12 m/1°. Acceptance-limit terminology is withdrawn.

11. PASS — the good-fix timeline is reconciled with `t_dr`. The claimed 2.5-minute authority bridge is explicitly withdrawn, and the definition of a position-constraining observation is routed as open.

12. PASS — the “one measurement clears all six” claim is explicitly withdrawn and replaced with a multi-row evidence matrix.

13. PASS — 11.4× is described as consistent with, not proof of, an assumed 10° heel.

14. PASS — `Δv=cδ` is correctly scoped to a raw single-observation screening bound; estimator/clock-state leakage is left for injection testing.

15. PASS — the claim that speed improves Doppler conditioning is explicitly withdrawn.

16. PASS — R5’s trim, RMS, peak, duration, VN-100, mounting, and response values are reconciled in a dedicated table. Departures are labeled judgment or `[UNVERIFIED]`.

17. PASS — the asserted `Q` multiplier is withdrawn. Planing `Q` is deferred to measured residual spectra and NEES/NIS validation.

18. PASS — `v^1.5–v²` and the resulting scaling are explicitly identified as unsourced judgment and `[UNVERIFIED]`, not an established craft law.

19. PASS — the claim that the cap lies specifically between 20 and 30 kn is withdrawn. The document correctly says the cap may be below 20 kn and requires a craft-specific budget.

20. PASS — antenna motion, spray, multipath, RF availability, power/EMI, crew effects, and sea-state coupling are added and routed.

21. PASS — architecture/contract ownership is now explicit for `t_dr` reset semantics, class selection, heave aiding, dynamic extrinsics, and clock-transient monitoring.

22. PASS — the double-negative textual defect is gone.

## Requested verdict checks

- 20 kn: PASS. The verdict remains “not presently supportable,” but now follows from missing hull/component/human/replay evidence, timer uncertainty, IMU characterization, position observability, and tuning—not a fabricated rectification integrity breach.
- 30 kn: PASS. It remains exploratory/manual-or-aided only. The document distinguishes sourced facts from the explicitly hypothetical speed extrapolation and withdraws the alleged hardware-class change and specifically located 20–30 kn cap.
- Legacy numbers: PASS. Values such as 3 g RMS, 5–50 ms, 10–15 dB, 0.09–0.14 m/s, ±16 g, and the hard 8000 Hz/s interpretation occur only in explicit correction/withdrawal context.
- New findings: none material. The conditional 14–23 kn illustration must not be mistaken for a validated cap, but the document repeatedly states that limitation.

PASS.
