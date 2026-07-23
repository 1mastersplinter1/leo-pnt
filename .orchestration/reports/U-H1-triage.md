(Grok triage seat — findings require non-Grok confirmation per D5/header.)

## Triage findings — `docs/design/HIGH_SPEED_ENVELOPE.md`

Sources checked: full U-H1 doc; `docs/research/R5-highspeed-dynamics.md`; `docs/design/PARAMS_PROPOSAL.md` §1–3 method; `DECISIONS.md` D45–D48. Arithmetic: ≥15 chains recomputed (speeds, cot, Δv=δ·c, trim residual, Γ₂ rectification, Doppler/heave stacking, timers 1/v, convergence, heading→PL, v³–v⁴, isolation dB).

---

### F1 — HIGH · confidence high · §1.4 / §6.1 / Verdict
**Claim:** Sustained slam environment `a_rms = 3 g` → `δ_DC = 9×10⁻¹¹` → `Δv = 0.027 m/s` “at the denied velocity PL”; 30 kn scales this to 0.091–0.137 m/s (3.3–4.9× over).
**Issue:** Arithmetic of the chain is correct *given* 3 g RMS, but R5’s published anchors are **~0.3–0.5 g RMS** (MK V head seas 0.44 g RMS; STANAG-class 0.2 g). A₁/₁₀ ≈ 2.7–3.2 g is a **peak statistic**, not continuous `a_rms` for `⟨a²⟩`. Using 3 g as RMS overstates `⟨a²⟩` by ~**(3/0.44)² ≈ 46×** vs R5’s verified point. Integrity-grade “at PL / 3–5× over” may be **order-of-magnitude high**.
**How to verify:** Recompute δ_DC with R5 RMS band (0.3–0.5 g) and separately with crest-factor models; require hull-measured PSD/`a_rms` before freezing isolation.

### F2 — HIGH · confidence high · §1.4 table / §1.5 isolator / R5 §1
**Claim:** Slam duration **5–50 ms**; dominant content **10–100 Hz** via `1/(2·duration)`; isolator corner **~10–15 Hz** to attenuate that band.
**Issue:** Material conflict with R5 VERIFIED NSWCCD: impact durations **100–450 ms** → energy largely **≲ few–10 Hz**. Doc durations ~**10× short**. Isolator design (corner above 0.5–3 Hz encounter, attenuate 10–100 Hz) **misses R5’s long-pulse content**; R5 explicitly warns long slams sit **below many isolator resonances** and need low-fn + large stroke.
**How to verify:** Side-by-side table doc vs R5 §1.1/§1.3/cross-topic; re-derive transmissibility against 100–450 ms half-sine SRS.

### F3 — HIGH · confidence high · §3.2 vs §1.4 vs §6.1
**Claim:** §1.4 LCG peaks **2–6 g**, bow **10–20+ g**; §6.1 “At 20 kn (**10 g LCG**, ~7400 Hz/s) the 128-sample block still barely coped.”
**Issue:** **Internal inconsistency.** 10 g is outside the §1.4 LCG band (it is the bow floor). Tracker stacking at 20 kn is argued with bow-class g labeled as LCG; 30 kn “LCG 10–14 g” only matches §1.4 if you take the `v²` upper edge of an already-stretched base.
**How to verify:** Fix station labels (LCG vs bow) and re-run stack table with consistent g bands; check mount location assumption (doc prefers LCG).

### F4 — HIGH · confidence high · §6.1 isolation dB
**Claim:** At 30 kn, isolation must deliver **10–15 dB more** attenuation than the 20 kn case to bring 0.09–0.14 m/s into the 0.028 m/s PL.
**Issue:** Ratio 0.091/0.028 ≈ 3.25×, 0.137/0.028 ≈ 4.9×. Because bias ∝ `⟨a²⟩`, required **power/`⟨a²⟩` attenuation is 10 log₁₀(ratio) ≈ 5.1–6.9 dB**, not 10–15 dB. 10–15 dB matches **20 log₁₀(ratio)** — treating the Δv ratio as if it scaled with `a` once, contradicting the doc’s own Γ₂/`a²` model. Overstates isolation hardness by ~2× in dB.
**How to verify:** Recompute required transmissibility `T` with `Δv ∝ T²⟨a_in²⟩`; report both amplitude dB (20 log T) and power dB consistently.

