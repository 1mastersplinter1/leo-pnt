use pnt_predictor::{predict, PredictError, ReceiverState, SatelliteState};

const C_MPS: f64 = 299_792_458.0;

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
