use chrono::{Duration, TimeZone, Utc};
use pnt_ephemeris::{EphemerisError, EphemerisStore};

#[test]
fn parses_local_tle_and_omm_json() {
    let tle = EphemerisStore::from_tle_file("tests/fixtures/iss.tle").unwrap();
    let omm = EphemerisStore::from_omm_json_file("tests/fixtures/iss.json").unwrap();
    assert!(tle.contains(25544));
    assert!(omm.contains(25544));
}

#[test]
fn default_six_hour_age_gate_is_typed() {
    let store = EphemerisStore::from_tle_file("tests/fixtures/iss.tle").unwrap();
    let epoch = store.epoch(25544).unwrap();
    assert!(store.propagate_ecef(25544, epoch + Duration::hours(6)).is_ok());
    assert!(matches!(store.propagate_ecef(25544, epoch + Duration::hours(6) + Duration::nanoseconds(1)), Err(EphemerisError::TooOld { .. })));
}

#[test]
fn vallado_reference_vector_case_00005() {
    // Vallado et al. SGP4 verification case 00005, t=0 min (SGP4-VER.TLE/tcppver.out).
    // Values are TEME km and km/s, not produced by this crate.
    let tle = "0 VANGUARD 1\n1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753\n2 00005  34.2682 331.5174 1849677 331.7664  19.3264 10.82419157413667\n";
    let store = EphemerisStore::from_tle_str(tle).unwrap();
    let state = store.propagate_teme(5, store.epoch(5).unwrap()).unwrap();
    let expected_p = [7022.465_292_66, -1400.082_967_55, 0.039_951_55];
    let expected_v = [1.893_841_015, 6.405_893_759, 4.534_807_250];
    for i in 0..3 { assert!((state.position_km[i] - expected_p[i]).abs() < 1e-6); assert!((state.velocity_kmps[i] - expected_v[i]).abs() < 1e-9); }
}

#[test]
fn ecef_rotation_has_known_quarter_turn_geometry() {
    let utc = Utc.timestamp_opt(946_728_000, 0).unwrap(); // J2000 instant
    let state = pnt_ephemeris::teme_to_ecef_at_gmst([1.0, 0.0, 0.0], [0.0; 3], core::f64::consts::FRAC_PI_2);
    assert!(state.position_m[0].abs() < 1e-9);
    assert!((state.position_m[1] + 1000.0).abs() < 1e-9);
    assert_eq!(utc.timestamp(), 946_728_000);
}