### F5 — HIGH · confidence high · §3.5 / §4.4 vs PARAMS §1.3
**Claim:** “2° aided heading **PL** (`PARAMS §1.3`)”; table “5° (`PARAMS §1.3` denied)” for denied PL breach timing.
**Issue:** PARAMS §1.3: **PL aided = 1.0°**, **PL denied = 2.5°**; **2° / 5° are acceptance errors**, not PLs. Citation is wrong. Direction of operational conclusion (tight times) **worsens** if one uses true PL (1°/2.5° error → shorter times to position PL) *or* softens if 5° was meant as worst acceptance — either way the table is mis-anchored.
**How to verify:** Recompute distance/time with θ = PL (1.0°/2.5°) and with θ = acceptance (2°/5°); label which; fix PARAMS cross-ref.

### F6 — MEDIUM · confidence high · §1.4(a)
**Claim:** 10 g LCG slam → instantaneous Δv = 3 m/s, suppressed ~150× over τ=1 s → “**sub-cm/s** in the integrated Doppler.”
**Issue:** 3 / 150 = **0.02 m/s = 2 cm/s**, not sub-cm/s. At 10 Hz content residual is worse (~0.1 m/s). Wording overclaims by ≥2×.
**How to verify:** 3/(π f τ) for f∈{10,50,100} Hz; correct “sub-cm/s” → “~cm/s-class” or give f-dependent residual.

### F7 — MEDIUM · confidence high · §1.3 trim vs R5 §2
**Claim:** Planing sustained trim typically **3–6°**; 30 kn “~3–5°.”
**Issue:** R5 VERIFIED efficient planing **2–4°**, optimum often **~2–3°**; **>6°** uneconomical. Doc’s high end sits at R5’s over-trim boundary. cot benefit still large at 2–4°, but “3–6° typical” is **high-biased** vs R5.
**How to verify:** Replace with R5 range or mark ASSUMED vs VERIFIED; recompute cot at 2°/4°.

### F8 — MEDIUM · confidence high · §1.4 peaks vs R5 §1
**Claim:** Bow **10–20+ g**; LCG **2–6 g** (order-of-magnitude estimates).
**Issue:** R5: crew/cox station peaks ~**6–9 g** class (MK V peak 8.62 g head seas), A₁/₁₀ 2.7–3.2 g “max safe,” equip lab **20 g / 23 ms**. Doc bow band is **more severe** than R5 full-scale HSC peaks at crew station (possible for small-RIB bow, but not R5-sourced). RMS/peak vocabulary conflation (F1) compounds this.
**How to verify:** Map each g number to R5 row (RMS / A₁/₁₀ / peak / lab pulse) with station (LCG/bow/crew).

### F9 — MEDIUM · confidence high · §1.5 / §3.4 / M-1 vs R5 §4.2–4.3
**Claim:** Shared shock isolation for **reference + IMU** on one plate is REQUIRED / preferred path.
**Issue:** R5 VERIFIED VN-100: **rigid mount preferred**; isolation “hard to get right” and can **degrade AHRS**; ~**4.5 g RMS** random can saturate accels. Doc’s shared-isolator BOM delta is in tension with IMU-vendor guidance (isolating the clock may still be right; co-isolating the AHRS is not free).
**How to verify:** Split M-1: isolate OCXO/Rb vs hard-mount or carefully designed IMU path; cite VN/SBG notes.

