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

## Verdict assessment

The narrow conclusion “20 kn denied operation is not supportable today” follows from the fail-closed status of the timer set, uncharacterized hull environment, unverified IMU/reference behavior, tracker production gaps, and heading/position limitations. It does **not** require the speculative `Γ₂ = 10⁻¹¹/g²` integrity breach, and that mechanism should not be presented as the central proven reason.

Likewise, “do not support 30 kn denied authority today” follows. The stronger claims that 30 kn necessarily requires 10–15 dB additional isolation, that no tracker block can survive, that loss of lock is unavoidable, and that the cap boundary lies specifically between 20 and 30 kn do not follow.

The report `.orchestration/reports/U-H1.md` faithfully repeats the document’s calculations, but it also repeats its unsupported 3 g RMS/`Γ₂` premise, 5–50 ms slam model, hard 8000 Hz/s tracker cap, timer scaling, and good-fix interpretation. It therefore does not provide an independent reconciliation.

FAIL
