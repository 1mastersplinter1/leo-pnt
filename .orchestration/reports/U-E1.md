# U-E1 report — ephemeris propagator and Doppler predictor

Built against contract v2 (2026-07-22). The two design documents still display a v1 label;
the binding v2 additions in `.orchestration/CONTRACTS.md` were applied.

## Changes

- Added `pnt-ephemeris`, a local-file-only TLE and CelesTrak OMM JSON store keyed by NORAD
  catalogue ID. Both a JSON object and array are accepted. Each entry retains its epoch.
- Added configurable, typed ephemeris age rejection. The default is six hours, from
  `docs/design/DESIGN_BASELINE.md` (approximately 0.94 km cited orbit-error datum). The
  absolute epoch/query separation is gated, including backward propagation.
- Added SGP4 TEME propagation using `sgp4` 2.4.0 in AFSPC compatibility mode. That selects
  WGS-72 constants and makes the included Vallado case 00005 comparison identical to the
  dependency's shipped `tests/test_cases.toml` reference vector.
- Added TEME-to-ECEF position and velocity conversion and documented its model limits.
- Added `pnt-predictor` with geometric range, range rate, ECEF line of sight, geocentric
  elevation/mask, receiver-clock drift, nominal carrier and per-SV nuisance bias.
  Sign convention is pinned: range rate is `d(range)/dt`; approach is negative range rate,
  and the first-order received-frequency Doppler is therefore positive.
- Added an end-to-end fixture test that propagates a real ISS TLE over a fixed Copenhagen
  receiver, finds a visible pass, and checks the closest-approach sign change, orbital-speed
  Doppler bound, and ten-second smoothness.

## Dependencies

- `sgp4` 2.4.0
- `chrono` resolved 0.4.45 (manifest compatibility floor 0.4.43)
- `serde_json` resolved 1.0.151
- `thiserror` resolved 2.0.19

No runtime or test code performs network access. Fixtures are under
`crates/pnt-ephemeris/tests/fixtures/`.

## Models, bounds, and assumptions

- SGP4 uses the crate's AFSPC compatibility path and WGS-72 geopotential. Its included
  Vallado reference is checked to 1e-6 km position and 1e-9 km/s velocity at epoch. This is
  an implementation-verification tolerance, not an ephemeris accuracy claim.
- TEME to Earth-fixed uses the IAU-1982 GMST polynomial/Vallado TEME-to-PEF rotation,
  constant Earth rotation rate 7.2921150e-5 rad/s, and the corresponding `omega cross r`
  velocity term. UTC is used as UT1. Since IERS keeps `|UT1-UTC| < 0.9 s`, the stated worst
  equatorial angular-position contribution is below approximately 420 m. Polar motion,
  current Earth-orientation parameters, length-of-day variation, and higher-fidelity
  transformations are omitted.
- `[UNVERIFIED]` The net TEME-to-ECEF error for the actual deployment epochs has not been
  validated against an IERS-EOP-aware reference implementation. Omitted polar motion is
  expected to be metre-scale, but no project-local validation fixture establishes a bound.
- `[UNVERIFIED]` The baseline's mapping from the six-hour/approximately-0.94-km orbit error
  to navigation integrity remains unresolved, as the baseline itself states.
- Elevation is relative to the receiver's geocentric radial direction. This is deliberate
  and documented; an ellipsoidal geodetic local-up conversion is needed if sub-degree mask
  accuracy matters.
- Receiver clock drift is represented as equivalent range rate in m/s. Per-SV nuisance
  bias is an additive correlation-frequency offset in Hz.

## Integration changes needed from U-I2

No existing crate was modified. U-I2 must add executive ports/adapters for ephemeris query,
predicted observation, and typed rejection reporting (review finding F7). It must map the
estimator's receiver clock-drift state into equivalent m/s with the same sign convention,
create/retire the per-SV per-pass nuisance state, and decide where geodetic elevation/local
up is supplied. Existing `pnt-types` has no satellite ECEF-state or predicted-Doppler bus
payload; integration needs those schema decisions without weakening the age gate.

## Evidence

TDD commits:

- `12394a0 test(ephemeris): specify propagation and Doppler contracts` (tests committed
  before library sources; initial run failed because the crates did not yet exist).
- `794156a feat(ephemeris): add SGP4 propagation and Doppler prediction`.

Final gate, 2026-07-22:

```text
$ cargo test
all workspace tests passed (new tests: pnt-ephemeris 5/5, pnt-predictor 3/3)
$ cargo clippy --all-targets -- -D warnings
finished successfully, zero warnings
$ cargo fmt --all -- --check
finished successfully
```