### F10 — MEDIUM · confidence medium · §3.3 unlabeled dynamics
**Claim:** Planing surge ~0.1–0.3 g, heave ~0.5–2 g RMS, slam peaks 3–10 g; “one to two orders of magnitude” vs displacement; planing `Q` likely 10²–10⁴×.
**Issue:** Magnitude bands drive the “D43 Q invalid” conclusion but are only partially tagged (Q has [UNVERIFIED]; g bands are bare). Heave “0.5–2 g **RMS**” again conflicts with R5 ~0.44 g RMS unless “in slam bursts” is meant.
**How to verify:** Tag every g figure estimate/UNVERIFIED; align RMS language with R5.

### F11 — MEDIUM · confidence high · §3.5
**Claim:** Fast turn **30°/s**, mag **~100 ms** latency → ~3° gyro bridge → breaches “2° aided PL.”
**Issue:** 30°/s and 100 ms are **unlabeled estimates**. Even with correct 1° PL (F5), need source for rate/latency. Arithmetic 30°/s × 0.1 s = 3° is fine.
**How to verify:** Cite AHRS/mag rate contract or mark [UNVERIFIED]; use PARAMS PL 1.0°.

### F12 — LOW · confidence high · §2.4 vs §6.1 timer class consistency
**Claim:** Class B `t_dr ≈ 40 s` (= 120×0.35 rounded from 42); exploratory 30 kn `t_dr ≈ 28 s` (= 120×0.233).
**Issue:** Pure 1/v from Class B 40 s → 40×(20/30) ≈ **26.7 s**, not 28 s. Harmless rounding split (raw 7 kn base vs rounded Class B). Document which base is normative.
**How to verify:** State “always scale from Class A raw” or “from frozen Class B.”

### F13 — LOW · confidence high · arithmetic spot-checks that **PASS**
Recomputed and consistent: speeds 7/20/30 kn; `v` ratios 2.86/4.29/1.50; cot(θ/2) 38×/19×/11×; Δv=δ·c table; trim residual 5.5×10⁻¹² → 1.6×10⁻³ m/s; vessel Doppler 136/389/583 Hz Ku; heave rates 370/3702 Hz/s; stacks 7420/9271/11122; timer 1/v and distances (103 m / 52 m / 78 m); convergence table; heading times ~637/223/149 s and 199/70/46 s; 500 km @ 20 kn = 13.5 h; 100 km @ 30 kn = 1.80 h; v^1.5/v^2 factors 1.84/2.25; 30 kn Δv_DC 0.091–0.137 if 3 g base held. D46/D47 scenario numbers (12 km / 1.8 h / 9–18 km) match.

### F14 — MEDIUM · confidence medium · unlabeled / lightly labeled numbers
Besides F10–F11: Froude 8–12 kn and 3–6° trim are “estimate” but many **derived integrity numbers** (isolation ≥20 dB, 10–15 Hz corner, Γ₂ 1×10⁻¹¹, a_rms 3 g) sit in prose where only some carry [UNVERIFIED]. Intro claims every introduced value is [UNVERIFIED]; not every numeric is so tagged in-line (process discipline gap, not always wrong physics).
**How to verify:** Pass for bare numbers without estimate/UNVERIFIED/sourced-json tag.

### F15 — HIGH · confidence high · missing-consequence (scope hole vs R5 §6 + ops)
**Doc largely omits high-speed effects that R5 and ops care about:**
1. **Antenna pointing / servo rate / slam attitude** and **spray–wash on L/Ku** (R5 §6; dead-end on planing spray trials — still a risk register row).
2. **RF multipath / sea-surface** at planing speeds (tracker integrity, not only Doppler rate).
3. **Power / EMI** at planing throttle (mag calibration called out; electrical noise on reference/bladeRF not).
4. **Crew / human factors** beyond `T_ack` (R5 A₁/₁₀ “extremely uncomfortable,” ability to take helm after slam train).
5. **Cooling / environmental** (spray, isolator foam, Rb warm-up under wet high-speed install).
6. **Sea-state (Hs) coupling** — all slam numbers fixed; class rules n_cg scales with **V and Hs** (R5).
7. **Speed-log / propulsion sensor** behavior on plane (ventilation, cavitation) — freshness timers unchanged without a sensor-physics note.
8. **Structural fatigue / isolator stroke** under repeated long-duration slams (R5).
9. **D48** bearings-only AoA is a related heading fix path (DECISIONS) — not required inside U-H1, but §4.4 heading crisis does not point at it.

