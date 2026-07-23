# Bill of Materials — LEO Signals-of-Opportunity Maritime PNT

Status: **handoff deliverable 5**
Contract version: v1 (2026-07-22)
Owner: U-B1
Normative parent: `docs/design/DESIGN_BASELINE.md` (approved baseline, contract v1)
Access date for all live web claims in this document: **2026-07-22**

This BOM is desk research only. No purchases and no quote requests were made. Every
price/availability claim carried in from `docs/research/R1-bladerf-market.md` (R1) or
`docs/research/R2-services-regs.md` (R2) was **live-verified** by re-opening the vendor
page in this unit (per `.orchestration/DECISIONS.md` D5: a non-Grok reviewer must confirm
research-doc claims before they become load-bearing). Items not covered by R1/R2 were
sourced fresh, also by opening the vendor page directly, and are tagged the same way.

**Verification-status legend** (per item, applied to price + availability together):
`[CONFIRMED <date>]` — vendor page opened this session, value matches or is a trivial
update; `[CORRECTED: <what changed>]` — vendor page opened this session, value differs
materially from what R1/R2 stated; `[UNVERIFIED — page unreachable]` — a live fetch was
attempted and failed (timeout / 403) and no substitute primary source could be reached in
time; `[UNVERIFIED — selection]` — a concrete candidate is proposed against a baseline row
that carries no specified part (IMU grade, magnetometer model, etc.); this tag does not mean
the price itself is unverified, only that the baseline has not endorsed this specific part.

**VAT assumption.** Where a vendor already displays a VAT-inclusive EU consumer price in
EUR, that price is used as-is in the "incl. VAT" column (the embedded national VAT rate
varies 19–22% by member state; treated as immaterial at BOM budgeting granularity). Where
a vendor quotes ex-VAT EUR, USD or GBP, this document adds an explicit **25% Danish
import-VAT assumption** (Denmark being the baseline's working operating-area assumption)
to produce the "est. incl. VAT" figure. No line item's landed cost (VAT + customs duty +
courier) was confirmed at an actual checkout; this mirrors R1's own caution on Nuand's
US-origin pricing. FX rates used (2026-07-22, mid-market, rounded): **1 USD ≈ €0.88**,
**1 GBP ≈ €1.17**.

---

## A. Coherent RF sampling — Nuand bladeRF 2.0 micro

Baseline role: "Two coherent RX channels on one external reference. RX allocation is
survey-dependent."

