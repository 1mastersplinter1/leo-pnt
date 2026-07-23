# U-R6 — Unknown-Emitter AoA Array: Feasibility Research Inputs

**Contract:** v5.1 report rules  
**Access date for all live sources:** 2026-07-23  
**Purpose:** Feasibility inputs for a 4–5 element coherent antenna array doing direction finding (DF) on **unknown land-based emitters** from a vessel at **7–30 kn**, fused as **bearings-only measurements** into an existing **error-state EKF** (bearings-only SLAM / multi-beacon map+pose).  
**Claim discipline:** Every factual claim is tagged **[V]** VERIFIED (primary/vendor/standard literature with URL) or **[A]** ASSUMED (engineering inference; not a measured claim from a cited source).

---

## 0. Executive synthesis (decision-relevant)

| Question | Bottom line | Confidence |
|---|---|---|
| Is COTS 4–5 ch coherent RX feasible at hobby/research cost? | **Yes.** KrakenSDR is the clear 5-ch COTS package (~USD 750 / EU stock). bladeRF/USRP-class 2-ch adds bandwidth & upper freq but **not** 5-element MUSIC quality alone. | High |
| Usable DF accuracy on a small mast array (1–2 m) at VHF/UHF? | **Instrumental CI/MUSIC ~1–2° RMS ideal; field marine 3–15°+** depending multipath, SNR, aperture in wavelengths, platform motion. | Medium–High |
| Can bearings fuse into error-state EKF as unknown emitters? | **Yes in principle (BO-TMA / bearings-only SLAM).** Observability requires **ownship acceleration / non-radial motion baseline**; pure constant-velocity straight runs are weakly or un-observable for range. | High (theory); Medium (marine RF field) |
| Baltic/Danish coastal emitter density? | **FM/DAB dense nearshore; DVB-T2 present; cellular fades beyond ~10–30+ km unless high-gain; AIS shore + VHF marine always coastal.** Best array match: **DAB Band III + cellular 700–900 MHz**; FM is power-rich but **aperture-starved** at 1–2 m. | Medium |
| Does 20–30 kn help? | **Net positive for observability (geometry rate)** if attitude/heading and array phase stability are controlled; **negative for integration time and vibration** at planing. | Medium |

---

## 1. COTS coherent multi-channel receivers

### 1.1 KrakenSDR (5-ch coherent RTL) — primary baseline

| Attribute | Value | Tag | Source |
|---|---|---|---|
| Channels | 5 RX, single LO, designed for phase-coherent DF | **[V]** | https://www.crowdsupply.com/krakenrf/krakensdr |
| Price (MSRP, unit) | **USD 749** (krakenrf.com); **USD 750** (Crowd Supply, aluminum case) | **[V]** | https://www.krakenrf.com/product-page/krakensdr ; https://www.crowdsupply.com/krakenrf/krakensdr |
| Antenna kit | 5× mag-mount telescopic whips + length-matched cables ~**USD 199** | **[V]** | Crowd Supply product options |
| Frequency | **24–1766 MHz** (R820T2 / RTL2832U class); IBW ~**2.56 MHz**/ch | **[V]** | Crowd Supply / KrakenRF product pages |
| Phase calibration | Shared clock + **internal noise source + RF switches**; automatic phase sync when freq/gain changes; software corrects residual phase | **[V]** | https://www.krakenrf.com/about-krakensdr ; Crowd Supply |
| DF algorithm (stock) | Correlative interferometry + **MUSIC** in open-source DSP | **[V]** | Crowd Supply; https://github.com/krakenrf/krakensdr_doa (linked from product) |
| Host | Raspberry Pi 4/5 supported; open DAQ/DSP | **[V]** | Product pages |
| EU availability | **Yes:** SDRstore.eu lists KrakenSDR category (EU shipping; price band shown ~€347–€1693 for kit variants); also Passion-Radio (FR) advertises KrakenRF line | **[V]** | https://www.sdrstore.eu/software-defined-radio/instruments/krakensdr/ ; https://www.passion-radio.com/suppliers/krakenrf-124 |
| Published DF accuracy (manufacturer) | **No official RMS-degree specification** on main product pages | **[V]** | Product pages (absence of number is itself observed) |
| Community localization claims | Forum anecdotes of **tens of metres** geo-location after multi-point mobile bearings (not single-fix bearing RMS) | **[A/V mixed]** | e.g. RadioReference thread citing “20–50 m” worst-case geo — treat as **anecdotal**, not metrology: http://forums.radioreference.com/threads/krakensdr-direction-finding-p25-is-it-possible.481595/ |