**How to verify:** Checklist R5 §§5–6 + BOM/environment against §5 consequence register; add rows or explicit “out of scope.”

### F16 — MEDIUM · confidence medium · §4.2 ephemeris / D45
**Claim:** 2.6 km @ 24 h → `σ_add(24 h) ≈ 18.42 m/s`; 24 h passage leaves 6 h to 30 h ceiling.
**Issue:** Passage timing arithmetic is fine and matches D45/D46 intent. `18.42 m/s` from 2.6 km is **not re-derived here** (U-P1-internal); inherited synthetic caveat is stated. No material D45 conflict found; cache-at-departure gate aligns with D45 “cached no later than departure.”
**How to verify:** U-P1 fit equation for σ_add(age).

### F17 — LOW · confidence high · 20 kn body vs 30 kn extension (consistency)
Aside from F3 (LCG g) and F4 (dB): ratios 1.5× on distance/time/Doppler/heading clocks are consistent; timer class exploration (cap vs third class) is coherent with Class B logic; good-fix vs cold-start narrative matches D47; rectification “class change” vs estimator “margin tighten” is a clear structure. **Main 20↔30 fractures are F3 and F4.**

### F18 — MEDIUM · confidence medium · §0 / Verdict posture vs evidence
**Claim:** Fail-closed to displacement until measured slam spectrum; 20 kn not supportable on present evidence; 30 kn exploratory + denied cap.
**Issue:** Posture matches SAFETY_CASE/PARAMS fail-closed method and D46/D47. But load-bearing “rectified bias at PL” (F1) and isolator feasibility (F2/F4) are on **weak/conflicted bases** — verdict direction (isolation mandatory, 30 kn harder) may survive, **quantitative severity may not**.
**How to verify:** After F1–F4 fixes, re-issue §5.5 / both verdicts.

---

### Spot-check summary (pass/fail per chain)

| # | Chain | Result |
|---|--------|--------|
| 1 | kn → m/s, v ratios | PASS |
| 2 | cot(θ/2) 3°/6°/10° | PASS |
| 3 | Δv=δ·c table | PASS |
| 4 | trim δ / Δv at 6° | PASS |
| 5 | Γ₂ rectification @ 3 g | PASS arith; FAIL vs R5 premise (F1) |
| 6 | Doppler vessel / heave rate | PASS |
| 7 | sat+heave stack sums | PASS |
| 8 | timer 1/v, distances | PASS |
| 9 | convergence v·T | PASS |
| 10 | heading→PL times 7/20/30 | PASS arith; FAIL PARAMS label (F5) |
| 11 | v³–v⁴ / 30 kn Δv_DC | PASS arith given 3 g |
| 12 | isolation “10–15 dB more” | FAIL (F4) |
| 13 | “sub-cm/s” after 150× | FAIL (F6) |
| 14 | passage 13.5 h / 1.80 h | PASS |

---

VERDICT: FAIL

**Rationale:** Multiple high-severity issues: R5-material conflicts on slam duration/spectrum and on `a_rms` driving the integrity case (F1–F2); internal LCG vs 10 g labeling (F3); wrong isolation dB conversion (F4); PARAMS heading PL mis-cite (F5); plus major missing high-speed consequences (antenna/spray, power, crew, Hs, cooling) (F15). Core cot/Δv/timer/convergence arithmetic is largely solid; document is not load-bearing-clean for non-Grok confirmation without rework.
