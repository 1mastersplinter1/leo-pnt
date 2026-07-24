# Real-TLE constellation geometry realism check

**REAL-PUBLISHED-TLE GEOMETRY CHECK [UNVERIFIED currency/provenance]. Endpoints come from the production Executive + FilterStub against synthetic generator truth; no result is clamped or target-fitted. The elements were grok-fetched and were not independently confirmed against CelesTrak.**

## Real result

The 40-element real fixture supports only two persistent LOS over the controlled five-minute leg; an N=8 rerun is not physically available from this sparse sample.

N=8 was not run, so no multi-satellite accuracy conclusion is available.

| geometry | N | fixed SVs | GDOP mean/p95 | endpoint position mean/p95/spread | velocity mean | accepted/rejected mean | class |
|---|---:|---|---:|---:|---:|---:|---|
| fixed single LOS; no handover | 1 | [41917] | unobservable/infinite/unobservable/infinite | 79.6/144.9/30.8-144.9 m | 0.364 m/s | 55.0/0.0 | 100-200 m |
| fixed simultaneous multi-LOS cohort; no handover | 2 | [41917, 44723] | unobservable/infinite/unobservable/infinite | 62.7/118.8/29.6-118.8 m | 0.325 m/s | 110.0/0.0 | 100-200 m |

## Controls and interpretation

- Seeds: [223617062, 223617063, 223617064, 223617065, 223617066, 223617067, 223617068, 223617069]; individual endpoint errors are retained in `results.json`.
- Dynamics: pnt-mission generator: 3 deg/s coordinated-turn command, wave/slam, and speed-scaled IMU at 7 kn [UNVERIFIED].
- Geometry: A single persistent real-TLE cohort is selected once per mission. N tiers use nested prefixes, all satellites remain above 5 deg for every denied Doppler epoch, and no tier hands over; only simultaneous distinct LOS count changes. The sparse 40-element sample supports N=1 and N=2 only. GDOP is the conventional instantaneous velocity-plus-common-clock geometry metric; N<4 is unobservable/infinite. This is a 40-SV sample, not complete operational constellations.
- Clock stress: receiver drift 0.030 m/s (0.100 ppb) and deterministic [UNVERIFIED] signed 0.35-1.05 Hz, fixed per SV and seed. These values and the noise model are [UNVERIFIED].
- Measurement stress: bounded ±0.5 Hz nominal error plus deterministic signed 12 Hz tracker outliers at about 1/17 observations [UNVERIFIED].
- The production chi-square gate is `Some(9.0)`; accepted/rejected counts come from integrity events.

## Realism verdict on synthetic 116 m / 554 m

The real-element run cannot validate or falsify the synthetic N=8 result: only two of the 40 sampled SVs remain simultaneously above 5° for the controlled five-minute no-handover leg. N=1 gives 79.6/144.9 m mean/p95 and N=2 gives 62.7/118.8 m, but both have infinite GDOP for a position-plus-clock solution and largely measure short-leg inertial propagation aided by underdetermined Doppler. Treating those smaller errors as “better than 116/554” would be dishonest. The material difference is coverage: the synthetic 960-SV Walker grid provides N=8 and GDOP about 1.8, while this sparse real subset does not. A complete dated constellation snapshot is required for a genuine real-vs-synthetic N=8 check.

## [UNVERIFIED]

- TLE source and currency: grok-fetched, not independently confirmed against CelesTrak; physical parse/propagation and shell inclinations are confirmed.
- Synthetic vessel truth, IMU/wave/turn model, clock drift, per-SV bias, cadence, and Doppler noise/outliers.
- Whether this 40-SV sample is representative of operational constellation coverage; it plainly is not a complete constellation snapshot.
