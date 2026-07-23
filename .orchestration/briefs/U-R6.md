# Brief U-R6 — Unknown-emitter AoA array research (live web, grok)

Contract v5.1 report rules. Purpose: feasibility inputs for a 4-5 element coherent antenna
array doing direction finding on UNKNOWN land-based emitters from a moving vessel at
7-30 kn, fused as bearings-only SLAM into an existing error-state EKF. Every claim: URL +
access date (2026-07-23), VERIFIED/ASSUMED split.

1. COTS coherent multi-channel receivers: KrakenSDR (5-ch coherent RTL) and comparable
   (price, EU availability, phase-calibration approach, frequency range, published DF
   accuracy); what a bladeRF-class 2-channel adds/can't do; array-extension options.
2. Direction-finding accuracy from small arrays on moving platforms: published MUSIC/
   correlative-interferometer accuracy vs SNR/aperture at VHF/UHF; multipath over water;
   mast-mounted marine DF experience (e.g. VHF DF, AIS localization work).
3. Bearings-only SLAM / unknown-emitter navigation literature: observability conditions
   (motion baselines), convergence rates, published vehicle/vessel experiments using
   terrestrial broadcast (FM/DAB/DVB-T/cellular) as unknown or partially-known beacons;
   TDOA/FDOA alternatives with a single moving receiver.
4. Emitter landscape in Danish/Baltic coastal water: FM/DAB/DVB-T2 transmitter density and
   typical received power at 10-50 km offshore; cellular coverage range offshore; AIS
   base stations; which bands are best for a 4-5 element array with ~1-2 m aperture.
5. High-speed benefit/cost: how platform speed affects bearings-only observability and DF
   (faster geometry vs shorter integration, antenna motion/vibration at planing speeds).

## Output format (stdout contract — write NO files)
`===R6-DOC===` then the document, `===R6-REPORT===` then the report.
