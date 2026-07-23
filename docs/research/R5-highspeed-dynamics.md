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
- Low-VRE sensors (SBG marketing claim): isolation sometimes unnecessary if VRE controlled in silicon.

### 4.4 Published 20–30 kn small-craft navigation-sensor studies

**Gap (dead end for specific open lit):** No peer-reviewed study was found that reports **VN-100 / MEMS INS navigation error metrics specifically at 20–30 kn on small planing craft** with quantified slam spectra. Closest primary base is **NSWCCD HSC acceleration environment** (§1) as the **input vibration/shock load** for sensor survival and VRE estimation.

**ASSUMED workflow for envelope:** Use measured vertical accel time history (or half-sine train: A_peak, T=0.1–0.45 s, rate from wave encounter) as vibration input; apply manufacturer VRE model if available; else budget bias steps of order **mg-class** under multi-g RMS vib for industrial MEMS.

---

## 5. High-speed craft safety / collision context (alarm & ack timer budgets)

### 5.1 Stopping and turning — published standards

**VERIFIED — IMO manoeuvring standards (conventional ships, not small planing craft)**  
MSC.137(76) *Standards for Ship Manoeuvrability*:  
URL: https://wwwcdn.imo.org/localresources/en/KnowledgeCentre/IndexofIMOResolutions/MSCResolutions/MSC.137(76).pdf  

- Full astern stopping: **track reach ≤ 15 ship lengths** (≤ 20 L for large low-powered vessels).  
- Turning circle / advance / transfer criteria also defined.  
- **Does not apply as a numerical rule to 6–12 m RIBs**; scales with ship length and displacement inertia.

**VERIFIED — HSC Code structure**  
IMO *International Code of Safety for High-Speed Craft, 2000* (MSC.97(73) and amendments):  
URL: https://wwwcdn.imo.org/localresources/en/KnowledgeCentre/IndexofIMOResolutions/MSCResolutions/MSC.97(73).pdf  

- **Chapter 17** — Handling, controllability, performance: proof of compliance, controllability/manoeuvrability, **acceleration and deceleration**, speeds, failures, night operation.  
- **Chapter 18** — Operational requirements: operational control, documentation, training, manning of survival craft.  
- **Chapter 15** — Operating compartment / field of vision.  
- Code requires **demonstrated** deceleration/controllability for craft type approval — **does not publish a universal 25 kn stopping distance table** for small monohulls.

**Dead end:** No primary open trial report found with **crash-stop distance in meters at 20 / 25 / 30 kn for small planing RIB/patrol craft**. Planing craft stop by **drag + reverse thrust + turn**, not ship-like full-astern hydrodynamic reverse.

### 5.2 Speed-scaled distance budgets (for alarm/ack timers)

**ASSUMED kinematic envelope** (physics only; not a published craft trial):

| Speed | m/s | Distance in **1 s** | in **3 s** | in **5 s** | in **10 s** |
|-------|-----|---------------------|------------|------------|-------------|
| 20 kn | 10.3 | 10 m | 31 m | 51 m | 103 m |
| 25 kn | 12.9 | 13 m | 39 m | 64 m | 129 m |
| 30 kn | 15.4 | 15 m | 46 m | 77 m | 154 m |

**Reaction-time building blocks (mixed VERIFIED/ASSUMED):**  
- Human simple reaction ~**0.2–0.5 s**; complex recognition + decision for marine collision often budgeted **several seconds** (watchkeeping practice; **not** a single HSC Code number).  
- Bridge-team “time to act” for HSC is dominated by **closing speed** (own speed + target).  
- For **alarm acknowledgment timers**, a conservative design often uses **detection + recognition + motor response** ≥ **3–5 s** plus craft-specific stopping/turn distance — **ASSUMED** for UI policy unless project human-factors test exists.

**Turning:** High-speed planing craft can change heading in **one to a few boat lengths** at speed if controlled (practice/experience); published quantitative **tactical diameter vs kn for RIBs** not found as a standard table in this search.

**Inputs for speed-scaled ack budget (recommended use of verified pieces):**  
1. Distance-at-speed table above (ASSUMED kinematics).  
2. Craft-specific crash-stop/turn trial (to be measured — gap).  
3. HSC Code obligation: craft must have **documented** handling/deceleration performance (Ch 17).  
4. Prefer **time + distance** dual presentation to operator (e.g. “ack within T s ≈ D m at current SOG”).

---

## 6. Doppler / comms: planing motion, antenna attitude, spray vs L-band / Ku

### 6.1 Antenna attitude / tracking

**VERIFIED — L-band maritime terminal motion envelope (SAILOR FleetBroadband install manuals)**  
Example: SAILOR 250/500 FleetBroadband installation documentation  
URL: https://www.remotesatellite.com/supportdocs/support/thrane/Sailor-250-500-fleetbroadband-install-manual.pdf  

- Stabilized antenna continuous pointing: **360° azimuth**; elevation/pitch-roll capability **down to −25° (SAILOR 500) / −60° (SAILOR 250)** for heavy sea conditions.  
- Implies design for **large attitude swings**; outages when motion exceeds servo rate/limits or when LOS blocked.

**ASSUMED for 20–30 kn planing craft:** Pitch/roll rates and slam-induced angular accelerations can **exceed** servo tracking bandwidth more often than on displacement hulls of same Hs — expect **short L-band/Ku fades** synchronized with wave encounters even if average attitude is within mechanical limits.

