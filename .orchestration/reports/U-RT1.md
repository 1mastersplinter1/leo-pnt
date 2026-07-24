# U-RT1 report — real-constellation geometry realism study

**Superseded by U-RT1.1 below.** The original 40-element run in this section is kept for the
historical record, with one correction: the ~53°/~87.9°/~86.4° reference inclinations quoted
against the parsed elements are **public knowledge** (operator/regulatory-filing shell
parameters), not sourced from `docs/research/R4-signal-structures.md` — that document covers
Ku-band downlink signal/frame structure, not orbital elements. The original text below
mis-attributed the cross-check to R4; this is the correction (review finding F1). See the
U-RT1.1 section for the current, adequately-resourced result.

## Fixture validation first (U-RT1, original 40-element run — superseded)

The grok-fetched fixture was validated before study implementation. All **40/40** records parse
through the `sgp4` crate, construct SGP4 constants, and propagate to finite epoch states:

- Starlink: **20/20**, inclination **53.0371–53.1608°** (public-knowledge ~53° shell, not R4 —
  see correction above).
- OneWeb: **10/10**, inclination **87.8496–87.9078°** (public-knowledge ~87.9° shell).
- Iridium NEXT: **10/10**, inclination **86.3927–86.3941°** (public-knowledge ~86.4° shell).

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

## Gates (U-RT1, original run)

- Real-TLE parse/propagation/inclination validation test.
- Deterministic real-pipeline simulation test.
- Production-gate-on test.
- Fixed visibility and nuisance-state isolation test.
- `cargo fmt`, `cargo test --workspace`, and workspace clippy.

---

# U-RT1.1 follow-up — adequate real data + review fixes

Disposition on the three review findings from U-RT1:

- **F1 (R4 mis-attribution, fixed):** corrected above and in code comments/STUDY.md — the
  Starlink ~53° shell inclination is public knowledge, not sourced from
  `docs/research/R4-signal-structures.md`.
- **F2 (no cohort-size lock, fixed):** `realtle::tests::fixture_size_and_n7_cohort_are_locked`
  now asserts the merged fixture's satellite count (150 = 120 SupGP + 30 plain-TLE supplement)
  and the persistent cohort size for **every** default seed, and additionally locks in the
  honest N=8-unavailable finding (asserts N=8 stays unreachable) so this can't silently drift.
- **F3 (hardcoded `SATELLITE_IDS`, fixed):** removed; `fixture_satellite_ids` now derives every
  satellite ID at runtime by parsing NORAD catalog numbers out of the fixture text, exactly as
  the multisat pattern derives its own synthetic IDs.

## Fixture switch

Primary fixture is now `starlink-supgp-120-2026-204.tle` (120 real, operator-supplied Starlink
SupGP records — accuracy-preferred per DESIGN_BASELINE), supplemented **only** by the 30
satellites in `starlink-150-2026-205.tle` that SupGP does not cover (all 120 SupGP catalog
numbers are a subset of the 150-satellite plain-TLE fixture). Every satellite that has a SupGP
record uses that record; the plain-TLE data is used solely for the 30 it does not cover, needed
to reach the largest real persistent cohort this sample supports. Both are
[UNVERIFIED: grok-fetched, not independently confirmed against CelesTrak].

## N=8 was searched for and is confirmed NOT reachable — honest result

The task assumption ("~120 Starlink from ~56N should give >=8 simultaneously visible") does not
hold once the check is done properly. A broad search — receiver latitude 25–60°N, the entire
48-hour TLE validity window, evaluated against the vessel's actual generated trajectory (not an
idealised fixed point) — finds a maximum **persistent** (all satellites above 5° for the entire
300 s no-handover leg, sampled every 30 s) cohort of **7**, not 8, and this is stable at 7 across
all 8 default seeds (each has a slightly different vessel path from seed-dependent wave/turn
noise). The receiver is placed at 43°N, 0°E — chosen from visibility geometry alone, before any
accuracy number was computed — via one fixed rigid rotation applied uniformly to every generated
ECEF position and IMU acceleration vector (local NED velocity/heading are already
position-independent and untouched); this is an exact relocation of the already-generated
scenario, not a re-simulation, and is disclosed in the module docs and STUDY.md.

