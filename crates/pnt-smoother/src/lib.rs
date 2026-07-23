//! Fixed-lag smoother scaffolding for the moving-regime estimator (code-change plan U4d/U4e).
//!
//! # Exclusive information ownership (the double-counting rule)
//!
//! In the smoother-refines-filter architecture the EKF and the smoother must **not** both
//! consume the same measurements — feeding a measurement to both and then reseeding the EKF
//! from the smoother reinjects that measurement's information, yielding an optimistically small
//! EKF covariance. Therefore, when the smoother is active, **measurements enter the smoother
//! only**; the EKF does IMU-prediction plus a guarded reseed. The single exception is the
//! kill-switch path (EKF-only mode), where measurements are routed back onto the EKF so it is
//! never left prediction-only. This module owns the reseed contract; the executive owns which
//! path a measurement takes.
//!
//! # The reseed is the autopilot surface
//!
//! The EKF state is what feeds the autopilot, so writing it from the smoother is safety-
//! critical. [`ReseedGate`] enforces, before any reseed is accepted:
//! - **lag alignment** — the smoothed estimate is at `t − L`; its covariance is grown by the
//!   elapsed lag before it is compared to or written into the live EKF (never reseed a live
//!   filter with a stale-tight covariance);
//! - a bound on the reseed step `Δx` (reject an implausible jump);
//! - **optimistic-covariance rejection** — after lag alignment the reseed covariance must be
//!   positive semidefinite and no tighter than a configured floor (it may not collapse to a
//!   degenerate near-zero confidence). A lag-grown smoother covariance that is *tighter than
//!   the live EKF* is legitimate — the smoother has fused more measurements — so tightness
//!   versus the EKF is deliberately **not** the rejection criterion; staleness is handled by
//!   the lag growth, and degenerate over-confidence by the floor;
//! - **fail-closed** — on any rejection the live EKF state is held unchanged.

use nalgebra::{DMatrix, DVector};

/// A reseed candidate produced by the fixed-lag smoother, valid at time `t − lag_seconds`.
#[derive(Clone, Debug)]
pub struct ReseedCandidate {
    /// Smoothed state estimate at `t − lag_seconds`.
    pub state: DVector<f64>,
    /// Smoothed covariance at `t − lag_seconds` (square, same dimension as `state`).
    pub covariance: DMatrix<f64>,
    /// How far in the past the smoothed estimate is, relative to the live EKF.
    pub lag_seconds: f64,
}

/// The live EKF estimate the reseed would overwrite, at time `t`.
#[derive(Clone, Debug)]
pub struct FilterEstimate {
    pub state: DVector<f64>,
    pub covariance: DMatrix<f64>,
}

