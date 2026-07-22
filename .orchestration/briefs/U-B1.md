# Brief U-B1 — Bill of materials with live-verified EU pricing

Contract version: v1. Read first: `.orchestration/CONTRACTS.md`, `docs/design/DESIGN_BASELINE.md`
(approved baseline — sensor set + Orbcomm independent-receiver requirement),
`docs/research/R1-bladerf-market.md`, `docs/research/R2-services-regs.md`,
`.orchestration/DECISIONS.md` (D5: Grok research claims are unconfirmed until a non-Grok
reviewer verifies them — that reviewer is YOU for every price/availability you carry into the BOM).

## Goal
`docs/design/BOM.md` — handoff deliverable 5. One line per item: item, role (trace to the
baseline's sensor/equipment table), candidate part, qty, unit price EUR, availability,
source URL + access date (2026-07-22), verification status. Rules:
- **Live-verify** (open the vendor page yourself) every price and availability claim you take
  from R1/R2; mark each `[CONFIRMED <date>]` or `[CORRECTED: <what changed>]` or
  `[UNVERIFIED — page unreachable]`. Do not silently trust the research docs.
- Cover: bladeRF 2.0 micro (state the xA4-vs-xA9 recommendation and its rationale from R1),
  ext-ref PLL LNB + dish/feed, Iridium L-band antenna + filter/LNA, independent Orbcomm
  receiver (137 MHz — cheap SDR + antenna; note its free-running clock per D10),
  rubidium/OCXO reference + distribution, IMU, dual magnetometers, speed log, host computer,
  ArduPilot autopilot hardware, helm kill-cord, power/enclosure/cabling allowance.
- Where the baseline marks an item [UNVERIFIED] (e.g. exact Orbcomm receiver), propose a
  concrete candidate and price it, keeping the [UNVERIFIED] selection status.
- Totals: subtotal per subsystem + grand total, pre- and est. incl.-VAT (state VAT assumption).
- No purchases, no quote requests — desk research only. For Iridium STL note R2's
  quote-path finding as the procurement action, not a priced line item.

## Files owned
Only: `docs/design/BOM.md`, `.orchestration/reports/U-B1.md`. No code, no git commit.

## Report
`.orchestration/reports/U-B1.md`: totals summary, items corrected vs R1/R2 (list each),
items you could not verify, assumptions, contract version.
