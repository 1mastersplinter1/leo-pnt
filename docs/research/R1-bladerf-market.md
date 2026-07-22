# R1 — bladeRF / LNB / host / tooling market research

**Access date for all live web claims:** 2026-07-22  
**Contract version:** v1  
**Unit:** U-R1  

Every section splits **VERIFIED** (page loaded / content read stating the claim) vs **ASSUMED** (inference or secondary summary not re-read as primary). Source URLs are listed per claim.

---

## 1. Nuand bladeRF 2.0 micro xA4 / xA9 — price, stock, specs

### 1.1 Prices and stock

#### VERIFIED

| Source | Model | Price | Stock / ship note | Currency notes |
|--------|-------|-------|-------------------|----------------|
| [Nuand xA4 product page](https://www.nuand.com/product/bladerf-xa4/) | bladeRF 2.0 micro xA4 | **USD 540.00** | “In stock - Usually ships out in 1 to 2 business days.” | USD; tax/shipping not shown on product page. |
| [Nuand xA9 product page](https://www.nuand.com/product/bladerf-xa9/) | bladeRF 2.0 micro xA9 | **USD 860.00** | Same in-stock / 1–2 business day ship claim | USD |
| [Nuand shop](https://www.nuand.com/shop/) | xA9 THERMAL | USD 1,850.00 | Listed with in-stock language on related pages | USD |
| [SDRstore.eu xA4](https://www.sdrstore.eu/software-defined-radio/instruments/bladerf/bladerf-2-0-micro-xa4-software-defined-radio-en/) | xA4 | **EUR 1,001** (HTML price node; search snippet previously showed €1,001.33) | **In stock**; free shipping over €415; NL destination estimate 7–8 business days | EUR storefront; VAT treatment not fully disclosed on product page (EU retailer — treat as retail incl. VAT until checkout proves otherwise). 5+ qty: €962.06 |
| [SDRstore.eu xA9](https://www.sdrstore.eu/software-defined-radio/instruments/bladerf/bladerf-2-0-micro-xa9-software-defined-radio-en/) | xA9 | **EUR 1,214** (unit price node); 5+ qty **€1,148.58** | **In stock** | EUR |
| [Lab401 bladeRF xA4 packs](https://lab401.com/products/bladerf-sdr-2-micro-xa4) | xA4 Starter / Deluxe packs | Pack pricing shown (DKK locale at fetch: Starter ~4 948.78 kr; Deluxe ~6 855.09 kr); page claims EU/US stock and EU dispatch | “In Stock (EU, US)”; “Import duties may apply.” | Packs include antennas/case/amps — not bare board USD equivalent. |

#### ASSUMED

- Nuand USD list prices are pre-tax; US state sales tax and international shipping are applied at checkout (not verified by completing a cart).
- Shipping Nuand (USA) → EU on a ≥€540 equivalent order: EU import **VAT always due**; customs **duty** typically applies above the €150 de minimis for commercial goods; IOSS may or may not be used by Nuand (not verified on Nuand checkout). Prefer EU stock (SDRstore / Lab401) to avoid dual VAT/customs friction.
- SDRstore ~EUR 1 001 for xA4 vs Nuand USD 540 is a large retail markup (~85% at ~1.08 USD/EUR); likely includes EU VAT + distributor margin + logistics — exact VAT split not verified at checkout.
- Rough bare-board EUR equivalent of Nuand list (FX only, no VAT): xA4 ~€500, xA9 ~€800 at ~1.08 USD/EUR — **not a live FX quote**.

### 1.2 EU distributors (concrete)

#### VERIFIED

| Distributor | Region signals | bladeRF offering | Notes |
|-------------|----------------|------------------|-------|
| **SDRstore.eu** | Phone +31…; NL destination estimates; DHL/FedEx; free ship >€415 | xA4, xA9, THERMAL variants, BT-100/200 | Live prices above; EU-facing webshop |
| **Lab401** | Claims dispatch from **Europe**; pack SKUs | xA4 Starter/Deluxe packs with accessories | Good for “no US customs” path; pack pricing |
| **Nuand direct** | Made in USA | Bare xA4/xA9 kits (board + USB 3 cable) | Official; international ship possible, customs on EU side |

#### ASSUMED

- Digi-Key / Mouser / SparkFun listings for WRL-15043 (xA4) are **obsolete / discontinued** for new sales (discontinuation notices and “obsolete” lifecycle seen in search results) — not a reliable EU procurement path in 2026.
- No major pan-EU catalogue distributor (RS, Farnell) was found with live in-stock bare micro xA4/xA9 at research time.

### 1.3 Key specs confirmation

#### VERIFIED (Nuand product pages + product brief overview)

| Spec | Claim | Source |
|------|-------|--------|
| 2×2 MIMO | “2×2 MIMO streaming” / “2×2 MIMO channels” | [xA4](https://www.nuand.com/product/bladerf-xa4/), [xA9](https://www.nuand.com/product/bladerf-xa9/), [overview](https://www.nuand.com/bladerf-2-0-micro/) |
| Coherent dual RX | Shared RFIC (AD9361) + shared clock architecture on one board; both RX ports part of the single 2×2 design | Product pages + block-diagram description; AD9361 stated as transceiver |
| 10 MHz ext ref | “on-board PLL allows the bladeRF 2.0 micro to tame its VCTCXO to a **10 MHz reference signal**” | [xA4](https://www.nuand.com/product/bladerf-xa4/), [xA9](https://www.nuand.com/product/bladerf-xa9/) |
| Tuning | Marketing: **47 MHz–6 GHz**; detailed table: **RX 70–6000 MHz**, **TX 47–6000 MHz** | [overview specs table](https://www.nuand.com/bladerf-2-0-micro/) |
| Max sample rate (standard) | **61.44 MSPS** ADC/DAC; filtered BW up to **56 MHz** | Overview table |
| Extended sample rate | Capable of up to **122.88 MSPS** (2023.02 release; overclock / 8-bit packing) | Product pages link to [2023.02 release](https://www.nuand.com/2023-02-release-122-88mhz-bandwidth/) |
| USB 3.0 | “USB 3.0 SuperSpeed”; Cypress FX3; “saturate the full duplex 5 Gbps USB 3.0 link” (marketing) | Product pages |
| ADC resolution | 12-bit | Overview table |
| Clock | Factory-calibrated **38.4 MHz** VCTCXO; taming via DAC or ADF4002 PLL | Overview + product text |
| Kit contents | Board + USB 3.0 SS cable (no case in base kit) | Product pages |

#### ASSUMED

- Dual RX is **coherent** (same LO/clock domain) by AD9361 architecture; Nuand does not use the word “coherent” on the product page, but a shared AD9361 2×2 is the industry-standard interpretation for this platform.
- External 10 MHz is accepted as sine/square within ADF4002 limits; exact amplitude/level window not re-verified from ADF4002 datasheet in this unit.
- For LEO Ku IF + L-band: both 950–2150 MHz IF and Iridium ~1616–1626.5 MHz sit inside RX 70–6000 MHz.

### 1.4 Shipping-to-EU implications

#### VERIFIED

- EU-internal purchase (SDRstore, Lab401 EU stock): no third-country import customs between EU member states for ordinary commercial circulation ([EU internal movement principle — general trade guidance](https://www.parcel-guide.eu/guides/customs-and-vat-international-shipping-guide/)).
- Imports from **outside** the EU: VAT applies from low values; duty de minimis historically **€150** for commercial goods (same guide family; confirm with local customs for 2026 updates).

#### ASSUMED

- Prefer **SDRstore.eu or Lab401** for EU projects despite higher EUR sticker, unless Nuand IOSS/pre-collect VAT is confirmed at checkout and total landed cost still wins.

---

## 2. xA4 vs xA9 FPGA sizing and on-board DSP

### 2.1 Logic-element counts

#### VERIFIED

| Resource | xA4 | xA9 | Source |
|----------|-----|-----|--------|
| Logic elements (kLE) | **49** | **301** | Nuand product pages + [overview FPGA table](https://www.nuand.com/bladerf-2-0-micro/) |
| User-programmable (approx.) | ~**32 kLE** (Nuand) / SDRstore also cites ~38 kLE in places | ~**292 kLE** | Nuand product text; SDRstore comparison blog |
| FPGA memory | **3 383 kbits** | **13 917 kbits** | Overview table |
| Variable-precision DSP blocks | **66** | **342** | Overview table |
| Embedded 18×18 multipliers | **132** | **684** | Overview table |
| Device class | Cyclone V (e.g. 5CEA4 class) | Cyclone V (e.g. 5CEA9 class) | PySDR / community docs; Nuand “Cyclone V” |

Nuand states explicitly that the **xA9 FPGA is for hardware accelerators** (FFTs, turbo decoders, TX modulators/filters, RX acquisition correlators) and that **no such accelerators ship with the product** — customer designs or third-party IP only ([xA9 page](https://www.nuand.com/product/bladerf-xa9/)).

RF front end, sample rate, USB, and MIMO are **the same** on xA4 and xA9 ([SDRstore comparison](https://www.sdrstore.eu/bladerf-2-0-micro-xa4-vs-xa9-which-sdr-should-you-buy/), 2026-06-12).

### 2.2 What channelisation / DSP fits realistically

#### VERIFIED (capacity facts + Nuand positioning)

- **xA4 (~32 kLE free after host bitstream):** suitable for modest custom HDL, learning projects, and the stock “hostedxA4” streaming image. Stock image already consumes a non-trivial fraction of the 49 kLE.
- **xA9 (~292 kLE free):** Nuand positions for multi-block modem chains (FFT + correlators + filters + decoders). Memory (~4×) and DSP blocks (~5×) matter as much as LE count for channelisers.

#### ASSUMED (resource budgeting for this project)

| DSP block (typical Cyclone V HDL) | Likely fits xA4 leftover? | Likely fits xA9? |
|-----------------------------------|---------------------------|------------------|
| Narrow FIR channel filter + decimate (few Msps after DDC) | Marginal / small only | Comfortable multi-channel |
| Multi-channel polyphase channeliser (e.g. 8–32 channels @ multi-MHz) | Unlikely after host core | Plausible with careful design |
| Long correlators for LEO beacons / PSS-SSS class acquisition | Tight | Primary reason to pick xA9 |
| Host-side DSP only (USB stream all IQ) | Yes (xA4 enough) | No FPGA benefit |

**Project implication (ASSUMED):** if all channelisation and Doppler tracking run on the Linux host at 2.5–5 MHz per observable (handoff brief), **xA4 is enough**. Prefer **xA9** only if a roadmap item needs on-FPGA multi-channel correlators or dual-band channelisation with low USB load.

### 2.3 Community evidence of custom FPGA DSP

#### VERIFIED

- Nuand **VHDL ADS-B decoder** runs decode on FPGA and ships source ([bladeRF ADS-B article](https://www.nuand.com/bladerf-vhdl-ads-b-decoder/), GitHub `Nuand/bladeRF-adsb`); originally demonstrated on bladeRF 1.x (x40/x115) and cited as the model for FPGA modem work. Larger FPGA (xA9 / x115 class) is repeatedly described as enabling *more* of this class of work ([Hackaday 2018](https://hackaday.com/2018/08/30/bladerf-2-0-micro-is-smaller-more-powerful/)).
- Open HDL tree: [github.com/Nuand/bladeRF/tree/master/hdl](https://github.com/Nuand/bladeRF/tree/master/hdl) (VHDL; Quartus Lite).
- Nuand marketing lists example accelerator *types* for xA9 (FFTs, turbo, correlators) without shipping them.

#### ASSUMED

- Production-quality third-party bladeRF 2.0 channeliser IP is scarce compared to USRP; expect in-house HDL or host DSP.

---

## 3. Sustained USB 3.0 throughput on Linux

### 3.1 Bandwidth arithmetic (VERIFIED calculation, ASSUMED operational margin)

SC16_Q11 (2 × int16 per IQ sample) ≈ **4 bytes/sample**.

| Mode | Aggregate sample rate | Approx. USB payload |
|------|----------------------|---------------------|
| 1× RX @ 5 MSPS | 5e6 | ~20 MB/s |
| 1× RX @ 30 MSPS | 30e6 | ~120 MB/s |
| 2× RX @ 30 MSPS | 60e6 | ~240 MB/s |
| 2× RX @ 61.44 MSPS | 122.88e6 | ~491 MB/s |
| 122.88 MSPS 8-bit mode (1 ch) | see Nuand 2023.02 | designed to ease USB limit |

USB 3.0 SuperSpeed theoretical ~5 Gbit/s; real host+stack sustained is lower. **491 MB/s dual full-rate 12-bit is aggressive** and host-dependent.

### 3.2 Real-world reports

#### VERIFIED

- Official wiki page **“Debugging dropped samples and identifying achievable sample rates”** (Nuand/bladeRF wiki, last edited 2019-08-30 per GitHub wiki metadata): states USB 3.0 controller quality varies widely; points to working/problematic host lists; notes USB 2.0 practical rates historically ~**5–8 MSPS** (some reports to 10). Page load of full body was incomplete via GitHub UI, but title/metadata and search snippets confirm content and USB-controller caveat. Snippet: “Some USB 3.0 controllers work far better than others.”
- Forum [USB errors / RX buffer overrun](https://nuand.com/forums/viewtopic.php?t=5140): user fixed issues by raising **buffers 32 / transfers 16** (from 16/8).
- PySDR bladeRF chapter ([pysdr.org](https://pysdr.org/content/bladerf.html)): example `sync_config` uses `num_buffers=16`, `buffer_size=8192`, `num_transfers=8`; notes occasional “Hit stall for buffer” as expected at stream stop; documents sample_rate_range max **61 440 000**.
- Nuand [2023.02 release](https://www.nuand.com/2023-02-release-122-88mhz-bandwidth/): 8-bit mode + packing specifically to support higher aggregate rates under USB constraints; overclock path “may affect system stability.”

#### ASSUMED

- On a **good xHCI USB 3.0 root port** (Intel/AMD desktop, device alone on controller, no hubs), **single-channel 30–40 MSPS SC16** is routinely sustainable; **dual-channel ~20–30 MSPS each** is a common working envelope for continuous capture.
- **Full dual 61.44 MSPS SC16** is not guaranteed; plan for host selection tests or 8-bit / lower rate / FPGA decimation.
- For this LEO mission (beacon correlation **2.5–5 MHz** BW per tracker): USB margin is large even for 2× RX — **not the primary risk**.

### 3.3 Host controller caveats and buffers

#### VERIFIED

- Prefer direct motherboard USB 3.x port; avoid unpowered hubs; controller chipset lists maintained on wiki “Troubleshooting” (linked from dropped-samples page).
- Tunables: `num_buffers`, `buffer_size`, `num_transfers`, stream timeout (libbladeRF / Python sync API).

#### ASSUMED

- Isolating bladeRF on its own USB controller root, disabling USB autosuspend for the device, and using a real-time-friendly userspace (or at least avoiding heavy desktop compositing during capture) reduces drop rate.

---

## 4. Software path maturity

### 4.1 Rust bindings / crates

#### VERIFIED

| Crate / repo | Latest observed | Role | Maintenance signal |
|--------------|-----------------|------|---------------------|
| [`bladerf`](https://docs.rs/crate/bladerf/0.1.2) / lib.rs | **0.1.2** (lib.rs: **2024-10-07**) | Safe bindings wrapping `bladerf-sys`; described as WIP wrapper for libbladeRF | Stale since late 2024 |
| [`bladerf-sys`](https://docs.rs/crate/bladerf-sys/0.1.2) | **0.1.2** | FFI / bindgen layer | Paired with `bladerf` |
| [`libbladerf-rs`](https://docs.rs/crate/libbladerf-rs/0.4.1) | **0.4.1** (**2026-06-20** per lib.rs) | **Pure Rust** driver; **BladeRF1 only (x40/x115)**; “BladeRF2 support —” incomplete; **no C libbladeRF dependency** | Active in 2026 but **wrong generation for micro 2.0** |
| [`bladerf-bindings`](https://docs.rs/crate/bladerf-bindings/0.0.13) | **0.0.13** | Alternate bindings crate | Low version; not primary ecosystem path |
| [Nuand/libbladeRF-rust](https://github.com/Nuand/libbladeRF-rust) | No releases; example `hello_libbladeRF` | Official-ish bindgen examples | Sparse (few commits); MIT; not a polished crates.io product |

#### ASSUMED

- For a **bladeRF 2.0 micro** Rust workspace: most reliable path is **FFI to system `libbladeRF`** (either via `bladerf`/`bladerf-sys` with local maintenance, or project-owned bindgen), **not** `libbladerf-rs` until BladeRF2 lands.
- Expect to own glue code (sync RX MIMO, timestamps, clock_ref) rather than depend on a mature high-level Rust SDR stack.

### 4.2 SoapySDR route

#### VERIFIED

- Plugin: [pothosware/SoapyBladeRF](https://github.com/pothosware/SoapyBladeRF) — SoapySDR module for bladeRF.
- Packaged in distros (e.g. Arch `soapybladerf`, conda-forge `soapysdr-module-bladerf`).
- Nuand product pages list **SoapySDR** among supported ecosystems; GNU Radio typically via **gr-osmosdr** / Soapy source blocks.

#### ASSUMED

- Soapy is the fastest path for GNU Radio / multi-SDR apps; slightly more layers than raw libbladeRF for low-latency MIMO + timestamps — measure before committing for production navigator.

### 4.3 Python

#### VERIFIED

- **Official** Python bindings ship in libbladeRF tree: `host/libraries/libbladeRF_bindings/python` (install via setup.py per [PySDR](https://pysdr.org/content/bladerf.html)).
- API surface: `from bladerf import _bladerf`, `BladeRF()`, channel config, `sync_config` / `sync_rx` / `sync_tx`, MIMO layouts `RX_X1` / `RX_X2`.
- Mature for prototyping; same C library as production C/C++.

#### ASSUMED

- Python is ideal for surveys and lab tools; production maritime stack likely Rust/C++ with Python offline analysis.

### 4.4 Maturity summary (ASSUMED ranking for this project)

1. **libbladeRF C + official Python** — production-proven.  
2. **SoapyBladeRF** — good interoperability.  
3. **Rust via FFI (`bladerf`/`bladerf-sys` or custom)** — usable but thin; plan ownership.  
4. **`libbladerf-rs` pure Rust** — active, but **not for bladeRF 2.0** today.

---

## 5. Ku LNB — free-running stable chain, external ref

### 5.1 Problem with DRO / crude LNBs

#### ASSUMED (domain knowledge aligned with handoff)

- Consumer DRO LNBs: LO error often **±~1 MHz** class with temperature drift — unusable for free-running Doppler SoOP without huge LO states.
- PLL LNBs lock LO to a crystal/TCXO (often **25 MHz** × 390 = **9750 MHz** LO low band; ×424 = **10600 MHz** high band for universal dual-LO designs).

### 5.2 PLL LNB with external reference (radio astronomy / QO-100 class)

#### VERIFIED

**A. Othernet / Bullseye BE01 (TCXO, 25 MHz REF OUT — not ext ref IN by default)**  
- [SDRstore Bullseye page](https://www.sdrstore.eu/software-defined-radio/instruments/rtl-sdr/qo-100-bullseye-tcxo-lnb-ultra-stable-lnb-for-qo-100-and-ku-band-satellites/): **EUR 62**, **In stock**.  
- Specs on page: input **10 489–12 750 MHz**; LO **9750 / 10600 MHz**; IF low **739–1950 MHz**, high **1100–2150 MHz**; PLL + **2 ppm TCXO**; factory cal within 1 kHz; outdoor stability claim within **10 kHz**; **25 MHz reference output** on secondary (red) F-connector.  
- Requires **12–19 V** bias-tee (not RTL-SDR Blog 4.5 V).  
- Support text on same page: 25 MHz is **output**, not a user-swept injection frequency.

**B. qro.cz / hamparts.shop “10 GHz LNB EXT OSC MK3” (true external 10 MHz → 25 MHz for LNB PLL)**  
- [hamparts.shop product](https://hamparts.shop/10-ghz-lnb-ext-osc-mk3.html): **EUR 168.19 incl. tax** / **EUR 139.00 excl. tax**; classic or POTY-adapter option.  
- **LNB needs external 25 MHz** over IF coax; bias-tee box accepts **10 MHz ref in** (SMA) **or** internal **0.2 ppm TCXO**; generates filtered 25 MHz for LNB.  
- LO **9750 MHz** examples: 10 489 → 739 MHz IF; applications list includes **Starlink experiments**, coherent receivers, QO-100.  
- Stability with ext 10 MHz: “depends on reference source”; with TCXO: 0.2 ppm ×390 (LO multiplication).  
- Czech/EU amateur supplier (qro.cz brand).

**C. DIY / community external-ref mods**  
- [Baltic Lab](https://baltic-lab.com/2023/07/lnb-modification-for-10-ghz-qo-100-satellite-reception/): modify cheap PLL LNB 25 MHz crystal path for external ref; documents ×390 / ×424.  
- [remoteqth.com LNB GPS/ref page](https://remoteqth.com/lnb_gps.php): documents MK3 architecture (mirrors hamparts product).  
- Avenger PLL321S-2 + external 27 MHz class refs appear in QO-100 writeups ([destevez.net](https://destevez.net/2022/08/measuring-the-qo-100-beacons-phase-difference/)).

#### ASSUMED

- For **shared free-running Rb/OCXO** disciplining **both** bladeRF (10 MHz in) and LNB:  
  - **Best commercial fit:** hamparts **EXT OSC MK3** (10 MHz in → 25 MHz to LNB) + same 10 MHz to bladeRF (splitter).  
  - **Bullseye** is excellent TCXO stability for QO-100 but **does not replace** an external free-running master unless modified; its 25 MHz **out** could feed other gear but does not lock the LNB to the ship Rb.
- Universal LO **9750 / 10600** covers Starlink/OneWeb Ku with IF up to **2150 MHz** (high band), matching handoff IF math.

### 5.3 LO / IF coverage summary

#### VERIFIED (Bullseye + universal dual-LO convention)

| LO | Ku RF example | IF |
|----|---------------|-----|
| 9750 MHz | 10.7 GHz | 950 MHz |
| 9750 MHz | 11.7 GHz | 1950 MHz |
| 10600 MHz | 11.7 GHz | 1100 MHz |
| 10600 MHz | 12.75 GHz | 2150 MHz |

#### ASSUMED

- Single dual-LO universal LNB + 22 kHz tone / voltage for high band is sufficient for 10.7–12.75 GHz if IF path to bladeRF is clean to 2150 MHz (bladeRF RX covers this).
- Bias-tee / LNB power must **not** rely on bladeRF BT-200 alone without verifying voltage (Bullseye needs 12–19 V).

### 5.4 Price & EU availability snapshot

| Item | EUR (access 2026-07-22) | EU buy link |
|------|-------------------------|-------------|
| Bullseye TCXO LNB | ~€62 incl. (SDRstore) | SDRstore.eu |
| LNB EXT OSC MK3 kit | €168.19 incl. / €139 ex VAT | hamparts.shop |
| Cheaper PLL+TCXO LNB (no ext ref) | ~€52 incl. on hamparts related SKU | hamparts.shop |

---

## 6. Reference oscillator — FE-5680A class and OCXO

### 6.1 FE-5680A surplus rubidium

#### VERIFIED

- Datasheet ([FE-5680A series PDF](https://www.miedema.dyndns.org/co/2019/rb/3rb/FE-5680A-Rubidium-datasheet.pdf)):  
  - Default model frequency **10 MHz**; **factory-settable 1 Hz–20 MHz**.  
  - Output: **0.5 Vrms into 50 Ω** (options for levels); square TTL-compatible for some bands; sine for 5–20 MHz class.  
  - Power: ~**11 W** steady @25 °C, peak ~32 W; **15–18 V** supply class.  
  - Stability examples: Allan ~1.4×10⁻¹¹/√τ class; aging ~2×10⁻⁹/year (datasheet figures).  
  - **Options by type** include non-10 MHz: **5 MHz (03), 15 MHz (04), 13 MHz (05), 2.048 MHz (06), 10.23 MHz (07), 50.255 MHz (01)**, customer frequency (08), etc.  
- Surplus market historically cheap ([ke5fx Rb notes](http://www.ke5fx.com/rb.htm) — 2012 era “&lt; $50”); modern eBay listings still appear (search: FE-5680A 10 MHz) with used pricing commonly in the **tens to low hundreds USD** depending on enclosure/PSU/tested status — **exact live ask prices fluctuate; no single stable MSRP**.

#### ASSUMED

- **Buy only units labeled or measured as 10 MHz sine** for bladeRF ref-in; reject or re-synthesise 5 MHz / 50.255 MHz / 2.048 MHz variants.  
- 10 MHz FE-5680A is suitable for bladeRF ADF4002 path when level is conditioned (often need buffering / amplitude matching).  
- For LNB chain: feed same 10 MHz into hamparts MK3 (or 10→25 MHz PLL such as DF9NP-class); **do not** expect FE-5680A to output 25 MHz without external conversion.  
- Maritime power budget: Rb warm-up current is non-trivial; plan 15 V rail and thermal.

### 6.2 OCXO alternatives (EU commercial)

#### VERIFIED

- [hamparts 10 MHz OCXO board](https://hamparts.shop/10-mhz-ocxo-board.html): **EUR 50.82 incl.** / **EUR 42 excl.**; “Last items in stock”; sine + square SMA; claims calibration vs GPSDO; stability claim **5×10⁻¹²** (verify in use); phase noise claim −140 dBc/Hz @100 Hz; explicitly “Reference for LNB EXT Bias tee”.  
- Related boxed OCXO / GPSDO products on same shop (GPSDO paths exist but **violate free-running premise** if used for continuous GNSS discipline — handoff forbids GPSDO as operational reference).

#### ASSUMED

- Good OCXO is adequate for voyage-scale free-running if pre-calibrated and temperature-controlled; Rb still preferred for multi-day aging if power allows.  
- Split 10 MHz with a distribution amp/splitter to bladeRF + LNB bias-tee simultaneously.

### 6.3 Frequency conversion map

| Device | Needs | From FE-5680A 10 MHz | From non-10 MHz FE-5680A |
|--------|-------|----------------------|---------------------------|
| bladeRF ref-in | 10 MHz (default refin) | Direct (level-condition) | Frequency conversion / DDS / reject unit |
| LNB PLL (25 MHz) | 25 MHz at LNB | ×2.5 PLL (MK3 / DF9NP / custom) | Extra conversion stages |
| Bullseye (stock) | Internal 25 MHz TCXO | Not locked to ship ref without mod | Same |

---

## Cross-cutting recommendations (ASSUMED synthesis)

1. **SDR:** Prefer **EU stock** (SDRstore xA4 ~€1001 or Lab401 pack) for simpler logistics; Nuand USD 540/860 if landed-cost model wins.  
2. **xA4 vs xA9:** **xA4** unless FPGA correlators/channelisers are committed. RF identical.  
3. **Host USB:** Validate dual-RX stream at planned Msps on target industrial PC; buffer defaults 16/8192/8, increase if overruns.  
4. **Software:** Production path **libbladeRF** (+ thin Rust FFI); Soapy for tools; do not plan on pure-Rust BladeRF2 yet.  
5. **LNB:** **hamparts EXT OSC MK3** (~€168 incl.) for true 10 MHz-referenced free-running chain; Bullseye (~€62) as TCXO-only alternative or secondary.  
6. **Clock:** Surplus **FE-5680A 10 MHz** or **hamparts OCXO €51**; split to bladeRF + LNB; never operational GPSDO.

---

## Source index (access 2026-07-22)

| ID | URL |
|----|-----|
| N1 | https://www.nuand.com/product/bladerf-xa4/ |
| N2 | https://www.nuand.com/product/bladerf-xa9/ |
| N3 | https://www.nuand.com/bladerf-2-0-micro/ |
| N4 | https://www.nuand.com/shop/ |
| N5 | https://www.nuand.com/2023-02-release-122-88mhz-bandwidth/ |
| N6 | https://www.nuand.com/bladerf-vhdl-ads-b-decoder/ |
| E1 | https://www.sdrstore.eu/software-defined-radio/instruments/bladerf/bladerf-2-0-micro-xa4-software-defined-radio-en/ |
| E2 | https://www.sdrstore.eu/software-defined-radio/instruments/bladerf/bladerf-2-0-micro-xa9-software-defined-radio-en/ |
| E3 | https://www.sdrstore.eu/bladerf-2-0-micro-xa4-vs-xa9-which-sdr-should-you-buy/ |
| E4 | https://www.sdrstore.eu/software-defined-radio/instruments/rtl-sdr/qo-100-bullseye-tcxo-lnb-ultra-stable-lnb-for-qo-100-and-ku-band-satellites/ |
| L1 | https://lab401.com/products/bladerf-sdr-2-micro-xa4 |
| H1 | https://hamparts.shop/10-ghz-lnb-ext-osc-mk3.html |
| H2 | https://hamparts.shop/10-mhz-ocxo-board.html |
| R1 | https://remoteqth.com/lnb_gps.php |
| P1 | https://pysdr.org/content/bladerf.html |
| G1 | https://github.com/Nuand/libbladeRF-rust |
| G2 | https://github.com/pothosware/SoapyBladeRF |
| G3 | https://github.com/Nuand/bladeRF/wiki/Debugging-dropped-samples-and-identifying-achievable-sample-rates |
| C1 | https://docs.rs/crate/bladerf/0.1.2 |
| C2 | https://docs.rs/crate/libbladerf-rs/0.4.1 |
| C3 | https://lib.rs/crates/bladerf |
| C4 | https://lib.rs/crates/libbladerf-rs |
| F1 | https://www.miedema.dyndns.org/co/2019/rb/3rb/FE-5680A-Rubidium-datasheet.pdf |
| F2 | http://www.ke5fx.com/rb.htm |
| B1 | https://baltic-lab.com/2023/07/lnb-modification-for-10-ghz-qo-100-satellite-reception/ |
| D1 | https://destevez.net/2022/08/measuring-the-qo-100-beacons-phase-difference/ |
| X1 | https://www.rtl-sdr.com/bladerf-micro-2-0-now-supports-up-to-122-88-mhz-of-bandwidth/ |
| K1 | https://hackaday.com/2018/08/30/bladerf-2-0-micro-is-smaller-more-powerful/ |
| V1 | https://www.parcel-guide.eu/guides/customs-and-vat-international-shipping-guide/ |

---

*End of R1 research note.*
