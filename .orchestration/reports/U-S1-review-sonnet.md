(Reviewer: Sonnet 5, fresh context, completeness seat, review round 1. Verdict: FAIL — majors B1/C1, B3, C2/D1.)

Majors (already dispatched in fix round): B1/C1 G3 mislabeled as covering ephemeris-age gate; B3 G1 arm input absent from architecture (→D13); C2/D1 "unexpired" frequency-reference calibration undefined and unmarked.
Minors:
- B2 minor/high SAFETY_CASE.md:150 — "Calibration ID missing/mismatched" row is not one of the baseline degradation table's 11 literal rows (sourced from extrinsics prose, DESIGN_BASELINE.md:126-130); accurate but breaks the 1:1 "per degradation row" mapping — label it as an additional row with its source.
- B5 minor/high SAFETY_CASE.md:143-146 — several §2.2 rows drop qualifying clauses from baseline text: :143 drops "IMU turn dynamics and any selected non-magnetic heading sensor" (baseline :139); :144 similar for both-mags-lost (baseline :140); :146 drops "journalling and recovery may continue" (baseline :142). Restore the dropped clauses.
- D2 minor/high SAFETY_CASE.md:123,197 — "stabilisation dwell" not tagged [UNVERIFIED] at first two mentions (only later in H3 residual and §5); add the tag at first use.
Also noted positively: 11/11 baseline rows mapped; [UNVERIFIED] discipline otherwise strong; all 7 mandated hazards present.
