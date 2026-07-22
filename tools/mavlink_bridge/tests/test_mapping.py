import math

import pymap3d as pm
from pymavlink.dialects.v20 import common as mavlink2
import pytest

from mavlink_bridge.mapping import SolutionEpoch, encode_yaw, map_epoch


def epoch(**changes):
    ecef = pm.geodetic2ecef(51.4778, -0.0014, 45.0, deg=True)
    values = {
        "monotonic_ns": 10_000_000_000,
        "position_ecef_m": tuple(ecef),
        "horizontal_velocity_ned_mps": (2.5, -1.25),
        "heading_rad": math.radians(90),
        "steering_authorised": True,
        "horiz_accuracy_m": 1.2,
        "speed_accuracy_mps": 0.15,
        "vert_accuracy_m": 2.4,
        "msl_alt_m": 7.5,
    }
    values.update(changes)
    return SolutionEpoch(**values)


def test_ecef_is_wgs84_and_altitude_is_explicit_msl():
    message = map_epoch(epoch(), 10_000_000_000)
    assert message.lat == pytest.approx(51.4778e7, abs=1)
    assert message.lon == pytest.approx(-0.0014e7, abs=1)
    assert message.alt == 7.5  # never substitute the 45 m ellipsoid height


def test_ned_velocity_accuracy_and_vertical_policy_map_independently():
    message = map_epoch(epoch(), 10_000_000_000)
    assert (message.vn, message.ve, message.vd) == (2.5, -1.25, 0.0)
    assert message.horiz_accuracy == 1.2
    assert message.speed_accuracy == 0.15
    assert message.vert_accuracy == 2.4
    assert not message.ignore_flags & mavlink2.GPS_INPUT_IGNORE_FLAG_VEL_VERT
    assert not message.ignore_flags & mavlink2.GPS_INPUT_IGNORE_FLAG_VEL_HORIZ


@pytest.mark.parametrize(
    ("radians", "encoded"),
    [(0.0, 36000), (2 * math.pi, 36000), (-2 * math.pi, 36000), (math.pi / 2, 9000), (None, 65535)],
)
def test_yaw_encoding_including_north_wrap(radians, encoded):
    assert encode_yaw(radians) == encoded


def test_stale_fill_inflates_uncertainty_and_degrades_fix():
    stale = map_epoch(epoch(), 12_000_000_000)
    assert stale.fix_type == 2
    assert stale.horiz_accuracy == pytest.approx(5.2)
    assert stale.speed_accuracy == pytest.approx(0.65)
    assert stale.vert_accuracy == pytest.approx(6.4)
    assert stale.lat == map_epoch(epoch(), 10_000_000_000).lat


def test_expired_fill_is_no_fix_and_marks_velocity_and_yaw_unavailable():
    expired = map_epoch(epoch(), 14_000_000_001)
    assert expired.fix_type == 1
    assert expired.ignore_flags & mavlink2.GPS_INPUT_IGNORE_FLAG_VEL_HORIZ
    assert expired.yaw == 65535


def test_revoked_authority_never_claims_a_fix():
    assert map_epoch(epoch(steering_authorised=False), 10_000_000_000).fix_type == 1


@pytest.mark.parametrize("name", ["horiz_accuracy_m", "speed_accuracy_mps", "vert_accuracy_m"])
def test_required_accuracies_must_be_positive(name):
    with pytest.raises(ValueError, match=name):
        epoch(**{name: 0.0}).validate()
