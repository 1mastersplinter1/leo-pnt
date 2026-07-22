# U-D1.1 report — design baseline fix pass, review round 1

Contract: v1 (2026-07-22)

## Finding dispositions

| Review finding | Disposition |
|---|---|
| Opus F1 / Sonnet F3 — Orbcomm receive path | **Fixed:** required a separate low-cost, non-coherent receiver, simultaneous in either bladeRF allocation; receiver selection remains a BOM `[UNVERIFIED]` item. |
| Opus F2 — vertical channel | **Fixed:** required a local-MSL sea-surface pseudo-measurement, `GPS_INPUT.alt` from that estimate, `vd = 0`, and consistent nonzero/bounded `vert_accuracy`; the local noise model is `[UNVERIFIED]`. |
| Opus F3 — lever-arm extrinsics | **Fixed:** mandated surveyed 3-D antenna/IMU/vessel-reference extrinsics, calibration-ID linkage, rotational velocity compensation, and authority denial on missing/mismatched calibration. |
| Opus F4 — observable and per-SV frequency | **Fixed:** defined correlation-peak Doppler as the estimator observable and a separate per-satellite, per-pass transmit-frequency nuisance-bias state with validated small random walk and pass-end retirement. |
| Opus F5 — SupGP age | **Fixed:** selected a conservative provisional 6-hour maximum from the supplied error-growth data; orbit-error-to-integrity mapping remains `[UNVERIFIED]`. |
| Opus F6 — indirect state observability | **Fixed:** clarified that documented direct or cross-covariance measurement paths permit observable IMU bias states. |
| Opus F7 — heading delivery | **Fixed:** required fused heading publication in `GPS_INPUT.yaw`, subject to the companion integrity gate. |
| Opus F8 — exclude ODOMETRY | **Fixed:** explicitly prohibited `ODOMETRY` for navigation injection because it cannot carry required per-epoch velocity uncertainty. |
| Opus F9 / Sonnet F4 — current treatment | **Fixed:** current is explicitly derived and journalled as ground velocity minus heading-rotated water-relative velocity, with propagated covariance; it is not a baseline EKF state. Scoring is mandatory, while a threshold remains `[UNVERIFIED]`. |
| Opus F10 — replay equivalence | **Fixed:** required bit-exact output under identical deterministic inputs/settings; non-deterministic backends need a frozen tolerance `[UNVERIFIED]`. |
| Opus F11 — multipath | **Fixed:** added sea-surface multipath rejection/uncertainty inflation and protection-limit authority response. |
| Sonnet F1 — heading justification | **Fixed:** documented the transverse-velocity rationale and marked the unvalidated 2/5 degree targets `[UNVERIFIED]`. |
| Sonnet F2 — aided 25 m provenance | **Rejected/no edit:** existing text already labels it an estimate inferred from an illustrative handoff limit and explicitly denies certification meaning; changing the value lacks evidence and was not requested by the unit brief. |
| Sonnet F5 — propagation wording | **Fixed:** replaced “no measurement-only propagation” with the explicit rule that measurement arrivals cannot be the sole propagation trigger. |
| Sonnet F6 — stale IMU estimator behaviour | **Fixed:** estimator stays running, does not fabricate propagation samples, and journals/recovery continue while authority is revoked. |
| Sonnet F7 — allocation qualifier | **Fixed:** clarified that direct L-band survival applies when the bladeRF allocation includes it and that independently received Orbcomm survives Ku loss. |

## Decisions taken

- Orbcomm uses an independent receiver. Non-coherence is accepted because front-end and receiver-clock diversity is valuable and both coherent bladeRF channels remain available for the survey-selected Ku/L-band allocation.
- Vertical position remains represented but is constrained to local mean sea level. The maritime system does not claim measured heave or vertical velocity; it publishes zero down velocity with an honest vertical bound.
- Per-satellite frequency uncertainty is estimated as a per-pass nuisance bias rather than epoch differencing. This preserves the absolute Doppler curve needed for position convergence while separating transmitter offsets from common receiver clock drift.
- Water current is a derived solution product, not a core estimator state. Both source velocities already have measurement paths, and covariance propagation makes the derived product auditable without adding a weakly modelled state.
- A provisional six-hour SupGP age limit is conservative relative to the supplied one-day and seven-day growth figures, but it is not yet an integrity proof.

## Remaining `[UNVERIFIED]` items introduced or retained by this pass

- Exact independent Orbcomm receiver/BOM choice.
- Mapping from SupGP orbit error and age to navigation protection limits; validate or tighten the provisional six-hour gate.
- Sea-surface pseudo-measurement variance covering geoid/chart datum, tide, waves and vessel motion in the actual operating area.
- Numeric process noise for the per-satellite transmit-frequency nuisance bias.
- Truth instrumentation, variability definition and pass/fail threshold for horizontal current-vector error.
- Deterministic replay tolerance for any backend that cannot produce bit-exact output.
- Heading acceptance targets (2 degrees aided, 5 degrees denied).

## Evidence

- Read, in required order: `.orchestration/CONTRACTS.md`, `docs/HANDOFF_PROMPT_BLADERF.md`, both design documents, then both review reports.
- Searched the edited documents for the required resolution terms using `rg` (Orbcomm path, vertical/MSL, correlation-peak and nuisance bias, lever arms/calibration ID, current vector, heading/ODOMETRY, multipath, ephemeris age and replay determinism).
- Documentation-only unit: no executable test suite applies. Final whitespace and diff checks were run over the three owned files.

## Assumptions

- The baseline vessel remains a displacement-hull vessel operating provisionally in the Danish straits.
- MAVLink `GPS_INPUT` fields named by the existing baseline are the intended ArduPilot interface; implementation-level units and validity encoding remain subordinate schema/code work.
