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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FilterState {
    pub position_ecef_m: [f64; 3],
    pub horizontal_velocity_ned_mps: [f64; 2],
    pub heading_rad: f64,
    pub receiver_clock_bias_m: f64,
    pub receiver_clock_drift_mps: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolutionEpoch {
    pub monotonic_ns: u64,
    pub state: FilterState,
    pub steering_authorised: bool,
}
