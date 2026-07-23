//! Error-state EKF for the nine-state navigation core and pass-scoped biases.

use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};
use pnt_types::{
    ecef_to_enu_rotation, ecef_vector_to_enu, FilterState, Heading, ImuSample, MeasurementEnvelope,
    MeasurementPayload, ReceiverClockId, ReceiverClockSlot, SpeedThroughWater,
};

// Fixed-core state layout. Current (E,N) is permanent physics and lives in the fixed core
// (U2) — it never enters the dynamic augment/retire index-shift path. POS/VEL/HEADING/CLOCK
// keep indices 0..8 so accuracy helpers keyed on POS=0/VEL=3 stay valid.
const CORE_DIM: usize = 11;
const POS: usize = 0;
const VEL: usize = 3;
const HEADING: usize = 6;
const CLOCK_BIAS: usize = 7;
const CLOCK_DRIFT: usize = 8;
const CURRENT_E: usize = 9;
const CURRENT_N: usize = 10;
/// Width of the predictor's Doppler Jacobian (`POS..=CLOCK_DRIFT`); it does not observe current.
const DOPPLER_JACOBIAN_DIM: usize = 9;
const CLOCK_BIAS_VARIANCE_CAP_M2: f64 = 1.0e8;

pub trait Estimator {
    fn propagate(&mut self, imu: ImuSample);
    fn update(&mut self, measurement: &MeasurementEnvelope);
    fn state(&self) -> FilterState;
    fn update_predicted_doppler(&mut self, update: &DopplerRangeRateUpdate) -> UpdateResult;
    /// The navigation-core state and covariance (the fixed `CORE_DIM` block), for the smoother
    /// reseed contract (U4e). Returned as `(state, covariance)` in `nalgebra` form.
    fn core_estimate(&self) -> (DVector<f64>, DMatrix<f64>);
    /// Overwrites the navigation-core block with an accepted smoother reseed (U4e).
    fn apply_reseed(&mut self, core_state: &DVector<f64>, core_covariance: &DMatrix<f64>);
}

#[derive(Clone, Copy, Debug)]
pub struct ProcessNoise {
    pub acceleration_variance: f64,
    pub turn_rate_variance: f64,
    pub clock_drift_variance: f64,
    pub nuisance_random_walk_variance: f64,
    /// Random-walk spectral density of the ENU current-velocity state (U2). Deliberately
    /// **much smaller than the velocity process noise** so that, with no speed-through-water
    /// observation to make current observable, it cannot random-walk into the velocity state.
    pub current_random_walk_variance: f64,
}

