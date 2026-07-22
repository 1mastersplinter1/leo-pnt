from __future__ import annotations

import argparse
import json
import selectors
import sys
import time

from pymavlink import mavutil

from .mapping import SolutionEpoch, map_epoch


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description="Publish JSON SolutionEpoch lines as GPS_INPUT")
    result.add_argument("--connect", default="udpout:127.0.0.1:14550")
    result.add_argument("--rate-hz", type=float, default=5.0)
    result.add_argument("--gps-id", type=int, default=0)
    result.add_argument("--stop-after-eof", type=float, metavar="SECONDS")
    return result


def run(args: argparse.Namespace) -> int:
    if args.rate_hz <= 0:
        raise ValueError("--rate-hz must be greater than zero")
    connection = mavutil.mavlink_connection(args.connect, source_system=245, source_component=191)
    selector = selectors.DefaultSelector()
    selector.register(sys.stdin, selectors.EVENT_READ)
    latest = None
    eof_at = None
    interval = 1.0 / args.rate_hz
    next_send = time.monotonic()
    while True:
        timeout = max(0.0, next_send - time.monotonic())
        if eof_at is None:
            for key, _mask in selector.select(timeout):
                line = key.fileobj.readline()
                if line:
                    try:
                        latest = SolutionEpoch.from_dict(json.loads(line))
                    except (ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
                        print(f"rejected input: {error}", file=sys.stderr)
                else:
                    eof_at = time.monotonic()
                    selector.unregister(sys.stdin)
        else:
            time.sleep(min(timeout, 0.02))
        now = time.monotonic()
        if now >= next_send:
            if latest is not None:
                message = map_epoch(latest, time.monotonic_ns(), args.gps_id)
                message.send(connection)
                print(json.dumps(message.__dict__, sort_keys=True), file=sys.stderr, flush=True)
            next_send += interval
            if next_send < now:
                next_send = now + interval
        if eof_at is not None and args.stop_after_eof is not None:
            if now - eof_at >= args.stop_after_eof:
                return 0


def main() -> None:
    raise SystemExit(run(parser().parse_args()))


if __name__ == "__main__":
    main()
