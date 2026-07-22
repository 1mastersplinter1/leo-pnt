#!/usr/bin/env python3
"""Launch pinned Rover SITL, inject bridge messages, and assert observable acceptance."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import subprocess
import sys
import time

from pymavlink import mavutil

from mavlink_bridge.mapping import map_epoch
from mavlink_bridge.synthetic import generate

EKF_POS_HORIZ_ABS = 1 << 4
EKF_CONST_POS_MODE = 1 << 7


def distance_m(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    radius = 6_371_008.8
    p1, p2 = math.radians(lat1), math.radians(lat2)
    dp = p2 - p1
    dl = math.radians(lon2 - lon1)
    a = math.sin(dp / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2
    return 2 * radius * math.asin(math.sqrt(a))


def set_param(connection, name: str, value: float) -> None:
    connection.mav.param_set_send(
        connection.target_system,
        connection.target_component,
        name.encode(),
        value,
        mavutil.mavlink.MAV_PARAM_TYPE_REAL32,
    )
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        reply = connection.recv_match(type="PARAM_VALUE", blocking=True, timeout=1)
        if reply and reply.param_id == name and abs(reply.param_value - value) < 0.01:
            return
    raise AssertionError(f"SITL did not confirm {name}={value}")


def run(binary: Path, evidence: Path, duration: float, tolerance_m: float, speed_mps: float) -> None:
    evidence.mkdir(parents=True, exist_ok=True)
    sitl_log = (evidence / "sitl.log").open("w", encoding="utf-8")
    process = subprocess.Popen(
        [
            str(binary),
            "--model",
            "rover",
            "--speedup",
            "1",
            "--wipe",
            "--defaults",
            str(Path(__file__).resolve().parent / "params.parm"),
        ],
        cwd=evidence,
        stdout=sitl_log,
        stderr=subprocess.STDOUT,
    )
    records = []
    try:
        connection = mavutil.mavlink_connection("tcp:127.0.0.1:5760", source_system=245)
        if not connection.wait_heartbeat(timeout=30):
            raise AssertionError("no SITL heartbeat within 30 seconds")
        for name, value in (("FRAME_CLASS", 2), ("GPS1_TYPE", 14), ("GPS2_TYPE", 0)):
            set_param(connection, name, value)

        epochs = generate(-35.363261, 149.165230, 584.0, speed_mps, math.ceil(duration * 5))
        latest_global = None
        ekf_flags = 0
        raw_accuracy_seen = False
        started = time.monotonic()
        final_epoch = None
        for index, epoch_dict in enumerate(epochs):
            deadline = started + index / 5
            time.sleep(max(0.0, deadline - time.monotonic()))
            from mavlink_bridge.mapping import SolutionEpoch

            final_epoch = SolutionEpoch.from_dict(epoch_dict)
            gps = map_epoch(final_epoch, final_epoch.monotonic_ns)
            gps.send(connection)
            while True:
                message = connection.recv_match(blocking=False)
                if message is None:
                    break
                kind = message.get_type()
                if kind == "GLOBAL_POSITION_INT":
                    latest_global = (message.lat / 1e7, message.lon / 1e7)
                elif kind == "EKF_STATUS_REPORT":
                    ekf_flags = int(message.flags)
                elif kind == "GPS_RAW_INT":
                    h_acc = getattr(message, "h_acc", 0)
                    raw_accuracy_seen |= message.fix_type == 3 and 700 <= h_acc <= 900
                    records.append({"type": kind, "fix_type": message.fix_type, "h_acc": h_acc})
        if final_epoch is None or latest_global is None:
            raise AssertionError("no injected epoch or GLOBAL_POSITION_INT observed")
        expected = map_epoch(final_epoch, final_epoch.monotonic_ns)
        error = distance_m(expected.lat / 1e7, expected.lon / 1e7, *latest_global)
        records.append({"type": "OBSERVED", "position_error_m": error, "ekf_flags": ekf_flags})
        (evidence / "mavlink.jsonl").write_text(
            "".join(json.dumps(record, sort_keys=True) + "\n" for record in records), encoding="utf-8"
        )
        if error > tolerance_m:
            raise AssertionError(f"position error {error:.2f} m exceeds {tolerance_m:.2f} m")
        if not (ekf_flags & EKF_POS_HORIZ_ABS) or ekf_flags & EKF_CONST_POS_MODE:
            raise AssertionError(f"EKF did not report absolute horizontal position: flags={ekf_flags}")
        if not raw_accuracy_seen:
            raise AssertionError("GPS_RAW_INT did not expose injected fix_type=3 and h_acc=800 mm")
        records.append({"type": "ACCEPTANCE", "position_error_m": error, "ekf_flags": ekf_flags})
        (evidence / "mavlink.jsonl").write_text(
            "".join(json.dumps(record, sort_keys=True) + "\n" for record in records), encoding="utf-8"
        )
        print(json.dumps(records[-1], sort_keys=True))
    finally:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
        sitl_log.close()


def main() -> None:
    here = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=here / "ardupilot/build/sitl/bin/ardurover")
    parser.add_argument("--evidence", type=Path, default=here / "evidence")
    parser.add_argument("--duration", type=float, default=45)
    parser.add_argument("--tolerance-m", type=float, default=10)
    parser.add_argument("--speed-mps", type=float, default=0.1)
    args = parser.parse_args()
    if not args.binary.is_file():
        sys.exit(f"missing SITL binary: run {here / 'build.sh'} first")
    run(args.binary, args.evidence, args.duration, args.tolerance_m, args.speed_mps)


if __name__ == "__main__":
    main()
