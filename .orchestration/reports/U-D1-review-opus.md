(Reviewer: Opus 4.8 seat, fresh context, adversarial. Verdict: FAIL — blockers F1, F2, F3; F4 must also be addressed.)

BLOCKERS:
- F1 major/high DESIGN_BASELINE.md:45,56-59 + ARCHITECTURE.md:12 — Orbcomm 137 MHz is a required front end but BOTH coherent RX channels are consumed in BOTH survey outcomes (Ku+Iridium, or dual-Ku+time-shared Iridium); no channel, no second-device path, no [UNVERIFIED] mark. Fix: one sentence — separate independent receiver (BOM item) OR time-shared with stated non-simultaneity.
- F2 major/med-high DESIGN_BASELINE.md:92-93,121-123,53 — vertical channel undefined: no vertical state, no sea-surface/MSL constraint, no producer for mandatory GPS_INPUT alt/vd/vert_accuracy; collides with the doc's own no-unobserved-states rule. Fix: state altitude treatment explicitly (e.g. MSL pseudo-measurement or fixed-excluded) and what U-M1 publishes.
- F3 major/med (absent everywhere) — no antenna phase-centre / IMU lever-arm extrinsics requirement. |omega x r| for r~5-10 m at 0.05-0.1 rad/s yaw = 0.25-1.0 m/s spurious velocity vs 0.02-0.04 m/s acceptance. Fix: mandate measured extrinsics as calibration input; envelope must carry/reference them.

MUST-ADDRESS (major, low-med confidence):
- F4 DESIGN_BASELINE.md:92-95 + ARCHITECTURE.md:29-30 — per-satellite transmit-frequency/oscillator offset untreated; observable definition (correlation Doppler vs carrier Doppler) not fixed at the estimator interface. One sentence fixing observable + per-SV nuisance treatment.

MINORS (recommended, non-blocking):
- F5 minor/high :61-63 — SupGP age gate called [UNVERIFIED] though handoff gives the SGP4 error-growth curve (0.94 km@6h / 2.6 km@1d / 38.5 km@7d) from which a default derives against the 200 m budget.
- F6 minor/med :92-95 — "measurement update that observes them" wording could mislead U-F1 into deleting IMU bias states (observed indirectly via cross-covariance); clarify.
- F7 minor/med :53,123 — fused heading has no stated delivery path to ArduPilot (GPS_INPUT.yaw?); heading acceptance unobservable at AP otherwise.
- F8 minor/med :53 — GPS_INPUT-over-ODOMETRY rationale + explicit ODOMETRY exclusion not recorded in the normative doc.
- F9 minor/med :49,92-95,119-126 — current set/drift (handoff's "strongest argument") optional with no acceptance criterion; at minimum state why.
- F10 minor/med :126 — replay "materially equivalent" tolerance unfrozen; consider bit-exact promise for identical config.
- F11 minor/low — sea-surface multipath unmentioned as maritime bias source; note whether integrity gate is expected to catch it.

Fidelity: all deliverable-1/3 elements present; all ten handoff failure modes guarded; no contradiction with handoff or between docs. Blockers are omissions of physical/interface facts, not errors in what is stated.
