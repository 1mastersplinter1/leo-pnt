# Report U-R1 — bladeRF / LNB / host / tooling market research

**Contract version:** v1 (read 2026-07-22 against `.orchestration/CONTRACTS.md`)  
**Access date:** 2026-07-22  
**Outputs written:**
- `docs/research/R1-bladerf-market.md` (full research)
- `.orchestration/reports/U-R1.md` (this report)

**Not done:** git commit (per brief). No code changes.

---

## Summary of findings

1. **bladeRF 2.0 micro pricing (live)**  
   - Nuand direct: **xA4 USD 540**, **xA9 USD 860**, both **in stock**, ship 1–2 business days.  
   - EU retail SDRstore.eu: **xA4 ~EUR 1 001**, **xA9 ~EUR 1 214**, both **in stock**, free ship over €415.  
   - Lab401 sells EU-dispatched **packs** (not bare-board list) with accessories.  
   - Key specs **confirmed**: 2×2 MIMO, USB 3.0, 61.44 MSPS (up to 122.88 MSPS advanced), 12-bit, tuning 47 MHz–6 GHz marketing / RX table 70–6000 MHz, **10 MHz external reference** via on-board PLL, AD9361 RFIC.

2. **xA4 vs xA9**  
   - FPGA **49 kLE vs 301 kLE**; ~32 vs ~292 kLE user; memory/DSP/multipliers scale ~4–5×.  
   - RF/USB/MIMO identical. xA9 for custom HDL only; no stock DSP accelerators.  
   - For host-side LEO correlation at a few MHz BW: **xA4 is enough** unless FPGA channelisers are planned.

3. **USB 3.0 sustained**  
   - Full dual 61.44 MSPS SC16 is host-sensitive (~0.5 GB/s class payload).  
   - Mission rates (2.5–5 MSPS-class per observable) are well inside USB 3.0.  
   - Caveats: controller chipset, buffers (`num_buffers` / `num_transfers`), avoid hubs. Wiki + forum evidence of overruns and buffer fixes.

4. **Software**  
   - **libbladeRF + official Python** mature.  
   - **SoapyBladeRF** mature interoperability path.  
   - Rust: `bladerf`/`bladerf-sys` **0.1.2 (2024-10)** thin; Nuand bindgen examples sparse; **`libbladerf-rs` 0.4.1 (2026-06) pure Rust but BladeRF1 only — not micro 2.0**.

5. **Ku LNB**  
   - True **external-ref** path: **hamparts/qro.cz LNB EXT OSC MK3** ~**€168 incl.** (10 MHz in → 25 MHz to LNB PLL, LO 9750).  
   - **Bullseye** ~**€62** excellent TCXO / 25 MHz **out**, not stock ext-ref-in.  
   - Dual LO 9750/10600 covers 10.7–12.75 GHz with IF to ~2150 MHz.

6. **Reference**  
   - FE-5680A datasheet: default **10 MHz**, factory options include **5 / 15 / 13 / 2.048 / 10.23 / 50.255 MHz** — must verify 10 MHz sine before buy. Surplus pricing volatile (historically cheap; currently used market).  
   - EU OCXO alternative: hamparts **~€51 incl.** 10 MHz board.  
   - Convert 10→25 MHz for LNB; do not use GPSDO operationally (handoff constraint).

---

## VERIFIED vs ASSUMED (aggregate)

| Class | What |
|-------|------|
| **VERIFIED** | Nuand USD prices & stock; SDRstore EUR price nodes & stock; Lab401 EU pack offering; official FPGA/RF/USB/10 MHz-ref specs; FPGA resource table; ADS-B FPGA precedent; SoapyBladeRF existence; Python bindings via libbladeRF; crate versions on docs.rs/lib.rs; Bullseye & MK3 LNB specs/prices; FE-5680A datasheet frequencies/options; hamparts OCXO price; USB buffer forum report; 122.88 MSPS 8-bit release notes |
| **ASSUMED** | Landed-cost / VAT details without checkout; coherent-MIMO wording (inferred from AD9361 2×2); dual full-rate SC16 sustainability envelope; xA4 sufficient for host DSP roadmap; pure-Rust BladeRF2 timeline; DRO ±1 MHz handoff claim not re-measured; FE-5680A current eBay street price as a single number; exact SDRstore VAT inclusion without invoice |

---

## Dead ends

| Attempt | Result |
|---------|--------|
| crates.io API | Rate-limited / data-access policy rejection; used docs.rs + lib.rs instead |
| Full GitHub wiki body for dropped samples | UI “Uh oh” / raw 404; used title, metadata, search snippets, related forum |
| Digi-Key/Mouser live bare micro | SparkFun SKUs obsolete/discontinued — not a buy path |
| Single stable FE-5680A market price | Surplus only; listings volatile; datasheet + historical notes used |
| Exact Nuand international cart VAT/IOSS | Not completed through checkout |

---

## Open uncertainties

1. Exact **SDRstore VAT-inclusive** vs ex-VAT invoice line (checkout needed).  
2. Nuand → EU **landed cost** (shipping + duty + VAT) for bare board vs SDRstore retail.  
3. **Sustained dual-RX MSPS** on the specific industrial PC to be used at sea (must bench).  
4. Whether any **third-party bladeRF 2.0 FPGA channeliser** is commercially available (none found).  
5. **`libbladerf-rs` BladeRF2** ETA / completeness.  
6. Live surplus **FE-5680A 10 MHz** unit price and health at purchase time.  
7. Bullseye **modification** feasibility for external 25 MHz lock vs buying MK3.  
8. bladeRF **external ref amplitude** window for FE-5680A 0.5 Vrms output without buffer amp.

---

## Evidence of method

- Live web search + page loads of Nuand, SDRstore, Lab401, hamparts, PySDR, docs.rs/lib.rs, FE-5680A PDF, SoapyBladeRF/GitHub, remoteqth, Baltic Lab, Nuand ADS-B article.  
- HTML price extraction via `curl` for SDRstore xA4 (€1,001), xA9 (€1,214), Bullseye (€62).  
- PDF read of FE-5680A datasheet for frequency options.  
- No hardware purchased; no git commit.

---

## Files owned (this unit only)

- `docs/research/R1-bladerf-market.md`  
- `.orchestration/reports/U-R1.md`
