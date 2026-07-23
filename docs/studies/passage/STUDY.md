# Passage endurance study

**SYNTHETIC ONLY.** Nine hours at 6 kn covers 100.01 km; GNSS is lost at hour 2 and ephemeris is cached at departure.

## Result

| handling | Doppler through | final position error | position class |
|---|---:|---:|---|
| hard 6 h | 6.0 h | 3050 m | dead-reckoning (>1 NM error) |
| graduated, 30 h ceiling | 9.0 h | 350 m | passage-held (<1 NM error) |

The binary gate kills Doppler three hours before arrival and the solution degrades to DR. Graduated weighting retains Doppler through the passage and holds the synthetic position class.

## D43 caveat

D43 applies: synthetic epoch aging aliases orbital phase and is a stand-in, not validation of real SupGP error growth.

The 350 m aided bound, 0.25 m/s DR error, SGP4 error curve, LOS-rate mapping, and 30 h ceiling are `[UNVERIFIED]` pending real-SupGP aging and at-sea replay.
