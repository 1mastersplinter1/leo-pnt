use pnt_predictor::{predict, PredictError, ReceiverState, SatelliteState};

const C_MPS: f64 = 299_792_458.0;

#[test]
fn geometric_range_rate_linearisation_matches_central_differences() {
    let satellite = SatelliteState {
        position_ecef_m: [7.1e6, -1.2e6, 2.3e6],
        velocity_ecef_mps: [1_100.0, 6_900.0, -800.0],
    };
    let receiver = ReceiverState {
        position_ecef_m: [3.5e6, 0.8e6, 5.2e6],
        velocity_ecef_mps: [4.0, -2.0, 1.0],
        clock_drift_mps: 3.0,
    };
    let analytic = pnt_predictor::geometric_range_rate_linearisation(satellite, receiver).unwrap();
    let model = |state: ReceiverState| {
        predict(satellite, state, 0.0, 1.6e9, -std::f64::consts::FRAC_PI_2)
            .unwrap()
            .range_rate_mps
            + state.clock_drift_mps
    };
    for index in 0..7 {
        let step = if index < 3 { 0.1 } else { 1.0e-4 };
        let (mut plus, mut minus) = (receiver, receiver);
        if index < 3 {
            plus.position_ecef_m[index] += step;
            minus.position_ecef_m[index] -= step;
        } else if index < 6 {
            plus.velocity_ecef_mps[index - 3] += step;
            minus.velocity_ecef_mps[index - 3] -= step;
        } else {
            plus.clock_drift_mps += step;
            minus.clock_drift_mps -= step;
        }
        let fd = (model(plus) - model(minus)) / (2.0 * step);
        let column = if index == 6 { 8 } else { index };
        assert!(
            (analytic[column] - fd).abs() < 1.0e-6,
            "column {column}: analytic={}, fd={fd}",
            analytic[column]
        );
    }
    assert!(analytic[6].abs() < f64::EPSILON);
    assert!(analytic[7].abs() < f64::EPSILON);
}

#[test]
fn approaching_is_positive_doppler() {
    let sat = SatelliteState {
        position_ecef_m: [20_000_000.0, 0.0, 0.0],
        velocity_ecef_mps: [-1_000.0, 0.0, 0.0],
    };
    let rx = ReceiverState {
        position_ecef_m: [6_400_000.0, 0.0, 0.0],
        velocity_ecef_mps: [0.0; 3],
        clock_drift_mps: 0.0,
    };
    // -90 rad is deliberately below asin's [-pi/2, pi/2] range, disabling the mask.
    let p = predict(sat, rx, 0.0, 1_500_000_000.0, -90.0).unwrap();
    // Independently: rho_dot = LOS dot (v_sat-v_rx) = -1000 m/s and
    // received Doppler = -f*rho_dot/c = +5003.461... Hz.
    assert!((p.range_rate_mps + 1_000.0).abs() < 1e-12);
    assert!((p.correlation_peak_hz - 1_500_000_000.0 * 1_000.0 / C_MPS).abs() < 1e-9);
    assert!((p.line_of_sight_ecef[0] - 1.0).abs() < 1e-15);
    assert_eq!(&p.line_of_sight_ecef[1..], &[0.0, 0.0]);
    assert!((p.range_m - 13_600_000.0).abs() < 1e-9);
    assert!((p.elevation_rad - core::f64::consts::FRAC_PI_2).abs() < 2e-8);
}

#[test]
fn clock_and_nuisance_terms_are_additive_frequency_offsets() {
    let sat = SatelliteState {
        position_ecef_m: [7_000_000.0, 0.0, 0.0],
        velocity_ecef_mps: [0.0; 3],
    };
    let rx = ReceiverState {
        position_ecef_m: [6_000_000.0, 0.0, 0.0],
        velocity_ecef_mps: [0.0; 3],
        clock_drift_mps: 2.0,
    };
    let p = predict(sat, rx, 7.0, C_MPS, 0.0).unwrap();
    assert!((p.correlation_peak_hz - 5.0).abs() < 1e-12);
}

#[test]
fn elevation_mask_rejects_below_horizon() {
    let sat = SatelliteState {
        position_ecef_m: [-7_000_000.0, 0.0, 0.0],
        velocity_ecef_mps: [0.0; 3],
    };
    let rx = ReceiverState {
        position_ecef_m: [6_000_000.0, 0.0, 0.0],
        velocity_ecef_mps: [0.0; 3],
        clock_drift_mps: 0.0,
    };
    assert!(matches!(
        predict(sat, rx, 0.0, 1.0, 0.0),
        Err(PredictError::BelowElevationMask { .. })
    ));
}
