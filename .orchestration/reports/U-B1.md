# U-B1 report — BOM with live-verified EU pricing

Contract version built against: **v1** (2026-07-22, as stated in the U-B1 brief).
Deliverable: `docs/design/BOM.md`.

## Totals summary

| Subsystem | Excl. VAT (EUR) | Est. incl. VAT (EUR) |
|---|---:|---:|
| A — bladeRF 2.0 micro xA4 (adopted over xA9) | 801.06 | 1,001.33 |
| B — Ku LNB + dish/feed | 171.50 | 208.82 |
| C — Iridium L-band antenna + filter/LNA | 61.52 | 76.90 |
| D — Independent Orbcomm receiver (SDR + antenna) | 50.12 | 62.65 |
| E — Frequency reference + distribution | 141.00 | 170.61 |
| F — IMU | 1,199.00 | 1,498.75 |
| G — Dual magnetometers (qty 2) | 70.00 | 87.50 |
| H — Speed log | 386.51 | 483.14 |
| I — Host computer | 797.28 | 996.60 |
| J — ArduPilot autopilot hardware | 391.20 | 489.00 |
| K — Helm kill-cord | 29.52 | 36.90 |
| L — Power/enclosure/cabling allowance | 800.00 | 1,000.00 |
| **Grand total** | **€4,898.71** | **€6,112.20** |

Iridium STL is not in this total — R2's quote-path finding was carried forward as a
procurement action (email pnt@iridium.com + VIAVI SecureTime LEO quote request), not a
priced line, per the brief's instruction.

xA4 was adopted over xA9 per R1's own analysis: RF front end/sample rate/USB/MIMO are
identical; xA9's extra FPGA fabric only matters for on-board correlators/channelisers that
the current design does all in host software (2.5–5 MHz per observable). xA9 priced as a
documented, not-in-total alternative.

## Items corrected vs R1/R2

- **bladeRF xA9, SDRstore.eu**: R1 rounded to €1,214; live page shows **€1,214.89**, marked
  down 21% from a €1,535.59 list price R1 didn't report. Immaterial to the recommendation
  (xA4 is adopted regardless) but the discount structure is new information.
- **Bullseye TCXO LNB, SDRstore.eu**: WebFetch's HTML→markdown conversion initially
  garbled the price to "€6296" (a superscript-cents rendering bug). Raw HTML confirms the
  true displayed price is **€62.96**, which does match R1's "~€62" once decoded — flagged
  as a correction because the first-pass tool output was wrong and needed a second,
  lower-level fetch (`curl` + grep) to resolve.
- **Airmar/Garmin DST810-PV-N2 speed log, SVB24**: this item wasn't sourced from R1/R2 (new
  line), but its own search-result snippet suggested ≈€394.92; the live page (confirmed via
  the page's own `<title>` tag) shows **€483.14**. Noted in the BOM as a correction against
  the snippet that surfaced it, since the two numbers could otherwise look like the same
  claim.

Everything else costed from R1 (Nuand xA4 $540/xA9 $860 direct-from-Nuand, SDRstore.eu xA4
€1,001.33, hamparts EXT OSC MK3 €168.19/€139.00, hamparts OCXO board €50.82/€42.00) was
**[CONFIRMED]** — live prices matched R1's reported figures exactly or to a cent-level
rounding difference not worth flagging as a correction.

## Items I could not verify

- **FE-5680A rubidium reference**: no live price obtained. Five distinct eBay listing
  fetches were attempted (WebFetch and `curl`); all either timed out (60s) or returned no
  usable content. `[UNVERIFIED — page unreachable]` in the BOM; the OCXO board was adopted
  as the priced/in-stock reference instead, with Rb left as a documented but unpriced
  alternative (R1's own "tens to low hundreds USD, no stable MSRP" characterization is
  quoted rather than a fabricated number).
- **OnLogic CL260 EU-store price**: the US store (`onlogic.com/store/cl260/`) gave a live,
  confirmed price (USD 906.00, via raw JSON in the page). Two attempts at the EU store
  (`onlogic.com/eu/store/cl260/` and `onlogic.com/eu-en/cl260/`) returned empty content or
  404. The BOM uses the US price FX-converted plus an assumed 25% DK import VAT, flagged
  explicitly as not a native EU quote.
- **CubePilot Cube Orange+, robotshop.com**: HTTP 403 to automated fetch. A different
  reseller (aeroboticshop.com, IT/EU) was successfully fetched and gave a real EUR price
  (€489.00) but that listing is currently sold out. Stock across other resellers (NW Blue,
  RaceDayQuads, GetFPV) was not independently re-checked this session.
- **wimo.com KE-137 QFH antenna**: HTTP 403 to automated fetch; abandoned in favor of the
  Passion Radio DP-KE137 listing (also fetched successfully, also out of stock) and the
  in-stock, cheaper V-dipole alternative that was adopted instead.
- **Passion Radio 80cm Ku dish**: price was live-fetched successfully (€40.63 incl. /
  €32.50 excl.) but the page itself states the SKU is discontinued ("This material is no
  longer available"). Priced as an order-of-magnitude reference only; flagged for
  re-sourcing before any purchase.

## Assumptions

- **VAT**: 25% Danish import VAT assumed when converting non-EU-VAT-inclusive source
  prices (USD Nuand/Nooelec/NavTechGPS/OnLogic, GBP Fox's Chandlery) to an "est. incl. VAT"
  figure. Where a vendor already displays a VAT-inclusive EU consumer price (SDRstore,
  hamparts, Passion Radio, SVB24, aeroboticshop), that real price is used as-is; the
  embedded national VAT rate in those cases actually varies 19–22% by member state, treated
  as immaterial at this budgeting granularity. No landed-cost checkout (VAT + customs duty
  + courier) was completed for any line — this mirrors R1's own caution about Nuand's
  US-origin pricing, extended to every non-EU line in this BOM.
- **FX rates** (2026-07-22, mid-market, rounded): 1 USD ≈ €0.88, 1 GBP ≈ €1.17.
- **IMU and dual-magnetometer candidates are proposals, not endorsed selections**
  (`[UNVERIFIED — selection]`): the baseline specifies no part or grade for either. VN-100
  Rugged pricing is a reseller devkit-range midpoint, not VectorNav's own list price (their
  OEM/Rugged unit page is "Call for Pricing"); a materially cheaper alternative (Xsens
  MTi-3 class, ~USD 550 list, though shown out of stock at DigiKey at fetch time) exists if
  VN-100-class performance isn't required — left as an open design tradeoff, not resolved
  here.
- **Orbcomm receiver/antenna** deliberately chosen for lowest verified cost per the brief's
  "cheap SDR + antenna" instruction and D10 (its clock is free-running and must not enter
  fusion without a second receiver-clock state) — not chosen for performance.
- **Power/enclosure/cabling** is a flat €800/€1,000 engineering allowance, not sourced from
  any vendor, per the brief's explicit instruction that this line is an allowance rather
  than a priced quote.
- Grand total excludes Iridium STL (no public price exists; treated as a procurement
  action per the brief) and excludes customs duty on non-EU lines (VAT-only estimate).

## Files owned / touched

- `docs/design/BOM.md` (created)
- `.orchestration/reports/U-B1.md` (this file)

No other files were modified. No git commit was made.
