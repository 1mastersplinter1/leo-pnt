# Brief U-R1 — bladeRF / LNB / host / tooling market research (live web)

Contract version: v1 (read `.orchestration/CONTRACTS.md` first).

## Goal
Live web research, written to `docs/research/R1-bladerf-market.md`. Questions:

1. Nuand bladeRF 2.0 micro xA4 and xA9: current price (USD and EUR), stock/availability from Nuand direct and EU distributors (list concrete distributors, prices, shipping-to-EU implications). Confirm key specs: 2×2 coherent MIMO RX, 10 MHz ext ref input, tuning range, max sample rate, USB 3.0.
2. xA4 vs xA9 FPGA sizing: logic-element counts, what on-board channelisation/DSP realistically fits in each; community evidence of custom FPGA DSP on either.
3. Sustained USB 3.0 throughput of bladeRF 2.0 on Linux hosts: real-world reports of sustained MSPS without drops, host controller caveats, buffer settings.
4. Software path maturity: libbladeRF Rust bindings (which crates, last release, maintenance state) vs SoapySDR route; note Python paths too.
5. Ku LNB for this use: the handoff requires a free-running-but-stable frequency chain. Standard DRO LNBs have ~±1 MHz LO error and drift; find PLL LNBs with EXTERNAL reference input (e.g. 25 MHz ref-in types used by radio astronomy / QO-100 community), price and EU availability, and note LO frequency options for 10.7–12.75 GHz coverage and IF up to ~2150 MHz.
6. Reference oscillator: FE-5680A-class surplus rubidium and OCXO alternatives — current surplus market price, availability, 10 MHz output suitability for both bladeRF ref-in and LNB ref chain (note any frequency conversion needed, e.g. FE-5680A's non-10MHz variants).

## Method & rules
- Web search allowed and expected. Every claim: source URL + access date (2026-07-22).
- Split every section into VERIFIED (you read/loaded the page stating it) vs ASSUMED (inference). No unlabeled claims.
- Prices: quote currency and date; note if pre-VAT.

## Files owned
Only: `docs/research/R1-bladerf-market.md`, `.orchestration/reports/U-R1.md`. Do NOT run git commit. No code.

## Report
`.orchestration/reports/U-R1.md`: summary of findings, VERIFIED/ASSUMED split, dead ends, open uncertainties, contract version.
