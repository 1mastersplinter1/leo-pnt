from __future__ import annotations

import argparse
import json
import math
import time

import pymap3d as pm


def generate(lat_deg: float, lon_deg: float, alt_m: float, speed_mps: float, count: int):
    start_ns = time.monotonic_ns()
    for index in range(count):
        elapsed = index / 5.0
        north_m = speed_mps * elapsed
        lat, lon, ellipsoid_alt = pm.enu2geodetic(
            0.0, north_m, 0.0, lat_deg, lon_deg, alt_m, deg=True
        )
        ecef = pm.geodetic2ecef(lat, lon, ellipsoid_alt, deg=True)
        yield {
            "monotonic_ns": start_ns + round(elapsed * 1_000_000_000),
            "state": {
                "position_ecef_m": list(ecef),
                "horizontal_velocity_ned_mps": [speed_mps, 0.0],
                "heading_rad": 0.0,
                "receiver_clock_bias_m": 0.0,
                "receiver_clock_drift_mps": 0.0,
            },
            "steering_authorised": True,
            "horiz_accuracy_m": 0.8,
            "speed_accuracy_mps": 0.1,
            "vert_accuracy_m": 1.5,
            "msl_alt_m": alt_m,
        }


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate a 5 Hz straight north trajectory")
    parser.add_argument("--lat", type=float, default=-35.363261)
    parser.add_argument("--lon", type=float, default=149.165230)
    parser.add_argument("--alt", type=float, default=584.0)
    parser.add_argument("--speed", type=float, default=1.0)
    parser.add_argument("--duration", type=float, default=30.0)
    parser.add_argument("--realtime", action="store_true")
    args = parser.parse_args()
    count = math.ceil(args.duration * 5)
    started = time.monotonic()
    for index, epoch in enumerate(generate(args.lat, args.lon, args.alt, args.speed, count)):
        if args.realtime:
            time.sleep(max(0, started + index / 5.0 - time.monotonic()))
        print(json.dumps(epoch), flush=True)


if __name__ == "__main__":
    main()
