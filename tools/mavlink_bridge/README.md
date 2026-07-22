# MAVLink GPS_INPUT bridge

This Python 3 package consumes one JSON object per line on standard input and emits
MAVLink 2 `GPS_INPUT` (message 232) at 5 Hz. It uses `pymavlink` 2.4.43 for transport and
`pymap3d` 3.2.0 for WGS 84 (EPSG:4978 ECEF to EPSG:4979 geodetic) conversion. The ECEF
ellipsoid height is deliberately discarded: `alt` is the estimator's separate
MSL-constrained `msl_alt_m` value. Velocity is NED and `vd` is always zero.

The [MAVLink common message specification](https://mavlink.io/en/messages/common.html#GPS_INPUT)
defines `yaw` as clockwise from Earth north in centidegrees: zero means unavailable and
36000 must represent north. This bridge uses 65535 when a configured heading is temporarily
unavailable, as prescribed by the current extension-field definition. ArduPilot documents
that [GPS1_TYPE must be 14](https://ardupilot.org/mavproxy/docs/modules/GPSInput.html).

## Input schema

Every line has this exact shape (JSON numbers must be finite):

```json
{
  "monotonic_ns": 123456789000,
  "state": {
    "position_ecef_m": [-4479000.0, 2670000.0, -3660000.0],
    "horizontal_velocity_ned_mps": [1.0, 0.0],
    "heading_rad": 0.0,
    "receiver_clock_bias_m": 0.0,
    "receiver_clock_drift_mps": 0.0
  },
  "steering_authorised": true,
  "horiz_accuracy_m": 0.8,
  "speed_accuracy_mps": 0.1,
  "vert_accuracy_m": 1.5,
  "msl_alt_m": 584.0
}
```

The first three fields mirror v2 `SolutionEpoch` and all five `FilterState` members are
carried in the JSON contract (the bridge currently needs position, horizontal velocity and
heading). `heading_rad` may be `null`. The three positive accuracy fields and MSL altitude
extend the current Rust type pending the estimator unit; all are required. Accuracy means
a one-standard-deviation bound in the units named by the field.

Fresh authorised epochs claim fix type 3. After 1 second without a new epoch the repeated
fill degrades to fix type 2; after 3 seconds it reports no fix, ignores horizontal velocity,
and marks yaw unavailable. A revoked `steering_authorised` value reports no fix immediately.
Horizontal and vertical position accuracy grow by 2 m/s of age and speed accuracy by
0.25 m/s per second of age. Publication itself continues, preventing transport silence;
the default 5 Hz interval is far inside the baseline's estimated 4-second timeout.
HDOP/VDOP are ignored. Vertical velocity is supplied as zero, exactly as required by the
baseline. `ODOMETRY` is never emitted.

## Run and test

```sh
DISABLE_MAVNATIVE=1 tools/.venv/bin/pip install -e tools/mavlink_bridge
tools/.venv/bin/pytest tools/mavlink_bridge
tools/.venv/bin/ruff check tools/mavlink_bridge
tools/.venv/bin/python -m mavlink_bridge.synthetic --realtime --duration 30 |
  tools/.venv/bin/python -m mavlink_bridge.cli --connect udpout:127.0.0.1:14550 --stop-after-eof 4
```
