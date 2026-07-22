# U-R2 Report

(Delivered via stdout contract U-R2b after file-write failures — see DECISIONS D6/D9.)

## Unit report — U-R2

**Contract version:** v1 (read `.orchestration/CONTRACTS.md`; reports are text-delivered per brief override — no files written).

### Summary
Live web research completed on Iridium STL ownership/branding (Iridium post-Satelles; **not Xona**), purchase path (partner + subscription; **pnt@iridium.com** / VIAVI quote; no public prices), performance (~20 m position = vendor-class claims; sub-100 ns timing = NIST-backed for good oscillators), terrestrial SoOP around the Danish-waters assumption (DVB-T2 SFN coastal candidate with unconfirmed GNSS discipline; R-Mode still testbed expanding early-2026; cellular good nearshore / Tampnet North Sea; eLoran not regional-operational), Danish MASS/test permits (DMA Future Lab + DEMA ≥4 weeks; COLREGs still apply to manned trials), and CelesTrak SupGP (Starlink ~10.8k rows, OneWeb ~651 rows live 2026-07-22; free HTTP with one-download-per-update discipline).

### VERIFIED vs ASSUMED (high level)

| VERIFIED (read primary pages) | ASSUMED (inference) |
|---|---|
| Iridium owns STL; brand Iridium STL / Iridium PNT since 2024 | Danish DVB-T2 sites are GNSS-disciplined (industry default only) |
| Xona Pulsar is a different LEO-PNT product | Exact R-Mode geometric usability of Danish straits without station coordinates |
| VIAVI STL modules require SecureTime LEO subscription | Manned autopilot always needs same DEMA path as unmanned drones (case-by-case) |
| NIST timing results sub-100 ns class | Dynamic maritime STL position accuracy |
| DLR R-Mode testbed + early-2026 standardised signals DE–FI–EE corridor | Continuous public 5G as SoOP mid-Baltic |
| DMA/DEMA test-permit process and Future Lab contacts | EMSA does not replace national permit (no counter-example found) |
| CelesTrak SupGP Starlink/OneWeb files live; 2 h check / usage limits | SupGP fit quality sufficient for all SoOP pipelines |

### Dead ends
- No public Iridium or VIAVI **price list** for STL service.  
- No official Danish broadcaster document stating **whether DTT SFN sync is GNSS-only or PTP-backed**.  
- No public **complete R-Mode station coordinate table** with Danish coverage contours on open web (map descriptions only).  
- **EMSA** did not yield a concrete EU sandbox permit product for Danish waters.  
- eLoran: UK funding/rebuild found; **no Baltic operational chain**.

### Open uncertainties
1. Which Iridium **European partners** stock maritime-suitable STL receivers **now** (2026-07).  
2. STL **dynamic** position error budget on a small craft in Øresund multipath.  
3. Whether DEMA/DMA treat **manned** semi-autonomous control differently from “sea drones” for the same risk envelope.  
4. R-Mode **receiver availability** and licensing for non-project participants.  
5. OneWeb SupGP “matching results: 1” meaning for ephemeris trust (investigate RMS tables before production use).  
6. Legal: radio/spectrum implications of passive SoOP receive-only vs any active RF experiments (not researched in depth).

### Method notes
Web search + page fetch + selective `curl` of CelesTrak/Starlink endpoints. No repo files written or edited. No git. No code. Access date stamped **2026-07-22** throughout.

### Contract compliance
- Contract **v1** stated.  
- VERIFIED/ASSUMED split present.  
- Regulatory not-legal-advice + authority contacts present.  
- Operating area assumption flagged.  
- Output format: `===R2-DOC===` / research / `===R2-REPORT===` / this report.
