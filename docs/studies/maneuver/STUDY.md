# Maneuver vs constant heading: denied-leg LEO-Doppler A/B

**SYNTHETIC MANEUVER-VS-CONSTANT A/B [UNVERIFIED]. Errors are production Executive + real FilterStub EKF state versus generator truth, production chi-square gate ON. No result is clamped, formula-generated, or target-fitted.**

Resolves the tension between the bladeRF handoff's "every manoeuvre resets convergence, hold constant heading" guidance and bearings-only/Doppler observability theory's "platform maneuvers aid position observability". Controlled A/B on the production `Executive` + `FilterStub` EKF, production chi-square gate ON, on the shared three-shell LEO Walker fixture, versus generator truth. Cross-reference: D55/D57 (leg-duration confounds), D68/D72 (the filter is overconfident/inconsistent -- the km-scale denied error is estimation inconsistency, not a physics floor), D69.

## Fixture

- 960 satellites, synthetic [UNVERIFIED].
  - Starlink-class: ~550 km, 53.0 deg, 15.064 rev/day
  - OneWeb-class: ~1200 km, 87.9 deg, 13.158 rev/day
  - Iridium-class: ~780 km, 86.4 deg, 14.342 rev/day

## Controls

- Seeds: 8 ([1296125526, 1296125527, 1296125528, 1296125529, 1296125530, 1296125531, 1296125532, 1296125533]).
- Real path: production `Executive` and `FilterStub` EKF state versus truth; production chi-square gate `Some(9.0)` (accept/reject counts are measured integrity events).
- 300 s shared GNSS-aided convergence, then GNSS withheld for the denied leg.
- Doppler cadence 30 s; common-mode receiver clock 1e-9 fractional.
- Geometry: Best-8 sticky-handover satellite schedule computed once per (seed, speed, leg) from the CONSTANT trajectory and reused for the maneuver arm, so the schedule (satellite selection) is held fixed and only the Doppler-curve evolution differs between arms. Turns move a boat's ground track by under ~1 km over a denied leg, negligible against 550-1200 km slant ranges. Per-epoch GDOP reported.
- 300 s shared GNSS-aided convergence, then GNSS withheld; both arms identical except denied-leg heading. Coordinated turns alternate direction (zig-zag), 30 s each, at the yaw rate carried by pnt_mission::CoordinatedTurnConfig.

## Cell: 7.0 kn / 30 min denied leg

Constant-heading baseline RMS-over-leg p50 = 491.8 m. Paired delta = per-seed (maneuver minus constant) RMS; MATERIAL only when the paired [p05,p95] excludes zero.

| arm | GDOP mean (max) | RMS p50 | RMS p95 | endpoint p50 | paired delta mean [p05,p95] | material | p95 delta | accept/reject | err/sigma (win/steady) | horiz NEES (win/steady) |
|---|---:|---:|---:|---:|---:|:--:|---:|---:|---:|---:|
| constant-heading | 2.60 (4.08) | 491.8 m | 964.8 m | 1118.9 m | +0.0 [0.0,0.0] m | - | +0 m | 357.0/131.0 | 3.9x (n/a/3.9x) | 48.56 (n/a/48.56) |
| turn 45deg every 5 min | 2.60 (4.08) | 491.3 m | 965.5 m | 1120.1 m | +10.0 [-0.7,80.2] m | no | +1 m | 356.4/131.6 | 3.9x (4.3x/3.9x) | 48.21 (53.83/47.59) |
| turn 90deg every 5 min | 2.60 (4.08) | 491.5 m | 966.4 m | 1123.4 m | +10.3 [-0.6,80.1] m | no | +2 m | 356.4/131.6 | 3.9x (4.3x/3.9x) | 48.24 (53.87/47.62) |
| turn 45deg every 10 min | 2.60 (4.08) | 491.4 m | 965.5 m | 1119.7 m | +10.0 [-0.5,80.2] m | no | +1 m | 356.4/131.6 | 3.9x (5.0x/3.9x) | 48.21 (62.01/47.49) |
| turn 90deg every 10 min | 2.60 (4.08) | 491.6 m | 966.4 m | 1122.1 m | +10.3 [-0.2,80.2] m | no | +2 m | 356.4/131.6 | 3.9x (5.0x/3.9x) | 48.24 (62.08/47.52) |
| turn 45deg every 15 min | 2.60 (4.08) | 491.4 m | 965.6 m | 1120.3 m | +10.0 [-0.8,80.1] m | no | +1 m | 356.4/131.6 | 3.9x (5.3x/3.9x) | 48.20 (73.92/47.33) |
| turn 90deg every 15 min | 2.60 (4.08) | 491.6 m | 966.5 m | 1123.6 m | +10.2 [-0.7,80.0] m | no | +2 m | 356.4/131.6 | 3.9x (5.3x/3.9x) | 48.23 (74.00/47.35) |

