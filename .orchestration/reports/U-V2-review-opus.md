(Reviewer: Opus scientific seat. Verdict: PASS with mandatory interpretation corrections before merge.)
Verified: bit-identical reproduction, correct NEES/NIS/LOS methodology, no truth leakage, replay-prior defect diagnosis faithful (gain-0.5 half-radius, fix routed), Q=4e-4 is a GENUINE interior optimum (reviewer's extended sweep: 0.2659/0.2616/0.2398/0.2365/0.3084/0.3753/0.6643 across 4e-7..0.4).
Corrections required (text only, empirical results stand):
- F1 MED/high — "default Q is 100x the injected acceleration-error variance / matched to injected scale" is FALSE: injected random variance (5e-4)^2=2.5e-7; optimum 4e-4 is ~1600x that. Restate as: Q=4e-4 empirically minimizes velocity RMS (interior optimum, cite F2 sweep); remove the "matches the injected scale" causal claim.
- F3 MED/med — state explicitly: the observed Doppler-degrades-velocity effect is CONTINGENT on the near-truth IMU (propagation error ~5e-4 m/s^2); at sea with realistic IMU error the sign could flip and the low-Q fix would be wrong. Scope the D39 answer accordingly.
- F4 LOW-MED — stale-ephemeris section: epoch-shifting aliases orbital phase (non-monotonic innovation RMS 5414/4395/3825 m/s at 1h/6h/24h), innovations are 3000-5000x the gate; data support "any nonzero staleness >=1h is rejected", NOT the 6h choice; delete "supports the gate being no looser than 6h", keep the honest report phrasing; note the missing HPH' term makes rejection an upper bound.
- F5 LOW-MED — maneuver-reset: state it is unfalsifiable by construction here (turn enters via near-truth IMU), a harness limitation, not evidence about the real filter.
- F6 LOW — add dispersion honesty: NEES epochs autocorrelated (effective N far below 57,600); 20-min crossover rests on 6 seeds, means only — label fragile.
- F7 LOW — label initial_radial_position_error_m as analytic annotation, not measurement.
- F8 disclosed — debug-assert panic on full study in debug builds: keep disclosed.
