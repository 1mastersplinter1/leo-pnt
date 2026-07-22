import pytest

from mavlink_bridge.mapping import SolutionEpoch


def valid_dict():
    return {
        "monotonic_ns": 42,
        "state": {
            "position_ecef_m": [1, 2, 3],
            "horizontal_velocity_ned_mps": [4, 5],
            "heading_rad": 0,
            "receiver_clock_bias_m": 6,
            "receiver_clock_drift_mps": 7,
        },
        "steering_authorised": True,
        "horiz_accuracy_m": 1,
        "speed_accuracy_mps": 2,
        "vert_accuracy_m": 3,
        "msl_alt_m": 4,
    }


def test_schema_accepts_solution_epoch_shape():
    assert SolutionEpoch.from_dict(valid_dict()).horizontal_velocity_ned_mps == (4.0, 5.0)


def test_schema_requires_accuracy():
    value = valid_dict()
    del value["horiz_accuracy_m"]
    with pytest.raises(KeyError):
        SolutionEpoch.from_dict(value)

