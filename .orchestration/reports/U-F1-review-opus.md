(Reviewer: Opus, fresh context, deep seat. Verdict: FAIL — fix round required.)

Verified correct: Joseph-form update, CWNA Q discretization, gain/innovation math, nuisance augment/retire index bookkeeping (rows/cols + slot decrement), v3 additive compatibility (workspace builds w/ unmodified executive), ArmCommand/D13 + receiver-clock/D10 encodings faithful. Branch-staleness diff noise benign (I1).

Findings requiring fix:
- H1 high/high pnt-types/src/lib.rs:169-192 — horiz/vert accuracies use raw ECEF X,Y,Z variances; at 56°N these are neither horizontal nor vertical (up is ~34° off Z). Also max-of-axes instead of a 2-D combination (DRMS/trace), ignoring P[0,1]. These feed the steering gate. Fix: rotate position covariance ECEF→ENU at the estimate's location; horizontal = sqrt of ENU E+N variances (DRMS) or stated convention; vertical = ENU U sigma. Unit test at a high-latitude point vs independent rotation.
- H2 high/high estimator/src/lib.rs:546-553 — magnitude-growth test passes with Q=0 (reproduced; growth comes from identity-P velocity coupling). Fix: start with zero velocity variance or assert the Q-attributable growth component (e.g. compare two propagations with/without Q, or P[0,0] growth with zero initial velocity covariance must equal q-driven terms).
- M1 med/high :518-543 — heading/GNSS/MSL Jacobian tests hand-rebuild H instead of invoking update_heading/update_gnss/update_msl_altitude. Fix: FD-check the REAL code paths (e.g. expose h-construction or test via update with crafted state/covariance so a wrong index/sign fails).
- M2 med/high :210-232 — update_doppler_for_receiver (D10 path) has zero tests; its drift-remap h[slot.drift]+=h[CLOCK_DRIFT];h[CLOCK_DRIFT]=0 unchecked. Fix: FD test on augmented-dim filter.
- M3 med/high :448 — horizontal_velocity_ned_mps populated with ECEF Vx,Vy. Fix: proper ECEF→NED rotation at estimate location (consistent with H1's frame machinery).
- M4 med/high-math :460-468 — speed_model = hypot of ECEF equatorial-plane velocity, not horizontal speed. Fix: horizontal speed in ENU at estimate location; keep FD test on the real code.
- M5 med — clock bias (state 7) has no measurement path (Doppler observes drift only; no pseudorange); variance unbounded. Fix per baseline states-rule: either document+bound it (v3 note: bias carried for future pseudorange/STL, with variance cap or soft constraint) or remove the state; decision goes to CONTRACTS v3 + report — pick, justify, mark [UNVERIFIED] mapping.
- L1 low :500-516 — transition FD test runs on 9-dim default only; run FD on an augmented filter (clock slot + nuisance) so coupling terms are perturbed.
- L2 low — ECEF vertical velocity weakly observable in denied mode; note in report/v3, consider constraining consistent with vd=0 rule.
- L3 low — symmetry/PSD debug_assert only (informational; keep, note in report).
- L4 low :414-417 — clock Q incomplete (no bias term); add standard two-state clock Q with [UNVERIFIED] coefficients.
- L5 low fusion-executive:76-103 — ArmCommand falls into `_` route → misrouted to Fusion until U-I2; add explicit note to U-I2 handoff (executive not owned by U-F1).
- I2 — update_doppler hardcodes nuisance variance 1e4 ignoring augment's variance param; honor the param.
- I3 — GNSS gating is 6×1-dof scalar gates w/ shared threshold; either document threshold semantics or implement joint gate; state choice in v3.
