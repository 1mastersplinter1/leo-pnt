use pnt_integrity::{
    matrix_successor, simultaneous_successor, AlarmLevel, AuthorityEvent as E, AuthorityParams,
    AuthorityProfile, AuthoritySolution, AuthorityState as S, AuthoritySupervisor,
    IntegrityAuthorityGate, ProtectionLimits,
};
use pnt_types::{ArmAction, ArmCommand, FilterState, SourceId};

fn params() -> AuthorityParams {
    AuthorityParams {
        aided: ProtectionLimits {
            horizontal_position_m: Some(10.0),
            horizontal_velocity_mps: Some(10.0),
            heading_rad: Some(2.0),
        },
        denied: ProtectionLimits {
            horizontal_position_m: Some(20.0),
            horizontal_velocity_mps: Some(20.0),
            heading_rad: Some(2.0),
        },
        t_lease_s: Some(1.0),
        t_dr_s: Some(5.0),
        t_eph_s: Some(10.0),
        dwell_clear_s: Some(1.0),
        dwell_rearm_s: Some(1.0),
        caution_enter: Some(2.0),
        caution_clear: Some(1.0),
        revoke_threshold: Some(9.0),
        t_ack_s: Some(1.0),
    }
}

fn state_with_accuracy(accuracy: f64) -> FilterState {
    let mut state = FilterState::default();
    state.covariance.fill(0.0);
    state.covariance[0] = accuracy * accuracy / 2.0;
    state.covariance[10] = accuracy * accuracy / 2.0;
    state.covariance[6 * 9 + 6] = 0.01;
    state.covariance[3 * 9 + 3] = 0.01;
    state.covariance[4 * 9 + 4] = 0.01;
    state
}

fn arm(supervisor: &mut AuthoritySupervisor, now: u64) {
    supervisor.arm_command(&ArmCommand {
        action: ArmAction::Arm,
        host_monotonic_ns: now,
        source_id: SourceId("helm".into()),
    });
}

fn solution(supervisor: &mut AuthoritySupervisor, sequence: u64, now: u64, state: &FilterState) {
    supervisor.solution(
        AuthoritySolution {
            sequence,
            state,
            profile: AuthorityProfile::Aided,
            last_absolute_observation_ns: Some(0),
            ephemeris_age_s: Some(0.0),
            calibration_id: Some("cal"),
        },
        now,
    );
}

#[test]
#[allow(clippy::many_single_char_names)]
fn exhaustive_safety_case_section_2_2_matrix() {
    let states = [
        S::Disarmed,
        S::Nominal,
        S::Caution,
        S::Warning,
        S::Escalated,
        S::LatchedSafe,
    ];
    let events = [
        E::Disarm,
        E::Arm,
        E::G2Fall,
        E::G2Rise,
        E::G3Fall,
        E::G3Rise,
        E::LeaseExpiry,
        E::AckTimeout,
        E::Acknowledge,
        E::CautionEnter,
        E::CautionClear,
    ];
    let d = S::Disarmed;
    let n = S::Nominal;
    let c = S::Caution;
    let w = S::Warning;
    let x = S::Escalated;
    let l = S::LatchedSafe;
    // Literal transcription of SAFETY_CASE.md §2.2, with guarded N? evaluated true.
    let expected = [
        [d, n, d, d, d, d, d, d, d, d, d],
        [d, n, w, n, w, n, w, n, n, c, n],
        [d, c, w, c, w, c, w, c, c, c, n],
        [w, w, w, w, w, w, w, x, l, w, w],
        [x, x, x, x, x, x, x, x, l, x, x],
        [l, n, l, l, l, l, l, l, l, l, l],
    ];
    for (row, state) in states.into_iter().enumerate() {
        for (column, event) in events.into_iter().enumerate() {
            assert_eq!(
                matrix_successor(state, event, true),
                expected[row][column],
                "§2.2 cell ({state:?}, {event:?})"
            );
        }
    }
    assert_eq!(matrix_successor(S::Disarmed, E::Arm, false), S::Disarmed);
    assert_eq!(
        matrix_successor(S::LatchedSafe, E::Arm, false),
        S::LatchedSafe
    );
    assert_eq!(
        simultaneous_successor(S::Nominal, &[E::Disarm, E::G2Fall], true),
        S::Warning
    );
    assert_eq!(
        simultaneous_successor(S::Warning, &[E::Acknowledge, E::AckTimeout], true),
        S::Escalated
    );
    for state in states {
        let (authorised, alarm) = state.output();
        assert_eq!(authorised, matches!(state, S::Nominal | S::Caution));
        assert_eq!(alarm == AlarmLevel::QuietArmed, state == S::Nominal);
    }
}

