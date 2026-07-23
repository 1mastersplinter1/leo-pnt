# U-R6 Report

(Stdout contract.)

# U-R6 Report — Unknown-Emitter AoA Array Research  
**Contract v5.1 | Access date: 2026-07-23 | Live web research**

## Mission restatement
Produce **feasibility inputs** (not a build) for a **4–5 element coherent array** on a **7–30 kn vessel**, DF’ing **unknown land emitters**, fusing **bearings-only** into an **error-state EKF**. Claims split **VERIFIED [V]** / **ASSUMED [A]** with URL + access date.

---

## Findings by brief section

### §1 Receivers — COTS coherent multi-channel
**Verdict: Feasible at research cost; KrakenSDR is the default 5-ch solution.**

| Item | Result | V/A |
|---|---|---|
| KrakenSDR | 5-ch coherent RTL, 24–1766 MHz, auto noise-source phase cal, MUSIC/CI software, **~$749–750**, EU stock (SDRstore.eu etc.) | **[V]** |
| Published DF ° RMS (Kraken) | **No official metrology number** on vendor pages; community geo anecdotes only | **[V]** absence / weak |
| bladeRF 2.0 xA4 | **$540**, 2×2 MIMO, 47 M–6 GHz, wide IBW — adds BW/freq, **not** N=5 MUSIC alone | **[V]** |
| USRP B210 | **~$2387**, coherent 2×2, research workhorse | **[V]** |
| Pro CI benchmark | R&S: **≤1° ideal**; WiNRADiO: **~2° RMS** field, **<0.5°** instrumental | **[V]** |
| Array extension | Multi-SDR LO/ref distro; dual-front-end hybrid; pro 5–9 el UCA | **[A/V]** |

**Cannot do with 2-ch alone:** full-rank 5-element spatial spectrum; expect dual-element interferometry only unless multiple synchronized units.

### §2 DF accuracy on small moving arrays
**Verdict: Instrumental 1–2° is realistic only in clean multipath and adequate D/λ; marine field often 5–20°+.**

- MUSIC/CI accuracy vs SNR/aperture is textbook-strong; **multipath correlation destroys superresolution** without smoothing. **[V]**  
- Sea paths: **two-ray / surface multipath** distort phase manifolds. **[V]**  
- Low-VHF clutter DF can be catastrophic (tens–100+°). Open water is better but not ideal. **[V/A]**  
- Mast multipath from superstructure is a first-order install risk (AIS community experience). **[V]**  

**Aperture match [A]:** 1–2 m is **weak at FM (~0.3–0.7 λ)**, **usable at DAB (~1 λ)**, **strong at 700–900 MHz (several λ)**.

### §3 Bearings-only SLAM / literature
**Verdict: Theoretically sound; observability is the gate; marine RF-unknown multi-beacon EKF demos are sparse.**

- **Nardone–Aidala:** unique BO solutions require **ownship acceleration** (with known pathological unobservable maneuvers). **[V]**  
- Static unknown emitters + multi-beacon geometry relax but do not remove need for **cross-range baseline**. **[A]**  
- Bearings-only SLAM literature (EKF, exploration policies, reverse BO-TMA for vehicle nav) supports the architecture. **[V]**  
- **TDOA** needs spatial baseline between sensors; **FDOA/Doppler** works from a single moving platform and is a natural **2-ch add-on**. **[V]**  
- Broadcast-as-beacon at sea is less published than land SoOp / passive radar — treat fusion performance as **experimental**. **[A]**

### §4 Danish / Baltic emitter landscape
**Verdict: Nearshore beacon-rich; mid-offshore thins to broadcast + AIS shore; cellular dies first.**

| Source | Density / reach | V/A |
|---|---|---|
| FM | **>590 transmitters** in DK (radiomap); kW-class powers; receivable tens of km | **[V/A]** |
| DAB+ | Nationwide 3 muxes since 2017; Band III | **[V]** |
| DVB-T2 | Coastal UHF towers; wideband | **[A/V]** |
| Cellular | ~**8 km** phone-like; **30–70 km** with maritime gain setups | **[V]** |
| AIS shore / VTS | Dense fairway coverage (HELCOM/DK VTS); VHF ~**20 NM** | **[V]** |

**Best bands for 1–2 m, 4–5 el [A]:** (1) cellular low-band, (2) DAB III, (3) DVB-T2 slice, (4) FM coarse, (5) AIS base. **Watch SFN** on DAB/DVB.

### §5 Speed 7–30 kn
**Verdict: Speed buys geometry, costs integration stability.**

- Baseline in fixed time scales with speed (**~4×** from 7→30 kn). **[A]**  
- Planing regime (**~12–18 kn+**) increases vibration/slam → phase center motion. **[A/V]**  
- Optimum for fusion likely **mid teens to low 20s kn** with short DF snapshots + AHRS; validate at planing separately. **[A]**

---

## Feasibility conclusions (for EKF fusion program)

1. **Hardware path is real and EU-accessible:** KrakenSDR 5-ch is sufficient to start AoA; bladeRF/B210 optional for wideband/FDOA. **[V]**  
2. **Do not plan on 1° marine bearings** from a 1–2 m mast without measurement; design EKF for **adaptive R** and outliers. **[A]**  
3. **Observability must be designed into the voyage** (maneuvers or multi-beacon coasts), not assumed from constant-velocity legs. **[V→A]**  
4. **Band strategy:** prioritize **DAB + low cellular** for array physics; use **FM** for power/detection; treat **SFN** carefully. **[A]**  
5. **High speed is a net win only if** mechanical phase stability and attitude estimation keep bearing quality above the information-theory floor. **[A]**

## Critical gaps / next measurements (not in brief, but gate feasibility)
- On-water **bearing RMS vs band/SNR/sea state** with the actual mast geometry.  
- **SFN bias** characterization for DK DAB multiplexes offshore.  
- Array **phase stability** at planing speeds (accelerometers on mast).  
- End-to-end **EKF NEES / map convergence** on a known coastal tower set (truth via public transmitter databases).

## Claim hygiene
All numbered vendor/spec/literature statements in the DOC section carry URLs and are tagged **[V]** or **[A]**. Access date for live retrieval: **2026-07-23**. No repository files were written (stdout contract only).
