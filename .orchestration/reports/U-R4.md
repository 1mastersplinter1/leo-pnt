# U-R4 Report

(Delivered via stdout contract per D9/D16.)

## Unit report — U-R4

**Contract version:** v5.1  
**Worker:** live web research only (no repo file writes per brief)

### Summary

Consolidated published signal structures for the four constellations the tracker must eventually correlate:

1. **Starlink:** Humphreys et al. fully publish **PSS (exact LFSR + hex) and SSS (hex)**, frame **1/750 s**, **240 MHz** channel OFDM numerology, eight Ku channels. **2.5–5 MHz** is sufficient for band-limited PSS/SSS Doppler (published LNBF practice); full 240 MHz improves delay resolution and gain. **Tone comb degraded post-2023**—do not depend on it. PSS is **not SV-unique**.

2. **Iridium:** Simplex **ring-alert / paging** bursts are the always-available SoOP, not continuous CW. **25 ksps DE-QPSK**, TDMA lattice, **12-symbol BPSK sync** (gr-iridium). Doppler via tone prefix and/or UW correlation / M-power. Exact burst length slightly inconsistent across sources.

3. **Orbcomm:** Continuous **4800 bps SDPSK** on **137–138 MHz** FDMA channels. Published SoOP trackers use **modulation wipe + FFT**, not a known global PRN. Best continuous Doppler source; sequence-based correlator is **not fully published**.

4. **OneWeb:** Beyond 10 ms demand-dependent repetition: Komodromos/Humphreys publish **single-carrier QPSK 230.4 Mbaud**, **1 ms / 400-symbol SS** common to all SVs, frame of 10×1 ms slots, channel centers, β=0.1. Default 10 ms payload is **load-dependent**. Narrowband 2.5–5 MHz needs **blind/capture beacon** strategy; full SS match wants wideband.

### VERIFIED (read primary or strong secondary full text this session)

- Humphreys Starlink structure PDF (parameters, PSS/SSS generators, channel layout).  
- Qin et al. arXiv HTML (frame rate, PSS+SSS coherent use, timing caveats).  
- Komodromos OneWeb PDF (symbol rate, SS hex, 10 ms / 1 ms model, demand-dependent default PL).  
- Kozhaya OneWeb first-look PDF (T0=10 ms ACF, 2.5 MHz capture setup, beam-specific beacons).  
- gr-iridium README (band, 12-sym BPSK sync, burst tooling).  
- Orabi/Kassas Iridium+Orbcomm PDF (burst tone model, Orbcomm continuous 4800 SD-QPSK/SDPSK, Doppler pipeline).  
- Kassas-line reports of **post-2023 Starlink tone power reduction**.

### ASSUMED / partial

- Decode Systems Iridium HTML exact numbers (fetch failed once; used search snippets).  
- Exact current Orbcomm channel plan vs OG1/OG2 differences at Danish latitudes.  
- Whether Starlink CSS full sequence has been published in a post-2023 paper not fully retrieved this session.  
- Legal acceptability of passive SoOP under Danish maritime law without counsel review.  
- That gr-iridium UW bit pattern remains valid on Iridium NEXT fleet without local re-verify.

### Dead ends

- MDPI full-text HTML for Tan Iridium SoOP paper: **Access Denied**.  
- decodesystems.com: intermittent fetch failure.  
- muccc wiki (Anubis bot wall).  
- No operator-published ICD found for Starlink / OneWeb / Iridium IRA / Orbcomm packet-level PRN.

### Open uncertainties for tracker implementation

1. Freeze Starlink PSS/SSS constants from Humphreys; plan **live regression capture** if SpaceX changes sync.  
2. Iridium: extract UW bits from gr-iridium source + **capture confirmation** on simplex.  
3. Orbcomm: implement **continuous residual Doppler**, not sequence correlator, unless RE finds stable headers.  
4. OneWeb: keep **survey gate**; implement SS only if RF path can support meaningful matched-filter bandwidth—or use Kassas-style blind beacon at 2.5–5 MHz.  
5. Licensing review before shipping any third-party decoder code in-tree.

### Evidence of work

- Live web search + PDF fetch/read of primary Starlink, OneWeb, Iridium/Orbcomm SoOP papers and gr-iridium README on **2026-07-23**.  
- No repository files written (per brief).
