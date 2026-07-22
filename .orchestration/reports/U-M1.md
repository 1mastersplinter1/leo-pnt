# U-M1 report — MAVLink GPS_INPUT publisher and Rover SITL

Contract: v2 (2026-07-22). Unit branch: `unit/U-M1`.

## VERIFIED

### Bridge

- Added the Python package in `tools/mavlink_bridge`, using pure-Python `pymavlink==2.4.43`
  (`DISABLE_MAVNATIVE=1`) and `pymap3d==3.2.0`. The datum is WGS 84: EPSG:4978 ECEF is
  converted to EPSG:4979 latitude/longitude; ellipsoid altitude is discarded and the
  required separate MSL-constrained altitude is published.
- The README defines newline-delimited JSON mirroring all current v2 `SolutionEpoch` and
  `FilterState` fields, plus required horizontal/speed/vertical accuracies and required MSL
  altitude. Heading may be null. Accuracy inputs must be finite and positive.
- The mapping publishes GPS_INPUT only (never ODOMETRY), NED `vn`/`ve`, valid `vd=0`, three
  independent accuracies, and yaw. MAVLink's official common-message specification says
  yaw zero means unavailable and 36000 represents north; tests cover positive and negative
  north wraps, east, and unavailable yaw.
- At age <=1 s an authorised epoch is fix type 3. It degrades to type 2 after 1 s and no-fix
  after 3 s, with position/vertical accuracy growing 2 m per second of age and speed
  accuracy growing 0.25 m/s per second. Publication remains at 5 Hz after stdin EOF when
  configured with a grace period, far inside the estimated 4 s GPS timeout.
- Commands run:

  ```text
  tools/.venv/bin/pytest -q tools/mavlink_bridge
  ...............  [100%]
  15 passed in 0.03s

  tools/.venv/bin/ruff check tools/mavlink_bridge tools/sitl
  All checks passed!
  ```

### Pinned SITL build and partial live integration

- Resolved official ArduPilot `Rover-4.6.3` to immutable commit
  `3fc7011a7d3dc047cbb17d8bd98ee94577d144c6`, cloned its submodules, configured `sitl`, and
  successfully built `ardurover` with GCC 15.2.0. No sudo or system package install was
  used; waf dependencies live in `tools/.venv` and are pinned in
  `tools/sitl/requirements-build.txt`.
- Built artifact SHA-256:
  `abd0088642cb85d4fd2e7511acd225d5c4626f6ccd9298d38140b7bb2cb3f499`.
- `tools/sitl/run_acceptance.py` loads and confirms `FRAME_CLASS=2`, `GPS1_TYPE=14`, and
  `GPS2_TYPE=0`, injects the package's 5 Hz synthetic straight leg, and fails closed unless
  all acceptance checks pass.
- A strict 60-second live run showed repeated actual telemetry excerpts:

  ```json
  {"fix_type": 3, "h_acc": 800, "type": "GPS_RAW_INT"}
  {"ekf_flags": 167, "position_error_m": 0.07783655661270385, "type": "OBSERVED"}
  ```

  Thus ArduPilot consumed the injected fix, exposed the injected 0.8 m horizontal accuracy,
  and its reported position tracked the final injected position to 0.08 m. This is real
  evidence from `tools/sitl/evidence/mavlink.jsonl` (ignored as a run artifact), not a
  fabricated acceptance record.

## ASSUMED / UNVERIFIED

- **Overall SITL acceptance did not pass.** After 60 seconds the external GPS showed a 3D
  fix, but `EKF_STATUS_REPORT.flags` remained 167: constant-position mode was set and the
  absolute-horizontal-position bit was absent. The harness raised
  `AssertionError: EKF did not report absolute horizontal position: flags=167`; therefore
  no `ACCEPTANCE` record was written. The requested “EKF reaches a 3D-fix-equivalent state”
  remains `[UNVERIFIED — run here but failed]`.
- The configured 0.1 m/s straight leg is intentionally slow because SITL's simulated IMU
  remains stationary. A 1 m/s attempt was correctly rejected by the 10 m tracking check
  with 45.44 m error. The final run's 0.08 m position result does not establish performance
  for dynamically consistent moving-vehicle simulation.
- Accuracy inflation rates and 1 s/3 s degradation thresholds are explicit conservative
  policy decisions because the normative design specifies honest inflation and continuous
  fill but no numerical growth model. They require system integrity-owner review.
- Current `SolutionEpoch` in Rust has no accuracy or MSL-altitude fields. The JSON additions
  are the agreed U-M1 boundary pending the later estimator unit; no files under `crates/`
  were changed.

## Open uncertainties

1. Determine why EKF3 remains in constant-position mode despite stable GPS_INPUT fix type 3,
   valid zero vertical velocity, visible accuracy, and close global-position tracking. The
   deterministic failing script and telemetry make this reproducible.
2. Freeze system-owned stale thresholds and covariance/accuracy growth after integrity
   policy is implemented.
3. Decide whether estimator UTC/GPS-week information should extend the JSON boundary; this
   bridge currently uses monotonic `time_usec` and zero GPS week fields.
