# U-I3 report — Doppler assimilation in replay

## Result

Implemented optional replay Doppler configuration using immutable TLE records, explicit
elevation-mask and innovation-gate choices, and an optional caller-owned receiver prior.
Every Executive receives a fresh ephemeris store and pipeline. Both paired modes still
consume clones of one already-loaded measurement vector. The prior is never derived from
measurement or truth journals and its initialization updates are excluded from the reported
journal `measurement_updates` count.

The mission study now emits aided, denied DR-only, and denied-with-Doppler summaries from
the same journal. This is a synthetic integration demonstration, not a performance claim.

## Seeded evidence

Default mission (`seed=1`, `duration_s=180`):

| run | Doppler fusion routes | measurement updates | position RMS (m) | speed RMS (m/s) |
|---|---:|---:|---:|---:|
| aided | 181 | 1448 | 0.8261608336425085 | 0.3491087994720640 |
| denied DR-only | 181 | 362 | 153.1263153931950 | 1.3118538039291545 |
| denied with Doppler | 181 | 543 | 91.79453277363858 | 2.186010970891528 |

Denied-with-Doppler accepted 181 additional journal-driven updates and improved synthetic
position RMS relative to DR-only. Speed RMS did not improve. These numbers demonstrate
wiring and a qualitative position result only; they do not establish real-signal accuracy.

## D35 carried items

- Closed comparison pairing accounting: schema v2 separately reports
  `excluded_no_paired_epoch` and `excluded_no_near_truth`.
- Retained direct aided-minus-withheld sign assertions and hand-derived statistics tests.
- Retained exact input-count identity assertions across authority modes.
- Extended bit-exact replay assertions to Production and denied-with-Doppler.

## Schema change

`ReplayReport.schema_version` is now `2`. `RunSummary` adds
`doppler_fusion_routes`; `ComparisonSummary` adds both exclusion counts. The README
documents these fields and configured replay semantics.

## Verification

- Focused: `cargo test -p pnt-replay -p pnt-mission` — PASS.
- `cargo test` — PASS.
- `cargo clippy --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.

## [UNVERIFIED]

- Real RF/capture behavior and operational performance.
- Surveyed antenna lever arm (the Executive still uses its documented zero lever-arm hook).
- Whether the synthetic position improvement generalizes to other geometries, missions,
  noise models, gate settings, or receiver priors.
