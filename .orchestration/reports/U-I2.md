# U-I2 report — executive integration and contracts v4

Built on branch `unit/U-I2` against contracts v3; authored v4.

## Delivered

- Appended contracts v4 with one geometric range-rate convention, estimator-owned clock
  terms, authority-only `ArmCommand` routing, the default-false `oneweb_enabled` key, and
  the exact U-M1 MAVLink bridge NDJSON field shape.
- Connected tracker Doppler ingress to local ephemeris lookup/age gating, SGP4 ECEF state,
  pure-geometric prediction from the current filter receiver state, estimator NIS gating,
  EKF update, decision journalling, solution construction, and owned line-oriented NDJSON
  output. The source ID is the NORAD catalogue ID and UTC is required for propagation.
- Added explicit journalled integrity decisions for policy and processing rejects. Added an
  authority port for arm/disarm; arm messages are journalled but never call filter update.
- Migrated executive epoch creation to `SolutionEpoch::new`; output materialises the v3
  horizontal, speed, and vertical accuracies.
- Restored Vallado case 00005 t=0 assertions beside t=360, tagged improved production mode's
  local validation gap, and documented geocentric versus geodetic latitude in ENU rotation.

## Carried-item dispositions

### D15 / U-C1 review

- F1: fixed. `oneweb_enabled: bool` parses with default false; disabled observations are
  rejected and journalled, enabled observations enter the tracker route.
- F2: fixed. off-mode GNSS, Orbcomm, OneWeb, missing/stale ephemeris, prediction/elevation,
  and NIS rejects create integrity journal records.
- F5: fixed with a `process()`-level off-mode GNSS test proving no filter update.
- F6: fixed with production GNSS dual-route and Heading/SpeedThroughWater route tests.
- F7: fixed with concrete ephemeris/predictor/gate/output seams in the executive and a
  distinct authority command method on the integrity/authority port.

### D21 / U-E1

- Restored independently stated Vallado t=0 position and velocity assertions without
  removing t=360.
- Added literal `[UNVERIFIED]` improved-mode validation note.
- The integration uses the ephemeris store's typed six-hour age gate and journals errors.

### D22 / U-F1

- N1: fixed in v4 and code. Predictor range rate is geometric only. Primary and independent
  receiver Doppler APIs put clock drift in H·x. A regression sets primary drift to nonzero
  and proves zero innovation for geometric + clock observation.
- N2: fixed with the geocentric-latitude approximation comment in `pnt-types`.
- U-F1 arm item: fixed; authority receives the command and estimator update count remains 0.
- U-F1 GNSS frame item: retained through the existing NED-to-ECEF estimator ingress path.
- Orbcomm remains rejected per D10 because no verified executive receiver allocation/source
  mapping is configured; the existence of the receiver-specific estimator API does not
  silently lift the safety gate.

## Test evidence

`PATH="$HOME/.cargo/bin:$PATH" cargo test` passed: fusion executive 12 tests, estimator 13,
ephemeris 6, predictor 3, types 3, and all remaining unit/doc tests.

The executive suite covers production GNSS fusion+truth, recorded-only and off modes,
Heading and speed routes, OneWeb disabled/enabled routing, Orbcomm rejection, authority-only
arm routing, stale-age rejection journalling, and fixture ephemeris through accepted Doppler
update to finite-accuracy JSON parse and bridge-schema key checks.

## U-I2.1 fix-round dispositions

1. Opus F1/F4 — fixed. The geometric range-rate linearisation was moved to
   `pnt-predictor`, matching ARCHITECTURE module 7 and making the exact function called by
   `process_doppler` independently testable. Its position, velocity, and receiver-clock-drift
   columns are checked against central differences. The executive fixture now supplies a
   0.5 m/s innovation and requires a finite velocity correction bounded between 1e-4 and
   1 m/s, so the update is no longer a zero-innovation decoy.
2. Opus F5 / Sonnet F-1 — fixed. `DopplerPipeline` now defaults to a 5-degree elevation
   mask, stored as `Option<f64>` radians; configuration crosses a validated degrees API and
   tests can explicitly disable screening. The value is `[UNVERIFIED]` pending link-budget
   and replay tuning. An executive test constructs below-mask geometry and proves a
   journalled rejection without a Doppler filter update. CONTRACTS v4.1 records the default.
3. Sonnet F-2 — fixed. A divergent correlation peak exercises the executive NIS path and
   proves a chi-square integrity rejection with no additional filter update.
4. Opus F2 — fixed. Policy rejects write the full ingress envelope before their integrity
   event. Off-GNSS, Orbcomm, and disabled-OneWeb tests assert measurement journal growth.
5. Opus F3 / Sonnet F-4 — fixed. Epoch emission checks the complete state, covariance, and
   accessor-derived accuracies for finiteness. A poisoned estimator proves that neither an
   NDJSON line nor a solution epoch is emitted and that an integrity event is written.
6. Sonnet F-3 — fixed. CONTRACTS v4.1 pins TrackerDoppler `source_id` to the decimal NORAD
   catalogue ID, requires valid RFC3339 `utc`, and corrects accuracy wording to accessor
   derivation at emission.
7. Sonnet weak-test note — fixed. The enabled OneWeb test installs a real ephemeris-backed
   Doppler pipeline with valid NORAD/UTC metadata and asserts both the measurement record and
   the downstream integrity decision, proving tracker-pipeline reachability.

## U-I2.1 gate evidence

Exact command:

`PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`

Exit status: 0. `cargo test` passed all suites: fusion executive 15, ephemeris 6,
estimator 13, predictor 4, types 3, with all remaining unit and doc-test targets also
passing (41 integration/unit tests total). Clippy completed the workspace all-targets pass
with warnings denied. The final fmt check produced no diff. Final gate output ended with:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
```

## `[UNVERIFIED]` list

- Lever-arm compensation currently has an explicit zero-lever-arm hook; surveyed rotational
  velocity compensation is not connected.
- Output `msl_alt_m` is 0.0 until the MSL constraint surface is carried into the solution.
- Predictor elevation remains geocentric rather than WGS-84 geodetic local-up.
- Process noise, clock caps, satellite nuisance initial variance, chi-square threshold, and
  propagation age remain engineering settings requiring replay/field tuning.
- Improved-mode SGP4 has no project-local literature-anchored numerical fixture; AFSPC
  compatibility alone has Vallado t=0 and t=360 reference coverage.
- Orbcomm clock allocation/source mapping is not verified, so fusion remains disabled.
