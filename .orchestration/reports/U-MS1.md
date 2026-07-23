# U-MS1.1 controlled multi-satellite fix report

The original U-MS1 headline is withdrawn. Its straight-line constant-velocity truth, zero
receiver/transmitter clock terms, N=1 handovers, and single seed confounded dead reckoning,
temporal LOS diversity, and simultaneous multi-LOS geometry.

## Controlled result

The rebuilt experiment uses the real `pnt-mission` generator at 7 kn with its 3 deg/s
coordinated-turn command, wave/slam, and speed-scaled IMU model. GNSS is removed at 300 s
during the generated turn. A fixed nested cohort supplies N=1/2/4/8 simultaneous LOS
observations; every selected satellite remains above the physical 5° mask for the entire
denied leg and no tier hands over. Thus within each seed the only tier variable is the
number of simultaneous distinct LOS directions.

Eight deterministic seeds inject an unknown 0.030 m/s receiver clock drift (0.100 ppb),
fixed signed 0.35–1.05 Hz per-SV transmit biases, bounded measurement error, and seeded
tracker outliers. All values are synthetic `[UNVERIFIED]`. The production chi-square gate
is `Some(9.0)` and produces measured rejections in the N=4 and N=8 tiers.

The real controlled N=8 endpoint distribution is:

- mean: 116.3 m
- p95/spread: 554.8 m / 7.8–554.8 m
- velocity-error mean: 1.144 m/s
- GDOP mean/p95: 1.79 / 1.93
- accepted/rejected updates per seed (mean): 144.4 / 31.6

Therefore controlled N=8 does **not** establish the 100–200 m class at p95. Good finite
GDOP confirms useful instantaneous LOS diversity, but the broad seed spread shows that the
current clock/per-SV nuisance observability, manoeuvre dynamics, cadence, and tracker-outlier
response remain limiting. This is the real result; no parameter was fitted to an accuracy
target.

The fixed N=1 control is also not an endurance reproduction of D51: this leg is five minutes
at 30 s Doppler cadence. It has mean/p95 118.3/222.3 m and unobservable instantaneous
velocity-plus-clock GDOP. A 15-minute pre-run found no satellite continuously above 5°, so
the five-minute duration was used to retain a genuine eight-SV no-handover cohort. This
duration limitation is explicit and prevents an endurance claim.

## D51 reconciliation and artifacts

D51 remains the evidence for a fixed-single-ISS, 100 km, 30-minute-cadence manoeuvring
fixture with tens-of-kilometres error. U-MS1.1 isolates the narrower D54 geometry question
over a short persistent-visibility interval; it does not close D51.

Machine-readable per-seed endpoints, GDOP, update counts, injected controls, and fixed SV
identities are in `docs/studies/multisat/results.json`; the generated narrative is in
`docs/studies/multisat/STUDY.md`. The orbit grid, clock/bias/error models, mission dynamics,
cadence, and selection rule remain `[UNVERIFIED]`; dated OMM/SupGP and captured residual
replay are still required.