#[test]
fn every_missing_parameter_is_fail_closed() {
    let good = state_with_accuracy(0.5);
    for index in 0..15 {
        let mut p = params();
        let fields = [
            &mut p.aided.horizontal_position_m,
            &mut p.aided.horizontal_velocity_mps,
            &mut p.aided.heading_rad,
            &mut p.denied.horizontal_position_m,
            &mut p.denied.horizontal_velocity_mps,
            &mut p.denied.heading_rad,
            &mut p.t_lease_s,
            &mut p.t_dr_s,
            &mut p.t_eph_s,
            &mut p.dwell_clear_s,
            &mut p.dwell_rearm_s,
            &mut p.caution_enter,
            &mut p.caution_clear,
            &mut p.revoke_threshold,
            &mut p.t_ack_s,
        ];
        *fields.into_iter().nth(index).unwrap() = None;
        let mut s = AuthoritySupervisor::with_calibration_validator(p, |id| id == "cal");
        arm(&mut s, 0);
        solution(&mut s, 1, 0, &good);
        assert!(
            !s.steering_authorised(&good, 0),
            "missing field {index} granted"
        );
    }
}

#[test]
fn lease_is_non_circular_and_requires_sequence_advance() {
    let good = state_with_accuracy(0.5);
    let mut s = AuthoritySupervisor::with_calibration_validator(params(), |_| true);
    arm(&mut s, 0);
    solution(&mut s, 1, 0, &good);
    assert!(s.steering_authorised(&good, 999_999_999));
    assert!(!s.steering_authorised(&good, 1_000_000_000));
    s.acknowledge(1_000_000_000);
    arm(&mut s, 2_000_000_000);
    solution(&mut s, 1, 2_000_000_000, &good);
    assert!(
        !s.steering_authorised(&good, 2_000_000_000),
        "same sequence renewed"
    );
    solution(&mut s, 2, 2_000_000_001, &good);
    assert!(s.steering_authorised(&good, 2_000_000_001));
}

#[test]
fn dwell_and_ack_boundaries_are_exact() {
    let good = state_with_accuracy(0.5);
    let caution = state_with_accuracy(2.5);
    let bad = state_with_accuracy(9.0);
    let mut s = AuthoritySupervisor::with_calibration_validator(params(), |_| true);
    arm(&mut s, 0);
    solution(&mut s, 1, 0, &good);
    solution(&mut s, 2, 1, &caution);
    solution(&mut s, 3, 10, &good);
    solution(&mut s, 4, 1_000_000_009, &good);
    assert_eq!(s.state(), S::Caution);
    solution(&mut s, 5, 1_000_000_010, &good);
    assert_eq!(s.state(), S::Nominal);
    solution(&mut s, 6, 1_000_000_011, &bad);
    assert_eq!(s.state(), S::Warning);
    assert!(!s.steering_authorised(&bad, 2_000_000_010));
    assert_eq!(s.state(), S::Warning);
    assert!(!s.steering_authorised(&bad, 2_000_000_011));
    assert_eq!(s.state(), S::Escalated);
}

#[test]
fn revocation_is_latched_until_ack_dwell_and_fresh_arm() {
    let good = state_with_accuracy(0.5);
    let bad = state_with_accuracy(9.0);
    let mut s = AuthoritySupervisor::with_calibration_validator(params(), |_| true);
    arm(&mut s, 0);
    solution(&mut s, 1, 0, &good);
    solution(&mut s, 2, 1, &bad);
    solution(&mut s, 3, 2, &good);
    assert!(!s.steering_authorised(&good, 2));
    s.acknowledge(2);
    arm(&mut s, 3);
    solution(&mut s, 4, 1_000_000_001, &good);
    assert!(!s.steering_authorised(&good, 1_000_000_001));
    solution(&mut s, 5, 1_000_000_002, &good);
    assert!(s.steering_authorised(&good, 1_000_000_002));
}

#[test]
fn random_sequences_never_authorise_with_a_false_g_condition() {
    let good = state_with_accuracy(0.5);
    let bad = state_with_accuracy(9.0);
    let mut seed = 0x5eed_u64;
    for _case in 0..32 {
        let mut s = AuthoritySupervisor::with_calibration_validator(params(), |id| id == "cal");
        let mut now = 0_u64;
        for sequence in 1..200 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            now += seed % 20_000_000;
            if seed.trailing_zeros() >= 3 {
                arm(&mut s, now);
            }
            if seed & 31 == 1 {
                s.arm_command(&ArmCommand {
                    action: ArmAction::Disarm,
                    host_monotonic_ns: now,
                    source_id: SourceId("helm".into()),
                });
            }
            let g2 = seed & 2 == 0;
            let g3 = seed & 4 == 0;
            let state = if g2 { &good } else { &bad };
            s.solution(
                AuthoritySolution {
                    sequence,
                    state,
                    profile: AuthorityProfile::Aided,
                    last_absolute_observation_ns: Some(now),
                    ephemeris_age_s: Some(0.0),
                    calibration_id: g3.then_some("cal"),
                },
                now,
            );
            let authorised = s.steering_authorised(state, now);
            assert!(
                !authorised || (g2 && g3),
                "authorised with false G at sequence {sequence}"
            );
        }
    }
}
