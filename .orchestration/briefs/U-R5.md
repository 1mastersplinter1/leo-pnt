# Brief U-R5 — High-speed marine dynamics research (live web, grok)

Contract v5.1 report rules. PURPOSE: replace the [UNVERIFIED] engineering estimates in the
high-speed envelope analysis (planing dynamics at 20-30 kn) with published values. Every
claim: source URL + access date (2026-07-23); VERIFIED (read it) vs ASSUMED split; prefer
primary sources (papers, standards, manufacturer datasheets).

Questions:
1. Planing-hull vertical accelerations and slam statistics at 20-30 kn in moderate sea
   states: published measurement campaigns / standards (e.g. high-speed craft design
   standards, RIB/patrol-boat trials) — peak g, rms g, impact duration, occurrence rates
   vs speed and significant wave height.
2. Sustained trim angles for planing hulls vs speed (typical ranges, sources).
3. Crystal/rubidium oscillator behavior under vibration and shock: g-sensitivity specs for
   FE-5680A-class Rb and quality OCXOs, vibration-induced phase noise formulas, and COTS
   shock/vibration isolation solutions used for precision oscillators in vehicles/marine
   (products, isolation frequency, EU availability, prices).
4. MEMS IMU behavior in high-vibration marine use: vibration rectification error (VRE)
   specs, which IMU classes (e.g. VN-100 class vs automotive vs tactical) publish marine/
   high-dynamics performance, anti-vibration mounting practice; any published 20-30 kn
   small-craft navigation-sensor studies.
5. High-speed craft safety/collision context: stopping and turning distances at 20-30 kn
   for small planing craft, watchkeeping/reaction-time guidance for high-speed craft
   (HSC Code or similar) — inputs for speed-scaled alarm/ack timer budgets.
6. Doppler/comms: any published effect of planing-craft motion (antenna attitude swings,
   spray/washdown) on L-band/Ku satellite reception quality at speed.

## Output format (stdout contract — write NO files)
Final answer text only: line `===R5-DOC===`, the research document, line `===R5-REPORT===`,
the unit report (summary, VERIFIED/ASSUMED, dead ends).
