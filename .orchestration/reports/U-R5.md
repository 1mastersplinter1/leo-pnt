# U-R5 Report

(Stdout contract per D9/D16; first run silent-exited, this is run 2.)

# U-R5 Unit Report — High-speed marine dynamics research

## Summary
Live-web research replaced several [UNVERIFIED] planing-envelope estimates with **NSWCCD full-scale HSC acceleration statistics**, **impact durations (100–450 ms)**, **coxswain A₁/₁₀ limits (2.7–3.2 g)**, **equipment shock quals (20 g/23 ms; 5 g×800)**, **planing trim 2–4°**, **OCXO Γ~1e-9/g + Filler phase-noise formula**, **FE-5680A datasheet limits (no Γ)**, **VN-100 shock/vib + rigid-mount guidance**, and **IMO/HSC Code** manoeuvring/operational structure. Two major gaps remain: **published small-craft crash-stop distances at 20–30 kn** and **peer-reviewed L/Ku satcom quality vs planing speed/spray**.

## VERIFIED (read primary / datasheet)
- MK V vertical accel table (peak 8.62 g, A₁/₁₀ 4.30 g, RMS 0.44 g head seas, Hs~0.9 m).  
- Impact duration 100–450 ms (NSWCCD-80-TR-2014/026).  
- Half-sine rigid-body model + StandardG analysis methods (NSWCCD-80-TR-2016/003).  
- A₁/₁₀ max-safe ~2.7–3.2 g; equip tests 20 g/23 ms & 5 g×800 (NSWCCD-80-TR-2017/002).  
- STANAG 4154 0.2 g RMS cited; HSC applicability questioned (RTO-MP-AVT-110).  
- Optimum planing trim ~2–3° (Ghassemi/Savitsky); practice 2–4°.  
- FE-5680A stability/PN specs; **Γ not specified**.  
- Γ~1e-9/g SC-cut typical; L(f)=20log(Γ a f₀/(2 f_v)) (Wenzel/NIST).  
- Isolation: fn, damping, wire-rope/elastomer practices (Wenzel).  
- VRE definition (ADI); VN-100 500 g shock, 6 g sine, rigid mount, 4.5 g RMS saturate.  
- MSC.137(76) ≤15 L track reach; HSC Code Ch 17/18 structure.  
- SAILOR FB pitch/roll tracking limits (−25°/−60°).

## ASSUMED
- Mapping MK V / large-HSC stats onto **small RIB at exactly 20–30 kn**.  
- Class-rule n_cg formulas (ABS/ISO/DNV) coefficients not re-derived from paid full text.  
- FE-5680A under slam vib behaves like VCXO-limited Γ (not datasheet).  
- EU prices for isolators/OCXOs (order-of-magnitude).  
- Kinematic distance-at-speed table for alarm timers.  
- Multi-second watchkeeping reaction budgets without HSC-specific HF trial.  
- Doppler negligible vs pointing/spray at ≤30 kn.  
- IMU VRE class ranking where datasheets omit numbers.

## Dead ends
1. **Open matrix of peak/RMS/A₁/₁₀ vs (V=20–30 kn)×(Hs)** for small RIB/patrol boats — not found as one primary table.  
2. **FE-5680A g-sensitivity / vib PN** — absent from commercial datasheet.  
3. **VN-100 (and many MEMS) VRE µg/g²** — not published.  
4. **Measured stopping/turning distances at 20–30 kn for small planing craft** — not found in open primary literature.  
5. **Peer-reviewed L-band/Ku C/N0 or BER vs planing speed + spray** — not found; only product install limits and marketing resilience claims.  
6. **Southampton full-scale slamming PDF** (Camilleri et al.) and some ISO 12215 paid texts — fetch failed or paywalled; not used as VERIFIED.

## Confidence for envelope replacement
- **High:** slam duration, half-sine model, equipment shock quals, trim 2–4°, phase-noise formula, OCXO Γ order, IMU mount practice.  
- **Medium:** absolute g levels at 20–30 kn moderate seas (use NSWCCD orders + craft trial).  
- **Low / open:** small-craft stop distances; satcom fade rates at speed; Rb Γ under slam.

## Recommended next measurements (if envelope must be fully VERIFIED)
1. Craft-specific seakeeping: 20/25/30 kn × Hs 0.5/1.0/1.5 m, head/bow — log A_peak, A₁/₁₀, RMS, impact rate, duration.  
2. Crash-stop and hard-over distances at those speeds.  
3. OCXO/Rb Γ on shaker + one-at-sea phase-noise recording.  
4. IMU dual-mount (rigid vs isolated) VRE/bias walk under same runs.  
5. L-band C/N0 time series vs spray events if comms is in scope.