## Cell: 7.0 kn / 20 min denied leg

Constant-heading baseline RMS-over-leg p50 = 266.4 m. Paired delta = per-seed (maneuver minus constant) RMS; MATERIAL only when the paired [p05,p95] excludes zero.

| arm | GDOP mean (max) | RMS p50 | RMS p95 | endpoint p50 | paired delta mean [p05,p95] | material | p95 delta | accept/reject | err/sigma (win/steady) | horiz NEES (win/steady) |
|---|---:|---:|---:|---:|---:|:--:|---:|---:|---:|---:|
| constant-heading | 2.45 (3.66) | 266.4 m | 840.1 m | 506.9 m | +0.0 [0.0,0.0] m | - | +0 m | 258.5/69.5 | 2.8x (n/a/2.8x) | 31.21 (n/a/31.21) |
| turn 90deg every 10 min | 2.45 (3.66) | 266.3 m | 840.5 m | 507.0 m | +0.1 [-0.1,0.3] m | no | +0 m | 258.5/69.5 | 2.8x (3.9x/2.8x) | 31.22 (43.69/30.58) |

## Cell: 7.0 kn / 40 min denied leg

Constant-heading baseline RMS-over-leg p50 = 1246.5 m. Paired delta = per-seed (maneuver minus constant) RMS; MATERIAL only when the paired [p05,p95] excludes zero.

| arm | GDOP mean (max) | RMS p50 | RMS p95 | endpoint p50 | paired delta mean [p05,p95] | material | p95 delta | accept/reject | err/sigma (win/steady) | horiz NEES (win/steady) |
|---|---:|---:|---:|---:|---:|:--:|---:|---:|---:|---:|
| constant-heading | 2.65 (4.32) | 1246.5 m | 2300.8 m | 1960.3 m | +0.0 [0.0,0.0] m | - | +0 m | 447.4/200.6 | 4.8x (n/a/4.8x) | 58.89 (n/a/58.89) |
| turn 90deg every 10 min | 2.65 (4.31) | 1251.0 m | 1803.7 m | 1986.9 m | -39.1 [-497.1,169.4] m | no | -497 m | 446.2/201.6 | 4.8x (5.4x/4.7x) | 57.70 (67.68/57.18) |

## Cell: 3.5 kn / 30 min denied leg

Constant-heading baseline RMS-over-leg p50 = 492.8 m. Paired delta = per-seed (maneuver minus constant) RMS; MATERIAL only when the paired [p05,p95] excludes zero.

| arm | GDOP mean (max) | RMS p50 | RMS p95 | endpoint p50 | paired delta mean [p05,p95] | material | p95 delta | accept/reject | err/sigma (win/steady) | horiz NEES (win/steady) |
|---|---:|---:|---:|---:|---:|:--:|---:|---:|---:|---:|
| constant-heading | 2.60 (4.07) | 492.8 m | 966.2 m | 1124.9 m | +0.0 [0.0,0.0] m | - | +0 m | 356.4/131.6 | 3.9x (n/a/3.9x) | 48.27 (n/a/48.27) |
| turn 90deg every 10 min | 2.60 (4.07) | 492.6 m | 967.0 m | 1126.3 m | +0.1 [-0.2,0.8] m | no | +1 m | 356.4/131.6 | 3.9x (5.0x/3.9x) | 48.28 (62.16/47.56) |

