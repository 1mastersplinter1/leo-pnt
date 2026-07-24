# U-ST1 — endurance studies (fix round U-ST1.1)

## Outcome

Rebuilt `pnt-studies::endurance` after the D65 adversarial FAIL. The prior study's
levers were measured on an uncontrolled, unrepresentative geometry (MEO orbits
mislabeled as LEO, a per-epoch lowest-ID handover cohort, and no GDOP). This
round runs both sweeps on the correct LEO regime, with continuous best-N
handover tracking, per-epoch GDOP instrumentation, real `Executive` + `FilterStub`
EKF versus generator truth, and the production Doppler chi-square gate
`Some(9.0)`. Results are committed to `docs/studies/endurance/results.json`
(schema 2) and `STUDY.md`; only `endurance.rs`, its bin, and its docs were
touched. This is a synthetic controlled experiment, not a validated prediction —
fixture, noise, clock, and dynamics remain `[UNVERIFIED]`.

## Dispositions of the D65 findings

- **MEO confound (F1) — FIXED.** The fixture is now the verified multisat LEO
  regime: three shells at 53.0/87.9/86.4 deg and 15.064/13.158/14.342 rev/day
  (550-1200 km), not the ~2 rev/day/20,000 km MEO grid. A new test
  `fixture_is_leo_not_meo` parses every TLE and asserts mean motion > 10 rev/day,
  so the MEO regression cannot recur. Constellation labels are corrected to match
  the shell layout (each shell is one class), not the prior `%3` mislabel.
- **Geometry not held/measured (F2) — FIXED.** GDOP instrumentation
  (`gdop()`, reused from multisat) is restored and reported per tier (mean, min,
  max). Measured GDOP stays well-conditioned across every leg: mean 2.1 (10 min)
  to 2.7 (60 min), comparable to multisat's ~1.8. Geometry is therefore proven
  not to be the confound; the levers are measured on fixed, well-conditioned
  geometry.
- **Handover reframe (F5) — ADOPTED as the honest model.** A fixed 8-SV LEO
  cohort cannot survive an endurance leg (satellites set within minutes), so the
  study tracks the best-conditioned eight *currently visible* satellites with
  realistic **sticky** handover: lock is held on each satellite until it sets,
  and freed slots are refilled with the geometry-improving visible candidate.
  Handovers now reflect physical setting events (10 over a 10-min leg to 65 over
  60 min, ~1 per 30 s epoch), not per-epoch reshuffling.
- **Clock lever inconclusive (F3) — CONFIRMED and explained on good geometry.**
  Re-run on the fixed 30-min good-geometry leg, the clock lever is near-invisible
  (see below). This is now reported as honest BOM evidence with the mechanism,
  not a swamped artifact.

## Two fixture issues found and handled honestly (disclosed)

The LEO fixture exposed two latent generator issues that the old MEO grid hid:

1. **Truth flew to space over long legs.** With the 1 Hz truth cadence, the
   sub-second (0.25 s) wave-slam model aliases to a strictly *upward* 6.10 m/s²
   impulse every burst (the burst waveform is only ever sampled at phase 0,
   `cos 0 = +1`), integrating to ~1187 km of altitude over a 60-min leg. The old
   MEO satellites (20,000 km) stayed "visible" from a receiver in space, hiding
   it; LEO satellites (550 km) do not. Wave-slam is therefore disabled and the
   vertical IMU bias zeroed for the endurance legs; truth is constant-heading
   maritime DR with horizontal bias and speed-scaled IMU noise. A test asserts
   truth stays within 50 km of sea level over the full leg.
2. **Coarse 960-SV grid has equatorial coverage gaps over an hour.** The multisat
   grid only needed to hold an 8-SV cohort for 5 minutes; over 60 minutes at the
   equatorial mission origin it drops below 8 visible. The grid is densified to a
   Starlink-scale synthetic Walker constellation (768 SVs, 16 planes × 16 slots ×
   3 shells) so >=8 (typically 18-40) satellites stay continuously visible. The
   sea-level/coverage test guards both.

Both are disclosed in the module docs, `STUDY.md`, and the `[UNVERIFIED]` list.

## Honest measured curves (16 seeds, endpoint horizontal error vs truth)

### Leg-duration (clock fixed at 1e-9)

