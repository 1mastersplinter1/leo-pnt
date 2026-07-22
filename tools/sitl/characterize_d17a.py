#!/usr/bin/env python3
"""Characterise Rover behaviour when an aided GPS_INPUT stream becomes silent."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import subprocess
import time

from pymavlink import mavutil

from mavlink_bridge.mapping import SolutionEpoch, map_epoch
from mavlink_bridge.synthetic import generate
from run_acceptance import EKF_CONST_POS_MODE, EKF_POS_HORIZ_ABS, set_param


def record_message(records: list[dict], message, start: float, phase: str) -> None:
    kind = message.get_type()
    base = {"elapsed_s": round(time.monotonic() - start, 3), "phase": phase, "type": kind}
    if kind == "HEARTBEAT":
        base.update(
            armed=bool(message.base_mode & mavutil.mavlink.MAV_MODE_FLAG_SAFETY_ARMED),
            custom_mode=int(message.custom_mode),
        )
    elif kind == "EKF_STATUS_REPORT":
        flags = int(message.flags)
        base.update(
            flags=flags,
            horiz_abs=bool(flags & EKF_POS_HORIZ_ABS),
            const_pos=bool(flags & EKF_CONST_POS_MODE),
        )
    elif kind == "GPS_RAW_INT":
        base.update(fix_type=int(message.fix_type), h_acc=int(getattr(message, "h_acc", 0)))
    elif kind == "SERVO_OUTPUT_RAW":
        base.update(
            servo1=int(message.servo1_raw),
            servo3=int(message.servo3_raw),
        )
    elif kind == "VFR_HUD":
        base.update(throttle=int(message.throttle), groundspeed=round(float(message.groundspeed), 3))
    elif kind == "STATUSTEXT":
        base.update(text=message.text)
    elif kind == "COMMAND_ACK":
        base.update(command=int(message.command), result=int(message.result))
    else:
        return
    records.append(base)


def drain(connection, records: list[dict], start: float, phase: str) -> int:
    flags = 0
    while True:
        message = connection.recv_match(blocking=False)
        if message is None:
            return flags
        record_message(records, message, start, phase)
        if message.get_type() == "EKF_STATUS_REPORT":
            flags = int(message.flags)


def set_mode(connection, mode: str) -> None:
    mode_id = connection.mode_mapping()[mode]
    connection.mav.set_mode_send(
        connection.target_system,
        mavutil.mavlink.MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
        mode_id,
    )


def run(
    binary: Path,
    evidence: Path,
    aid_timeout: float,
    silence_s: float,
    hold_delay_s: float,
    label: str,
) -> None:
    evidence.mkdir(parents=True, exist_ok=True)
    sitl_log = (evidence / "d17a-sitl.log").open("w", encoding="utf-8")
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
    records: list[dict] = []
    try:
        connection = mavutil.mavlink_connection("tcp:127.0.0.1:5760", source_system=245)
        if not connection.wait_heartbeat(timeout=30):
            raise AssertionError("no SITL heartbeat")
        for name, value in (("FRAME_CLASS", 2), ("GPS1_TYPE", 14), ("GPS2_TYPE", 0)):
            set_param(connection, name, value)
        connection.mav.request_data_stream_send(
            connection.target_system,
            connection.target_component,
            mavutil.mavlink.MAV_DATA_STREAM_ALL,
            10,
            1,
        )

        epochs = generate(-35.363261, 149.165230, 584.0, 0.0, math.ceil(aid_timeout * 5))
        start = time.monotonic()
        aided = False
        for index, epoch_dict in enumerate(epochs):
            time.sleep(max(0.0, start + index / 5 - time.monotonic()))
            epoch = SolutionEpoch.from_dict(epoch_dict)
            map_epoch(epoch, epoch.monotonic_ns).send(connection)
            flags = drain(connection, records, start, "aiding")
            aided |= bool(flags & EKF_POS_HORIZ_ABS and not flags & EKF_CONST_POS_MODE)
            if aided and time.monotonic() - start >= 35:
                break
        if not aided:
            raise AssertionError("EKF did not become aided before D17a experiment")

        set_mode(connection, "GUIDED")
        connection.mav.command_long_send(
            connection.target_system,
            connection.target_component,
            mavutil.mavlink.MAV_CMD_COMPONENT_ARM_DISARM,
            0,
            1,
            21196,
            0,
            0,
            0,
            0,
            0,
        )
        transition = time.monotonic()
        while time.monotonic() - transition < 3:
            drain(connection, records, start, "guided-armed")
            time.sleep(0.05)

        silence_start = time.monotonic()
        records.append({"elapsed_s": round(silence_start - start, 3), "type": "STREAM_STOP"})
        hold_sent = None
        while time.monotonic() - silence_start < silence_s:
            if hold_sent is None and time.monotonic() - silence_start >= hold_delay_s:
                hold_sent = time.monotonic()
                records.append(
                    {"elapsed_s": round(hold_sent - start, 3), "type": "HOLD_COMMAND_SENT"}
                )
                set_mode(connection, "HOLD")
            drain(connection, records, start, "gps-input-silent")
            time.sleep(0.05)

        if hold_sent is None:
            raise AssertionError("companion HOLD delay exceeds experiment duration")
        transition = time.monotonic()
        while time.monotonic() - transition < 3:
            drain(connection, records, start, "companion-hold")
            time.sleep(0.05)

        (evidence / f"d17a-{label}-mavlink.jsonl").write_text(
            "".join(json.dumps(record, sort_keys=True) + "\n" for record in records),
            encoding="utf-8",
        )
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
    parser.add_argument("--aid-timeout", type=float, default=50)
    parser.add_argument("--silence-s", type=float, default=20)
    parser.add_argument("--hold-delay-s", type=float, default=20)
    parser.add_argument("--label", default="native")
    args = parser.parse_args()
    run(
        args.binary,
        args.evidence,
        args.aid_timeout,
        args.silence_s,
        args.hold_delay_s,
        args.label,
    )


if __name__ == "__main__":
    main()
