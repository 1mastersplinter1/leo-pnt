# Brief U-MS1 — Multi-satellite fixture study (demonstrate the real denied position class)

Contract v5.1. Worktree branch `unit/U-MS1`. Commit there; never merge to main; NO
Co-Authored-By/Claude-Session trailers. Read first: DECISIONS.md D51/D52/D54 (the single-sat
limitation this closes; the honesty corrections), docs/studies/highspeed + passage (the
patterns; the single-sat divergence lesson), crates/pnt-ephemeris (fixture loading, TLE/OMM),
crates/pnt-predictor + pnt-estimator (Doppler geometry, EKF, per-SV nuisance augmentation),
crates/pnt-studies (harness), crates/pnt-mission (mission generator), docs/research/
R4-signal-structures.md (real constellation orbital params — Starlink 53deg/550km,
OneWeb 87.9deg/1200km, Iridium 86.4deg/780km — for realistic fixtures).

## The core question
Does LINE-OF-SIGHT DIVERSITY from multiple simultaneously-visible LEO satellites make DENIED
position observable at the ~100-200 m class the literature claims, where the single-satellite
fixture could not (D51: tens of km)? Answer it honestly with the real pipeline.

## Goal
1. **Multi-satellite fixture**: build a fixture with N (sweep N in {1,2,4,8}) satellites of
   realistic orbital DIVERSITY — mix inclinations/altitudes/RAAN so their LOS directions from
   the vessel differ (this is the whole point; co-planar sats add little). Use real published
   TLEs/OMM where available (cite), or synthesize orbital elements with documented realistic
   params [UNVERIFIED where synthetic]. Compute per-epoch visibility (elevation mask) so the
   study only uses satellites actually above the horizon — report visible-count over time.
2. **The observability study** (pnt-studies, new module `multisat`): run the REAL
   Executive + FilterStub EKF with the PRODUCTION chi-square gate Some(9.0) on:
   - a long constant-heading denied leg (the handoff's 10-20 min position-convergence
     regime; go longer, e.g. 30-60 min) at displacement speed;
   - N = 1/2/4/8 visible satellites (the headline sweep — position error vs satellite count);
   - the D45 100 km passage with multi-sat.
   Measure denied position/velocity error vs truth, convergence-vs-time curve, and the
   position-error-class per N. Also do the single-vs-multi contrast that isolates geometry.
3. **Honesty (mandatory, per D51/D52)**: production gate ON (prove via rejection counts);
   honest error_class incl. DIVERGED; NO clamped toy estimator (use the real FilterStub, same
   as U-H2); NO formula/closed-form outputs — every number from real filter state vs truth;
   if the filter still can't reach 100-200 m even with 8 satellites, REPORT THAT (it would
   mean the stub EKF or the Doppler-only observability needs more — route it), do not
   target-fit. State the GDOP/geometry intuition. Nuisance-state augmentation per satellite
   as the design intends.
4. **Tests**: TDD; determinism (bit-identical reruns); the visibility computation; the gate
   genuinely on; whole-workspace gate green.

## Files owned
crates/pnt-studies/** (multisat module + bin only), crates/pnt-ephemeris/** (ONLY if a
multi-record fixture loader addition is needed — additive, legacy callers unbroken),
docs/studies/multisat/**, .orchestration/reports/U-MS1.md. Fixtures under the ephemeris
crate's tests/fixtures/ or docs/studies/multisat/.

## Report
The headline: does multi-sat reach the 100-200 m class, at what satellite count and leg
duration? position-error-vs-N table, convergence curves, the single-vs-multi contrast, the
GDOP story, honest [UNVERIFIED] list, and — if it does NOT reach the class — the diagnosed
reason and routed next step. No target-fitting; report the real result whatever it is.