**What KrakenSDR is good for (this program):**  
5 simultaneous coherent channels → natural fit to 5-element circular/UCA array; auto-cal; mature DF stack; EU buy path; cost well under professional DF. **[V/A]**

**What it cannot do well:**  
Narrow IBW (~2.56 MHz) limits simultaneous multi-signal / wideband DVB-T capture; no TX; dynamic range and phase noise are RTL-class, not lab-grade; upper edge 1.766 GHz misses mid-band 5G FR1. **[A]** grounded in published specs **[V]**.

### 1.2 bladeRF-class 2-channel (AD9361 MIMO)

| Attribute | Value | Tag | Source |
|---|---|---|---|
| Product | bladeRF 2.0 micro **xA4** | **[V]** | https://www.nuand.com/product/bladerf-xa4/ |
| Price | **USD 540** (xA4 list) | **[V]** | Nuand product page |
| Channels | **2×2 MIMO** (2 RX + 2 TX), shared RFIC LO → **in-device coherent** dual RX | **[V]** | Nuand; https://www.sdrstore.eu/2x2-mimo-sdr-explained-usrp-b210-pluto-bladerf-limesdr/ |
| Frequency | **47 MHz – 6 GHz** | **[V]** | Nuand |
| Sample rate / BW | up to **61.44 MS/s** (and higher modes documented by vendor); ~56 MHz filtered IBW class | **[V]** | Nuand |

**What 2-ch bladeRF *adds* vs KrakenSDR**  
- Much wider instantaneous bandwidth (DVB-T2, multi-carrier cellular observation).  
- Continuity into **L-band / S-band** (GNSS spoofers, some cellular, radar illuminators).  
- TX capability (calibration beacons, self-test, not needed for pure passive DF).  
- Better path to **TDOA/FDOA** on wideband waveforms (phase/timing quality of AD9361 class). **[A]** from architecture **[V]**.

**What 2-ch *cannot* do alone for this brief**  
- True **N=4–5 MUSIC/CI** needs ≥4–5 phase-coherent elements; 2-ch yields interferometer bearing (ambiguous / low DoF), not full-array spectrum.  
- Multi-unit phase coherence across separate bladeRFs requires external LO/ref distribution and careful calibration (forum-level difficulty acknowledged in Nuand discussions). **[V/A]** https://nuand.com/forums/viewtopic.php?t=5004  

### 1.3 Comparable coherent platforms (price ladder)

| Platform | Ch | Freq | ~Price | Coherence model | Notes | Tag |
|---|---|---|---|---|---|---|
| KerberosSDR (legacy) | 4 | 24–1766 MHz | EoL / secondary market | Shared clock; manual-ish cal vs Kraken | Predecessor | **[V]** Crowd Supply comparison table |
| KrakenSDR | 5 | 24–1766 | ~$750 | Auto noise-source cal | Best COTS 5-ch DF package | **[V]** |
| bladeRF 2.0 xA4 | 2 | 47 M–6 G | $540 | On-chip MIMO LO | BW/freq upgrade, not N=5 | **[V]** |
| USRP B210 | 2 | 70 M–6 G | **~$2,387** board kit | Fully coherent 2×2 MIMO (AD9361 both chains) | Research standard | **[V]** https://www.ettus.com/all-products/ub210-kit/ |
| Epiq Sidekiq X4 | 4 | ~1–6 G | Lab / $10k+ class (vendor claims high-end) | Naturally coherent | Custom SW; cited in Kraken comparison | **[V]** Crowd Supply comparison |
| WiNRADiO WD-7200 class | 2 RX + commutated array | HF–VHF/UHF options | Professional | Quasi-coherent correlative interferometer | Spec: **typ. 2° RMS**, instrumental **<0.5°** (reflection-free) | **[V]** https://winradio.com/home/wd7200.htm |
| R&S portable DF (DDF007/PR200 class) | Multi | wide | $100k-class (vendor marketing contrast) | Correlative interferometry | High-end benchmark | **[V]** Crowd Supply comparison |

### 1.4 Phase-calibration approaches (summary)

