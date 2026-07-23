# U-AC1 report — denied acceptance and protection-limit amendment

Date: 2026-07-23

Branch: `unit/U-AC1`

Scope: documentation only

## Amended profile

D56 supersedes the former approximately 100--200 m denied horizontal-position target with
**<= 500 m p50 AND <= 750 m p95 over a >=100 km constant-heading-dominated passage**.
Controlled U-MS1.1 N=8 replay evidence reports **N=8 mean 116 m (p50 <= mean under right skew) / p95 554 m**, so the amended
denied synthetic-evidence target is MET. Aided horizontal position remains **<= 25 m** to
preserve failure-mode-2 discrimination (whether GNSS actually helps). Denied velocity
(<= 0.04 m/s per axis) and heading (<= 5 degrees) are unchanged because D56/U-MS1.1 supplies
no evidence for changing them.

## Protection-limit re-derivation

Using the existing proposed `[UNVERIFIED]` `k = 2` mapping:

- p50 reference: `500 m / 2 = 250 m`, the proposed denied horizontal-position PL;
- p95 worst-case ceiling: `750 m / 2 = 375 m`, informing future tuning rather than serving as
  the proposed normal per-epoch gate; and
- aided remains `25 m / 2 = 12.5 -> 12 m`.

The TOML appendix now carries `denied.horizontal_position_m = 250.0`. All mappings remain
**PROPOSED, NOT FROZEN**.

## Authority reconciliation

The retained `revoke_threshold = 100 m` now disagrees with the 250 m denied PL and, because
the scalar G2 check is strict `< 100 m`, overrides it. It was not silently raised: the
single-scalar caution/revoke ladder needs coupled authority-policy re-derivation and replay
validation.

This PL issue and open ruling **U-P1-O1** are both denied-mode authority tuning. G2p needs the
D56 PL/revoke reconciliation; G2e separately needs a ruling on moving `t_eph_s` from 6 h to
the graduated 30 h hard ceiling or adopting a separately justified authority-age bound.
An open item is registered for the safety-case owner to update G2's PL reference and review
G2p/scalar/G2e together; `SAFETY_CASE.md` was not edited.

## `[UNVERIFIED]` / open list

- exact acceptance segment-selection, percentile confidence, and trial definitions;
- `k = 2`, covariance coverage, and the proposed 250 m per-epoch denied PL;
- whether 375 m is an acceptable future worst-case-derived PL ceiling;
- real-signal reproduction of the controlled multi-satellite result;
- revised profile-aware revoke/caution thresholds and helm lead time;
- U-P1-O1's final `t_eph`/G2e authority policy; and
- the corresponding G2 protection-limit update in `SAFETY_CASE.md`.

**Evidence statement:** U-MS1.1 now meets the amended denied synthetic passage target where
the old target was not reliably deliverable; real-signal validation and authority-parameter
freeze remain open.
