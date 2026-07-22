//! Estimator port and observable propagation stub.

use pnt_types::{FilterState, ImuSample, MeasurementEnvelope};

pub trait Estimator {
    fn propagate(&mut self, imu: ImuSample);
    fn update(&mut self, measurement: &MeasurementEnvelope);
    fn state(&self) -> FilterState;
}

#[derive(Debug, Default)]
pub struct FilterStub {
    state: FilterState,
    propagations: u64,
    covariance_growth_count: u64,
    measurement_updates: u64,
}

impl FilterStub {
    #[must_use]
    pub const fn propagations(&self) -> u64 {
        self.propagations
    }

    #[must_use]
    pub const fn covariance_growth_count(&self) -> u64 {
        self.covariance_growth_count
    }

    #[must_use]
    pub const fn measurement_updates(&self) -> u64 {
        self.measurement_updates
    }
}

impl Estimator for FilterStub {
    fn propagate(&mut self, imu: ImuSample) {
        self.propagations += 1;
        self.covariance_growth_count += 1;
        self.state.horizontal_velocity_ned_mps[0] += imu.acceleration_mps2[0] / 100.0;
        self.state.horizontal_velocity_ned_mps[1] += imu.acceleration_mps2[1] / 100.0;
    }

    fn update(&mut self, _measurement: &MeasurementEnvelope) {
        self.measurement_updates += 1;
    }

    fn state(&self) -> FilterState {
        self.state
    }
}
