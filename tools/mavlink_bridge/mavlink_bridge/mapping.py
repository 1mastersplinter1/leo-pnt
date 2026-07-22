"""Pure mapping and staleness policy for MAVLink GPS_INPUT (message 232)."""

from __future__ import annotations

from dataclasses import dataclass
import math
from typing import Any

import pymap3d as pm
from pymavlink.dialects.v20 import common as mavlink2

NANOSECONDS = 1_000_000_000
FRESH_NS = NANOSECONDS
NO_FIX_NS = 3 * NANOSECONDS
ACCURACY_GROWTH_MPS = 2.0
SPEED_ACCURACY_GROWTH_MPS2 = 0.25


@dataclass(frozen=True)
class SolutionEpoch:
    monotonic_ns: int
    position_ecef_m: tuple[float, float, float]
    horizontal_velocity_ned_mps: tuple[float, float]
    heading_rad: float | None
    steering_authorised: bool
    horiz_accuracy_m: float
    speed_accuracy_mps: float
    vert_accuracy_m: float
    msl_alt_m: float

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> SolutionEpoch:
        state = value["state"]
        epoch = cls(
            monotonic_ns=int(value["monotonic_ns"]),
            position_ecef_m=_vector(state["position_ecef_m"], 3, "position_ecef_m"),
            horizontal_velocity_ned_mps=_vector(
                state["horizontal_velocity_ned_mps"], 2, "horizontal_velocity_ned_mps"
            ),
            heading_rad=None if state.get("heading_rad") is None else float(state["heading_rad"]),
            steering_authorised=bool(value["steering_authorised"]),
            horiz_accuracy_m=float(value["horiz_accuracy_m"]),
            speed_accuracy_mps=float(value["speed_accuracy_mps"]),
            vert_accuracy_m=float(value["vert_accuracy_m"]),
            msl_alt_m=float(value["msl_alt_m"]),
        )
        epoch.validate()
        return epoch

    def validate(self) -> None:
        scalars = (*self.position_ecef_m, *self.horizontal_velocity_ned_mps, self.msl_alt_m)
        if not all(math.isfinite(item) for item in scalars):
            raise ValueError("position, velocity, and altitude must be finite")
        if self.heading_rad is not None and not math.isfinite(self.heading_rad):
            raise ValueError("heading_rad must be finite or null")
        for name in ("horiz_accuracy_m", "speed_accuracy_mps", "vert_accuracy_m"):
            value = getattr(self, name)
            if not math.isfinite(value) or value <= 0:
                raise ValueError(f"{name} must be finite and greater than zero")


def _vector(value: Any, length: int, name: str) -> tuple:
    if not isinstance(value, (list, tuple)) or len(value) != length:
        raise ValueError(f"{name} must contain exactly {length} numbers")
    return tuple(float(item) for item in value)


@dataclass(frozen=True)
class GpsInput:
    time_usec: int
    gps_id: int
    ignore_flags: int
    time_week_ms: int
    time_week: int
    fix_type: int
    lat: int
    lon: int
    alt: float
    hdop: float
    vdop: float
    vn: float
    ve: float
    vd: float
    speed_accuracy: float
    horiz_accuracy: float
    vert_accuracy: float
    satellites_visible: int
    yaw: int

    def send(self, connection: Any) -> None:
        connection.mav.gps_input_send(**self.__dict__)


def encode_yaw(heading_rad: float | None) -> int:
    """Encode radians clockwise from north as MAVLink centidegrees."""
    if heading_rad is None:
        return 65535
    centidegrees = round(math.degrees(heading_rad) % 360.0 * 100.0)
    if centidegrees in (0, 36000):
        return 36000
    return centidegrees


def map_epoch(epoch: SolutionEpoch, now_monotonic_ns: int, gps_id: int = 0) -> GpsInput:
    epoch.validate()
    age_ns = max(0, now_monotonic_ns - epoch.monotonic_ns)
    age_s = age_ns / NANOSECONDS
    lat_deg, lon_deg, _ellipsoid_alt_m = pm.ecef2geodetic(*epoch.position_ecef_m, deg=True)
    ignore = (
        mavlink2.GPS_INPUT_IGNORE_FLAG_HDOP
        | mavlink2.GPS_INPUT_IGNORE_FLAG_VDOP
    )
    fix_type = 3 if epoch.steering_authorised else 1
    yaw = encode_yaw(epoch.heading_rad)
    if age_ns > FRESH_NS:
        fix_type = min(fix_type, 2)
    if age_ns > NO_FIX_NS:
        fix_type = 1
        ignore |= mavlink2.GPS_INPUT_IGNORE_FLAG_VEL_HORIZ
        yaw = 65535

    return GpsInput(
        time_usec=now_monotonic_ns // 1000,
        gps_id=gps_id,
        ignore_flags=ignore,
        time_week_ms=0,
        time_week=0,
        fix_type=fix_type,
        lat=round(lat_deg * 10_000_000),
        lon=round(lon_deg * 10_000_000),
        alt=epoch.msl_alt_m,
        hdop=65535.0,
        vdop=65535.0,
        vn=epoch.horizontal_velocity_ned_mps[0],
        ve=epoch.horizontal_velocity_ned_mps[1],
        vd=0.0,
        speed_accuracy=epoch.speed_accuracy_mps + SPEED_ACCURACY_GROWTH_MPS2 * age_s,
        horiz_accuracy=epoch.horiz_accuracy_m + ACCURACY_GROWTH_MPS * age_s,
        vert_accuracy=epoch.vert_accuracy_m + ACCURACY_GROWTH_MPS * age_s,
        satellites_visible=10 if fix_type >= 3 else 0,
        yaw=yaw,
    )