| Method | How | Used by | Strength / limit | Tag |
|---|---|---|---|---|
| Shared LO + switched noise injection | Inject common noise, measure inter-channel phase, apply digital correction | KrakenSDR | Auto, works after retune; residual cable/antenna mismatch remain | **[V]** |
| On-chip dual chain (single RFIC) | AD9361 two RX share LO | bladeRF, B210 | Excellent short-baseline coherence; only 2 ch | **[V]** |
| External 10 MHz / 1 PPS + LO distribution | Chain multiple SDRs | Multi-USRP MIMO | Scales N; cable phase drift with temp | **[A]** standard SDR practice |
| Correlative DF with element commutation | One/two RX time-share elements | Some pro HF/VHF DF (WiNRADiO style) | Cheaper RX count; slower on short bursts | **[V]** WD-7200 page |

### 1.5 Array-extension options

1. **Stay on one Kraken (N=5):** circular 5-whip ~0.5–1.5 m diameter — matches brief aperture. **[A]**  
2. **Dual Kraken / multi-station cloud DF:** KrakenRF cloud mapper concept for geographically separated bearings (not single-platform array extension). **[V]** Crowd Supply “Ongoing Work”.  
3. **Hybrid:** Kraken for VHF/UHF N=5 MUSIC + bladeRF/B210 for wideband cellular/DVB observation or dual-antenna FDOA. **[A]**  
4. **Lab scale-up:** Multiple USRPs with OctoClock/ref distribution for N≥4 at GHz (cost multi-k$–10k$). **[A]**  
5. **Professional N=5–9 UCA in radome:** R&S-class correlative arrays (odd element count 5–9 is industry practice). **[V]** https://cdn.rohde-schwarz.com/am/us/campaigns_2/a_d/Intro-to-direction-finding-methodologies.pdf  

**Feasibility pick for prototype:** KrakenSDR + matched 5-element array as primary AoA front-end; optional bladeRF for band expansion / waveform research. **[A]**

---

## 2. Direction-finding accuracy: small arrays, moving platforms

### 2.1 Published method performance (instrumental / ideal)

| Method | Typical accuracy claim | Conditions | Tag | Source |
|---|---|---|---|---|
| Correlative interferometry (5-el example) | **~1° or less** “ideal environment” | Professional CI; odd-count circular array | **[V]** | R&S intro PDF (above URL) |
| WiNRADiO WD-7200 correlative | **Typ. 2° RMS**; instrumental **<0.5°** after cal | Reflection-free; specific antenna | **[V]** | https://winradio.com/home/wd7200.htm |
| MUSIC superresolution | Accuracy improves with **SNR, snapshots, aperture**; degrades with **correlated multipath** (spatial smoothing needed) | Classic array processing | **[V]** | https://www.mathworks.com/help/phased/ug/music-super-resolution-doa-estimation.html ; https://en.wikipedia.org/wiki/MUSIC_(algorithm) |
| High-resolution interferometry analysis | **0.1° RMS** may need **~50 dB SNR** at λ/2 spacing — rarely available | Academic thesis result | **[V]** | https://open.metu.edu.tr/bitstream/handle/11511/112712/index.pdf |

**Aperture physics (for brief’s 1–2 m array)** — wavelength context **[A]** from λ=c/f:

| Band | f | λ | 1–2 m aperture in λ | DF implication |
|---|---|---|---|---|
| FM broadcast | 100 MHz | 3.0 m | **0.33–0.67 λ** | Electrically small; poor phase slope → large bearing σ |
| DAB Band III | ~200 MHz | 1.5 m | **0.67–1.33 λ** | Marginal–usable for 5-el CI |
| Cellular low | 800 MHz | 0.375 m | **2.7–5.3 λ** | Good phase leverage for 1–2 m |
| Cellular mid | 1800 MHz | 0.17 m | **6–12 λ** | Excellent aperture; Kraken top is 1.766 GHz — OK near edge |
| AIS / marine VHF | 162 MHz | 1.85 m | **0.5–1.1 λ** | Similar to DAB; strong coastal signals |

**Rule of thumb (engineering):** RMS bearing error scales roughly as  
σ_θ ≈ k / (SNR^{1/2} · (D/λ) · √N_eff)  
with multipath bias often dominating thermal noise. **[A]**

### 2.2 Multipath over water

