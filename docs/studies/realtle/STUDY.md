# Real-TLE constellation geometry realism check

**REAL-PUBLISHED-TLE GEOMETRY CHECK [UNVERIFIED currency/provenance]. Endpoints come from the production Executive + FilterStub against synthetic generator truth; no result is clamped or target-fitted. The elements were grok-fetched and were not independently confirmed against CelesTrak. The receiver is placed at a fixed mid-latitude via a single rigid coordinate rotation (see module docs) so a real Starlink shell has adequate simultaneous coverage; this is a placement choice made from visibility geometry alone, before any accuracy result was computed.**

## Real result

N=8 was searched for (receiver latitude 25-60 deg and the full 48h TLE validity window, checked against the vessel's actual generated trajectory rather than an idealised fixed point) and is not physically available from this real fixture for the full five-minute persistent no-handover leg; N=7 is the confirmed maximum, reported here instead of forcing an unreachable tier. Controlled N=7 manoeuvring result on REAL Starlink geometry: mean 35.8 m, p95 56.4 m, range 16.2-56.4 m across 8 seeds (<100 m).

N=7 on REAL Starlink geometry reaches the D56 usable denied target (settled reference: p50 <=500 m / p95 <=750 m over >=100 km-class legs); here mean 35.8 m / p95 56.4 m over a 5-minute leg. GDOP p95 is 13.86. Caveat: real GDOP p95 (13.86) is substantially WORSE than the synthetic study's ~1.8, yet the real endpoint error is smaller than the synthetic 554 m; a range-domain GDOP snapshot does not evidently predict this Doppler-EKF's realized accuracy over a short aided-then-denied leg, and this decoupling is itself [UNVERIFIED] -- it is reported, not explained away. This is a real-geometry realism check on the synthetic multisat N=8 result (D57: mean 116 m / p95 554 m, GDOP ~1.8), not a replacement for it -- vessel dynamics, clock, and measurement noise remain synthetic [UNVERIFIED].

| geometry | N | fixed SVs | GDOP mean/p95 | endpoint position mean/p95/spread | velocity mean | accepted/rejected mean | class |
|---|---:|---|---:|---:|---:|---:|---|
| fixed single LOS; no handover | 1 | [44741] | unobservable/infinite/unobservable/infinite | 52.5/88.2/28.8-88.2 m | 0.199 m/s | 55.0/0.0 | <100 m |
| fixed simultaneous multi-LOS cohort; no handover | 2 | [44741, 45366] | unobservable/infinite/unobservable/infinite | 51.9/92.4/9.2-92.4 m | 0.216 m/s | 110.0/0.0 | <100 m |
| fixed simultaneous multi-LOS cohort; no handover | 4 | [44741, 45366, 45368, 45377] | 46.66/199.60 | 39.4/58.6/15.4-58.6 m | 0.170 m/s | 220.0/0.0 | <100 m |
| fixed simultaneous multi-LOS cohort; no handover | 7 | [44741, 45366, 45368, 45377, 45387, 45405, 45580] | 8.75/13.86 | 35.8/56.4/16.2-56.4 m | 0.190 m/s | 385.0/0.0 | <100 m |

## Controls and interpretation

- Seeds: [223617062, 223617063, 223617064, 223617065, 223617066, 223617067, 223617068, 223617069]; individual endpoint errors are retained in `results.json`.
- Dynamics: pnt-mission generator: 3 deg/s coordinated-turn command, wave/slam, and speed-scaled IMU at 7 kn [UNVERIFIED].
- Geometry: A single persistent real-TLE cohort of 7 satellites is selected once per mission from the 150-satellite merged fixture. N tiers use nested prefixes, all satellites remain above 5 deg for every denied Doppler epoch, and no tier hands over; only simultaneous distinct LOS count changes.. GDOP is the conventional instantaneous velocity-plus-common-clock geometry metric; N<4 is unobservable/infinite. This is a 150-satellite real sample, not a complete operational constellation.
- Receiver placement: fixed at 43.0 deg N, 0 deg E via a single rigid coordinate rotation applied to every generated ECEF position and IMU acceleration vector -- chosen from visibility geometry alone (Starlink's ~53 deg shell -- public knowledge, not sourced from docs/research/R4-signal-structures.md, which covers signal/frame structure, not orbital elements -- has its densest simultaneous coverage near its own inclination latitude, not at the equator where the synthetic generator's default origin sits), before any accuracy number was computed.
- Clock stress: receiver drift 0.030 m/s (0.100 ppb) and deterministic [UNVERIFIED] signed 0.35-1.05 Hz, fixed per SV and seed. These values and the noise model are [UNVERIFIED].
- Measurement stress: bounded ±0.5 Hz nominal error plus deterministic signed 12 Hz tracker outliers at about 1/17 observations [UNVERIFIED].
- The production chi-square gate is `Some(9.0)`; accepted/rejected counts come from integrity events.

## Realism verdict on synthetic 116 m / 554 m

N=8 was searched for (receiver latitude 25-60 deg and the full 48h TLE validity window, against the vessel's actual generated trajectory) and is not physically available from this real fixture for the full five-minute persistent leg; N=7 is the confirmed maximum. Real N=7 gives mean 35.8 m / p95 56.4 m with GDOP mean/p95 8.75/13.86 against the synthetic N=8 multisat result of mean 116 m / p95 554 m, GDOP ~1.8 (D57). The real result also clears the D56 500 m p50 / 750 m p95 usable denied target, so real orbital geometry does not undermine the synthetic finding at this sample size and leg length. Caveat: real GDOP p95 (13.86) is substantially WORSE than the synthetic study's ~1.8 -- this Starlink cohort's LOS directions are more correlated (single shell, near its own inclination latitude) than the synthetic 3-shell Walker grid's -- yet the real endpoint error is smaller than the synthetic 554 m. A range-domain GDOP snapshot does not evidently predict this Doppler-EKF's realized accuracy over a short aided-then-denied leg; this decoupling is reported, not explained away, and it remains one real-geometry data point, not a replacement for the synthetic controlled study. Vessel dynamics/clock/measurement noise stay synthetic.

## SupGP vs plain TLE: why the geometry check is valid on either product

SupGP is operator-supplied and materially more accurate than SGP4-on-plain-TLE (plain TLE/SGP4 position error is commonly kilometre-scale; SupGP tracks are tighter). For this study's question -- does real orbital LOS geometry (visible count, GDOP) resemble the synthetic Walker fixture's -- the two products are effectively interchangeable: at a shared epoch, the line-of-sight *directions* from a fixed receiver to a given real satellite differ negligibly between SupGP and plain-TLE propagation of the same object, because both track the same real orbit to well within the angular resolution that matters for elevation-mask visibility and GDOP. Track quality (SupGP vs plain TLE) matters far more for the *absolute* position/Doppler accuracy budget used in the real-signal acceptance/age-gate work than for this geometry question -- which is why SupGP is used as primary (accuracy-preferred, per DESIGN_BASELINE) while the 30-satellite plain-TLE supplement (SupGP does not cover them) is used solely to complete the persistent N=7 cohort -- every satellite actually used in the table above happens to have a SupGP record, so this table's real accuracy numbers are on pure operator-supplied tracks; the supplement was searched over but not needed for the realized result.

## [UNVERIFIED]

- TLE/SupGP source and currency: grok-fetched, not independently confirmed against CelesTrak; physical parse/propagation and shell inclinations are confirmed.
- Synthetic vessel truth, IMU/wave/turn model, clock drift, per-SV bias, cadence, and Doppler noise/outliers.
- Whether this 150-satellite sample is representative of full operational Starlink coverage; it is not a complete constellation snapshot.
- The receiver-latitude relocation is an exact rigid-rotation reinterpretation of the synthetic vessel's already-generated dynamics (see module docs), not a re-simulation at that latitude from first principles.
