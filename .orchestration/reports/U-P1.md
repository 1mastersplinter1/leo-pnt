# U-P1 report — graduated ephemeris aging

## Disposition

Implemented the D45 graduated model. Legacy ephemeris callers retain the inclusive 6 h
gate; the Doppler executive uses a typed propagation result carrying signed age, nominal
weighting through 6 h, additive variance through an inclusive 30 h ceiling, and hard
rejection above it. Future-dated records carry a typed lead flag and produce an executive
integrity `NOTE`. Aging parameters are parsed into `Config` and used by normal executive
construction rather than replaced by constructor defaults.

The authority semantics did change: Doppler epochs now carry true `ephemeris_age_s` rather
than `0.0`, so G2e can revoke authority at `t_eph_s`. This is fail-closed, but the current
provisional 6 h authority parameter conflicts with D45 even though measurement eligibility
continues to 30 h.

## Derivation

The line through 0.94 km at 6 h and 2.6 km at 24 h is
`sigma_r=0.386667+0.0922222*a_h km`. With the reference geometry
`|u_dot|=v_rel/range=7.6/1000=0.0076 rad/s`, independent added uncertainty beyond the
nominal fresh model is
`sigma_add=|u_dot|*sqrt(sigma_r(a)^2-sigma_r(6h)^2)`. A central finite-difference test of
rotating-LOS range agrees with the implementation.

## Passage comparison

The committed deterministic synthetic run covers 100.01 km in 9 h at 6 kn, with GNSS lost
at 2 h and ephemeris cached at departure. The same seed and measurement generator drive
both config-selected policies through the integrated executive, SGP4 propagation, Doppler
prediction, estimator update, and integrity journal paths. Runtime is honestly decimated:
IMU propagation is 1 Hz and three synthetic Doppler geometries are sampled every 60 s.

Measured endpoint results from the actual filter states against mission truth:

- hard 6 h: 1,083 accepted / 540 rejected Doppler observations, last accepted at 6.0 h,
  3,636.8 m final 3D error (DR class);
- graduated 30 h: 1,623 accepted / 0 rejected, last accepted at 9.0 h, 1,632.6 m final 3D
  error (passage-held class).

No endpoint error law or post-run error constant is applied. D43 remains controlling: this
is synthetic availability evidence, not real ephemeris-aging validation.

## Open ruling

**U-P1-O1 — routed to PARAMS/SAFETY_CASE owners.** Reconcile G2e and
`PARAMS_PROPOSAL.md`'s provisional `t_eph_s=21600` with the 30 h measurement ceiling.
Recommended ruling: move `t_eph_s` to the hard ceiling and let graduated measurement
inflation carry the accuracy honesty. Alternative: define and justify a separate
authority-age bound. The parameter register and safety claim must change together; this
unit did not edit `SAFETY_CASE.md`.

## Review findings

- F1: fixed; passage evidence is an integrated, seeded two-policy run with measured errors.
- F2: fixed; the determinism test serializes two complete independent pipeline runs and
  compares their bytes.
- F3: corrected; true age changes G2e authority semantics in a fail-closed direction and is
  documented in the dated design amendment and this report.
- F4: surfaced and registered as open ruling U-P1-O1, routed to PARAMS/SAFETY_CASE owners;
  no unowned safety-case edit was made.
- F5: fixed; `Config` carries, parses, validates, and supplies `EphemerisAgingConfig` to the
  executive construction path.
- F6: fixed; the graduated propagation API flags future-dated ephemeris with a typed lead
  value and the executive journals an integrity `NOTE`.
- F7: fixed; non-finite/negative ages and fresh-above-ceiling configurations classify
  fail-closed, while parsed misordered configuration is a hard error.

## `[UNVERIFIED]`

- The two-point linear SGP4/SupGP error-growth fit and extrapolation to 30 h.
- Reference relative speed, range, isotropic orbit-error model, and Doppler mapping.
- The 30 h hard ceiling and all default inflation coefficients.
- Seeded IMU bias/noise, synthetic Doppler geometry/noise, 1 Hz/60 s decimation, study
  estimator gain/correction cap, and position-class proxy.
- Real-SupGP aging, constellation availability, real tracker residuals, sensor-rate
  execution, and at-sea replay.