/// Outcome of evaluating a reseed candidate against the live filter.
#[derive(Clone, Debug, PartialEq)]
pub enum ReseedDecision {
    /// Accepted: overwrite the EKF with this lag-aligned state and covariance.
    Accept {
        state: DVector<f64>,
        covariance: DMatrix<f64>,
    },
    /// Rejected (fail-closed: hold the live EKF), with a machine-readable reason.
    Reject(ReseedRejection),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReseedRejection {
    /// The reseed moved a state component beyond the configured bound.
    StepTooLarge,
    /// The reseed covariance was not positive semidefinite.
    NonPsdCovariance,
    /// The reseed claimed more confidence than the lag-grown EKF covariance (Löwner order).
    OptimisticCovariance,
    /// The candidate was structurally invalid (dimension mismatch, non-finite, negative lag).
    Malformed,
}

/// Enforces the reseed contract described in the crate docs.
#[derive(Clone, Copy, Debug)]
pub struct ReseedGate {
    /// Maximum accepted per-component state jump.
    pub max_step: f64,
    /// Per-second isotropic process growth added to the smoothed covariance during lag
    /// alignment (a conservative bound; the caller passes the dominant process spectral density).
    pub lag_growth_per_s: f64,
    /// Minimum accepted covariance eigenvalue after lag alignment — the floor that prevents a
    /// degenerate, over-confident (near-zero-variance) reseed from reaching the EKF.
    pub covariance_floor: f64,
    /// Tolerance for the PSD eigenvalue check.
    pub eigenvalue_tolerance: f64,
}

impl Default for ReseedGate {
    fn default() -> Self {
        Self {
            max_step: 1_000.0,
            lag_growth_per_s: 1.0,
            covariance_floor: 1.0e-6,
            eigenvalue_tolerance: 1.0e-9,
        }
    }
}

impl ReseedGate {
    /// Evaluates a reseed candidate against the live filter, fail-closed.
    #[must_use]
    pub fn evaluate(&self, candidate: &ReseedCandidate, live: &FilterEstimate) -> ReseedDecision {
        let n = live.state.len();
        if candidate.state.len() != n
            || candidate.covariance.nrows() != n
            || candidate.covariance.ncols() != n
            || live.covariance.nrows() != n
            || live.covariance.ncols() != n
            || !candidate.lag_seconds.is_finite()
            || candidate.lag_seconds < 0.0
            || candidate.state.iter().any(|v| !v.is_finite())
            || candidate.covariance.iter().any(|v| !v.is_finite())
        {
            return ReseedDecision::Reject(ReseedRejection::Malformed);
        }

        // Δx bound.
        if candidate
            .state
            .iter()
            .zip(live.state.iter())
            .any(|(a, b)| (a - b).abs() > self.max_step)
        {
            return ReseedDecision::Reject(ReseedRejection::StepTooLarge);
        }

        // Lag alignment: grow the smoothed covariance by the elapsed lag before it is trusted.
        let growth = self.lag_growth_per_s * candidate.lag_seconds;
        let aligned = &candidate.covariance + DMatrix::identity(n, n) * growth;

        // The aligned covariance must be PSD.
        let min_eig = min_eigenvalue(&aligned);
        if min_eig < -self.eigenvalue_tolerance {
            return ReseedDecision::Reject(ReseedRejection::NonPsdCovariance);
        }

        // Optimistic-covariance rejection: a lag-grown smoother covariance tighter than the
        // live EKF is legitimate (more measurements fused), so tightness-vs-EKF is not a
        // rejection criterion. What is rejected is a *degenerate* over-confident reseed whose
        // covariance has collapsed below the configured floor — false certainty reaching the
        // autopilot-facing state.
        if min_eig < self.covariance_floor {
            return ReseedDecision::Reject(ReseedRejection::OptimisticCovariance);
        }

        ReseedDecision::Accept {
            state: candidate.state.clone(),
            covariance: aligned,
        }
    }
}

/// Minimum eigenvalue of the symmetric part of `m`.
fn min_eigenvalue(m: &DMatrix<f64>) -> f64 {
    let symmetric = (m + m.transpose()) * 0.5;
    symmetric.symmetric_eigenvalues().min()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diag(values: &[f64]) -> DMatrix<f64> {
        DMatrix::from_diagonal(&DVector::from_column_slice(values))
    }

    fn live(state: &[f64], variances: &[f64]) -> FilterEstimate {
        FilterEstimate {
            state: DVector::from_column_slice(state),
            covariance: diag(variances),
        }
    }

    #[test]
    fn accepts_a_lag_aligned_consistent_reseed() {
        let gate = ReseedGate::default();
        let live = live(&[0.0, 0.0], &[100.0, 100.0]);
        let candidate = ReseedCandidate {
            state: DVector::from_column_slice(&[1.0, -1.0]),
            covariance: diag(&[50.0, 50.0]),
            lag_seconds: 5.0,
        };
        match gate.evaluate(&candidate, &live) {
            ReseedDecision::Accept { state, covariance } => {
                assert!((state[0] - 1.0).abs() < f64::EPSILON);
                // Covariance was grown by lag: 50 + 1.0 * 5 = 55.
                assert!((covariance[(0, 0)] - 55.0).abs() < 1.0e-9);
            }
            other @ ReseedDecision::Reject(_) => panic!("expected Accept, got {other:?}"),
        }
    }

    #[test]
    fn rejects_an_implausible_state_jump() {
        let gate = ReseedGate {
            max_step: 10.0,
            ..ReseedGate::default()
        };
        let live = live(&[0.0, 0.0], &[100.0, 100.0]);
        let candidate = ReseedCandidate {
            state: DVector::from_column_slice(&[1000.0, 0.0]),
            covariance: diag(&[50.0, 50.0]),
            lag_seconds: 1.0,
        };
        assert_eq!(
            gate.evaluate(&candidate, &live),
            ReseedDecision::Reject(ReseedRejection::StepTooLarge)
        );
    }

    #[test]
    fn accepts_a_reseed_tighter_than_the_live_filter() {
        // A smoother that has fused more measurements is legitimately more confident than the
        // live EKF; tightness-vs-EKF is not a rejection criterion.
        let gate = ReseedGate {
            lag_growth_per_s: 0.0,
            ..ReseedGate::default()
        };
        let live = live(&[0.0, 0.0], &[100.0, 100.0]);
        let candidate = ReseedCandidate {
            state: DVector::from_column_slice(&[1.0, 1.0]),
            covariance: diag(&[1.0, 1.0]), // far tighter than live, but above the floor
            lag_seconds: 0.0,
        };
        assert!(matches!(
            gate.evaluate(&candidate, &live),
            ReseedDecision::Accept { .. }
        ));
    }

    #[test]
    fn rejects_a_degenerate_overconfident_reseed_below_the_floor() {
        // A reseed whose covariance has collapsed below the floor is false certainty — reject.
        let gate = ReseedGate {
            lag_growth_per_s: 0.0,
            covariance_floor: 1.0e-3,
            ..ReseedGate::default()
        };
        let live = live(&[0.0, 0.0], &[100.0, 100.0]);
        let candidate = ReseedCandidate {
            state: DVector::from_column_slice(&[1.0, 1.0]),
            covariance: diag(&[1.0e-9, 1.0e-9]),
            lag_seconds: 0.0,
        };
        assert_eq!(
            gate.evaluate(&candidate, &live),
            ReseedDecision::Reject(ReseedRejection::OptimisticCovariance)
        );
    }

    #[test]
    fn rejects_a_non_psd_covariance() {
        let gate = ReseedGate {
            lag_growth_per_s: 0.0,
            ..ReseedGate::default()
        };
        let live = live(&[0.0, 0.0], &[100.0, 100.0]);
        let candidate = ReseedCandidate {
            state: DVector::from_column_slice(&[1.0, 1.0]),
            covariance: diag(&[-1.0, 50.0]), // negative variance
            lag_seconds: 0.0,
        };
        assert_eq!(
            gate.evaluate(&candidate, &live),
            ReseedDecision::Reject(ReseedRejection::NonPsdCovariance)
        );
    }

    #[test]
    fn rejects_a_malformed_candidate() {
        let gate = ReseedGate::default();
        let live = live(&[0.0, 0.0], &[100.0, 100.0]);
        let wrong_dimension = ReseedCandidate {
            state: DVector::from_column_slice(&[1.0, 1.0, 1.0]),
            covariance: diag(&[50.0, 50.0, 50.0]),
            lag_seconds: 1.0,
        };
        assert_eq!(
            gate.evaluate(&wrong_dimension, &live),
            ReseedDecision::Reject(ReseedRejection::Malformed)
        );
        let negative_lag = ReseedCandidate {
            state: DVector::from_column_slice(&[1.0, 1.0]),
            covariance: diag(&[50.0, 50.0]),
            lag_seconds: -1.0,
        };
        assert_eq!(
            gate.evaluate(&negative_lag, &live),
            ReseedDecision::Reject(ReseedRejection::Malformed)
        );
    }

    #[test]
    fn lag_growth_rescues_a_reseed_that_is_degenerate_only_before_alignment() {
        // A reseed at the floor at t-L is lifted above it by lag growth, so it is accepted.
        let gate = ReseedGate {
            lag_growth_per_s: 2.0,
            covariance_floor: 1.0,
            ..ReseedGate::default()
        };
        let live = live(&[0.0, 0.0], &[100.0, 100.0]);
        let candidate = ReseedCandidate {
            state: DVector::from_column_slice(&[1.0, 1.0]),
            covariance: diag(&[0.5, 0.5]), // below the 1.0 floor at t-L
            lag_seconds: 5.0,              // grows 0.5 -> 0.5 + 2*5 = 10.5 > floor
        };
        assert!(matches!(
            gate.evaluate(&candidate, &live),
            ReseedDecision::Accept { .. }
        ));
    }
}