## Cell: 12.0 kn / 30 min denied leg

Constant-heading baseline RMS-over-leg p50 = 493.0 m. Paired delta = per-seed (maneuver minus constant) RMS; MATERIAL only when the paired [p05,p95] excludes zero.

| arm | GDOP mean (max) | RMS p50 | RMS p95 | endpoint p50 | paired delta mean [p05,p95] | material | p95 delta | accept/reject | err/sigma (win/steady) | horiz NEES (win/steady) |
|---|---:|---:|---:|---:|---:|:--:|---:|---:|---:|---:|
| constant-heading | 2.60 (4.09) | 493.0 m | 1535.4 m | 1355.0 m | +0.0 [0.0,0.0] m | - | +0 m | 356.0/132.0 | 3.9x (n/a/3.9x) | 48.79 (n/a/48.79) |
| turn 90deg every 10 min | 2.61 (4.08) | 492.8 m | 1537.2 m | 1352.8 m | +0.2 [-0.7,1.8] m | no | +2 m | 356.1/131.9 | 3.9x (5.2x/3.9x) | 48.84 (65.72/47.97) |

## Honest answers

- HEADLINE A/B (primary cell 7 kn / 30 min, 8 seeds, PAIRED per-seed): no maneuver schedule crossed the materiality threshold (paired [p05,p95] excluding zero). Every swept arm's paired mean is POSITIVE (maneuver hurts) -- a sign-consistent but small convergence-RESET signature, DIRECTIONALLY matching the handoff and contradicting the observability-aid hypothesis. But the magnitude is tail-driven, not central: the largest paired mean is +10.3 m (arm 'turn 90deg every 10 min') but its MEDIAN seed delta is only +0.1 m with [p05,p95] = [-0.2, 80.2] m -- the mean is pulled by a single tail seed and does NOT reproduce across the neighbouring speed/leg cells (see sweeps below), so it is not a robust effect. Against the constant-heading baseline RMS p50 492 m (p95 965 m) and the km-scale long-leg error, any maneuver effect is <=~2% and mostly unmeasurable. VERDICT: on this production EKF the maneuver observability AID predicted by bearings-only theory does NOT materialise (it does not reduce the true error), and the convergence RESET the handoff warns of is at most a small, tail-driven inflation -- because both are swamped by the filter's own inconsistency (D68/D72). The handoff's OPERATIONAL bottom line (hold constant heading) is upheld; its stated MECHANISM ('every manoeuvre resets convergence') is directionally visible only as a weak, non-dominant signature.

- Turn-frequency x magnitude structure (paired per-seed RMS mean [p05,p95], + = hurt): [turn 45deg every 5 min] mean +10.0 (median +0.0) [-0.7,80.2]; [turn 90deg every 5 min] mean +10.3 (median +0.1) [-0.6,80.1]; [turn 45deg every 10 min] mean +10.0 (median +0.0) [-0.5,80.2]; [turn 90deg every 10 min] mean +10.3 (median +0.1) [-0.2,80.2]; [turn 45deg every 15 min] mean +10.0 (median +0.0) [-0.8,80.1]; [turn 90deg every 15 min] mean +10.2 (median +0.1) [-0.7,80.0]; No arm crosses the materiality threshold: no usable turn-frequency/magnitude lever on the TRUE error -- the swept schedules are all within seed noise of constant heading.

- Worst-case tail (RMS p95 delta vs constant): 7.0kn/30min 'turn 45deg every 5 min' p95 delta +1 m; 7.0kn/30min 'turn 90deg every 5 min' p95 delta +2 m; 7.0kn/30min 'turn 45deg every 10 min' p95 delta +1 m; 7.0kn/30min 'turn 90deg every 10 min' p95 delta +2 m; 7.0kn/30min 'turn 45deg every 15 min' p95 delta +1 m; 7.0kn/30min 'turn 90deg every 15 min' p95 delta +2 m; 7.0kn/20min 'turn 90deg every 10 min' p95 delta +0 m; 7.0kn/40min 'turn 90deg every 10 min' p95 delta -497 m; 3.5kn/30min 'turn 90deg every 10 min' p95 delta +1 m; 12.0kn/30min 'turn 90deg every 10 min' p95 delta +2 m. Any large negative here (e.g. the long-leg arms) is a tail-trimming effect worth noting, but with 8 seeds treat p95 deltas as indicative, not established.