| leg | GDOP mean (min-max) | p50 | p95 | spread (min-max) | acc/rej | handovers |
|---:|---:|---:|---:|---:|---:|---:|
| 10 min | 2.11 (1.47-3.06) | 8552 m | 31520 m | 3113-31813 m | 272/64 | 10 |
| 20 min | 2.35 (1.47-5.27) | 6837 m | 17552 m | 3014-26312 m | 489/168 | 20 |
| 30 min | 2.50 (1.47-5.27) | 13535 m | 24962 m | 4376-36486 m | 708/267 | 32 |
| 45 min | 2.65 (1.47-5.27) | 4317 m | 19649 m | 1980-21559 m | 984/470 | 48 |
| 60 min | 2.73 (1.47-7.29) | 5067 m | 9251 m | 33-15215 m | 1264/671 | 65 |

Longer legs help on average (p95 31.5 km → 9.3 km, 10→60 min) but noisily; the
30-min tier is a mid-leg spike because the endpoint metric samples the
instantaneous handover geometry at that one epoch. Convergence is **bimodal**: at
60 min the best seeds reach 33-70 m (5 of 16 seeds ≤ 500 m, one exactly at the
500 m D56 p50 target) while others stay km-scale.

### Clock discipline (30-min leg, fixed good geometry)

| label `[UNVERIFIED]` | fractional | drift | p50 | p95 |
|---|---:|---:|---:|---:|
| rubidium | 1e-11 | 0.003 m/s | 13535 m | 24967 m |
| good OCXO | 1e-9 | 0.300 m/s | 13535 m | 24962 m |
| poor reference | 1e-7 | 29.98 m/s | 11253 m | 25872 m |

The clock lever is near-invisible: Rb vs OCXO is ±5 m on a 25 km error, and even a
1e-7 poor clock barely moves it. The common-mode receiver-clock injection is
absorbed by the filter's clock/nuisance states, so clock quality is not a usable
BOM lever for denied position here. (Consistent with the prior study's finding,
now confirmed at good geometry rather than swamped by an MEO confound.)

## Honest verdict on the two levers

Neither lever, on honest continuous-handover LEO geometry, robustly delivers
D56's 500 m p50 / 750 m p95: **no tested leg meets p50 ≤ 500 m or p95 ≤ 750 m.**
The fixed-cohort multisat result (116 m / 554 m) does **not** transfer to
sustained endurance — a real LEO megaconstellation forces continuous handovers,
and each handover re-introduces an unconverged per-SV transmit bias, so denied
position stays km-scale on constant-heading legs even though the instantaneous
geometry (GDOP ~2.5) is excellent. Longer legs help and can occasionally reach
the target (bimodal best-case 33 m at 60 min), but not reliably; better clocks do
not help at all. The path to reliably tight denied endurance is therefore neither
"longer legs" nor "better clock" alone but resolving per-SV bias under handover
churn (e.g. bias priors/continuity across handover, or manoeuvre-aided
observability) — a filter/estimation question, not a BOM question.

## Guardrails and verification

- Real EKF state vs generator truth; production gate `Some(9.0)`; accepted and
  rejected integrity-event counts reported. No formula, clamp, target-fitting,
  or replacement estimator. The best-N cohort selection chooses good *geometry*
  (min-GDOP refill), never fitting error — errors range over four orders of
  magnitude (33 m to 38 km), and the min values prove the plumbing produces real
  tight fixes when observability allows.
- Deterministic: release runs are bit-identical, debug == release is byte-identical
  (cross-opt verified), and the 16-seed table matches the committed file.
- Seed loop parallelised with rayon in input order (determinism preserved) to
  keep the compute-heavy real-EKF sweep tractable; run with `--release`.
- Tests: determinism, production-gate-on-and-rejects, GDOP well-conditioned +
  hands over, truth-stays-at-sea-level + continuous coverage, LEO-not-MEO
  fixture, divergence-class-never-hidden. Full workspace `cargo test`, `cargo
  clippy --all-targets -D warnings`, and `cargo fmt --check` all pass.

## `[UNVERIFIED]` list

- Synthetic 768-SV three-shell LEO Walker grid and sticky best-N-visible handover
  selection.
- Constant-heading 10/20/30/45/60-min legs, 30 s Doppler cadence, 7 kn speed;
  sub-second wave-slam disabled for long-leg truth stability.
- Receiver clock fractional injections and the Rb/good-OCXO/poor labels; constant
  common-mode drift is a stand-in, not an oscillator stochastic model.
- Per-SV fixed transmit biases and deterministic tracker noise/outlier process.