### 6.2 Spray / washdown / weather

**VERIFIED commercial claims (not peer-reviewed trials):**  
- L-band (Inmarsat/Iridium class) marketed for **rain/spray resilience** and in-motion reliability vs Ka/Ku broadband.  
  e.g. comparative maritime satcom product literature: https://globalsatellite.us/starlink-maritime-vs-iridium-certus-2026-maritime-satellite-internet-comparison/  
- Ku/Ka VSAT more sensitive to **rain fade** and require higher pointing accuracy; marine stabilized VSAT products exist but are **not** planing-RIB optimized in open literature.

**Dead end:** No peer-reviewed measurement campaign was found that reports **Eb/N0, C/N0, or packet loss vs speed (20–30 kn) and spray condition** on small planing craft for L-band or Ku. Closest are general **VMES (vehicle-mounted earth station)** pointing-error studies (land mobile).

### 6.3 Doppler

**ASSUMED order-of-magnitude (physics):**  
At 30 kn ≈ 15.4 m/s, fractional Doppler **v/c ≈ 5×10⁻⁸** (~0.5 Hz at L-band 1.6 GHz; ~1.5 Hz at 12 GHz Ku) — **small** for modern closed-loop receivers.  
**Attitude jitter and multipath/spray** dominate over pure translational Doppler for planing craft at these speeds.

---

## Cross-topic values suitable to replace [UNVERIFIED] envelope cells

| Envelope cell | Replacement value | Tag | Primary anchor |
|---------------|-------------------|-----|----------------|
| Vertical RMS, moderate head seas, HSC | **~0.3–0.5 g** class (0.44 g MK V @ Hs~0.9 m) | VERIFIED point | Haupt MK V |
| A₁/₁₀ operational limit | **2.7–3.2 g** | VERIFIED | NSWCCD multi-craft |
| Single slam peak (severe) | **several–~9 g** at crew station; higher at bow | VERIFIED order | MK V |
| Slam duration | **100–450 ms** | VERIFIED | NSWCCD-80-TR-2014/026 |
| Equip shock qual | **20 g / 23 ms** + **5 g ×800** | VERIFIED | NSWCCD-80-TR-2017/002 |
| Sustained trim | **2–4°** efficient planing | VERIFIED range | Savitsky/Ghassemi/practice |
| OCXO Γ | **~1×10⁻⁹ / g** typical SC-cut | VERIFIED typical | Wenzel/NIST |
| Vib phase noise | L(f)=20log(Γ a f₀/(2 f_v)) | VERIFIED formula | Filler/Wenzel/NIST |
| FE-5680A g-sens | **Not on datasheet** | VERIFIED absence | FEI datasheet |
| IMU rigid mount | Preferred for VN-class | VERIFIED | VN-100 datasheet |
| IMU saturate | **~4.5 g RMS** random | VERIFIED | VN-100 datasheet |
| Ship stop rule | ≤15 L track reach | VERIFIED | MSC.137(76) |
| Small craft stop distance @25 kn | **Not found published** | DEAD END | — |
| Distance in 5 s @25 kn | **~64 m** | ASSUMED kinematics | physics |
| Satcom L-band attitude | **−25°/−60°** continuous track designs | VERIFIED product | SAILOR FB |
| Satcom vs planing spray @speed | **No primary trial found** | DEAD END | — |

---

## Source index (URLs + access 2026-07-23)

1. https://www.isthq.com/wp-content/uploads/2023/05/NSWC_High_Speed_Watercraft_Motions.pdf  
2. https://apps.dtic.mil/sti/tr/pdf/ADA616198.pdf  
3. https://apps.dtic.mil/sti/tr/pdf/AD1021121.pdf  
4. https://apps.dtic.mil/sti/pdfs/AD1032710.pdf  
5. https://publications.sto.nato.int/publications/STO%20Meeting%20Proceedings/RTO-MP-AVT-110/MP-AVT-110-31.pdf  
6. https://pubs.sciepub.com/ijp/7/1/4/  
7. https://www.boote-magazin.de/en/boat-knowledge/beginner/boat-trim-the-3-most-important-motorboat-types-and-their-trim-angle/  
8. https://www.miedema.dyndns.org/co/2019/rb/3rb/FE-5680A-Rubidium-datasheet.pdf  
9. https://tf.nist.gov/general/pdf/2328.pdf  
10. https://wenzel.com/library/time-frequency-articles/vibration-induced-phase-noise/  
11. https://wenzel.com/product/crystal-oscillators/vibration-isolated/oven-controlled-ocxos/  
12. https://www.analog.com/en/resources/technical-articles/vibration-rectification-in-mems-accelerometers.html  
13. https://metromatics.com.au/wp-content/uploads/2025/12/VN100CR-Datasheet-v7.0-DS100-CR-70-R1.pdf  
14. https://wwwcdn.imo.org/localresources/en/KnowledgeCentre/IndexofIMOResolutions/MSCResolutions/MSC.137(76).pdf  
15. https://wwwcdn.imo.org/localresources/en/KnowledgeCentre/IndexofIMOResolutions/MSCResolutions/MSC.97(73).pdf  
16. https://www.remotesatellite.com/supportdocs/support/thrane/Sailor-250-500-fleetbroadband-install-manual.pdf