- Over-sea VHF/UHF is often **line-of-sight + sea-surface reflection (two-ray / multi-ray)**; constructive/destructive fading and **phase distortion of the array manifold**. **[V]** e.g. https://www.mdpi.com/2072-4292/14/19/4753 ; over-sea channel survey literature.  
- Multipath causes **AoA bias and multi-peak MUSIC spectra**; CRFS notes multipath as primary DF/geolocation error driver. **[V]** https://www.crfs.com/blog/how-can-multipath-affect-df-and-geolocation  
- Low-VHF land DF field report: **tens of degrees to ~170°** gross errors near strong reflectors (wall/rail). **[V]** https://www.lineofdeparture.army.mil/Journals/Gray-Space/Archive/Fall-2025/Challenges-in-Low-VHF-Direction-Finding/  
- **Open water** reduces *terrestrial* clutter relative to urban, but **specular sea multipath + coastal ducting** remain; mast height and sea state matter. **[A]**

**Marine implication for 7–30 kn vessel:** Expect **time-varying multipath** (wave motion of platform and sea surface), so single-shot bearings should be **quality-weighted** (eigenvalue spread, MUSIC peak sharpness, residual) before EKF update. **[A]**

### 2.3 Mast-mounted marine DF / AIS experience

- Marine **VHF DF** remains standard for SAR/VTS even in the AIS era (bearing to voice distress). **[V]** https://www.porttechnology.org/technical-papers/vhf_direction_finding_in_the_age_of_ais/  
- AIS VHF range is **~20 NM class** ship–ship/shore depending antenna height; shore networks densify coverage. **[V]** https://www.navcen.uscg.gov/how-ais-works  
- AIS **antenna multipath** on vessels is a well-known installation problem (nearby metal/masts). **[V]** https://www.milltechmarine.com/VHF-Antenna-The-most-important-factor-for-getting-AIS-to-work-well_b_32.html  
- AIS shore stations used as **known** VHF beacons for ranging/positioning research exist (coastal systems). **[V]** e.g. PMC articles on AIS shore-based positioning.  

**For unknown-emitter SLAM:** AIS *ship* emitters are mobile (bad as map landmarks); AIS **base stations** and coastal VHF/DCS are fixed but may be partially known. Broadcast FM/DAB towers are better **static unknown/semi-known** landmarks if associated over time. **[A]**

### 2.4 Working accuracy budget for this program **[A]**

| Regime | Expected bearing σ (1σ) | Drivers |
|---|---|---|
| High SNR, UHF, calm, 1.5 m UCA, good cal | **2–5°** | Near pro CI floor |
| FM, electrically small array, multipath | **8–20°+** | Aperture + two-ray |
| Planing sea state, mast flex, intermittent SNR | **10–30°** or outliers | Motion + multipath |
| Gross multipath / array shadowing by superstructure | **unusable / multimodal** | Geometry of install |

These are **planning envelopes**, not measured on the target vessel.

---

## 3. Bearings-only SLAM / unknown-emitter navigation literature

### 3.1 Observability (classic BO-TMA → map landmarks)

**Foundational result (Nardone & Aidala):** For bearings-only target motion analysis from a single moving observer, **unique tracking solutions require ownship acceleration** (maneuver); certain maneuver classes leave the system unobservable. **[V]**  
- DTIC PDF: https://apps.dtic.mil/sti/tr/pdf/ADA102073.pdf  
- Survey citations throughout BOT literature (IEEE TAC lineage).

**Translation to error-state EKF with *static unknown emitters* (map points):**  
- Static landmarks improve observability vs moving targets, but **range still poorly observed on a constant-velocity radial approach**.  
- Need **cross-range baseline**: course changes, orbits, or long along-track runs with non-collinear geometry to multiple beacons. **[A]** from BO-TMA theory **[V]**.  
- Multi-beacon geometry (several land emitters at different bearings) partially substitutes for ownship maneuver. **[A]**

### 3.2 Bearings-only SLAM / navigation results

