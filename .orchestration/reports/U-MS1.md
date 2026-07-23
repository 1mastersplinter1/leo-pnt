# U-MS1 execution report

Implemented `pnt-studies::multisat` and the `multisat-study` binary. The study runs the
production `Executive<FilterStub>` with chi-square threshold `Some(9.0)`, per-SV nuisance
augmentation, a physical 5° elevation mask, and real SGP4/predictor/executive/filter updates.
No endpoint is clamped or computed from a closed-form accuracy law.

## Honest headline

The synthetic real-filter study reached the 100–200 m class or better in every tested case.
The strongest simultaneous-diversity result was N=8: 6.1 m after the 60-minute denied leg
and 11.4 m after the 100 km passage. N=1 also ended below 200 m (33.8 m and 2.4 m), but this
is not the D51 fixed-single-ISS case: it uses one currently visible SV per epoch and satellite
handovers provide time-varying LOS diversity. The growing nuisance-state counts expose those
handoffs. Endpoint behavior was non-monotonic for N=1/2/4, so this single deterministic
synthetic seed is capability evidence, not a performance distribution.

All cases had genuine gate rejections. Visibility was 45–54 satellites over time; only the
requested N=1/2/4/8 highest-elevation visible satellites were used at each epoch.

## Artifacts and caveats

Results and convergence curves are in `docs/studies/multisat/results.json` and `STUDY.md`.
The 960 synthetic records use Starlink-class 53°/550 km, OneWeb-class 87.9°/1200 km, and
Iridium-class 86.4°/780 km shells. Synthetic RAAN/anomaly grids, epoch, near-circular
eccentricity, receiver track, measurement/IMU errors, 10 s integration decimation, and 30 s
Doppler cadence are `[UNVERIFIED]`.

Routed next step: replay dated real multi-constellation OMM/SupGP records and captured tracker
residuals, then run multiple seeds before treating the measured class as an accuracy claim.
