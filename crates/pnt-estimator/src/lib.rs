//! Error-state EKF for the nine-state navigation core and pass-scoped biases.

use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};
use pnt_types::{
    ecef_to_enu_rotation, ecef_vector_to_enu, FilterState, Heading, ImuSample, MeasurementEnvelope,
    MeasurementPayload, ReceiverClockId, ReceiverClockSlot, SpeedThroughWater,
};

const CORE_DIM: usize = 9;
const POS: usize = 0;
const VEL: usize = 3;
const HEADING: usize = 6;
const CLOCK_BIAS: usize = 7;
const CLOCK_DRIFT: usize = 8;
const CLOCK_BIAS_VARIANCE_CAP_M2: f64 = 1.0e8;

pub trait Estimator {
    fn propagate(&mut self, imu: ImuSample);
    fn update(&mut self, measurement: &MeasurementEnvelope);
    fn state(&self) -> FilterState;
}

#[derive(Clone, Copy, Debug)]
pub struct ProcessNoise {
    pub acceleration_variance: f64,
    pub turn_rate_variance: f64,
    pub clock_drift_variance: f64,
    pub nuisance_random_walk_variance: f64,
}

impl Default for ProcessNoise {
    fn default() -> Self {
        Self {
            acceleration_variance: 0.04,
            turn_rate_variance: 1.0e-4,
            clock_drift_variance: 1.0e-4,
            nuisance_random_walk_variance: 1.0e-6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UpdateResult {
    pub innovation: f64,
    pub innovation_variance: f64,
    pub normalized_innovation_squared: f64,
    pub accepted: bool,
}

#[derive(Clone, Debug)]
pub struct DopplerRangeRateUpdate {
    pub satellite_id: String,
    pub measured_range_rate_mps: f64,
    /// Predictor module output at the current receiver state.
    pub predicted_range_rate_mps: f64,
    /// Predictor linearisation with respect to the nine core states.
    pub core_jacobian: [f64; CORE_DIM],
    pub variance_mps2: f64,
    pub chi_square_threshold: Option<f64>,
    /// Initial variance for a newly observed satellite's range-rate bias.
    pub satellite_bias_variance_mps2: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct GnssUpdate {
    pub position_ecef_m: [f64; 3],
    pub velocity_ecef_mps: [f64; 3],
    pub position_variance_m2: [f64; 3],
    pub velocity_variance_mps2: [f64; 3],
    pub aided_mode: bool,
    pub chi_square_threshold: Option<f64>,
}

#[derive(Clone, Copy, Debug)]
pub struct MslAltitudeUpdate {
    pub altitude_m: f64,
    pub up_ecef: [f64; 3],
    pub variance_m2: f64,
    pub chi_square_threshold: Option<f64>,
}

/// Kept under the historical name so the unmodified executive remains source-compatible.
#[derive(Debug)]
pub struct FilterStub {
    x: DVector<f64>,
    covariance: DMatrix<f64>,
    process_noise: ProcessNoise,
    dt_seconds: f64,
    propagations: u64,
    covariance_growth_count: u64,
    measurement_updates: u64,
    nuisance_slots: HashMap<String, usize>,
    receiver_clocks: HashMap<ReceiverClockId, ReceiverClockSlot>,
}

impl Default for FilterStub {
    fn default() -> Self {
        Self::new(0.01, ProcessNoise::default())
    }
}

impl FilterStub {
    /// Constructs a filter with a fixed IMU propagation interval.
    ///
    /// # Panics
    ///
    /// Panics when `dt_seconds` is non-finite or not strictly positive.
    #[must_use]
    pub fn new(dt_seconds: f64, process_noise: ProcessNoise) -> Self {
        assert!(dt_seconds.is_finite() && dt_seconds > 0.0);
        Self {
            x: DVector::zeros(CORE_DIM),
            covariance: DMatrix::identity(CORE_DIM, CORE_DIM),
            process_noise,
            dt_seconds,
            propagations: 0,
            covariance_growth_count: 0,
            measurement_updates: 0,
            nuisance_slots: HashMap::new(),
            receiver_clocks: HashMap::new(),
        }
    }

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
    #[must_use]
    pub fn covariance(&self) -> &DMatrix<f64> {
        &self.covariance
    }

    pub fn reserve_receiver_clock(&mut self, id: ReceiverClockId) -> ReceiverClockSlot {
        if let Some(slot) = self.receiver_clocks.get(&id) {
            return *slot;
        }
        let bias_index = self.augment_state(0.0, 1.0e6);
        let drift_index = self.augment_state(0.0, 1.0e2);
        let slot = ReceiverClockSlot {
            bias_index,
            drift_index,
        };
        self.receiver_clocks.insert(id, slot);
        slot
    }

    #[must_use]
    pub fn receiver_clock_slot(&self, id: &ReceiverClockId) -> Option<ReceiverClockSlot> {
        self.receiver_clocks.get(id).copied()
    }

    pub fn augment_satellite_bias(
        &mut self,
        satellite_id: impl Into<String>,
        variance: f64,
    ) -> usize {
        let id = satellite_id.into();
        if let Some(index) = self.nuisance_slots.get(&id) {
            return *index;
        }
        let index = self.augment_state(0.0, variance);
        self.nuisance_slots.insert(id, index);
        index
    }

    pub fn retire_satellite_bias(&mut self, satellite_id: &str) -> bool {
        let Some(index) = self.nuisance_slots.remove(satellite_id) else {
            return false;
        };
        self.remove_state(index);
        for slot in self.nuisance_slots.values_mut() {
            if *slot > index {
                *slot -= 1;
            }
        }
        for slot in self.receiver_clocks.values_mut() {
            if slot.bias_index > index {
                slot.bias_index -= 1;
            }
            if slot.drift_index > index {
                slot.drift_index -= 1;
            }
        }
        true
    }

    pub fn update_doppler(&mut self, update: &DopplerRangeRateUpdate) -> UpdateResult {
        let nuisance =
            self.augment_satellite_bias(&update.satellite_id, update.satellite_bias_variance_mps2);
        let mut h = DVector::zeros(self.x.len());
        h.rows_mut(0, CORE_DIM)
            .copy_from(&DVector::from_column_slice(&update.core_jacobian));
        h[nuisance] = 1.0;
        let predicted = update.predicted_range_rate_mps + self.x[nuisance];
        self.scalar_update(
            update.measured_range_rate_mps,
            predicted,
            &h,
            update.variance_mps2,
            update.chi_square_threshold,
        )
    }

    /// Applies a predictor result in an independent receiver clock domain.
    pub fn update_doppler_for_receiver(
        &mut self,
        update: &DopplerRangeRateUpdate,
        receiver: ReceiverClockId,
    ) -> UpdateResult {
        let slot = self.reserve_receiver_clock(receiver);
        let nuisance =
            self.augment_satellite_bias(&update.satellite_id, update.satellite_bias_variance_mps2);
        let mut h = DVector::zeros(self.x.len());
        h.rows_mut(0, CORE_DIM)
            .copy_from(&DVector::from_column_slice(&update.core_jacobian));
        h[slot.drift_index] += h[CLOCK_DRIFT];
        h[CLOCK_DRIFT] = 0.0;
        h[nuisance] = 1.0;
        let predicted = update.predicted_range_rate_mps
            + update.core_jacobian[CLOCK_DRIFT] * self.x[slot.drift_index]
            + self.x[nuisance];
        self.scalar_update(
            update.measured_range_rate_mps,
            predicted,
            &h,
            update.variance_mps2,
            update.chi_square_threshold,
        )
    }

    pub fn update_heading(
        &mut self,
        heading: Heading,
        variance: f64,
        gate: Option<f64>,
    ) -> UpdateResult {
        let mut h = DVector::zeros(self.x.len());
        h[HEADING] = 1.0;
        let innovation = wrap_angle(heading.radians - self.x[HEADING]);
        self.scalar_update_with_innovation(innovation, &h, variance, gate)
    }

    pub fn update_speed_through_water(
        &mut self,
        speed: SpeedThroughWater,
        variance: f64,
        gate: Option<f64>,
    ) -> UpdateResult {
        let (predicted, h) = speed_model(&self.x);
        self.scalar_update(speed.metres_per_second, predicted, &h, variance, gate)
    }

    pub fn update_msl_altitude(&mut self, update: MslAltitudeUpdate) -> UpdateResult {
        let mut h = DVector::zeros(self.x.len());
        h.rows_mut(POS, 3)
            .copy_from(&DVector::from_column_slice(&update.up_ecef));
        let predicted = update
            .up_ecef
            .iter()
            .zip(self.x.rows(POS, 3).iter())
            .map(|(a, b)| a * b)
            .sum();
        self.scalar_update(
            update.altitude_m,
            predicted,
            &h,
            update.variance_m2,
            update.chi_square_threshold,
        )
    }

    pub fn update_gnss(&mut self, update: GnssUpdate) -> Vec<UpdateResult> {
        if !update.aided_mode {
            return Vec::new();
        }
        let mut results = Vec::with_capacity(6);
        for component in 0..6 {
            let index = if component < 3 {
                POS + component
            } else {
                VEL + component - 3
            };
            let measured = if component < 3 {
                update.position_ecef_m[component]
            } else {
                update.velocity_ecef_mps[component - 3]
            };
            let variance = if component < 3 {
                update.position_variance_m2[component]
            } else {
                update.velocity_variance_mps2[component - 3]
            };
            let mut h = DVector::zeros(self.x.len());
            h[index] = 1.0;
            results.push(self.scalar_update(
                measured,
                self.x[index],
                &h,
                variance,
                update.chi_square_threshold,
            ));
        }
        results
    }

    fn scalar_update(
        &mut self,
        measured: f64,
        predicted: f64,
        h: &DVector<f64>,
        variance: f64,
        gate: Option<f64>,
    ) -> UpdateResult {
        self.scalar_update_with_innovation(measured - predicted, h, variance, gate)
    }

    fn scalar_update_with_innovation(
        &mut self,
        innovation: f64,
        h: &DVector<f64>,
        variance: f64,
        gate: Option<f64>,
    ) -> UpdateResult {
        assert!(variance.is_finite() && variance > 0.0);
        let ph = &self.covariance * h;
        let innovation_variance = h.dot(&ph) + variance;
        let nis = innovation * innovation / innovation_variance;
        let accepted = gate.is_none_or(|threshold| nis <= threshold);
        if accepted {
            let gain = ph / innovation_variance;
            self.x += &gain * innovation;
            let identity = DMatrix::identity(self.x.len(), self.x.len());
            let a = identity - &gain * h.transpose();
            self.covariance =
                &a * &self.covariance * a.transpose() + (&gain * gain.transpose()) * variance;
            self.measurement_updates += 1;
            self.debug_assert_covariance();
        }
        UpdateResult {
            innovation,
            innovation_variance,
            normalized_innovation_squared: nis,
            accepted,
        }
    }

    fn augment_state(&mut self, value: f64, variance: f64) -> usize {
        let old = self.x.len();
        self.x = self.x.clone().insert_row(old, value);
        self.covariance = self
            .covariance
            .clone()
            .insert_row(old, 0.0)
            .insert_column(old, 0.0);
        self.covariance[(old, old)] = variance;
        old
    }

    fn remove_state(&mut self, index: usize) {
        self.x = self.x.clone().remove_row(index);
        self.covariance = self
            .covariance
            .clone()
            .remove_row(index)
            .remove_column(index);
    }

    fn transition_matrix(&self, dt: f64) -> DMatrix<f64> {
        let mut f = DMatrix::identity(self.x.len(), self.x.len());
        for axis in 0..3 {
            f[(POS + axis, VEL + axis)] = dt;
        }
        f[(CLOCK_BIAS, CLOCK_DRIFT)] = dt;
        for slot in self.receiver_clocks.values() {
            f[(slot.bias_index, slot.drift_index)] = dt;
        }
        f
    }

    fn debug_assert_covariance(&self) {
        debug_assert!((&self.covariance - self.covariance.transpose()).amax() < 1.0e-8);
        debug_assert!(self.covariance.clone().symmetric_eigen().eigenvalues.min() >= -1.0e-8);
    }

    fn cap_clock_bias_variance(&mut self, index: usize) {
        let variance = self.covariance[(index, index)];
        if variance > CLOCK_BIAS_VARIANCE_CAP_M2 {
            let scale = (CLOCK_BIAS_VARIANCE_CAP_M2 / variance).sqrt();
            for column in 0..self.covariance.ncols() {
                self.covariance[(index, column)] *= scale;
            }
            for row in 0..self.covariance.nrows() {
                self.covariance[(row, index)] *= scale;
            }
            self.covariance[(index, index)] = CLOCK_BIAS_VARIANCE_CAP_M2;
        }
    }
}

impl Estimator for FilterStub {
    fn propagate(&mut self, imu: ImuSample) {
        let dt = self.dt_seconds;
        let old_position_variance = self.covariance[(POS, POS)];
        for axis in 0..3 {
            self.x[POS + axis] +=
                self.x[VEL + axis] * dt + 0.5 * imu.acceleration_mps2[axis] * dt * dt;
            self.x[VEL + axis] += imu.acceleration_mps2[axis] * dt;
        }
        self.x[HEADING] = wrap_angle(self.x[HEADING] + imu.angular_rate_rps[2] * dt);
        self.x[CLOCK_BIAS] += self.x[CLOCK_DRIFT] * dt;
        for slot in self.receiver_clocks.values() {
            self.x[slot.bias_index] += self.x[slot.drift_index] * dt;
        }
        let f = self.transition_matrix(dt);
        let mut q = DMatrix::zeros(self.x.len(), self.x.len());
        for axis in 0..3 {
            q[(POS + axis, POS + axis)] =
                self.process_noise.acceleration_variance * dt.powi(3) / 3.0;
            q[(POS + axis, VEL + axis)] =
                self.process_noise.acceleration_variance * dt.powi(2) / 2.0;
            q[(VEL + axis, POS + axis)] = q[(POS + axis, VEL + axis)];
            q[(VEL + axis, VEL + axis)] = self.process_noise.acceleration_variance * dt;
        }
        q[(HEADING, HEADING)] = self.process_noise.turn_rate_variance * dt;
        let clock_pairs = std::iter::once((CLOCK_BIAS, CLOCK_DRIFT)).chain(
            self.receiver_clocks
                .values()
                .map(|slot| (slot.bias_index, slot.drift_index)),
        );
        for (bias, drift) in clock_pairs {
            let spectral_density = self.process_noise.clock_drift_variance;
            q[(bias, bias)] += spectral_density * dt.powi(3) / 3.0;
            q[(bias, drift)] += spectral_density * dt.powi(2) / 2.0;
            q[(drift, bias)] += spectral_density * dt.powi(2) / 2.0;
            q[(drift, drift)] += spectral_density * dt;
        }
        for index in self.nuisance_slots.values().copied() {
            q[(index, index)] = self.process_noise.nuisance_random_walk_variance * dt;
        }
        self.covariance = &f * &self.covariance * f.transpose() + q;
        self.cap_clock_bias_variance(CLOCK_BIAS);
        let receiver_biases: Vec<_> = self
            .receiver_clocks
            .values()
            .map(|slot| slot.bias_index)
            .collect();
        for bias in receiver_biases {
            self.cap_clock_bias_variance(bias);
        }
        self.propagations += 1;
        if self.covariance[(POS, POS)] > old_position_variance {
            self.covariance_growth_count += 1;
        }
        self.debug_assert_covariance();
    }

    fn update(&mut self, measurement: &MeasurementEnvelope) {
        let variance = measurement
            .covariance
            .first()
            .copied()
            .unwrap_or(1.0)
            .max(f64::EPSILON);
        match measurement.payload {
            MeasurementPayload::Heading(value) => {
                self.update_heading(value, variance, None);
            }
            MeasurementPayload::SpeedThroughWater(value) => {
                self.update_speed_through_water(value, variance, None);
            }
            MeasurementPayload::Gnss(value) => {
                let rotation =
                    ecef_to_enu_rotation([self.x[POS], self.x[POS + 1], self.x[POS + 2]]);
                let ned = value.velocity_ned_mps;
                let enu = [ned[1], ned[0], -ned[2]];
                let velocity_ecef_mps = std::array::from_fn(|column| {
                    (0..3).map(|row| rotation[row][column] * enu[row]).sum()
                });
                self.update_gnss(GnssUpdate {
                    position_ecef_m: value.position_ecef_m,
                    velocity_ecef_mps,
                    position_variance_m2: [variance; 3],
                    velocity_variance_mps2: [variance; 3],
                    aided_mode: true,
                    chi_square_threshold: None,
                });
            }
            _ => {}
        }
    }

    fn state(&self) -> FilterState {
        let position_ecef_m = [self.x[0], self.x[1], self.x[2]];
        let velocity_ecef_mps = [self.x[3], self.x[4], self.x[5]];
        let velocity_enu_mps = ecef_vector_to_enu(position_ecef_m, velocity_ecef_mps);
        FilterState {
            position_ecef_m,
            velocity_ecef_mps,
            horizontal_velocity_ned_mps: [velocity_enu_mps[1], velocity_enu_mps[0]],
            heading_rad: self.x[HEADING],
            receiver_clock_bias_m: self.x[CLOCK_BIAS],
            receiver_clock_drift_mps: self.x[CLOCK_DRIFT],
            covariance: (0..self.x.len())
                .flat_map(|row| (0..self.x.len()).map(move |column| self.covariance[(row, column)]))
                .collect(),
            covariance_dimension: self.x.len(),
        }
    }
}

fn speed_model(x: &DVector<f64>) -> (f64, DVector<f64>) {
    let model = |state: &DVector<f64>| {
        let enu = ecef_vector_to_enu(
            [state[POS], state[POS + 1], state[POS + 2]],
            [state[VEL], state[VEL + 1], state[VEL + 2]],
        );
        enu[0].hypot(enu[1])
    };
    let speed = model(x);
    let mut h = DVector::zeros(x.len());
    if speed > 1.0e-12 {
        for index in POS..VEL + 3 {
            let step = 1.0e-6 * x[index].abs().max(1.0);
            let mut plus = x.clone();
            plus[index] += step;
            let mut minus = x.clone();
            minus[index] -= step;
            h[index] = (model(&plus) - model(&minus)) / (2.0 * step);
        }
    }
    (speed, h)
}

fn wrap_angle(angle: f64) -> f64 {
    (angle + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;
    use pnt_types::{
        ArmAction, ArmCommand, Frame, GnssFix, Provenance, QualityFlags, SourceId, TimeTag,
        TrackerDoppler,
    };

    const FD_STEP: f64 = 1.0e-6;
    const JACOBIAN_TOLERANCE: f64 = 2.0e-6;

    fn central_difference(
        function: impl Fn(&DVector<f64>) -> DVector<f64>,
        x: &DVector<f64>,
    ) -> DMatrix<f64> {
        let output = function(x);
        let mut result = DMatrix::zeros(output.len(), x.len());
        for column in 0..x.len() {
            let mut plus = x.clone();
            plus[column] += FD_STEP;
            let mut minus = x.clone();
            minus[column] -= FD_STEP;
            result.set_column(
                column,
                &((function(&plus) - function(&minus)) / (2.0 * FD_STEP)),
            );
        }
        result
    }

    #[test]
    fn transition_jacobian_matches_central_difference() {
        let filter = augmented_filter();
        let dt = 0.01;
        let x = filter.x.clone();
        let numeric = central_difference(
            |x| {
                let mut y = x.clone();
                for axis in 0..3 {
                    y[POS + axis] += x[VEL + axis] * dt;
                }
                y[CLOCK_BIAS] += x[CLOCK_DRIFT] * dt;
                for slot in filter.receiver_clocks.values() {
                    y[slot.bias_index] += x[slot.drift_index] * dt;
                }
                y
            },
            &x,
        );
        assert!((numeric - filter.transition_matrix(dt)).amax() < JACOBIAN_TOLERANCE);
    }

    #[test]
    fn all_measurement_jacobians_match_central_difference() {
        let x = DVector::from_vec(vec![10.0, 20.0, 30.0, 3.0, 4.0, 0.0, 0.4, 2.0, 0.1]);
        let (_, speed_h) = speed_model(&x);
        let numeric_speed =
            central_difference(|value| DVector::from_element(1, speed_model(value).0), &x);
        assert!((numeric_speed.row(0).transpose() - speed_h).amax() < JACOBIAN_TOLERANCE);

        for index in 0..CORE_DIM {
            let numeric = central_difference(|value| DVector::from_element(1, value[index]), &x);
            let mut analytic = DMatrix::zeros(1, CORE_DIM);
            analytic[(0, index)] = 1.0;
            assert!((numeric - analytic).amax() < JACOBIAN_TOLERANCE);
        }
        let up = DVector::from_vec(vec![0.2, -0.3, 0.932_737_905]);
        let numeric_altitude = central_difference(
            |value| DVector::from_element(1, up.dot(&value.rows(0, 3))),
            &x,
        );
        let mut analytic_altitude = DMatrix::zeros(1, CORE_DIM);
        analytic_altitude
            .row_mut(0)
            .columns_mut(0, 3)
            .copy_from(&up.transpose());
        assert!((numeric_altitude - analytic_altitude).amax() < JACOBIAN_TOLERANCE);
    }

    #[test]
    fn dead_reckoning_grows_position_variance_by_magnitude() {
        let mut with_q = FilterStub::default();
        let mut without_q = FilterStub::new(
            0.01,
            ProcessNoise {
                acceleration_variance: 0.0,
                turn_rate_variance: 0.0,
                clock_drift_variance: 0.0,
                nuisance_random_walk_variance: 0.0,
            },
        );
        with_q.covariance.fill(0.0);
        without_q.covariance.fill(0.0);
        for _ in 0..100 {
            with_q.propagate(ImuSample::default());
            without_q.propagate(ImuSample::default());
        }
        assert!(with_q.covariance()[(0, 0)] > without_q.covariance()[(0, 0)] + 0.01);
    }

    #[test]
    fn nuisance_state_is_augmented_updated_and_retired() {
        let mut filter = FilterStub::default();
        let core = filter.covariance().nrows();
        let result = filter.update_doppler(&DopplerRangeRateUpdate {
            satellite_id: "SV-1".into(),
            measured_range_rate_mps: 2.0,
            predicted_range_rate_mps: 1.0,
            core_jacobian: [0.0; CORE_DIM],
            variance_mps2: 1.0,
            chi_square_threshold: Some(10.0),
            satellite_bias_variance_mps2: 1.0e4,
        });
        assert!(result.accepted);
        assert_eq!(filter.covariance().nrows(), core + 1);
        assert!(filter.retire_satellite_bias("SV-1"));
        assert_eq!(filter.covariance().nrows(), core);
    }

    #[test]
    fn gate_rejects_outlier_without_changing_state() {
        let mut filter = FilterStub::default();
        let before = filter.state();
        let result = filter.update_heading(Heading { radians: 3.0 }, 0.01, Some(1.0));
        assert!(!result.accepted);
        assert_eq!(filter.state(), before);
    }

    #[test]
    fn independent_receiver_gets_distinct_clock_slot() {
        let mut filter = FilterStub::default();
        let slot = filter.reserve_receiver_clock(ReceiverClockId("orbcomm".into()));
        assert_eq!(
            filter.receiver_clock_slot(&ReceiverClockId("orbcomm".into())),
            Some(slot)
        );
        assert_ne!(slot.bias_index, CLOCK_BIAS);
    }

    fn augmented_filter() -> FilterStub {
        let mut filter = FilterStub::default();
        filter
            .x
            .rows_mut(0, 6)
            .copy_from(&DVector::from_column_slice(&[
                3_492.09, 742.235, 5_283.455, 2.0, -3.0, 4.0,
            ]));
        filter.reserve_receiver_clock(ReceiverClockId("orbcomm".into()));
        filter.augment_satellite_bias("SV-A", 11.0);
        filter.augment_satellite_bias("SV-B", 22.0);
        filter.augment_satellite_bias("SV-C", 33.0);
        assert!(filter.retire_satellite_bias("SV-B"));
        filter
    }

    fn envelope(payload: MeasurementPayload) -> MeasurementEnvelope {
        MeasurementEnvelope {
            schema_version: 2,
            source_id: SourceId("test".into()),
            sequence: 1,
            sample_time: TimeTag::DeviceNanoseconds(1),
            host_receive_monotonic_ns: 1,
            utc: None,
            payload,
            frame: Frame::FrameIndependent,
            covariance: vec![0.25],
            quality: QualityFlags::VALID,
            calibration_id: "test".into(),
            provenance: Provenance::DerivedRecord("test".into()),
        }
    }

    #[test]
    fn real_update_paths_have_finite_difference_jacobians_on_augmented_filter() {
        let base = augmented_filter().x;
        let numeric = |update: fn(&mut FilterStub) -> f64| {
            central_difference(
                |x| {
                    let mut filter = augmented_filter();
                    filter.x.copy_from(x);
                    DVector::from_element(1, update(&mut filter))
                },
                &base,
            )
        };

        let heading = numeric(|filter| {
            filter
                .update_heading(Heading { radians: 0.7 }, 1.0, None)
                .innovation
        });
        assert!((heading[(0, HEADING)] + 1.0).abs() < JACOBIAN_TOLERANCE);
        assert_eq!(heading.ncols(), base.len());

        let gnss = numeric(|filter| {
            filter.update_gnss(GnssUpdate {
                position_ecef_m: [1.0, 2.0, 3.0],
                velocity_ecef_mps: [4.0, 5.0, 6.0],
                position_variance_m2: [1.0; 3],
                velocity_variance_mps2: [1.0; 3],
                aided_mode: true,
                chi_square_threshold: Some(0.0),
            })[4]
                .innovation
        });
        assert!((gnss[(0, VEL + 1)] + 1.0).abs() < JACOBIAN_TOLERANCE);

        let msl = numeric(|filter| {
            filter
                .update_msl_altitude(MslAltitudeUpdate {
                    altitude_m: 0.0,
                    up_ecef: [0.2, -0.3, 0.932_737_905],
                    variance_m2: 1.0,
                    chi_square_threshold: Some(0.0),
                })
                .innovation
        });
        for (axis, expected) in [0.2, -0.3, 0.932_737_905].iter().enumerate() {
            assert!((msl[(0, POS + axis)] + expected).abs() < JACOBIAN_TOLERANCE);
        }

        let receiver_doppler = numeric(|filter| {
            filter
                .update_doppler_for_receiver(
                    &DopplerRangeRateUpdate {
                        satellite_id: "SV-A".into(),
                        measured_range_rate_mps: 2.0,
                        predicted_range_rate_mps: 1.0,
                        core_jacobian: [0.0, 0.0, 0.0, 0.2, -0.3, 0.4, 0.0, 0.0, 1.0],
                        variance_mps2: 1.0,
                        chi_square_threshold: Some(0.0),
                        satellite_bias_variance_mps2: 11.0,
                    },
                    ReceiverClockId("orbcomm".into()),
                )
                .innovation
        });
        let slot = augmented_filter()
            .receiver_clock_slot(&ReceiverClockId("orbcomm".into()))
            .unwrap();
        assert!(receiver_doppler[(0, CLOCK_DRIFT)].abs() < JACOBIAN_TOLERANCE);
        assert!((receiver_doppler[(0, slot.drift_index)] + 1.0).abs() < JACOBIAN_TOLERANCE);
    }

    #[test]
    fn speed_update_jacobian_uses_local_horizontal_velocity() {
        let base = augmented_filter().x;
        let numeric = central_difference(
            |x| {
                let mut filter = augmented_filter();
                filter.x.copy_from(x);
                DVector::from_element(
                    1,
                    filter
                        .update_speed_through_water(
                            SpeedThroughWater {
                                metres_per_second: 5.0,
                            },
                            1.0,
                            Some(0.0),
                        )
                        .innovation,
                )
            },
            &base,
        );
        assert!(numeric[(0, VEL + 2)].abs() > 1.0e-3);
    }

    #[test]
    fn estimator_update_dispatches_every_payload_type() {
        let cases = [
            (MeasurementPayload::Imu(ImuSample::default()), 0),
            (MeasurementPayload::Heading(Heading { radians: 0.2 }), 1),
            (
                MeasurementPayload::SpeedThroughWater(SpeedThroughWater {
                    metres_per_second: 2.0,
                }),
                1,
            ),
            (MeasurementPayload::Gnss(GnssFix::default()), 6),
            (
                MeasurementPayload::TrackerDoppler(TrackerDoppler {
                    constellation: pnt_types::Constellation::Starlink,
                    correlation_peak_hz: 1.0,
                    nominal_carrier_hz: 2.0,
                }),
                0,
            ),
            (
                MeasurementPayload::ArmCommand(ArmCommand {
                    action: ArmAction::Arm,
                    host_monotonic_ns: 1,
                    source_id: SourceId("helm".into()),
                }),
                0,
            ),
        ];
        for (payload, expected_updates) in cases {
            let mut filter = augmented_filter();
            filter.update(&envelope(payload));
            assert_eq!(filter.measurement_updates(), expected_updates);
        }
    }

    #[test]
    fn retire_middle_preserves_two_nuisances_and_receiver_slot_indices() {
        let filter = augmented_filter();
        assert_eq!(filter.nuisance_slots.len(), 2);
        assert!(filter.nuisance_slots["SV-A"] < filter.nuisance_slots["SV-C"]);
        let receiver = filter
            .receiver_clock_slot(&ReceiverClockId("orbcomm".into()))
            .unwrap();
        assert!(receiver.drift_index < filter.nuisance_slots["SV-A"]);
        assert_eq!(filter.covariance.nrows(), CORE_DIM + 2 + 2);
    }

    #[test]
    fn doppler_uses_requested_nuisance_variance_and_real_augmented_path() {
        let mut filter = augmented_filter();
        let index = filter.augment_satellite_bias("SV-D", 47.0);
        let result = filter.update_doppler(&DopplerRangeRateUpdate {
            satellite_id: "SV-D".into(),
            measured_range_rate_mps: 1.0,
            predicted_range_rate_mps: 1.0,
            core_jacobian: [0.0; CORE_DIM],
            variance_mps2: 3.0,
            chi_square_threshold: None,
            satellite_bias_variance_mps2: 47.0,
        });
        assert!((result.innovation_variance - 50.0).abs() < f64::EPSILON);
        assert_eq!(filter.nuisance_slots["SV-D"], index);
    }

    #[test]
    fn clock_bias_variance_is_bounded_after_full_two_state_process_noise() {
        let mut filter = augmented_filter();
        filter.covariance[(CLOCK_BIAS, CLOCK_BIAS)] = CLOCK_BIAS_VARIANCE_CAP_M2 * 2.0;
        let receiver = filter
            .receiver_clock_slot(&ReceiverClockId("orbcomm".into()))
            .unwrap();
        filter.covariance[(receiver.bias_index, receiver.bias_index)] =
            CLOCK_BIAS_VARIANCE_CAP_M2 * 2.0;
        filter.propagate(ImuSample::default());
        assert!(filter.covariance[(CLOCK_BIAS, CLOCK_BIAS)] <= CLOCK_BIAS_VARIANCE_CAP_M2);
        assert!(
            filter.covariance[(receiver.bias_index, receiver.bias_index)]
                <= CLOCK_BIAS_VARIANCE_CAP_M2
        );
        assert!(filter.covariance[(CLOCK_BIAS, CLOCK_DRIFT)] > 0.0);
        assert!(filter.covariance[(receiver.bias_index, receiver.drift_index)] > 0.0);
    }
}