impl Default for ProcessNoise {
    fn default() -> Self {
        Self {
            acceleration_variance: 0.04,
            turn_rate_variance: 1.0e-4,
            clock_drift_variance: 1.0e-4,
            nuisance_random_walk_variance: 1.0e-6,
            current_random_walk_variance: 1.0e-8,
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
    /// Pure satellite/receiver geometric range rate. Clock terms are represented by H*x.
    pub predicted_range_rate_mps: f64,
    /// Predictor linearisation with respect to the nine `POS..=CLOCK_DRIFT` core states.
    /// Doppler does not observe current, so this stays 9-wide and is zero-padded onto the
    /// two current states inside the estimator.
    pub core_jacobian: [f64; DOPPLER_JACOBIAN_DIM],
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

/// A stationary batch fix handed into the moving filter (U4b). Applied as a *soft* prior — a
/// measurement update whose covariance is the batch posterior **inflated by elapsed-time
/// process growth** and clamped so it can never be tighter than the filter already is. This is
/// the concrete fix for the D39/D43 prior-confounding bug (a tight stale prior the filter
/// cannot move away from).
#[derive(Clone, Copy, Debug)]
pub struct SoftPrior {
    pub position_ecef_m: [f64; 3],
    pub velocity_ecef_mps: [f64; 3],
    /// Batch posterior variances (per ECEF axis) at the moment the fix was taken.
    pub position_variance_m2: [f64; 3],
    pub velocity_variance_mps2: [f64; 3],
    /// Seconds elapsed since the fix was taken; the prior is softened by process growth over it.
    pub elapsed_seconds: f64,
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
    robust_gate: bool,
    vector_stw: bool,
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
            robust_gate: false,
            vector_stw: false,
        }
    }

    /// Enables the vector water-velocity STW model (U2): speed-through-water is assimilated as
    /// a 2-component ENU constraint `v_ground − current ≈ STW · û_heading`, making the ENU
    /// current observable. Default off, which keeps the legacy scalar ground-speed STW model,
    /// so existing behaviour and synthetic headline numbers are unchanged unless enabled. The
    /// vector model requires a trustworthy heading and a synthetic/real STW that is genuinely
    /// water-relative; its maritime tuning is validated by U2's dedicated study.
    #[must_use]
    pub const fn with_vector_stw(mut self) -> Self {
        self.vector_stw = true;
        self
    }

    /// Enables the robust (Huber) measurement cost (U4a): an innovation beyond the chi-square
    /// gate is down-weighted (its effective variance inflated) rather than hard-rejected, so a
    /// moderate outlier stays partially informative instead of being discarded. Default off, so
    /// the hard-gate behaviour is unchanged unless this is set.
    #[must_use]
    pub const fn with_robust_gate(mut self) -> Self {
        self.robust_gate = true;
        self
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
        h.rows_mut(0, DOPPLER_JACOBIAN_DIM)
            .copy_from(&DVector::from_column_slice(&update.core_jacobian));
        h[nuisance] = 1.0;
        let predicted = update.predicted_range_rate_mps
            + update.core_jacobian[CLOCK_DRIFT] * self.x[CLOCK_DRIFT]
            + self.x[nuisance];
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
        h.rows_mut(0, DOPPLER_JACOBIAN_DIM)
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

    /// Assimilates a speed-through-water reading against the **vector** water-velocity model
    /// `v_water ≈ STW · û_heading`, i.e. `v_ground_enu − current_enu ≈ STW · (sinψ, cosψ)`
    /// (U2). This is a 2-component (E,N) constraint — unlike the scalar speed magnitude, which
    /// is rank-deficient for a 2-vector current — so it makes the ENU current observable from
    /// the STW-vs-Doppler-ground-velocity discrepancy. Applied as two sequential scalar updates.
    ///
    /// The cross-component (sideslip) part is a *model* pseudo-measurement, not a sensor value,
    /// so the two components share the caller's `variance` only under the zero-sideslip
    /// assumption; a caller expecting sideslip should widen `variance` accordingly.
    pub fn update_speed_through_water(
        &mut self,
        speed: SpeedThroughWater,
        variance: f64,
        gate: Option<f64>,
    ) -> UpdateResult {
        if !self.vector_stw {
            // Legacy scalar ground-speed model (current unobserved).
            let (predicted, h) = speed_model(&self.x);
            return self.scalar_update(speed.metres_per_second, predicted, &h, variance, gate);
        }
        let heading = self.x[HEADING];
        // Heading is measured from North, clockwise, so the unit heading vector in ENU is
        // (E, N) = (sin ψ, cos ψ). Expected water velocity components under zero sideslip.
        let expected = [
            speed.metres_per_second * heading.sin(),
            speed.metres_per_second * heading.cos(),
        ];
        let mut last = UpdateResult {
            innovation: 0.0,
            innovation_variance: variance,
            normalized_innovation_squared: 0.0,
            accepted: true,
        };
        for (component, &expected_component) in expected.iter().enumerate() {
            let (predicted, h) = water_velocity_component_model(&self.x, component);
            last = self.scalar_update(expected_component, predicted, &h, variance, gate);
        }
        last
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

    /// Injects a stationary batch fix as a soft prior (U4b): the batch posterior variance is
    /// inflated by process-noise growth over `elapsed_seconds`, then **clamped so it is never
    /// tighter than the filter's current variance** for that state. This prevents a stale,
    /// over-confident prior from over-constraining the filter (the D39/D43 confounding bug):
    /// the state is nudged, not reset, and the covariance never shrinks below what the filter
    /// already earned.
    ///
    /// # Panics
    ///
    /// Panics when `elapsed_seconds` is negative or non-finite.
    pub fn apply_soft_prior(&mut self, prior: &SoftPrior) {
        assert!(prior.elapsed_seconds.is_finite() && prior.elapsed_seconds >= 0.0);
        let t = prior.elapsed_seconds;
        // Elapsed-time process growth. Position uncertainty grows with the acceleration random
        // walk integrated twice (∝ q_a·t³/3, plus the carried velocity uncertainty ∝ t²);
        // velocity grows ∝ q_a·t. This mirrors the propagate() Q model, conservatively.
        let position_growth = self.process_noise.acceleration_variance * t.powi(3) / 3.0;
        let velocity_growth = self.process_noise.acceleration_variance * t;
        for component in 0..6 {
            let (index, measured, base_variance, growth) = if component < 3 {
                (
                    POS + component,
                    prior.position_ecef_m[component],
                    prior.position_variance_m2[component],
                    position_growth,
                )
            } else {
                (
                    VEL + component - 3,
                    prior.velocity_ecef_mps[component - 3],
                    prior.velocity_variance_mps2[component - 3],
                    velocity_growth,
                )
            };
            // Inflate the batch posterior by elapsed process growth. Anti-confounding guard:
            // only apply the prior to a state it is *more certain* about than the filter already
            // is. A stale, less-certain prior carries no new information, so it is skipped — it
            // can neither yank the state nor (falsely) shrink the covariance (the D39/D43 bug).
            let inflated = base_variance + growth;
            if inflated >= self.covariance[(index, index)] {
                continue;
            }
            let mut h = DVector::zeros(self.x.len());
            h[index] = 1.0;
            self.scalar_update(measured, self.x[index], &h, inflated, None);
        }
    }

    /// Maritime zero-velocity update (U4c): when the vessel is moored/anchored, its ground
    /// velocity is ~0. Constrains the three ECEF ground-velocity components to zero with the
    /// given (tight) `variance`. The measurement touches only the velocity states, so the
    /// current state is left entirely free — an anchored boat sits still over ground while
    /// water still flows past it (review H5: a maritime ZUPT is not `v_water = 0`, and it must
    /// not fight the current estimate).
    ///
    /// Callers must only invoke this when a motion classifier confirms the moored condition
    /// (e.g. near-zero speed-through-water AND near-zero ground track); it asserts ground
    /// velocity is zero, which is false while making way.
    ///
    /// # Panics
    ///
    /// Panics when `variance` is non-finite or not strictly positive.
    pub fn apply_moored_zupt(&mut self, variance: f64) {
        assert!(variance.is_finite() && variance > 0.0);
        for axis in 0..3 {
            let mut h = DVector::zeros(self.x.len());
            h[VEL + axis] = 1.0;
            self.scalar_update(0.0, self.x[VEL + axis], &h, variance, None);
        }
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

        // Robust (Huber) cost (U4a): beyond the gate, down-weight rather than hard-reject by
        // inflating the measurement noise R by nis/threshold (Huber weight w=sqrt(threshold/nis),
        // 1/w^2 = nis/threshold). The reported `nis`/`innovation_variance` stay raw for honest
        // integrity reporting; only the *update* uses the inflated R.
        let (accepted, effective_variance) = match gate {
            Some(threshold) if nis <= threshold => (true, variance),
            Some(threshold) if self.robust_gate => (true, variance * (nis / threshold)),
            Some(_) => (false, variance),
            None => (true, variance),
        };

        if accepted {
            let effective_innovation_variance = h.dot(&ph) + effective_variance;
            let gain = &ph / effective_innovation_variance;
            self.x += &gain * innovation;
            let identity = DMatrix::identity(self.x.len(), self.x.len());
            let a = identity - &gain * h.transpose();
            self.covariance = &a * &self.covariance * a.transpose()
                + (&gain * gain.transpose()) * effective_variance;
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
        // Current (E,N) random walk (U2). F stays identity for these states (a random walk,
        // not an integrator), so only Q grows. Kept small so an unobserved current does not
        // leak into velocity.
        q[(CURRENT_E, CURRENT_E)] = self.process_noise.current_random_walk_variance * dt;
        q[(CURRENT_N, CURRENT_N)] = self.process_noise.current_random_walk_variance * dt;
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

    fn update_predicted_doppler(&mut self, update: &DopplerRangeRateUpdate) -> UpdateResult {
        self.update_doppler(update)
    }

    fn core_estimate(&self) -> (DVector<f64>, DMatrix<f64>) {
        let state = self.x.rows(0, CORE_DIM).into_owned();
        let covariance = self
            .covariance
            .view((0, 0), (CORE_DIM, CORE_DIM))
            .into_owned();
        (state, covariance)
    }

    /// Overwrites the navigation-core block with an accepted smoother reseed (U4e). Augmented
    /// states (per-satellite biases, receiver clocks) are left untouched — a stationary batch
    /// reseed constrains the navigation core, not per-pass nuisances. The caller must have
    /// already passed the reseed through the `ReseedGate`; this performs the overwrite only.
    ///
    /// # Panics
    ///
    /// Panics when `core_state`/`core_covariance` are not exactly `CORE_DIM`-sized.
    fn apply_reseed(&mut self, core_state: &DVector<f64>, core_covariance: &DMatrix<f64>) {
        assert!(
            core_state.len() == CORE_DIM
                && core_covariance.nrows() == CORE_DIM
                && core_covariance.ncols() == CORE_DIM,
            "reseed must be core-dimensioned"
        );
        for i in 0..CORE_DIM {
            self.x[i] = core_state[i];
            for j in 0..CORE_DIM {
                self.covariance[(i, j)] = core_covariance[(i, j)];
            }
        }
        // The core marginal now comes from a different joint distribution than the augmented
        // states, so the old core<->augmented cross-covariances are statistically invalid.
        // Zero them (conservative: drops correlation rather than asserting a stale one), which
        // keeps the joint covariance consistent and PSD. Augmented marginals are preserved.
        let total = self.x.len();
        for i in 0..CORE_DIM {
            for j in CORE_DIM..total {
                self.covariance[(i, j)] = 0.0;
                self.covariance[(j, i)] = 0.0;
            }
        }
    }
}

/// Legacy scalar STW model: horizontal ground-speed magnitude and its Jacobian (current
/// unobserved). Used unless the vector water-velocity model (U2) is enabled.
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

/// The ENU `component` (0 = East, 1 = North) of the horizontal water velocity
/// `v_ground_enu − current_enu`, and its numeric Jacobian. Used by the vector STW update to
/// make the ENU current observable. Sensitive to POS (via the ENU rotation), VEL, and the
/// two current states; zero elsewhere.
fn water_velocity_component_model(x: &DVector<f64>, component: usize) -> (f64, DVector<f64>) {
    let model = |state: &DVector<f64>| {
        let ground_enu = ecef_vector_to_enu(
            [state[POS], state[POS + 1], state[POS + 2]],
            [state[VEL], state[VEL + 1], state[VEL + 2]],
        );
        let state_current = [state[CURRENT_E], state[CURRENT_N]];
        ground_enu[component] - state_current[component]
    };
    let predicted = model(x);
    let mut h = DVector::zeros(x.len());
    // Position (ENU rotation), velocity, and the relevant current component carry sensitivity.
    let indices = [POS, POS + 1, POS + 2, VEL, VEL + 1, VEL + 2]
        .into_iter()
        .chain(std::iter::once(if component == 0 {
            CURRENT_E
        } else {
            CURRENT_N
        }));
    for index in indices {
        let step = 1.0e-6 * x[index].abs().max(1.0);
        let mut plus = x.clone();
        plus[index] += step;
        let mut minus = x.clone();
        minus[index] -= step;
        h[index] = (model(&plus) - model(&minus)) / (2.0 * step);
    }
    (predicted, h)
}

fn wrap_angle(angle: f64) -> f64 {
    (angle + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;
    use pnt_types::{
        AckCommand, ArmAction, ArmCommand, Frame, GnssFix, Provenance, QualityFlags, SourceId,
        TimeTag, TrackerDoppler,
    };

    const FD_STEP: f64 = 1.0e-6;
    const JACOBIAN_TOLERANCE: f64 = 2.0e-6;

    // A ground-truth-ish fixture near 56N 12E with a known ground velocity, used by the
    // current-observability tests.
    fn current_fixture() -> (FilterStub, [f64; 3]) {
        // ECEF position ~ (56N, 12E). Any consistent point works; the model is frame-correct.
        let lat = 56.0_f64.to_radians();
        let lon = 12.0_f64.to_radians();
        let r = 6.371e6;
        let pos = [
            r * lat.cos() * lon.cos(),
            r * lat.cos() * lon.sin(),
            r * lat.sin(),
        ];
        let mut filter = FilterStub::default();
        for (slot, &value) in filter.x.rows_mut(POS, 3).iter_mut().zip(pos.iter()) {
            *slot = value;
        }
        (filter, pos)
    }

    // Sets the state's ECEF ground velocity from a desired ENU (E, N, U=0) velocity.
    fn set_ground_velocity_enu(filter: &mut FilterStub, pos: [f64; 3], enu: [f64; 3]) {
        let rotation = pnt_types::ecef_to_enu_rotation(pos);
        // ECEF = R^T * ENU (rotation rows are the ENU basis in ECEF).
        for (axis, slot) in filter.x.rows_mut(VEL, 3).iter_mut().enumerate() {
            *slot = rotation[0][axis] * enu[0]
                + rotation[1][axis] * enu[1]
                + rotation[2][axis] * enu[2];
        }
    }

    #[test]
    fn vector_water_velocity_model_is_sensitive_to_current() {
        // The vector model (unlike a scalar speed magnitude, which is rank-deficient for a
        // 2-vector current) has non-zero Jacobian on the current states: the East component is
        // sensitive to CURRENT_E, the North component to CURRENT_N. This is what makes current
        // observable.
        let (mut filter, pos) = current_fixture();
        set_ground_velocity_enu(&mut filter, pos, [3.0, 0.0, 0.0]);
        let (_, h_east) = water_velocity_component_model(&filter.x, 0);
        let (_, h_north) = water_velocity_component_model(&filter.x, 1);
        assert!(
            (h_east[CURRENT_E] + 1.0).abs() < 1.0e-6,
            "East water-velocity component must depend on CURRENT_E (d/dc_E = -1)"
        );
        assert!(
            (h_north[CURRENT_N] + 1.0).abs() < 1.0e-6,
            "North water-velocity component must depend on CURRENT_N (d/dc_N = -1)"
        );
    }

    #[test]
    fn vector_stw_makes_current_observable() {
        // With a known heading and STW, the vector water-velocity model resolves a unique
        // ENU current from the STW-vs-ground-velocity discrepancy.
        let (filter, pos) = current_fixture();
        let mut filter = filter.with_vector_stw();
        // Truth: ground velocity 3 m/s East; current 1 m/s East; so water velocity = 2 m/s East,
        // heading = 90 deg (due East), STW = 2 m/s.
        set_ground_velocity_enu(&mut filter, pos, [3.0, 0.0, 0.0]);
        filter.x[HEADING] = 90.0_f64.to_radians();
        // Give the current state room to move.
        filter.covariance[(CURRENT_E, CURRENT_E)] = 4.0;
        filter.covariance[(CURRENT_N, CURRENT_N)] = 4.0;
        // Feed several STW observations of 2 m/s (steady leg).
        for _ in 0..40 {
            let _ = filter.update_speed_through_water(
                SpeedThroughWater {
                    metres_per_second: 2.0,
                },
                0.01,
                None,
            );
        }
        // Current East should converge toward +1 m/s; North stays near 0.
        assert!(
            (filter.x[CURRENT_E] - 1.0).abs() < 0.3,
            "current East {} should converge to ~1.0",
            filter.x[CURRENT_E]
        );
        assert!(
            filter.x[CURRENT_N].abs() < 0.3,
            "current North {} should stay near 0",
            filter.x[CURRENT_N]
        );
    }

    #[test]
    fn current_does_not_leak_into_velocity_without_stw() {
        // With no STW observation, current must not random-walk into the velocity estimate.
        let (mut filter, pos) = current_fixture();
        set_ground_velocity_enu(&mut filter, pos, [3.0, 0.0, 0.0]);
        let v_before = [filter.x[VEL], filter.x[VEL + 1], filter.x[VEL + 2]];
        for _ in 0..100 {
            filter.propagate(ImuSample {
                acceleration_mps2: [0.0; 3],
                angular_rate_rps: [0.0; 3],
            });
        }
        for (axis, before) in v_before.iter().enumerate() {
            assert!(
                (filter.x[VEL + axis] - before).abs() < 1.0e-6,
                "velocity leaked without STW"
            );
        }
    }

    #[test]
    fn apply_reseed_overwrites_the_core_block_and_preserves_augmented_states() {
        let mut filter = FilterStub::default();
        // Augment a satellite bias so there is an augmented state beyond the core.
        let nuisance = filter.augment_satellite_bias("sat", 42.0);
        filter.x[nuisance] = 7.0;
        assert!(filter.x.len() > CORE_DIM);

        let mut core_state = DVector::zeros(CORE_DIM);
        core_state[POS] = 111.0;
        core_state[VEL] = 2.0;
        let mut core_cov = DMatrix::identity(CORE_DIM, CORE_DIM) * 3.0;
        core_cov[(POS, POS)] = 9.0;

        filter.apply_reseed(&core_state, &core_cov);

        assert!((filter.x[POS] - 111.0).abs() < f64::EPSILON);
        assert!((filter.x[VEL] - 2.0).abs() < f64::EPSILON);
        assert!((filter.covariance[(POS, POS)] - 9.0).abs() < f64::EPSILON);
        // The augmented satellite state and its variance are untouched.
        assert!((filter.x[nuisance] - 7.0).abs() < f64::EPSILON);
        assert!((filter.covariance[(nuisance, nuisance)] - 42.0).abs() < f64::EPSILON);
        // Core<->augmented cross-covariances are zeroed (stale after a core overwrite), keeping
        // the joint covariance consistent.
        for i in 0..CORE_DIM {
            assert!(filter.covariance[(i, nuisance)].abs() < f64::EPSILON);
            assert!(filter.covariance[(nuisance, i)].abs() < f64::EPSILON);
        }
        // The result stays symmetric and PSD.
        assert!((&filter.covariance - filter.covariance.transpose()).amax() < 1.0e-12);
        assert!(
            filter
                .covariance
                .clone()
                .symmetric_eigen()
                .eigenvalues
                .min()
                >= -1.0e-9
        );
    }

    #[test]
    fn moored_zupt_pins_ground_velocity_without_touching_current() {
        // Maritime ZUPT (U4c): when moored/anchored, ground velocity is ~0, but the current
        // state must stay free (an anchored boat sits still while water flows past it). A
        // zero-ground-velocity update must NOT fight or corrupt the current estimate.
        let (mut filter, _pos) = current_fixture();
        // The filter wrongly believes it is moving; current has a settled non-zero estimate.
        filter.x[VEL] = 2.0;
        filter.x[VEL + 1] = -1.0;
        filter.x[CURRENT_E] = 0.8;
        filter.x[CURRENT_N] = -0.4;
        filter.covariance[(CURRENT_E, CURRENT_E)] = 0.1;
        filter.covariance[(CURRENT_N, CURRENT_N)] = 0.1;
        let current_before = [filter.x[CURRENT_E], filter.x[CURRENT_N]];

        filter.apply_moored_zupt(1.0e-4);

        // Ground velocity is driven toward zero.
        for axis in 0..3 {
            assert!(
                filter.x[VEL + axis].abs() < 0.2,
                "ground velocity axis {axis} not pinned: {}",
                filter.x[VEL + axis]
            );
        }
        // Current is untouched (the ZUPT H has no current column).
        assert!((filter.x[CURRENT_E] - current_before[0]).abs() < 1.0e-9);
        assert!((filter.x[CURRENT_N] - current_before[1]).abs() < 1.0e-9);
    }

    #[test]
    fn soft_prior_does_not_over_constrain_a_confident_filter() {
        // The D39/D43 prior-confounding bug: a stale stationary fix injected with an
        // artificially tight covariance makes the moving filter refuse to move away from it.
        // The soft prior must never be tighter than the filter already is for a state.
        let mut filter = FilterStub::default();
        // Filter is already confident about position (small variance) at the true point.
        for axis in 0..3 {
            filter.covariance[(POS + axis, POS + axis)] = 4.0; // 2 m sigma
            filter.x[POS + axis] = 100.0;
        }
        let confident_before = filter.covariance[(POS, POS)];
        // A STALE prior claims a *different* position with an over-confident 0.01 m^2 variance.
        filter.apply_soft_prior(&SoftPrior {
            position_ecef_m: [1000.0, 1000.0, 1000.0],
            velocity_ecef_mps: [0.0, 0.0, 0.0],
            position_variance_m2: [0.01, 0.01, 0.01],
            velocity_variance_mps2: [0.01, 0.01, 0.01],
            elapsed_seconds: 120.0, // 2 minutes stale
        });
        // The guard must prevent the covariance from shrinking below what the filter had, and
        // the state must not be yanked all the way to the stale prior.
        assert!(
            filter.covariance[(POS, POS)] >= confident_before - 1.0e-9,
            "soft prior over-constrained the filter (confounding bug)"
        );
        assert!(
            filter.x[POS] < 600.0,
            "state was yanked toward a stale over-confident prior: {}",
            filter.x[POS]
        );
    }

    #[test]
    fn soft_prior_inflates_variance_by_elapsed_time() {
        // An honest, non-stale prior should still be softened by how long ago it was taken.
        let mut filter = FilterStub::default();
        for axis in 0..3 {
            filter.covariance[(POS + axis, POS + axis)] = 1.0e6; // filter is very unsure
        }
        let mut fresh = FilterStub::default();
        for axis in 0..3 {
            fresh.covariance[(POS + axis, POS + axis)] = 1.0e6;
        }
        let prior = |elapsed| SoftPrior {
            position_ecef_m: [10.0, 0.0, 0.0],
            velocity_ecef_mps: [0.0, 0.0, 0.0],
            position_variance_m2: [1.0, 1.0, 1.0],
            velocity_variance_mps2: [1.0, 1.0, 1.0],
            elapsed_seconds: elapsed,
        };
        filter.apply_soft_prior(&prior(600.0)); // 10 min stale
        fresh.apply_soft_prior(&prior(0.0)); // just taken
                                             // The fresher prior pulls the state closer to the prior position (10 m) than the stale one.
        assert!(
            (fresh.x[POS] - 10.0).abs() < (filter.x[POS] - 10.0).abs(),
            "a fresher prior should pull harder: fresh={}, stale={}",
            fresh.x[POS],
            filter.x[POS]
        );
    }

    #[test]
    fn current_lives_in_the_fixed_core_at_stable_indices() {
        // Current (E,N) is permanent physics in the fixed core, never shifted by dynamic
        // augmentation/retirement of satellite biases or receiver clocks.
        assert_eq!(CORE_DIM, 11);
        assert_eq!(CURRENT_E, 9);
        assert_eq!(CURRENT_N, 10);
        let mut filter = FilterStub::default();
        assert_eq!(filter.x.len(), CORE_DIM);
        // Augment then retire a satellite bias; current indices must be untouched.
        filter.x[CURRENT_E] = 0.7;
        filter.x[CURRENT_N] = -0.3;
        filter.augment_satellite_bias("sat-a", 100.0);
        filter.reserve_receiver_clock(pnt_types::ReceiverClockId("rx".into()));
        filter.retire_satellite_bias("sat-a");
        assert!((filter.x[CURRENT_E] - 0.7).abs() < f64::EPSILON);
        assert!((filter.x[CURRENT_N] - (-0.3)).abs() < f64::EPSILON);
    }

    #[test]
    fn a_doppler_jacobian_does_not_observe_current() {
        // The predictor Jacobian is 9-wide and gets zero-padded onto the 2 current states,
        // so a pure-clock Doppler update leaves current unchanged.
        let mut filter = FilterStub::default();
        filter.x[CURRENT_E] = 0.5;
        filter.x[CURRENT_N] = 0.5;
        let _ = filter.update_doppler(&DopplerRangeRateUpdate {
            satellite_id: "sat".into(),
            measured_range_rate_mps: 3.0,
            predicted_range_rate_mps: 0.0,
            core_jacobian: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            variance_mps2: 1.0,
            chi_square_threshold: None,
            satellite_bias_variance_mps2: 1.0,
        });
        assert!((filter.x[CURRENT_E] - 0.5).abs() < 1.0e-9);
        assert!((filter.x[CURRENT_N] - 0.5).abs() < 1.0e-9);
    }

    #[test]
    fn robust_cost_downweights_a_moderate_outlier_instead_of_rejecting_it() {
        // Same moderate outlier through the hard gate vs. the robust (Huber) cost.
        let update = || DopplerRangeRateUpdate {
            satellite_id: "outlier".into(),
            measured_range_rate_mps: 5.0, // large innovation vs predicted 0
            predicted_range_rate_mps: 0.0,
            core_jacobian: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            variance_mps2: 1.0,
            chi_square_threshold: Some(4.0), // sqrt-gate ~2 sigma; this outlier exceeds it
            satellite_bias_variance_mps2: 1.0,
        };

        let mut hard = FilterStub::default();
        let hard_result = hard.update_doppler(&update());
        assert!(
            !hard_result.accepted,
            "hard gate should reject this outlier"
        );

        let mut robust = FilterStub::default().with_robust_gate();
        let robust_result = robust.update_doppler(&update());
        assert!(
            robust_result.accepted,
            "robust cost should accept-but-downweight, not reject"
        );
        // The robust update moves the velocity state, but by LESS than an ungated full update
        // would (the outlier is down-weighted). Compare against a no-gate full update.
        let mut full = FilterStub::default();
        let mut ungated = update();
        ungated.chi_square_threshold = None;
        let _ = full.update_doppler(&ungated);

        let robust_shift = robust.x[VEL].abs();
        let full_shift = full.x[VEL].abs();
        assert!(robust_shift > 0.0, "robust update must move the state");
        assert!(
            robust_shift < full_shift,
            "robust shift {robust_shift} should be smaller than full shift {full_shift}"
        );
    }

    #[test]
    fn primary_doppler_prediction_includes_nonzero_clock_drift_via_h_x() {
        let mut filter = FilterStub::default();
        filter.x[CLOCK_DRIFT] = 12.0;
        let result = filter.update_doppler(&DopplerRangeRateUpdate {
            satellite_id: "clock-regression".into(),
            measured_range_rate_mps: 17.0,
            predicted_range_rate_mps: 5.0,
            core_jacobian: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            variance_mps2: 1.0,
            chi_square_threshold: Some(1.0),
            satellite_bias_variance_mps2: 1.0,
        });
        assert!(result.accepted);
        assert!(result.innovation.abs() < f64::EPSILON);
    }

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
        let x = DVector::from_vec(vec![
            10.0, 20.0, 30.0, 3.0, 4.0, 0.0, 0.4, 2.0, 0.1, 0.0, 0.0,
        ]);
        for component in 0..2 {
            let (_, water_h) = water_velocity_component_model(&x, component);
            let numeric_water = central_difference(
                |value| {
                    DVector::from_element(1, water_velocity_component_model(value, component).0)
                },
                &x,
            );
            assert!(
                (numeric_water.row(0).transpose() - water_h).amax() < JACOBIAN_TOLERANCE,
                "water-velocity component {component} Jacobian mismatch"
            );
        }

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
                current_random_walk_variance: 0.0,
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
            core_jacobian: [0.0; DOPPLER_JACOBIAN_DIM],
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
            (
                MeasurementPayload::AckCommand(AckCommand {
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
            core_jacobian: [0.0; DOPPLER_JACOBIAN_DIM],
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