- Geometry control: the shared best-8 schedule stays well-conditioned (constant-arm GDOP mean 2.60, max 4.08), and the maneuver arms reuse the same schedule, so the A/B isolates the Doppler-curve/dynamics effect from satellite selection.

- Covariance consistency around maneuvers (D68/D72, representative 90 deg/10 min arm): whole-leg error/sigma ratio 3.9x (constant arm 3.9x) -- the filter stays OVERCONFIDENT in both arms, consistent with the endurance/consistency finding that the km-scale denied error is estimator inconsistency, not a physics floor. Maneuver-window ratio 4.3x vs steady 3.9x: the filter's overconfidence is essentially unchanged by the turn (maneuver-window and steady error/sigma ratios are comparable). 2-dof horizontal NEES (expected 2): maneuver-window 53.87 vs steady 47.62.

- Leg-length sweep (maneuver 90 deg/10 min vs constant, PAIRED per-seed RMS delta): 20.0 min: baseline 266 m, paired delta +0.1 m [-0.1,0.3]; 30.0 min: baseline 492 m, paired delta +10.3 m [-0.2,80.2]; 40.0 min: baseline 1247 m, paired delta -39.1 m [-497.1,169.4]; 

- Speed sweep (maneuver 90 deg/10 min vs constant, PAIRED per-seed RMS delta): 3.5 kn: baseline 493 m, paired delta +0.1 m [-0.2,0.8]; 7.0 kn: baseline 492 m, paired delta +10.3 m [-0.2,80.2]; 12.0 kn: baseline 493 m, paired delta +0.2 m [-0.7,1.8]; 

- OPERATIONAL RECOMMENDATION: HOLD CONSTANT HEADING -- but for a corrected reason. No tested turn schedule changed the TRUE denied-leg position error by a robustly material margin (no paired [p05,p95] excludes zero; the largest paired mean, ~+10 m, is tail-seed-driven and does not reproduce across neighbouring speed/leg cells). Maneuvering to AID observability buys NOTHING measurable here -- the observability-aid hypothesis is not realised on this filter -- while constant heading loses nothing. The direction of what little effect exists is consistently HURTING (a weak convergence-reset signature), so the handoff's operational bottom line (build the campaign around constant-heading legs) holds. But its stated MECHANISM only appears as a weak, non-dominant signature: the denied-leg error is dominated by ESTIMATOR INCONSISTENCY (D68/D72 overconfidence, here ~4-5x error/sigma and ~25-30x the 2-dof NEES expectation of 2), which is far larger than any maneuver-induced observability change. Maneuvering also measurably worsens covariance CONSISTENCY in the ~2 min after each turn (error/sigma and NEES rise in the maneuver window, growing with inter-turn interval) -- a second reason not to maneuver gratuitously. ACTIONABLE: the lever that matters is fixing filter consistency in the estimator (bias continuity/retirement across handover, covariance-consistency correction, Q retuning); only on a consistent filter is it worth re-running this A/B to see whether the maneuver observability aid then becomes exploitable.

## [UNVERIFIED] inputs

- Synthetic 960-satellite three-shell LEO Walker grid, reused unchanged from the multi-satellite/endurance studies; sticky best-N-visible handover.
- Synthetic maritime constant-speed dead-reckoning truth with horizontal IMU bias/noise; coordinated turns are the only dynamics difference between arms.
- Per-SV fixed transmit biases, deterministic measurement noise/outlier process, and a 1e-9 (good-OCXO label) common-mode receiver clock drift.
- Turn magnitude is applied over a fixed 30 s coordinated turn; the periodic schedule is study-side while the per-turn rate reuses pnt-mission's CoordinatedTurnConfig semantics.
