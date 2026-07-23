# Brief U-P1 — Passage endurance: graduated ephemeris aging (100 km denied requirement)

Contract version: v5.1. Worktree branch `unit/U-P1`. Commit there; never merge to main; NO
Co-Authored-By/Claude-Session trailers. Read first: DECISIONS.md D45 (the requirement) +
D43 (aging-evidence caveat), docs/design/DESIGN_BASELINE.md (ephemeris age section),
crates/pnt-ephemeris (age gate), crates/fusion-executive (process_doppler age handling),
crates/pnt-mission + pnt-studies (harness), docs/design/PARAMS_PROPOSAL.md t_eph section.

## Goal
1. **Design amendment (docs)**: a dated amendment to DESIGN_BASELINE's ephemeris paragraph:
   graduated model — age <= t_fresh (6 h): nominal weighting; t_fresh < age <= t_ceiling:
   observation accepted with measurement-noise inflation sigma_add(age) derived from the
   SGP4 error-growth curve (0.94 km@6h, 2.6 km@24h — derive the Doppler-domain mapping via
   the geometry, show the derivation, mark constants [UNVERIFIED pending real-SupGP study]);
   age > t_ceiling (propose >= 30 h with rationale): hard reject. State explicitly this
   serves D45's 100 km/9 h passage with pre-departure caching + margin.
2. **Implementation**: pnt-ephemeris returns age alongside propagation (typed, no behavior
   change to existing callers beyond the new path); the executive's Doppler path applies
   the inflation between t_fresh and t_ceiling and journals an integrity NOTE (not reject)
   with the age and applied inflation; config carries t_fresh/t_ceiling/inflation
   coefficients as [UNVERIFIED]-defaulted parameters (fail-closed rule untouched — these
   feed measurement weighting, not steering authority).
3. **Passage study** (pnt-studies): a >= 9 h synthetic passage at 6 kn, GPS lost at t=2 h,
   ephemeris cached at t=0: compare hard-6h-gate behavior (Doppler dies mid-passage,
   position degrades to DR) vs graduated handling (position class held), with the D43
   aliasing caveat stated — synthetic aging is a stand-in, not validation. Deterministic,
   committed JSON + STUDY section.
4. **Tests**: TDD for the inflation math (FD-checked against the derivation), the
   age-band routing (fresh/inflated/rejected boundaries exact +/-1ns), journal notes, and
   the study harness determinism. Whole-workspace gate green.

## Files owned
crates/pnt-ephemeris/**, crates/fusion-executive/**, crates/pnt-config/**,
crates/pnt-studies/** (passage module only), docs/design/DESIGN_BASELINE.md (amendment
only), docs/studies/passage/**, .orchestration/reports/U-P1.md.

## Report
Derivation, before/after passage numbers, dispositions, [UNVERIFIED] list.
