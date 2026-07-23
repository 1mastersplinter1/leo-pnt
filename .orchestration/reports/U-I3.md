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

## U-I3.1 fix round (dispositions; completed by coordinator after worker interruption — code was worker-authored, this section + one doc lint by Fable)

Corrected four-way table (seed 1, 180 s; horizontal position RMS m / speed RMS m/s / n / measurement_updates):

| run | pos RMS | speed RMS | n | updates |
|---|---:|---:|---:|---:|
| denied DR-only (no prior, no Doppler) | 153.126 | 1.312 | 543 | 362 |
| denied prior-only (Doppler suppressed) | 116.085 | 0.277 | 543 | 362 |
| denied prior + Doppler | 91.795 | 2.186 | 724 | 543 |
| denied no-prior + Doppler | 13216.216 | 6.948 | 724 | 543 |
| aided (reference) | 0.826 | — | — | 1448 |

Attribution (in the JSON `attribution` block): the disclosed receiver prior — which is
truth-equivalent for this synthetic fixture — contributes 37.0 m of the position improvement
by itself; Doppler given the prior contributes a further 24.3 m. **Doppler assimilation
degrades the speed RMS by 1.91 m/s against the same-initialization baseline (0.277 → 2.186).**
The plausible mechanism — strong LEO position observability with weak/adverse constraint on
the small receiver horizontal velocity under the stub filter's [UNVERIFIED] noise tuning —
requires a dedicated tuning study before any velocity claim; recorded as open. The n
difference (724 vs 543) is disclosed: accepted Doppler emits additional epochs, so the
prior+Doppler row mixes fix frequency with fix quality (F4).

Dispositions: F1 fixed (four-way + disclosed prior + separate attribution, above and in JSON);
F2 fixed (degradation stated plainly, mechanism [UNVERIFIED]); F3 fixed (tests now assert
prior+Doppler position < prior-only, and record the speed direction honestly); F4 disclosed;
F5 routes column footnoted in README (routing ≠ assimilation; updates delta is the evidence);
F6 no code change (no sign bug; unchanged update path); F7 Orbcomm-reject-with-pipeline test
added. Numbers above generated from the committed code via `mission-study --seed 1
--duration 180`.