| Theme | Result | Tag | Source |
|---|---|---|---|
| Bearings-only SLAM + EKF | Stable exploration policies needed; naive motion yields poor maps | **[V]** | Sim, “Stable Exploration for Bearings-only SLAM”, ICRA (ResearchGate index: https://www.researchgate.net/publication/224625880_Stable_Exploration_for_Bearings-only_SLAM) |
| GES Kalman SLAM with bearings | Existing large body of bearings-only SLAM (EKF, filter banks, RBPF) | **[V]** | Johansen et al. FUSION notes: https://torarnj.folk.ntnu.no/FUSION.pdf |
| Reverse BO-TMA for vehicle nav | Use bearings to *known* or estimated emitters to improve AUV nav | **[V]** | Alexandri & Diamant, IEEE TMC / WPNC lineage: https://www.computer.org/csdl/journal/tm/2019/03/08367860/17D45VTRozr |
| Crude AoA still useful for 3-D passive localization | Multi-platform / multi-fix fusion | **[V]** | Academic literature (e.g. Ibal et al. index pages) |

**Convergence rates:** No universal constant; classical result is **range uncertainty shrinks with baseline / range ratio and bearing information content** (Fisher information ∝ 1/σ_θ² and geometry). Expect **minutes of diverse geometry** at 7–15 kn for kilometre-scale coastal ranges, faster at 25–30 kn *if* bearings remain valid. **[A]**

### 3.3 Terrestrial broadcast as beacons (FM/DAB/DVB-T/cellular)

Published work is stronger on **signals-of-opportunity positioning** (SoOp / opportunistic) and **passive radar** than on pure **bearings-only vessel SLAM** with fully unknown emitters:

- **FM / DVB-T as illuminators of opportunity** for passive radar and propagation studies are mature research themes (many IEEE/MDPI papers).  
- **Cellular** opportunistic positioning exists in terrestrial robotics/phone literature; offshore SNR drops fast (see §4).  
- **KrakenSDR mobile car DF** demos continuous bearing logging + intersection localization of *unknown* transmitters — closest COTS operational analogue to the vessel use case (land, not sea). **[V]** Crowd Supply Android DF description.

**Gap (honest):** Peer-reviewed **at-sea, multi-unknown-emitter, bearings-only EKF SLAM** using FM/DAB arrays is **thin**; program should treat marine fusion performance as **to-be-measured**, with BO-TMA theory as the observability backbone. **[A]**

### 3.4 TDOA / FDOA alternatives with a **single moving receiver**

| Mode | Single moving RX feasibility | Notes | Tag |
|---|---|---|---|
| **AoA (this brief)** | Yes with array | Needs N≥4–5 coherent channels | **[V/A]** |
| **TDOA** | Needs ≥2 spatially separated sensors (or multipath/known structure) | Single-antenna TDOA not available without bistatic baseline | **[V]** standard geolocation texts |
| **FDOA / Doppler** | Yes: motion induces Doppler vs geometry | CRFS: FDOA uses moving receivers; Doppler shift ∝ v·cosθ | **[V]** https://pages.crfs.com/hubfs/whitepapers/Principles%20of%20Geolocation%20Techniques%20White%20Paper.pdf?hsLang=en |
| **Single-platform multi-antenna FDOA** | Possible with dual coherent channels (bladeRF/B210) | Airborne feasibility studies exist | **[V]** Karlsson thesis-class: https://www.diva-portal.org/smash/get/diva2:968154/FULLTEXT01.pdf |
| **Combined T/FDOA** | Usually multi-sensor; single-sat/platform variants researched | Sensitive to geometry & freq stability | **[V]** e.g. DTIC CubeSat geo study AD1055305 |

**For one vessel:**  
- **Primary:** multi-element **AoA** stream into EKF.  
- **Secondary (2-ch add-on):** **FDOA / range-rate** from wideband emitters if frequency stability and ownship velocity are well known.  
- Pure single-antenna TDOA is **not** a substitute. **[A]**

---

## 4. Emitter landscape: Danish / Baltic coastal waters

### 4.1 FM

- Radiomap.eu (Nordic–Baltic overview): Denmark has **five commercial radio networks**, **>300 local community stations**, and **over 590 FM radio transmitters**. **[V]** https://radiomap.eu/map/nordic-baltic  
- European FM high-power sites commonly run **kW-class ERP** (EBU cost/benefit tables use 1–10+ kW ERP categories for “medium/large” FM). **[V]** https://tech.ebu.ch/docs/techreview/EBU_Tech_Review_2017_Cost-benefit_analysis_of_FM_DAB_and_Broadband.pdf  
- **Received power at 10–50 km offshore:** depends on ERP, antenna height, ducting. Order-of-magnitude free-space path loss at 100 MHz:  
  - FSPL(dB) ≈ 32.4 + 20log₁₀(f_MHz) + 20log₁₀(d_km) → ~ **92 dB @10 km**, **106 dB @50 km**.  
  - 10 kW (40 dBW) ERP → rough isotropic Pr ≈ **−52 dBm @10 km**, **−66 dBm @50 km** before antenna gains / two-ray / earth curvature.  
  Real coastal links often better or worse than FSPL (height gain, sea multipath, horizon). **[A]** calculation; FSPL formula standard.

### 4.2 DAB / DAB+

- Denmark: regular DAB since 2002; **nationwide DAB→DAB+ switch 1 Oct 2017**; **three multiplexes** (public/commercial/regional). Official claim of **full national coverage**. **[V]** https://en.wikipedia.org/wiki/Countries_using_DAB/DMB  
- Historical Danish planning materials cited **~88 transmitter sites** (older dual-mux era; site count evolves). **[V]** EBU radio summit materials (e.g. Jensen PDF via EBU).  
- DAB Band III **~174–240 MHz**; OFDM, continuous-ish carriers useful for DF correlation. **[A/V]** band plan known; DF utility **[A]**.  
- TX powers: European DAB examples often **~kW RMS class** per site (much lower than peak FM in some comparisons). **[V]** industry comparison PDFs (GatesAir / ITU-D presentation classes).

### 4.3 DVB-T2

- Denmark uses DVB-T2 for digital terrestrial TV; coastal towers are high and high-power but **UHF** (typically 470–694 MHz region in EU).  
- Bandwidth **~8 MHz** channels — **exceeds Kraken IBW**; better matched to bladeRF/B210 for waveform features, while Kraken can still DF a **slice** of the channel. **[A]**

### 4.4 Cellular offshore

| Claim | Value | Tag | Source |
|---|---|---|---|
| Phone-like LTE offshore | **~8 km** practical without special gear | **[V]** | https://weconnect.one/blogs/how-far-offshore-does-4g-work-long-distance-maritime-internet-explained/ |
| Long-range maritime LTE setups | up to **~70 km** with dedicated antennas/amplifiers | **[V]** | same |
| General tower design reach | often **30–50 km** max terrestrial planning; offshore degrades | **[V]** | industry summaries citing LTE maritime research / Wikipedia cellular |

**Implication:** Cellular is excellent **nearshore** SoOp (strong, many sites, good D/λ on 1–2 m array) but **unreliable as sole beacon set beyond ~20–40 km** without high-gain maritime antennas. **[A]**

### 4.5 AIS base stations / VHF maritime

- HELCOM Baltic AIS network: regional shore monitoring mandate and information exchange. **[V]** Port Technology AIS/VTS paper citing HELCOM 2005 national systems.  
- Denmark: mandatory reporting + **VTS Great Belt**, **Sound Traffic** (with Sweden), etc. **[V]** https://www.dma.dk/safety-at-sea/safety-of-navigation/mandatory-ship-reporting-systems-msrs-and-vessel-traffic-services-vts ; Baltic routing guide.  
- AIS physical layer: **VHF 161.975 / 162.025 MHz**, horizon-limited ~**20 NM** typical. **[V]** USCG NavCen.  
- Shore base stations are **fixed, dense near fairways** — useful as **known or semi-known** VHF landmarks if MMSI/base IDs decoded; pure *unknown* DF still works without decode. **[A]**

### 4.6 Best bands for 4–5 element, ~1–2 m aperture

| Rank | Band | Why | Caveat |
|---|---|---|---|
| 1 | **Cellular 700–900 MHz** (and 1800 if RX allows) | D/λ large; many sites near coast | Range offshore limited; Kraken to 1.766 GHz |
| 2 | **DAB+ Band III (~200 MHz)** | National coverage DK; continuous OFDM; aperture ~1 λ | SFN multipath (multiple TX same freq) can bias AoA |
| 3 | **DVB-T2 UHF** | High power towers | Wideband; may need higher-end SDR; also SFN issues |
| 4 | **FM 88–108 MHz** | Highest power density / many TX | **Electrically small array**; multipath; still useful as *detection* + coarse bearing |
| 5 | **AIS / marine VHF** | Always-on coastal infrastructure | Ship AIS mobile; base stations better as landmarks |

**SFN warning (DAB/DVB):** Single-frequency networks present **multiple transmitters on one frequency** → MUSIC may lock to **composite/virtual direction** or multi-peaks. Prefer **non-SFN FM carriers** or **decodeable cell IDs / AIS base IDs** for data association when possible. **[A]**

---

## 5. High-speed benefit / cost (7–30 kn)

### 5.1 Observability benefit of speed

- Bearing rate |dθ/dt| ≈ (v_⊥ / r) for static emitter; higher speed → **faster information accumulation** and earlier range observability after a maneuver. **[A]** from geometry / BO-TMA.  
- At **30 kn ≈ 15.4 m/s**, a 10-minute leg yields **~9.3 km** baseline; at **7 kn ≈ 3.6 m/s**, only **~2.2 km**. Same time → **~4× baseline** advantage at high speed. **[A]** arithmetic.  
- Planing transition often **~12–18 kn** for small craft — regime change in motion dynamics. **[V]** planing hull discussions, e.g. https://soundingsonline.com/boats/semiplaning-boats-can-meet-many-needs/

### 5.2 DF / integration cost of speed

| Effect | Mechanism | Impact | Tag |
|---|---|---|---|
| Shorter coherent dwell | Platform rotates/translates during snapshot | Blurs MUSIC peak if integration too long | **[A]** |
| Attitude/heading noise | Yaw rate couples array frame to NED bearings | Needs high-rate AHRS / GNSS heading before EKF | **[A]** |
| Mast vibration / slam | Planing impact loads; local antenna motion | Phase center jitter → bearing wander | **[A]**; ship vibration guidance exists (ABS notes) https://ww2.eagle.org/content/dam/eagle/rules-and-guides/current/conventional_ocean_service/147-guidance-notes-on-ship-vibration/147-ship-vibration-gn-sep-23.pdf |
| Multipath dynamics | Sea surface + hull multipath change faster | Heavier outlier rejection | **[A]** |
| Doppler | Useful for FDOA; confuses narrowband phase tracking if unmodeled | Model or short snapshots | **[A]** |

### 5.3 Net recommendation **[A]**

- **7–12 kn (displacement):** better DF stability, slower observability — good for calibration/survey.  
- **15–25 kn:** best **compromise** for coastal SLAM if array is rigidly mounted and attitude is measured.  
- **25–30 kn planing:** use **short snapshots + robust weighting**; expect higher bearing σ; leverage geometry rate only if mechanical phase stability is validated on-water.

---

## 6. Integration notes for existing error-state EKF (feasibility inputs)

1. **Measurement model:** z = atan2(p_e − p̂_e, p_n − p̂_n) + b_array + v, with R adaptive from DF quality metrics. **[A]**  
2. **States:** per-emitter East/North (and optional bias); do **not** assume known tower coords initially (SLAM); optional soft priors if tower database available. **[A]**  
3. **Association:** frequency + MUSIC peak track ID; SFN bands need extra care. **[A]**  
4. **Observability scheduling:** inject **course changes** when only 1–2 beacons visible; multi-beacon coasts of DK/SE/DE help. **[A]**  
5. **Calibration:** online antenna manifold + phase bias estimation (Kraken auto-cal is necessary but not sufficient for mast multipath). **[A]**  
6. **Dual-front-end option:** Kraken continuous AoA on DAB/FM slice; bladeRF opportunistic FDOA on wideband cellular/DVB. **[A]**

---

## 7. Source index (access 2026-07-23)

Primary URLs used above include:  
Crowd Supply KrakenSDR; KrakenRF product/about; Nuand bladeRF; Ettus B210; SDRstore.eu Kraken category; WiNRADiO WD-7200; R&S DF methodologies PDF; MathWorks MUSIC; CRFS multipath & geolocation whitepapers; Nardone/Aidala DTIC ADA102073; radiomap.eu Nordic–Baltic; Wikipedia DAB countries; USCG AIS; DMA Denmark VTS; WeConnect maritime LTE; Port Technology VHF DF; Line of Departure low-VHF DF field report; ABS ship vibration guidance.

---

## 8. VERIFIED vs ASSUMED roll-up

| Class | Examples |
|---|---|
| **[V] VERIFIED** | Kraken price/freq/cal method; bladeRF/B210 specs/prices; R&S ~1° CI claim; WiNRADiO 2° RMS; BO-TMA observability theorems; DK DAB+ national mux structure; >590 DK FM transmitters claim on radiomap; AIS ~20 NM; LTE offshore range articles |
| **[A] ASSUMED** | Field bearing σ budgets on *this* vessel; FSPL Pr numbers as planning aids; ranking of bands for 1–2 m aperture; EKF measurement details; planing vibration impact magnitude; convergence time scales; SFN bias severity without measurement |
