# U-M1 report — MAVLink GPS_INPUT publisher and Rover SITL

Contract: v2 (2026-07-22). Unit branch: `unit/U-M1`. Fix round: U-M1.1.

## Finding dispositions

1. **Yaw sentinel (Sonnet F1 / Opus F1): FIXED, VERIFIED.** Corrected both red-first
   oracles and implementation: `encode_yaw(None)` and expired fill now send 0. Tests still
   preserve 36000 for north. README now follows common.xml `GPS_INPUT.yaw`; the unrelated
   `GPS_RAW_INT` 65535 sentinel is explicitly excluded.
2. **Required schema tests (Sonnet F2): FIXED, VERIFIED.** Parameterised missing-field tests
   cover `horiz_accuracy_m`, `speed_accuracy_mps`, `vert_accuracy_m`, and `msl_alt_m`.
3. **Clock domain (Sonnet F3): FIXED, VERIFIED.** README states that producer and bridge
   `monotonic_ns` must share the host monotonic clock domain.
4. **Duration consistency (Sonnet F4): FIXED.** Removed the false 60-second/fixed-duration
   narrative. The acceptance command defaults to 45 seconds; the latest successful run used
   that default. The bridge's 4-second timeout statement is now tied to pinned
   `AP_GPS.cpp:74`, not described as an estimate.
5. **Endpoint-only acceptance (Sonnet F5): FIXED, VERIFIED.** The harness now checks every
   sampled position after EKF aiding as well as the endpoint. Latest evidence contains 15
   continuous samples, maximum error 0.07836 m.
6. **HDOP root cause (Opus F2): FIXED, VERIFIED.** The bridge no longer ignores HDOP. It
   supplies the finite operational mapping `HDOP = horiz_accuracy_m / 1 m`, including stale
   inflation. `AP_GPS_MAV.cpp:77-79` writes it to `state.hdop`; the unchanged
   `EK3_GPS_CHECK` gate in `AP_NavEKF3_VehicleStatus.cpp:170` then passed. A red-first unit
   test asserts finite sent HDOP. No check was weakened in `params.parm`.
7. **D17a absent (Opus F3): FIXED, VERIFIED with scoped exception.** Native silence and an
   early companion-HOLD candidate were run in armed GUIDED mode; telemetry, timing, mode,
   EKF, and actuation findings are below and in `tools/sitl/evidence/D17a.md`. Alternative
   MANUAL/disarm candidates remain **[UNVERIFIED]** because they have different safety
   consequences and no selected Case-A policy.
8. **README false yaw claim (Opus F4): FIXED.** Removed the extension-field justification.
9. **Wrong test oracle (Opus F5): FIXED red-first.** Both unavailable-heading oracles were
   changed to 0 before implementation; the intentional red run was 3 failed / 16 passed,
   followed by green.
10. **Position prose (Opus F6): FIXED.** The 0.078 m result is described only as
    driver/telemetry ingestion and tracking evidence. It is not fusion-performance evidence
    for a dynamically representative moving vehicle.
11. **Checksum caveat (Opus F7): FIXED.** `tools/sitl/README.md` states that the observed
    binary SHA-256 is machine/build dependent, not a universal upstream checksum. The
    immutable source commit remains the reproducibility identity.
12. **GPS week/time (Opus F8): FIXED, VERIFIED.** The bridge anchors measurement age to host
    UTC and emits real GPS week/TOW with the current 18-second GPS−UTC offset. A unit test
    verifies delayed fill retains the measurement epoch rather than publication time. The
    README records the future-leap-second maintenance requirement.

## Acceptance — VERIFIED

Pinned official ArduPilot Rover-4.6.3 commit:
`3fc7011a7d3dc047cbb17d8bd98ee94577d144c6`. The local GCC 15.2.0 artifact produced
SHA-256 `abd0088642cb85d4fd2e7511acd225d5c4626f6ccd9298d38140b7bb2cb3f499`; this is only
the observed machine/build checksum.

The unchanged parameters load and confirm `FRAME_CLASS=2`, `GPS1_TYPE=14`, and
`GPS2_TYPE=0`. The successful run wrote a real `ACCEPTANCE` record:

```json
{"continuous_samples": 15, "ekf_flags": 831, "max_tracking_error_m": 0.07836298428997139, "position_error_m": 0.07836298428997139, "type": "ACCEPTANCE"}
```

Flags 831 contain absolute-horizontal-position and do not contain constant-position mode.
The 15 post-aid position observations all met the 10 m gate. `GPS_RAW_INT` repeatedly exposed
fix type 3 and injected horizontal accuracy 800 mm. These facts verify ingestion, EKF aiding
state, and the static-SITL telemetry path; the 0.078 m value does not establish estimator
fusion accuracy during realistic vehicle motion.

## D17a characterisation — VERIFIED excerpts

The native run entered armed GUIDED (mode 15) after aiding, then stopped `GPS_INPUT` at
44.443 s. Relative to stop:

- +1.019 s: `GPS_RAW_INT.fix_type=1`;
- +4.212 s: EKF flags changed from 831 to 39, losing absolute horizontal position;
- +5.071 s: HEARTBEAT changed to armed HOLD (mode 4), with `EKF variance` and
  `EKF failsafe` STATUSTEXT;
- +12.141 s: both EKF lanes reported `stopped aiding`; later flags were 167.

Throttle remained 0 and `SERVO_OUTPUT_RAW` channels 1/3 remained 1500/1500 throughout the
observed transition. Rover remained armed. This timing matches pinned source: GPS timeout is
4000 ms (`AP_GPS.cpp:74,888-894`); armed `ekf_position_ok()` rejects loss of absolute/relative
position (`Rover/ekf_check.cpp:124-146`); ten 10 Hz failed checks debounce the event, and the
default `FS_EKF_ACTION=HOLD` changes a position-requiring mode to HOLD
(`Rover/ekf_check.cpp:150-180`).

In the separate Case-A candidate run, the companion sent HOLD +1.531 s after silence while
GUIDED. HEARTBEAT showed HOLD 51 ms later, approximately 3.49 s before native failsafe timing.
Actuation stayed neutral. Full records: `tools/sitl/evidence/d17a-mavlink.jsonl` and
`d17a-case-a-mavlink.jsonl`; concise evidence: `tools/sitl/evidence/D17a.md`.

## Test evidence

```text
Red-first bridge run:
3 failed, 16 passed

tools/.venv/bin/pytest -q tools/mavlink_bridge
....................                                                     [100%]
20 passed in 0.03s

tools/.venv/bin/ruff check tools/mavlink_bridge tools/sitl
All checks passed!

tools/sitl/run.sh
{"continuous_samples": 15, "ekf_flags": 831, "max_tracking_error_m": 0.07836298428997139, "position_error_m": 0.07836298428997139, "type": "ACCEPTANCE"}
```

## Remaining ASSUMED / UNVERIFIED

- Accuracy inflation rates and 1 s/3 s degradation thresholds remain explicit conservative
  policy choices pending integrity-owner approval.
- The HDOP proxy is an operational compatibility mapping, not a geometric DOP calculation;
  system integrity review must approve it for sea trials.
- Dynamically consistent moving-vehicle fusion performance and alternative Case-A actions
  (MANUAL/disarm) are **[UNVERIFIED]**.
