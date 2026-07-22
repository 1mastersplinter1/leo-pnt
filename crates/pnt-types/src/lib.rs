//! Shared, versioned measurement-bus types.

pub const SCHEMA_VERSION: u16 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeTag {
    DeviceNanoseconds(u64),
    HostMonotonicNanoseconds(u64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct UtcTime {
    pub rfc3339: String,
    pub uncertainty_ns: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Frame {
    EarthCenteredEarthFixed,
    LocalNorthEastDown,
    VesselReference,
    Sensor,
    AntennaPhaseCenter,
    FrameIndependent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QualityFlags(pub u32);

impl QualityFlags {
    pub const VALID: Self = Self(1);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Provenance {
    CaptureRecord(String),
    SourceRecord(String),
    DerivedRecord(String),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImuSample {
    pub acceleration_mps2: [f64; 3],
    pub angular_rate_rps: [f64; 3],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Heading {
    pub radians: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpeedThroughWater {
    pub metres_per_second: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GnssFix {
    pub position_ecef_m: [f64; 3],
    pub velocity_ned_mps: [f64; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Constellation {
    Starlink,
    Iridium,
    OneWeb,
    Orbcomm,
}

/// Identifies a physically independent receiver clock domain.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReceiverClockId(pub String);

/// Slot reserved by the estimator for a receiver's clock bias and drift.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiverClockSlot {
    pub bias_index: usize,
    pub drift_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmAction {
    Arm,
    Disarm,
}

/// Human helm command. The executive, not the estimator, routes this message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArmCommand {
    pub action: ArmAction,
    pub host_monotonic_ns: u64,
    pub source_id: SourceId,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackerDoppler {
    pub constellation: Constellation,
    pub correlation_peak_hz: f64,
    pub nominal_carrier_hz: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MeasurementPayload {
    Imu(ImuSample),
    Heading(Heading),
    SpeedThroughWater(SpeedThroughWater),
    Gnss(GnssFix),
    TrackerDoppler(TrackerDoppler),
    ArmCommand(ArmCommand),
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeasurementEnvelope {
    pub schema_version: u16,
    pub source_id: SourceId,
    pub sequence: u64,
    pub sample_time: TimeTag,
    pub host_receive_monotonic_ns: u64,
    pub utc: Option<UtcTime>,
    pub payload: MeasurementPayload,
    pub frame: Frame,
    pub covariance: Vec<f64>,
    pub quality: QualityFlags,
    pub calibration_id: String,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilterState {
    pub position_ecef_m: [f64; 3],
    pub velocity_ecef_mps: [f64; 3],
    pub horizontal_velocity_ned_mps: [f64; 2],
    pub heading_rad: f64,
    pub receiver_clock_bias_m: f64,
    pub receiver_clock_drift_mps: f64,
    /// Row-major covariance. Its dimension is `covariance_dimension`; dynamic
    /// nuisance states follow the nine core states.
    pub covariance: Vec<f64>,
    pub covariance_dimension: usize,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            position_ecef_m: [0.0; 3],
            velocity_ecef_mps: [0.0; 3],
            horizontal_velocity_ned_mps: [0.0; 2],
            heading_rad: 0.0,
            receiver_clock_bias_m: 0.0,
            receiver_clock_drift_mps: 0.0,
            covariance: (0..Self::CORE_DIMENSION)
                .flat_map(|row| {
                    (0..Self::CORE_DIMENSION).map(move |column| f64::from(u8::from(row == column)))
                })
                .collect(),
            covariance_dimension: Self::CORE_DIMENSION,
        }
    }
}

impl FilterState {
    pub const CORE_DIMENSION: usize = 9;

    #[must_use]
    pub fn horizontal_accuracy_m(&self) -> f64 {
        covariance_accuracy(&self.covariance, self.covariance_dimension, &[0, 1])
    }

    #[must_use]
    pub fn speed_accuracy_mps(&self) -> f64 {
        covariance_accuracy(&self.covariance, self.covariance_dimension, &[3, 4])
    }

    #[must_use]
    pub fn vertical_accuracy_m(&self) -> f64 {
        covariance_accuracy(&self.covariance, self.covariance_dimension, &[2])
    }
}

fn covariance_accuracy(covariance: &[f64], dimension: usize, indices: &[usize]) -> f64 {
    indices
        .iter()
        .filter_map(|index| covariance.get(index * dimension + index))
        .copied()
        .fold(0.0_f64, f64::max)
        .max(0.0)
        .sqrt()
}

#[derive(Clone, Debug, PartialEq)]
pub struct SolutionEpoch {
    pub monotonic_ns: u64,
    pub state: FilterState,
    pub steering_authorised: bool,
}

impl SolutionEpoch {
    #[must_use]
    pub fn horizontal_accuracy_m(&self) -> f64 {
        self.state.horizontal_accuracy_m()
    }

    #[must_use]
    pub fn speed_accuracy_mps(&self) -> f64 {
        self.state.speed_accuracy_mps()
    }

    #[must_use]
    pub fn vertical_accuracy_m(&self) -> f64 {
        self.state.vertical_accuracy_m()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_accuracies_are_derived_from_its_full_covariance() {
        let mut covariance = vec![0.0; FilterState::CORE_DIMENSION.pow(2)];
        covariance[0] = 9.0;
        covariance[FilterState::CORE_DIMENSION + 1] = 16.0;
        covariance[2 * FilterState::CORE_DIMENSION + 2] = 25.0;
        covariance[3 * FilterState::CORE_DIMENSION + 3] = 0.04;
        covariance[4 * FilterState::CORE_DIMENSION + 4] = 0.09;
        let epoch = SolutionEpoch {
            monotonic_ns: 1,
            state: FilterState {
                covariance,
                covariance_dimension: FilterState::CORE_DIMENSION,
                ..FilterState::default()
            },
            steering_authorised: false,
        };
        assert!((epoch.horizontal_accuracy_m() - 5.0).abs() < f64::EPSILON);
        assert!((epoch.speed_accuracy_mps() - 0.360_555_127_546_398_9).abs() < f64::EPSILON);
        assert!((epoch.vertical_accuracy_m() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn accuracies_rotate_ecef_covariance_to_local_frame_at_high_latitude() {
        let latitude = 56.0_f64.to_radians();
        let longitude = 12.0_f64.to_radians();
        let radius = 6_378_137.0;
        let mut state = FilterState {
            position_ecef_m: [
                radius * latitude.cos() * longitude.cos(),
                radius * latitude.cos() * longitude.sin(),
                radius * latitude.sin(),
            ],
            velocity_ecef_mps: [2.0, -3.0, 4.0],
            covariance: vec![0.0; FilterState::CORE_DIMENSION.pow(2)],
            ..FilterState::default()
        };
        let covariance_ecef = [[9.0, 1.5, -0.5], [1.5, 16.0, 2.0], [-0.5, 2.0, 25.0]];
        for row in 0..3 {
            for column in 0..3 {
                state.covariance[row * FilterState::CORE_DIMENSION + column] =
                    covariance_ecef[row][column];
                state.covariance[(row + 3) * FilterState::CORE_DIMENSION + column + 3] =
                    covariance_ecef[row][column] / 100.0;
            }
        }

        let east = [-longitude.sin(), longitude.cos(), 0.0];
        let north = [
            -latitude.sin() * longitude.cos(),
            -latitude.sin() * longitude.sin(),
            latitude.cos(),
        ];
        let up = [
            latitude.cos() * longitude.cos(),
            latitude.cos() * longitude.sin(),
            latitude.sin(),
        ];
        let projected_variance = |axis: [f64; 3]| {
            (0..3)
                .flat_map(|row| (0..3).map(move |column| (row, column)))
                .map(|(row, column)| axis[row] * covariance_ecef[row][column] * axis[column])
                .sum::<f64>()
        };
        let expected_horizontal =
            (projected_variance(east) + projected_variance(north)).sqrt();
        let expected_vertical = projected_variance(up).sqrt();

        assert!((state.horizontal_accuracy_m() - expected_horizontal).abs() < 1.0e-12);
        assert!((state.vertical_accuracy_m() - expected_vertical).abs() < 1.0e-12);
        assert!((state.speed_accuracy_mps() - expected_horizontal / 10.0).abs() < 1.0e-12);
    }

    #[test]
    fn arm_command_is_a_bus_payload() {
        let command = ArmCommand {
            action: ArmAction::Disarm,
            host_monotonic_ns: 42,
            source_id: SourceId("helm".into()),
        };
        assert_eq!(
            MeasurementPayload::ArmCommand(command.clone()),
            MeasurementPayload::ArmCommand(command)
        );
    }
}
