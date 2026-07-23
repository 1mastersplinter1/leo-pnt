# pnt-replay

Deterministic offline replay of a `pnt-journal` run. `replay_paired` reads the measurement
stream once, then feeds clones of that exact ordered vector to fresh executives in
`production` and `recorded_only` modes. Truth is used only for scoring, never as an
estimator-to-estimator reference.

The JSON report has `schema_version` (`1`), `run_uuid`, `config_hash`,
`max_truth_offset_ns`, and `input_measurement_count` at its root. `aided` and `withheld`
each contain the authority `mode`, routing/update counts, matched/excluded epoch counts,
and `horizontal_position_error_m` / `horizontal_speed_error_mps` statistics. Each
statistics object contains `n`, `mean`, `rms`, `p50`, `p95`, and `max`. `comparison`
contains aided minus withheld values for the two error statistics; negative deltas mean
the aided run was better. Percentiles use linear interpolation at index
`p * (n - 1)` in a sorted sample. An empty sample has `n: 0` and null numeric fields.

Truth matching selects the smallest absolute monotonic-time difference at or below the
caller-provided maximum; ties select the earlier truth record. Horizontal position error
is the norm of the ECEF error after rotation to local ENU at truth. Horizontal speed error
is the norm of the estimate/truth north-east velocity difference.
