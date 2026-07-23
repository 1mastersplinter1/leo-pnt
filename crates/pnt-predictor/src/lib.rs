#![doc = "Geometric range-rate and correlation-peak Doppler prediction in ECEF."]

const SPEED_OF_LIGHT_MPS: f64 = 299_792_458.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SatelliteState {
    pub position_ecef_m: [f64; 3],
    pub velocity_ecef_mps: [f64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReceiverState {
    pub position_ecef_m: [f64; 3],
    pub velocity_ecef_mps: [f64; 3],
    /// Receiver-clock drift expressed as an equivalent positive range rate (m/s). This affects
    /// `correlation_peak_hz` only; `Prediction::range_rate_mps` remains purely geometric.
    pub clock_drift_mps: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prediction {
    /// Correlation-peak offset from nominal carrier. Positive means received frequency is high.
    pub correlation_peak_hz: f64,
    /// Geometric `d(range)/dt`; negative for an approaching satellite.
    pub range_rate_mps: f64,
    /// Unit vector from receiver to satellite, in ECEF.
    pub line_of_sight_ecef: [f64; 3],
    pub elevation_rad: f64,
    pub range_m: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PredictError {
    InvalidInput,
    CoincidentPositions,
    BelowElevationMask { elevation_rad: f64, mask_rad: f64 },
}

/// Predicts the observed correlation-peak Doppler.
///
/// The non-relativistic convention is `doppler = -carrier *
/// (geometric_range_rate + receiver_clock_drift) / c + nuisance_bias`.
/// Thus approach has negative range rate and positive Doppler. The nuisance bias is a
/// transmit-frequency offset in Hz and is additive. Elevation uses the geocentric ECEF
/// radial direction; callers needing sub-degree geodetic elevation should supply their
/// own ellipsoidal local frame.
///
/// # Errors
///
/// Returns [`PredictError::InvalidInput`] for non-finite values or a non-positive carrier,
/// [`PredictError::CoincidentPositions`] for undefined geometry, and
/// [`PredictError::BelowElevationMask`] when the satellite is below the requested mask.
pub fn predict(
    satellite: SatelliteState,
    receiver: ReceiverState,
    nuisance_bias_hz: f64,
    nominal_carrier_hz: f64,
    elevation_mask_rad: f64,
) -> Result<Prediction, PredictError> {
    if !nominal_carrier_hz.is_finite()
        || nominal_carrier_hz <= 0.0
        || !nuisance_bias_hz.is_finite()
        || !elevation_mask_rad.is_finite()
        || !receiver.clock_drift_mps.is_finite()
        || satellite
            .position_ecef_m
            .iter()
            .chain(satellite.velocity_ecef_mps.iter())
            .chain(receiver.position_ecef_m.iter())
            .chain(receiver.velocity_ecef_mps.iter())
            .any(|v| !v.is_finite())
    {
        return Err(PredictError::InvalidInput);
    }

    let delta = sub(satellite.position_ecef_m, receiver.position_ecef_m);
    let range_m = norm(delta);
    let receiver_radius = norm(receiver.position_ecef_m);
    if range_m == 0.0 || receiver_radius == 0.0 {
        return Err(PredictError::CoincidentPositions);
    }
    let los = scale(delta, 1.0 / range_m);
    let up = scale(receiver.position_ecef_m, 1.0 / receiver_radius);
    let elevation_rad = dot(los, up).clamp(-1.0, 1.0).asin();
    if elevation_rad < elevation_mask_rad {
        return Err(PredictError::BelowElevationMask {
            elevation_rad,
            mask_rad: elevation_mask_rad,
        });
    }
    let range_rate_mps = dot(
        los,
        sub(satellite.velocity_ecef_mps, receiver.velocity_ecef_mps),
    );
    let correlation_peak_hz = -nominal_carrier_hz * (range_rate_mps + receiver.clock_drift_mps)
        / SPEED_OF_LIGHT_MPS
        + nuisance_bias_hz;
    Ok(Prediction {
        correlation_peak_hz,
        range_rate_mps,
        line_of_sight_ecef: los,
        elevation_rad,
        range_m,
    })
}

/// Linearises geometric range rate plus receiver clock drift with respect to the
/// nine-state navigation core: position, velocity, heading, clock bias, clock drift.
///
/// # Errors
///
/// Returns [`PredictError`] when the receiver and satellite geometry cannot produce a
/// finite, non-degenerate range-rate prediction.
pub fn geometric_range_rate_linearisation(
    satellite: SatelliteState,
    receiver: ReceiverState,
) -> Result<[f64; 9], PredictError> {
    let prediction = predict(satellite, receiver, 0.0, 1.0, -std::f64::consts::FRAC_PI_2)?;
    let relative_velocity = sub(satellite.velocity_ecef_mps, receiver.velocity_ecef_mps);
    let position_gradient: [f64; 3] = std::array::from_fn(|i| {
        -(relative_velocity[i] - prediction.line_of_sight_ecef[i] * prediction.range_rate_mps)
            / prediction.range_m
    });
    let mut jacobian = [0.0; 9];
    jacobian[..3].copy_from_slice(&position_gradient);
    for i in 0..3 {
        jacobian[3 + i] = -prediction.line_of_sight_ecef[i];
    }
    jacobian[8] = 1.0;
    Ok(jacobian)
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
