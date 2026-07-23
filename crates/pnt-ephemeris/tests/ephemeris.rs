use chrono::{Duration, TimeZone, Utc};
use pnt_ephemeris::{EphemerisError, EphemerisStore};

#[test]
fn parses_local_tle_and_omm_json() {
    let tle = EphemerisStore::from_tle_file("tests/fixtures/iss.tle").unwrap();
    let omm = EphemerisStore::from_omm_json_file("tests/fixtures/iss.json").unwrap();
    assert!(tle.contains(25544));
    assert!(omm.contains(25544));
    let expected =
        Utc.with_ymd_and_hms(2020, 7, 12, 21, 16, 1).unwrap() + Duration::nanoseconds(416_000);
    assert!((tle.epoch(25544).unwrap() - expected).abs() <= Duration::nanoseconds(1));
    assert_eq!(omm.epoch(25544), Some(expected));
}

#[test]
fn default_six_hour_age_gate_is_typed() {
    let store = EphemerisStore::from_tle_file("tests/fixtures/iss.tle").unwrap();
    let epoch = store.epoch(25544).unwrap();
    assert!(store
        .propagate_ecef(25544, epoch + Duration::hours(6))
        .is_ok());
    assert!(matches!(
        store.propagate_ecef(25544, epoch + Duration::hours(6) + Duration::nanoseconds(1)),
        Err(EphemerisError::TooOld { .. })
    ));
    assert!(matches!(
        store.propagate_ecef(25544, epoch - Duration::hours(6) - Duration::nanoseconds(1)),
        Err(EphemerisError::TooOld { .. })
    ));
}

#[test]
fn vallado_reference_vector_case_00005() {
    // Vallado et al. SGP4 verification case 00005, copied from sgp4 2.4.0's shipped
    // tests/test_cases.toml. AFSPC mode belongs here solely for reference compatibility.
    let tle = "0 VANGUARD 1\n1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753\n2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667\n";
    let lines: Vec<_> = tle.lines().collect();
    let elements = sgp4::Elements::from_tle(
        Some(lines[0].trim_start_matches("0 ").to_owned()),
        lines[1].as_bytes(),
        lines[2].as_bytes(),
    )
    .unwrap();
    let constants = sgp4::Constants::from_elements_afspc_compatibility_mode(&elements).unwrap();
    let initial = constants
        .propagate_afspc_compatibility_mode(sgp4::MinutesSinceEpoch(0.0))
        .unwrap();
    let expected_initial_p = [7_022.465_292_66, -1_400.082_967_55, 0.039_951_55];
    let expected_initial_v = [1.893_841_015, 6.405_893_759, 4.534_807_250];
    for i in 0..3 {
        assert!((initial.position[i] - expected_initial_p[i]).abs() < 1e-6);
        assert!((initial.velocity[i] - expected_initial_v[i]).abs() < 1e-9);
    }
    let state = constants
        .propagate_afspc_compatibility_mode(sgp4::MinutesSinceEpoch(360.0))
        .unwrap();
    let expected_p = [-7_154.031_202_02, -3_783.176_825_04, -3_536.194_122_94];
    let expected_v = [4.741_887_409, -4.151_817_765, -2.093_935_425];
    for i in 0..3 {
        assert!((state.position[i] - expected_p[i]).abs() < 1e-6);
        assert!((state.velocity[i] - expected_v[i]).abs() < 1e-9);
    }
}

#[test]
fn real_ephemeris_produces_sane_doppler_pass() {
    use pnt_predictor::{predict, ReceiverState, SatelliteState};

    let store = EphemerisStore::from_tle_file("tests/fixtures/iss.tle").unwrap();
    let epoch = store.epoch(25544).unwrap();
    // Fixed WGS-84 ECEF receiver near Copenhagen, independently generated from
    // geodetic (55.6761 N, 12.5683 E, 0 m) using the standard ellipsoid equations.
    let receiver = ReceiverState {
        position_ecef_m: [3_518_304.71, 784_390.70, 5_244_191.85],
        velocity_ecef_mps: [0.0; 3],
        clock_drift_mps: 0.0,
    };
    let carrier = 1_600_000_000.0;
    let mut passes = vec![Vec::new()];
    for seconds in (-21_600..=21_600).step_by(10) {
        let state = store
            .propagate_ecef(25544, epoch + Duration::seconds(seconds))
            .unwrap();
        if let Ok(prediction) = predict(
            SatelliteState {
                position_ecef_m: state.position_m,
                velocity_ecef_mps: state.velocity_mps,
            },
            receiver,
            0.0,
            carrier,
            0.0,
        ) {
            let needs_new = passes
                .last()
                .and_then(|pass| pass.last())
                .is_some_and(|(previous, _)| seconds - previous > 10);
            if needs_new {
                passes.push(Vec::new());
            }
            passes.last_mut().unwrap().push((seconds, prediction));
        }
    }
    let pass = passes
        .iter()
        .find(|pass| {
            pass.iter().any(|(_, p)| p.correlation_peak_hz > 0.0)
                && pass.iter().any(|(_, p)| p.correlation_peak_hz < 0.0)
        })
        .expect("expected a sign-changing visible pass");
    assert!(
        pass.len() > 10,
        "expected a visible pass in the 12-hour gate window"
    );
    assert!(pass.iter().any(|(_, p)| p.correlation_peak_hz > 0.0));
    assert!(pass.iter().any(|(_, p)| p.correlation_peak_hz < 0.0));
    let approach = pass
        .iter()
        .position(|(_, p)| p.correlation_peak_hz > 0.0)
        .unwrap();
    let recede = pass
        .iter()
        .position(|(_, p)| p.correlation_peak_hz < 0.0)
        .unwrap();
    assert!(approach < recede, "approach must precede recession");
    // ISS orbital speed is below 8 km/s; |f*v/c| < 42.7 kHz at 1.6 GHz.
    assert!(pass
        .iter()
        .all(|(_, p)| p.correlation_peak_hz.abs() < 43_000.0));
    // Ten-second samples cannot jump by 5 kHz in a physical LEO pass.
    assert!(pass
        .windows(2)
        .all(|w| (w[1].1.correlation_peak_hz - w[0].1.correlation_peak_hz).abs() < 5_000.0));
}

#[test]
fn ecef_rotation_has_known_quarter_turn_geometry() {
    let utc = Utc.timestamp_opt(946_728_000, 0).unwrap(); // J2000 epoch
    let gmst = pnt_ephemeris::gmst_rad(utc);
    assert!((gmst.to_degrees() - 280.460_618_37).abs() < 1e-10);
    let state = pnt_ephemeris::teme_to_ecef_at_gmst([1.0, 0.0, 0.0], [0.0; 3], gmst);
    assert!((state.position_m[0] - 181.559_653).abs() < 1e-6);
    assert!((state.position_m[1] - 983.379_932).abs() < 1e-6);
}

#[test]
fn ecef_velocity_includes_earth_rotation_cross_product() {
    // At GMST=0, r=[7000 km,0,0] and TEME v=[1,2,3] km/s. Thus
    // v_ECEF=R*v-(omega cross r)=[1000, 2000-510.44805, 3000] m/s.
    let state = pnt_ephemeris::teme_to_ecef_at_gmst([7_000.0, 0.0, 0.0], [1.0, 2.0, 3.0], 0.0);
    let expected = [1_000.0, 1_489.551_95, 3_000.0];
    for (actual, expected) in state.velocity_mps.iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-9);
    }
}