**xA4 vs xA9 recommendation: adopt xA4.** Per R1 §2, RF front end, sample rate, USB
interface and MIMO capability are identical between the two boards — only FPGA fabric
differs (xA4 ≈49 kLE / ~32 kLE free vs xA9 ≈301 kLE / ~292 kLE free, plus ~4× memory and
~5× DSP blocks on xA9). The baseline and rate contract place all channelisation and
Doppler tracking (2.5–5 MHz processing bandwidth per observable) on the Linux host, not
on-FPGA; xA9's extra fabric exists to support on-board correlators/channelisers that
nothing in the current design calls for. Reserve xA9 as a specific future upgrade only if
a roadmap item requires on-FPGA multi-channel correlation or dual-band channelisation to
cut USB load — not a day-one requirement.

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| bladeRF 2.0 micro **xA4** (adopted) | Nuand bladeRF 2.0 micro xA4, via SDRstore.eu (EU stock) | 1 | €1,001.33 incl. VAT (est. €801.06 excl.) | In stock | [SDRstore.eu xA4](https://www.sdrstore.eu/software-defined-radio/instruments/bladerf/bladerf-2-0-micro-xa4-software-defined-radio-en/), 2026-07-22 | [CONFIRMED 2026-07-22] — raw page price node reads "1,001<sup>33</sup>"; matches R1's €1,001 to the cent-level detail R1 didn't capture |

Alternates priced but **not** in the subtotal:

| Alternate | Candidate | Unit price | Availability | Source / access date | Status |
|---|---|---|---|---|---|
| xA4, US-origin | Nuand direct | USD 540.00 (≈€475.20 FX-only, no import VAT/duty) | "In stock - Usually ships out in 1 to 2 business days." | [nuand.com/product/bladerf-xa4](https://www.nuand.com/product/bladerf-xa4/), 2026-07-22 | [CONFIRMED 2026-07-22] |
| xA9, EU stock | SDRstore.eu | €1,214.89 incl. VAT (est. €971.91 excl.) | In stock | [SDRstore.eu xA9](https://www.sdrstore.eu/software-defined-radio/instruments/bladerf/bladerf-2-0-micro-xa9-software-defined-radio-en/), 2026-07-22 | [CORRECTED: €1,214.89 vs R1's rounded €1,214; page also now shows a "-21%" markdown from a €1,535.59 list price that R1 did not report] |
| xA9, US-origin | Nuand direct | USD 860.00 (≈€756.80 FX-only) | "In stock - Usually ships out in 1 to 2 business days." | [nuand.com/product/bladerf-xa9](https://www.nuand.com/product/bladerf-xa9/), 2026-07-22 | [CONFIRMED 2026-07-22] |

**Subtotal A (excl. VAT / est. incl. VAT):** €801.06 / €1,001.33

---

## B. Ku LNB and antenna — Starlink / conditional OneWeb reception

Baseline role: "10.7–12.75 GHz downconverted to approximately 950–2150 MHz IF... do not
depend on the degraded tone comb." Correlation-Doppler tracking requires a phase-stable
LO, hence an **externally referenced** LNB fed from the same 10 MHz master as the bladeRF
(see subsystem E).

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| Ext-ref PLL LNB (adopted) | hamparts.shop "10 GHz LNB EXT OSC MK3" — accepts external 10 MHz in, synthesises 25 MHz to the LNB PLL; 9750/10600 MHz universal LO | 1 | €168.19 incl. VAT (€139.00 excl., vendor-stated split) | "Add to cart" active; explicit in-stock/lead-time string not present on page (unlike the OCXO board's "Last items in stock") | [hamparts.shop MK3](https://hamparts.shop/10-ghz-lnb-ext-osc-mk3.html), 2026-07-22 | [CONFIRMED 2026-07-22] — price matches R1 exactly; availability language is weaker than R1 implied, downgraded from "in stock" to "add-to-cart present, no explicit stock string" |
| Dish + feed | Generic 80 cm offset Ku dish, Megasat-branded example (Passion Radio) | 1 | €40.63 incl. VAT (€32.50 excl.) | **"This material is no longer available"** | [Passion Radio 80cm dish](https://www.passion-radio.com/lnb-pll/0500238-2729.html), 2026-07-22 | [UNVERIFIED — SKU discontinued]. New line, not costed in R1/R2. 80–85 cm offset Ku dishes are a commodity product widely stocked at EU satellite retailers in the same €30–€90 incl.-VAT band; this specific SKU is priced only as an order-of-magnitude reference and must be re-sourced before purchase. |

Not adopted (does not meet the external-reference requirement, priced for completeness):
Bullseye TCXO LNB, SDRstore.eu, **€62.96 incl. VAT**, in stock — [Bullseye TCXO LNB](https://www.sdrstore.eu/software-defined-radio/instruments/rtl-sdr/qo-100-bullseye-tcxo-lnb-ultra-stable-lnb-for-qo-100-and-ku-band-satellites/), 2026-07-22 — `[CORRECTED: WebFetch's markdown conversion initially garbled this to "€6296"; raw HTML confirms the true displayed price is €62.96 (span "62"<sup>"96"</sup>), matching R1's "~€62" once decoded]`. Its internal TCXO is not locked to the ship's Rb/OCXO, so it cannot deliver the phase coherence the correlation tracker needs; kept only as a cheap non-coherent spare.

**Subtotal B (excl. VAT / est. incl. VAT):** €171.50 / €208.82

---

## C. Iridium L-band reception

Baseline role: "Direct 1616–1626.5 MHz reception, bypassing the Ku LNB."

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| L-band antenna (adopted) | Nooelec 1620MHz Iridium PCB patch antenna, 80 MHz+ BW, 3.0 dBi+ | 1 | est. €32.95 incl. VAT (USD 29.95 ≈ €26.36 excl., FX + 25% DK-import-VAT assumption) | "Orders usually shipped in 1 business day!" | [Nooelec Iridium antenna](https://www.nooelec.com/store/sdr/iridium-antenna.html), 2026-07-22 | [CONFIRMED 2026-07-22] — new item, not costed in R1/R2 |
| Filter/LNA (adopted) | Nooelec SAWbird+ IR — cascaded ultra-low-noise LNA + SAW filter, 1620 MHz centre, for Iridium/Inmarsat | 1 | est. €43.95 incl. VAT (USD 39.95 ≈ €35.16 excl. + 25% est.) | "Orders usually shipped in 1 business day!" | [Nooelec SAWbird+ IR](https://www.nooelec.com/store/sawbird-ir.html), 2026-07-22 | [CONFIRMED 2026-07-22] — new item, not costed in R1/R2 |

Alternate antenna (higher gain, multi-band, priced for completeness, not adopted):
SparkFun Iridium/GPS/GLONASS passive antenna, **USD 80.95**, in stock —
[SparkFun](https://www.sparkfun.com/iridium-gps-glonass-passive-antenna.html), 2026-07-22 —
`[CONFIRMED 2026-07-22]`.

**Subtotal C (excl. VAT / est. incl. VAT):** €61.52 / €76.90

---

## D. Independent Orbcomm receiver (137 MHz)

Baseline role: "A separate, non-coherent receiver provides continuous
constellation/front-end diversity without consuming either bladeRF coherent RX channel;
exact receiver is a BOM dependency `[UNVERIFIED]`." Per DECISIONS D10: this receiver
carries its own **free-running, unmodeled clock**; Orbcomm observations must not enter
fusion until a second receiver-clock state or per-receiver nuisance term exists in the
estimator. The candidate below is chosen specifically to be cheap and simple — it is not
disciplined to anything, which is the point.

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| Cheap SDR (adopted) | Nooelec NESDR SMArt v5 (RTL2832U/R820T2, 0.5 ppm TCXO, free-running — no GNSS discipline) | 1 | est. €46.15 incl. VAT (USD 41.95 ≈ €36.92 excl. + 25% est.) | "Orders usually shipped in 1 business day!" | [Nooelec NESDR SMArt v5](https://www.nooelec.com/store/sdr/sdr-receivers/nesdr-smart-sdr.html), 2026-07-22 | [CONFIRMED 2026-07-22]; `[UNVERIFIED — selection]` (baseline leaves the exact receiver open) |
| 137 MHz antenna (adopted) | V-dipole antenna kit for 137 MHz weather/Orbcomm-band reception | 1 | est. €16.50 incl. VAT (USD 15.00, "EU VATs Exclude" per vendor, ≈€13.20 excl. + 25% est.) | "Add to cart" active; explicit stock count not shown | [elekitsorparts.com V-dipole](https://elekitsorparts.com/product/v-dipole-antenna-kit-for-weather-satellites-137mhz-v-dipole-noaa-satellite-antennas-for-sdr/), 2026-07-22 | [CONFIRMED 2026-07-22]; `[UNVERIFIED — selection]` |

Alternate antenna (better hemispherical/low-elevation pattern, materially more expensive
and currently unbuyable — priced for completeness, not adopted): Diamond DP-KE137 QFH,
**€269.79 incl. VAT** (€215.83 excl.), **"Out of stock, available in 2 to 3 months"** —
[Passion Radio DP-KE137](https://www.passion-radio.com/adsb/dp-ke137-1039.html),
2026-07-22 — `[CONFIRMED unavailable 2026-07-22]`. A second EU listing for the same
Diamond antenna (Moonraker) also showed the item **out of stock** at a lower
€66.57-marked-down-from-€83.44 price during this session — treat both as currently
non-procurable; the cheap V-dipole is the only in-stock candidate found.

**Subtotal D (excl. VAT / est. incl. VAT):** €50.12 / €62.65

---

## E. SDR frequency reference + distribution

Baseline role: "10 MHz external reference, calibrated only before deployment; no GNSS
discipline. Mount its resultant acceleration-sensitivity vector vertically under the
displacement-hull assumption." Feeds both the bladeRF (subsystem A) and the Ku LNB
(subsystem B) from one master, per R1 §6.

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| 10 MHz reference (adopted) | hamparts.shop 10 MHz OCXO board (sine + square, claimed 5×10⁻¹² stability, −140 dBc/Hz @100 Hz, explicitly marketed "Reference for LNB EXT Bias tee") | 1 | €50.82 incl. VAT (€42.00 excl., vendor-stated split) | **"Last items in stock"** | [hamparts.shop OCXO board](https://hamparts.shop/10-mhz-ocxo-board.html), 2026-07-22 | [CONFIRMED 2026-07-22] — matches R1 exactly |
| 10 MHz distribution (adopted) | hamparts.shop 10 MHz 8-way active splitter (10 dBm ±2 dBm @ 50 Ω per output) | 1 | €119.79 incl. VAT (€99.00 excl., vendor-stated split) | "in stock, shipped in 2-3 days" | [hamparts.shop 8-way splitter](https://hamparts.shop/10-mhz-8-way-active-splitter.html), 2026-07-22 | [CONFIRMED 2026-07-22] — new item, not costed in R1/R2 |

Not adopted / not price-confirmed this session: **FE-5680A rubidium** (the baseline's own
"free-running rubidium (FE-5680A class)... or good OCXO" framing treats these as
alternatives; better long-term aging than the OCXO but power-hungrier and its live market
price could not be pinned down). R1 characterised surplus FE-5680A asking prices as "tens
to low hundreds USD, no single stable MSRP." This unit attempted to open five specific
eBay listings; all five either timed out or could not be confirmed with a legible price
string in the time available. **`[UNVERIFIED — page unreachable]`** — no FE-5680A price
is carried into the totals below; if Rb is later selected over the OCXO, budget roughly
USD 50–250 (R1's range) plus a buffering/level-matching stage R1 flags as likely necessary,
and re-verify a specific listing before purchase.

**Subtotal E (excl. VAT / est. incl. VAT):** €141.00 / €170.61

---

## F. IMU

Baseline role: "Drives every estimator propagation; it does not carry passage-scale
position unaided." The baseline does not pin a grade or part, so the candidate below is a
proposal, not an endorsed selection.

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| IMU | VectorNav VN-100 Rugged IMU/AHRS (industrial/tactical-adjacent MEMS AHRS, enclosed housing suitable for a marine mount) | 1 | est. €1,498.75 incl. VAT (USD 1,362.50 midpoint of a USD 1,200–1,525 reseller range ≈€1,199.00 excl. + 25% est.) | Reseller listing, no explicit stock count | [NavTechGPS VectorNav page](https://www.navtechgps.com/brands/vectornav_technologies/), 2026-07-22 | [CONFIRMED 2026-07-22] price range is a **reseller** quote, not VectorNav's own list price — VectorNav's own site shows the OEM/Rugged unit as "Call for Pricing" (no live number obtainable) and only its bundled Development Kits carry a displayed range; the DevKit price likely overstates a bare production unit's cost since it includes eval breakout/cabling. `[UNVERIFIED — selection]`: baseline does not specify IMU grade; a lower-cost tactical-adjacent MEMS AHRS (e.g. Movella/Xsens MTi-3 class, list-priced ~USD 550 at DigiKey but shown "In-Stock: 0, no backorders" at fetch time — [DigiKey MTi-3-T](https://www.digikey.com/en/products/detail/xsens-technologies-b-v/MTI-3-T/9607411), 2026-07-22, `[CONFIRMED unavailable]`) is a materially cheaper alternative if VN-100-class performance is not required; that tradeoff is unresolved and should be a design decision, not a BOM default. |

**Subtotal F (excl. VAT / est. incl. VAT):** €1,199.00 / €1,498.75

---

## G. Dual magnetometers

Baseline role: "Calibration includes a propulsion-current deviation term." Two units for
redundancy per the baseline's degradation table ("One magnetometer lost/rejected... Both
magnetometers lost/rejected").

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| Magnetometer | PNI RM3100 3-axis breakout board (magneto-inductive, low-noise, SPI/I²C) via Unitronic GmbH (DE, EU-shippable) | 2 | €35.00 excl. VAT / est. €43.75 incl. (25% est., vendor page did not display an incl.-VAT figure) | "31 vorrätig (mehr kann nachbestellt werden)" — 31 in stock, more backorderable | [Unitronic.de RM3100](https://www.unitronic.de/produkt/rm3100-pni-3-axis-breakout-board/), 2026-07-22 | [CONFIRMED 2026-07-22] `[UNVERIFIED — selection]`. New item, not costed in R1/R2. Note: PNI's own official store lists the same board at **USD 25.00** but states "online orders only for shipment within the US" — not EU-actionable — [pnisensor.com](https://www.pnisensor.com/product/rm3100-breakout-board/), 2026-07-22, `[CONFIRMED, not EU-usable]`. |

**Subtotal G (excl. VAT / est. incl. VAT), qty 2:** €70.00 / €87.50

---

## H. Speed log

Baseline role: "Compared with LEO-derived speed over ground to estimate current set and
drift."

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| Speed log | Airmar/Garmin DST810-PV-N2 (Gen2 through-hull, NMEA2000 + Bluetooth, paddlewheel speed + depth + temp) via SVB24 (DE) | 1 | €483.14, vendor-stated "VAT inc., ex-works" (est. €386.51 excl., approximated at 25%; Germany's actual VAT rate is 19%, so the true ex-VAT figure is somewhat higher — the €483.14 incl. figure is the reliable one) | "more than 30 in stock" | [SVB24 DST810-PV-N2](https://www.svb24.com/en/airmar-dst810-gen2-through-hull-transducer-nmea2000-bluetooth.html), 2026-07-22 | [CORRECTED: a search-result snippet for this same page had suggested ≈€394.92; the live page (confirmed via `<title>` tag, "...only 483,14 € \| SVB") shows €483.14 — the snippet price was stale/cached or for a different variant]. New item, not costed in R1/R2. |

**Subtotal H (excl. VAT / est. incl. VAT):** €386.51 / €483.14

---

## I. Host computer

Baseline role: "Companion computer monotonic clock — Runtime ordering and watchdogs." Also
carries the USB-3 host role R1 flags as sensitive to controller quality (avoid hubs;
prefer isolated root ports).

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| Host computer | OnLogic CL260 fanless industrial mini PC, base config (Intel N150/N250, 4× USB 3.2 Gen1 Type-A + 2× USB-C, dual 1GbE, 12–24 V DC input) | 1 | est. €996.60 incl. VAT (USD 906.00 ≈€797.28 excl. + 25% est.) | Confirmed via US store live pricing JSON; EU store page (`onlogic.com/eu/...`) returned empty/404 on both attempted URLs this session | [OnLogic CL260 (US store)](https://www.onlogic.com/store/cl260/), 2026-07-22 | [CONFIRMED 2026-07-22] price and spec confirmed on the US store (raw JSON: `"displayValue":"$906.00"`; "4x USB 3.2 Gen1 Type-A" confirmed). `[UNVERIFIED — page unreachable]` for a direct EU-store EUR price; the figure above is FX-converted + assumed DK import VAT, not a native EU quote. |

**Subtotal I (excl. VAT / est. incl. VAT):** €797.28 / €996.60

---

## J. ArduPilot autopilot hardware

Baseline role: "Receives MAVLink `GPS_INPUT` (message 232, `GPS1_TYPE=14`)."

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| Autopilot | CubePilot "The Cube Orange+" Standard Set (STM32H757, requires a carrier board — included in the Standard Set) | 1 | €489.00 (vendor-displayed EUR retail price) | **"Out of stock" / "Sold out"** | [aeroboticshop.com Cube Orange+ Standard Set](https://aeroboticshop.com/products/cube-orange-standard-set-ads), 2026-07-22 | [CONFIRMED price 2026-07-22, unavailable]. Two other candidate EU/US resellers (robotshop.com, wimo.com is unrelated — the actual comparison was robotshop.com and a generic search) returned HTTP 403 to automated fetch and could not be independently re-verified this session — `[UNVERIFIED — page unreachable]` for those. Re-check stock across NW Blue, RaceDayQuads, GetFPV before ordering; this is a widely stocked product and the sold-out status at one reseller is unlikely to reflect market-wide unavailability. |

**Subtotal J (excl. VAT / est. incl. VAT):** €391.20 (est. ex.) / €489.00

---

## K. Helm kill-cord

Baseline role (Mission and operating assumptions, not the equipment table): "A qualified
person shall remain aboard with a physical, controller-independent manual override." This
line is that physical override at the helm.

| Item | Candidate | Qty | Unit price EUR | Availability | Source / access date | Status |
|---|---|---|---:|---:|---|---|
| Kill-cord | Universal outboard kill switch with coiled kill-cord lanyard | 1 | €36.90 (GBP 31.54, vendor-stated VAT-inclusive at UK's 20% rate, FX-converted; not re-based to DK's 25%) | "Estimated Delivery: 1 - 2 Working Days UK Mainland" | [Fox's Chandlery](https://foxschandlery.com/products/outboard-kill-switch-with-kill-cord-lanyard), 2026-07-22 | [CONFIRMED 2026-07-22]. New item, not costed in R1/R2. UK-to-DK import handling (post-Brexit) not modelled beyond the FX conversion; this is a low-value, low-risk line either way. |

**Subtotal K (excl. VAT / est. incl. VAT):** €29.52 (est. ex.) / €36.90

---

## L. Power / enclosure / cabling allowance

No baseline table row; supports every subsystem above (DC-DC power conditioning, fusing,
marine-rated enclosure/box for the bladeRF + host PC + reference chain, bulkhead glands,
RF and data cable runs, connectors). This is explicitly an **allowance**, not a priced
quote, per the brief's own instruction ("no purchases, no quote requests").

| Item | Basis | Qty | Amount EUR | Status |
|---|---|---:|---:|---|
| Power/enclosure/cabling allowance | Flat engineering estimate, not vendor-sourced | 1 (lump) | €800.00 excl. / €1,000.00 est. incl. (25% assumption applied for consistency with the rest of this document, though no real vendor transaction underlies it) | `[ESTIMATE — allowance, no vendor source]`. Order-of-magnitude only; a real figure requires a wiring/enclosure BOM this unit was not scoped to produce. |

**Subtotal L (excl. VAT / est. incl. VAT):** €800.00 / €1,000.00

---

## Iridium STL — procurement note, not a priced line item

> **Superseded (2026-07-23, D42): STL is rejected by user decision — owned equipment only,
> no service fees. Do not pursue the quote steps below; retained for the record only.**

Per the brief and per R2 §1.4: no public list price exists for the Iridium STL service
subscription or for VIAVI's SecureTime LEO receiver modules (STL-2600 / STL-1000). Both
require a vendor quote. **Procurement action, not a BOM line:**

1. Email **pnt@iridium.com** describing the use case (manned experimental maritime PNT,
   Danish waters/EU, research + operational dual role), approximate volume (1 unit for
   trials), and whether timing, position, or both are required.
2. In parallel, request a quote from VIAVI for SecureTime LEO service + an STL-2600 or
   STL-1000 receiver module via
   [viavisolutions.com/en-us/contact-sales](https://www.viavisolutions.com/en-us/contact-sales).
3. Optionally contact Adtran or Safran for a bundled GNSS+STL appliance path.

This was not re-verified live in this unit beyond confirming R2's own citations remain
internally consistent with the brief's instruction to treat it as a procurement action;
no vendor quote page carries a public number to check.

---

## Totals

| Subsystem | Excl. VAT (EUR) | Est. incl. VAT (EUR) |
|---|---:|---:|
| A — bladeRF 2.0 micro xA4 | 801.06 | 1,001.33 |
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

Iridium STL is excluded from this total (procurement action, no priced line, see above).

These totals are a **desk-research budgetary estimate**, not a landed-cost quote: no
checkout was completed for any line, customs duty is not modelled anywhere (only VAT), and
several lines carry an assumed 25% Danish VAT rate applied to non-EU or ex-VAT source
prices rather than a confirmed rate. Two lines have real availability problems today
(Ku dish/feed SKU discontinued; Cube Orange+ sold out at the checked reseller) that do not
block pricing but do block an actual order without re-sourcing.

---

## Open items carried forward

- FE-5680A rubidium reference: no live price obtained this session (`[UNVERIFIED — page
  unreachable]`); OCXO substituted as the priced/adopted reference.
- Ku dish/feed: priced SKU discontinued; a replacement in-stock SKU should be re-sourced
  before ordering.
- Cube Orange+: sold out at the one EU reseller successfully fetched; two other resellers
  could not be reached this session (HTTP 403).
- Host computer EU-store price: only the US-store price was confirmed live; the EU-store
  page did not return content.
- IMU and dual-magnetometer selections are proposals against baseline rows that specify no
  part (`[UNVERIFIED — selection]`), not confirmed design decisions.
