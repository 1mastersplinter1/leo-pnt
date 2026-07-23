# Controlled multi-satellite LOS-diversity study

**SYNTHETIC CONTROLLED EXPERIMENT [UNVERIFIED]. Endpoints come from the production Executive + FilterStub against generator truth; no result is clamped or target-fitted.**

## Real result

Controlled N=8 manoeuvring result: mean 116.3 m, p95 554.8 m, range 7.8-554.8 m across 8 seeds (200 m-1 km).

N=8 does not reach the 100-200 m class under proper controls (p95 554.8 m). The finite GDOP (1.93) shows distinct instantaneous geometry, but clock/per-SV bias observability, manoeuvre dynamics, cadence, and the 5-minute leg still limit the present filter. D51's single-satellite limitation is therefore not closed.

| geometry | N | fixed SVs | GDOP mean/p95 | endpoint position mean/p95/spread | velocity mean | accepted/rejected mean | class |
|---|---:|---|---:|---:|---:|---:|---|
| fixed single LOS; no handover | 1 | [70049] | unobservable/infinite/unobservable/infinite | 118.3/222.3/10.8-222.3 m | 0.493 m/s | 22.0/0.0 | 200 m-1 km |
| fixed simultaneous multi-LOS cohort; no handover | 2 | [70049, 70219] | unobservable/infinite/unobservable/infinite | 109.5/287.4/16.8-287.4 m | 0.529 m/s | 44.0/0.0 | 200 m-1 km |
| fixed simultaneous multi-LOS cohort; no handover | 4 | [70049, 70219, 70346, 70366] | 9.31/30.82 | 220.8/523.0/12.4-523.0 m | 2.045 m/s | 81.0/7.0 | 200 m-1 km |
| fixed simultaneous multi-LOS cohort; no handover | 8 | [70049, 70219, 70346, 70366, 70367, 70386, 70516, 70536] | 1.79/1.93 | 116.3/554.8/7.8-554.8 m | 1.144 m/s | 144.4/31.6 | 200 m-1 km |

## Controls and interpretation

- Seeds: [223617062, 223617063, 223617064, 223617065, 223617066, 223617067, 223617068, 223617069]; individual endpoint errors are retained in `results.json`.
- Dynamics: pnt-mission generator: 3 deg/s coordinated-turn command, wave/slam, and speed-scaled IMU at 7 kn [UNVERIFIED].
- Geometry: A single persistent satellite cohort is selected once per mission. N tiers use nested prefixes, all satellites remain above 5 deg for every denied Doppler epoch, and no tier hands over; only simultaneous distinct LOS count changes. GDOP is the conventional instantaneous velocity-plus-common-clock geometry metric; N<4 is reported as unobservable/infinite. Per-SV nuisance biases make actual observability weaker than GDOP alone.
- Clock stress: receiver drift 0.030 m/s (0.100 ppb) and deterministic [UNVERIFIED] signed 0.35-1.05 Hz, fixed per SV and seed. These values and the noise model are [UNVERIFIED].
- Measurement stress: bounded ±0.5 Hz nominal error plus deterministic signed 12 Hz tracker outliers at about 1/17 observations [UNVERIFIED].
- The production chi-square gate is `Some(9.0)` and accepted/rejected counts are measured from integrity events.
- Duration limitation: a 15-minute trial found zero SVs continuously above 5°; the five-minute denied leg is the tested interval that retained an eight-SV no-handover cohort. It covers the generator turn across GNSS loss but is not endurance evidence.

## D51 reconciliation

D51 used a fixed single ISS, a much longer 100 km leg, and 30-minute Doppler cadence. This experiment changes none of its findings. It answers the narrower D54 question by holding the mission, cadence, persistent SV identities, clock errors, noise distribution, filter, gate, and outlier process fixed while changing only the number of simultaneous LOS directions. The old U-MS1 constant-velocity/zero-clock/handover headline was confounded and is withdrawn.

The 960-orbit shell grid, selection rule, clock values, transmit biases, cadence, manoeuvre/wave parameters, and measurement errors remain synthetic [UNVERIFIED]. Dated OMM/SupGP and captured residual replay are still required.