Per the honesty mandate, this is reported plainly rather than forced: `RealTleConfig::default`
sweeps N=1/2/4/7, not 1/2/4/8, and the study's headline/diagnosis/verdict text all state the
N=8 search and its negative result explicitly.

## Real result (N=1/2/4/7, REAL Starlink geometry, production gate on, 8 seeds)

| N | fixed SVs | GDOP mean/p95 | endpoint position mean/p95 | class |
|---:|---|---:|---:|---|
| 1 | [44741] | unobservable/infinite | 52.5 / 88.2 m | <100 m |
| 2 | [44741, 45366] | unobservable/infinite | 51.9 / 92.4 m | <100 m |
| 4 | [44741, 45366, 45368, 45377] | 46.66 / 199.60 | 39.4 / 58.6 m | <100 m |
| 7 | [44741, 45366, 45368, 45377, 45387, 45405, 45580] | 8.75 / 13.86 | 35.8 / 56.4 m | <100 m |

Every satellite actually used above happens to have a SupGP record — the plain-TLE supplement
was searched over (it is what lets the cohort search reach N=7 at all in the idealised sense) but
none of it ended up in the realized N=1/2/4/7 cohorts, so this table's real accuracy numbers are
on pure operator-supplied SupGP tracks. `results.json`'s
`fixture.realized_cohort_is_pure_supgp` field makes this machine-checkable so it can't silently
go stale on a fixture change.

N=7 reaches the D56 usable denied target (p50 ≤500 m / p95 ≤750 m) with wide margin, and is
numerically better than the synthetic multisat N=8 result (D57: mean 116 m / p95 554 m).
**This is not reported as "real beats synthetic"** — GDOP p95 (13.86) is nearly 8× worse than
the synthetic study's ~1.8 (this single-shell, near-inclination-latitude Starlink cohort's LOS
directions are more correlated than the synthetic 3-shell Walker grid's), yet the real endpoint
error is smaller. A range-domain GDOP snapshot does not evidently predict this Doppler-EKF's
realized accuracy over a short aided-then-denied leg; that decoupling is reported as an
[UNVERIFIED] open observation, not resolved or explained away, and N=7/one real-geometry sample
is not a substitute for the synthetic controlled N=8 study. Full per-seed numbers, controls, and
this caveat are in `docs/studies/realtle/results.json` / `STUDY.md`.

## TLE vs SupGP note (STUDY.md)

Added to STUDY.md: SupGP is operator-supplied and materially more accurate than SGP4-on-plain-TLE
(plain TLE/SGP4 position error is commonly kilometre-scale; SupGP tracks are tighter), but for
this study's GEOMETRY question (visible count, GDOP) the two products are effectively
interchangeable — line-of-sight *directions* from a fixed receiver to a given real satellite
differ negligibly between SupGP and plain-TLE propagation of the same object at a shared epoch.
Track quality matters far more for the *absolute* accuracy/age-gate budget than for this geometry
question, which is why SupGP is primary and the plain-TLE supplement is used only to complete the
persistent cohort.

## [UNVERIFIED] (U-RT1.1)

- TLE/SupGP source and currency: grok-fetched, not independently confirmed against CelesTrak.
- Synthetic vessel truth, IMU/wave/turn dynamics, receiver clock drift, per-SV bias, cadence,
  Doppler noise/outlier model.
- Whether the 150-satellite sample is representative of full operational Starlink coverage.
- The GDOP-vs-realized-accuracy decoupling noted above.

## Gates (U-RT1.1)

- All U-RT1 gate tests retained and updated for the new fixture/derived IDs (no hardcoded
  satellite list remains).
- New: `fixture_size_and_n7_cohort_are_locked` — locks fixture size, N=7 cohort availability
  across all 8 default seeds, and the honest N=8-unavailability finding.
- `cargo test --workspace`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all -- --check` all pass. Committed on `unit/U-RT1`, not merged to main, no
  attribution trailers.
