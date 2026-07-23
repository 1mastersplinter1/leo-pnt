# U-AU1 report — D59 authority reconciliation

## Reconciliations

1. Denied acceptance and G2p now use the D56/D58 profile: horizontal position
   `<= 500 m p50` and `<= 750 m p95` over a `>= 100 km`
   constant-heading-dominated passage, with a proposed denied per-epoch PL of 250 m
   (`500 / k=2`).
2. G2e `t_eph_s` is proposed at 108000 s (30 h), the graduated-aging hard ceiling.
   The former 6 h authority-cliff interpretation is superseded per D59.
3. `revoke_threshold` is proposed at 250 m, matching the denied PL and replacing the
   superseded 100 m scalar that made the D56 availability relaxation inert.

## Three-way age disambiguation

- **G2p accuracy governor:** ephemeris-age-derived measurement-noise inflation continuously
  widens the protection limit and revokes on accuracy when the active profile is exceeded.
- **G2e freshness backstop:** `t_eph = 30 h`; ephemerides beyond the ceiling are too ancient
  for authority.
- **Inflation fresh-window:** `t_fresh = 6 h`; weighting is nominal through this boundary
  and inflated afterward. This is not an authority timeout.

## [UNVERIFIED] / PROPOSED-NOT-FROZEN

- D56 acceptance percentile, confidence, and segment-selection definitions.
- Coverage factor `k=2` and the aided/denied per-epoch protection-limit mappings, including
  the 250 m denied G2p limit.
- Real-SupGP orbit-error to range-rate inflation model and real-signal behavior through 30 h.
- `t_eph_s = 108000` boundary behavior and support from the intended caching cadence.
- `revoke_threshold = 250 m`, strict boundary semantics, and real-signal coverage.
- The retained 60/75 m caution band, its useful helm lead time, and all related hazard and
  human-factors evidence.
- All other authority-contract parameters and open safety-case register items remain at
  their existing `[UNVERIFIED]` status.

The reconciliations make D56 and D45 self-consistent, but the fail-closed gate still blocks
steering authority pending real-signal validation and a signed freeze of every
authority-contract parameter.

## Validation performed

- Searched owned design documents for stale active 6 h G2e, 100 m revoke, and 200 m denied
  references; remaining occurrences are explicitly historical/superseded or describe the
  distinct 6 h inflation fresh-window.
- Updated the machine-readable TOML proposal to `t_eph_s = 108000.0` and
  `revoke_threshold = 250.0`.
- `git diff --check` passes.
