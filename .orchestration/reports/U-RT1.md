# U-RT1 report — real-constellation geometry realism study

## Fixture validation first

The grok-fetched fixture was validated before study implementation. All **40/40** records parse
through the `sgp4` crate, construct SGP4 constants, and propagate to finite epoch states:

- Starlink: **20/20**, inclination **53.0371–53.1608°** versus the R4 ~53° shell.
- OneWeb: **10/10**, inclination **87.8496–87.9078°** versus the R4 ~87.9° shell.
- Iridium NEXT: **10/10**, inclination **86.3927–86.3941°** versus the R4 ~86.4° shell.

This confirms physical usability, not provenance or currency. The elements remain real published
elements that were grok-fetched and **not independently confirmed against CelesTrak**.

## Controlled real-TLE result

The production Executive + real error-state EKF was run against generator truth with the
production chi-square gate `Some(9.0)`, eight deterministic seeds, a fixed no-handover cohort,
receiver clock drift, deterministic per-SV transmit bias, tracker noise/outliers, and the same
five-minute manoeuvring denied leg as the corrected multisat control.

The 40-element fixture is too sparse for the intended N=8 replication. The best scanned window
retains only **two** satellites above the 5° mask for the whole denied leg:

| Real geometry | mean endpoint | p95 endpoint | GDOP | accepted/rejected mean |
|---|---:|---:|---:|---:|
| N=1, fixed Iridium 41917 | 79.6 m | 144.9 m | infinite/unobservable | 55/0 |
| N=2, + Starlink 44723 | 62.7 m | 118.8 m | infinite/unobservable | 110/0 |

The nuisance-state count is exactly N, demonstrating that the real Doppler observations reached
the estimator. Results are deterministic.

## Verdict on synthetic 116 m / 554 m

The real fixture **cannot validate or falsify** the synthetic N=8 mean 116 m / p95 554 m result.
N=1/N=2 have no finite position-plus-clock GDOP and largely reflect short-leg inertial
propagation aided by underdetermined Doppler. Calling their numerically smaller endpoint errors
“better than 116/554” would be dishonest.

The material real-vs-synthetic difference is coverage: the synthetic 960-SV Walker fixture
supplies a persistent N=8 cohort with GDOP about 1.8; this 40-SV real-element sample supplies at
most N=2 and no finite GDOP. A complete dated constellation snapshot is required for the requested
real N=8 geometry check. The 116/554 result therefore remains a synthetic controlled result, not a
real-constellation-validated headline.

## [UNVERIFIED]

- TLE source/currency and representativeness versus current CelesTrak operational catalogs.
- Synthetic vessel truth, IMU/wave/turn dynamics, receiver clock drift, per-SV bias, cadence,
  Doppler noise, and outlier model.
- Exact operational visibility and GDOP of the full Starlink/OneWeb/Iridium constellations.

## Gates

- Real-TLE parse/propagation/inclination validation test.
- Deterministic real-pipeline simulation test.
- Production-gate-on test.
- Fixed visibility and nuisance-state isolation test.
- `cargo fmt`, `cargo test --workspace`, and workspace clippy.
