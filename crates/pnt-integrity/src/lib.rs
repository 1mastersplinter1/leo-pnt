//! Deterministic solution-integrity and steering-authority supervisor.

use pnt_types::{ArmAction, ArmCommand, FilterState};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProtectionLimits {
    pub horizontal_position_m: Option<f64>,
    pub horizontal_velocity_mps: Option<f64>,
    pub heading_rad: Option<f64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AuthorityParams {
    pub aided: ProtectionLimits,
    pub denied: ProtectionLimits,
    pub t_lease_s: Option<f64>,
    pub t_dr_s: Option<f64>,
    pub t_eph_s: Option<f64>,
    pub dwell_clear_s: Option<f64>,
    pub dwell_rearm_s: Option<f64>,
    pub caution_enter: Option<f64>,
    pub caution_clear: Option<f64>,
    pub revoke_threshold: Option<f64>,
    pub t_ack_s: Option<f64>,
}

impl AuthorityParams {
    #[must_use]
    pub fn is_complete(self) -> bool {
        [
            self.aided.horizontal_position_m,
            self.aided.horizontal_velocity_mps,
            self.aided.heading_rad,
            self.denied.horizontal_position_m,
            self.denied.horizontal_velocity_mps,
            self.denied.heading_rad,
            self.t_lease_s,
            self.t_dr_s,
            self.t_eph_s,
            self.dwell_clear_s,
            self.dwell_rearm_s,
            self.caution_enter,
            self.caution_clear,
            self.revoke_threshold,
            self.t_ack_s,
        ]
        .into_iter()
        .all(|value| value.is_some_and(|value| value.is_finite() && value >= 0.0))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityProfile {
    Aided,
    Denied,
}

#[derive(Clone, Copy, Debug)]
pub struct AuthoritySolution<'a> {
    pub sequence: u64,
    pub state: &'a FilterState,
    pub profile: AuthorityProfile,
    pub last_absolute_observation_ns: Option<u64>,
    pub ephemeris_age_s: Option<f64>,
    pub calibration_id: Option<&'a str>,
}

pub trait IntegrityAuthorityGate {
    fn solution(&mut self, _solution: AuthoritySolution<'_>, _monotonic_ns: u64) {}
    fn steering_authorised(&mut self, state: &FilterState, monotonic_ns: u64) -> bool;
    fn arm_command(&mut self, _command: &ArmCommand) {}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct IntegrityStub;

impl IntegrityAuthorityGate for IntegrityStub {
    fn steering_authorised(&mut self, _state: &FilterState, _monotonic_ns: u64) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityState {
    Disarmed,
    Nominal,
    Caution,
    Warning,
    Escalated,
    LatchedSafe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityEvent {
    Disarm,
    Arm,
    G2Fall,
    G2Rise,
    G3Fall,
    G3Rise,
    LeaseExpiry,
    AckTimeout,
    Acknowledge,
    CautionEnter,
    CautionClear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlarmLevel {
    QuietDisarmed,
    QuietArmed,
    PreAlert,
    LoudDemandAck,
    MaximumContinuous,
    SteadyFault,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorityOutput {
    pub steering_authorised: bool,
    pub state: AuthorityState,
    pub alarm: AlarmLevel,
    pub transition: Option<(AuthorityState, AuthorityState, AuthorityEvent)>,
}

impl AuthorityState {
    #[must_use]
    pub const fn output(self) -> (bool, AlarmLevel) {
        match self {
            Self::Disarmed => (false, AlarmLevel::QuietDisarmed),
            Self::Nominal => (true, AlarmLevel::QuietArmed),
            Self::Caution => (true, AlarmLevel::PreAlert),
            Self::Warning => (false, AlarmLevel::LoudDemandAck),
            Self::Escalated => (false, AlarmLevel::MaximumContinuous),
            Self::LatchedSafe => (false, AlarmLevel::SteadyFault),
        }
    }
}

/// Literal successor function from `SAFETY_CASE.md` §2.2. Guarded arm cells are
/// represented by `arm_guard`; all other dots are self-loops.
#[must_use]
pub const fn matrix_successor(
    state: AuthorityState,
    event: AuthorityEvent,
    arm_guard: bool,
) -> AuthorityState {
    use AuthorityEvent as E;
    use AuthorityState as S;
    match (state, event) {
        (S::Disarmed | S::LatchedSafe, E::Arm) if arm_guard => S::Nominal,
        (S::Nominal | S::Caution, E::Disarm) => S::Disarmed,
        (S::Nominal | S::Caution, E::G2Fall | E::G3Fall | E::LeaseExpiry) => S::Warning,
        (S::Nominal, E::CautionEnter) => S::Caution,
        (S::Caution, E::CautionClear) => S::Nominal,
        (S::Warning, E::AckTimeout) => S::Escalated,
        (S::Warning | S::Escalated, E::Acknowledge) => S::LatchedSafe,
        _ => state,
    }
}

/// Resolves simultaneous events using the normative §2.2 priority order.
#[must_use]
pub fn simultaneous_successor(
    state: AuthorityState,
    events: &[AuthorityEvent],
    arm_guard: bool,
) -> AuthorityState {
    use AuthorityEvent::{
        AckTimeout, Acknowledge, Arm, CautionClear, CautionEnter, Disarm, G2Fall, G2Rise, G3Fall,
        G3Rise, LeaseExpiry,
    };
    const PRIORITY: [AuthorityEvent; 11] = [
        G3Fall,
        G2Fall,
        LeaseExpiry,
        Disarm,
        AckTimeout,
        Acknowledge,
        Arm,
        G2Rise,
        G3Rise,
        CautionEnter,
        CautionClear,
    ];
    for event in PRIORITY {
        if events.contains(&event) {
            let successor = matrix_successor(state, event, arm_guard);
            if successor != state {
                return successor;
            }
        }
    }
    state
}

#[allow(clippy::struct_excessive_bools)]
pub struct AuthoritySupervisor {
    params: AuthorityParams,
    state: AuthorityState,
    armed: bool,
    validator: Box<dyn Fn(&str) -> bool + Send + Sync>,
    last_sequence: Option<u64>,
    lease_deadline_ns: Option<u64>,
    g2: bool,
    g3: bool,
    warning_since_ns: Option<u64>,
    latched_since_ns: Option<u64>,
    caution_clear_since_ns: Option<u64>,
    pending_arm_edge: bool,
    transition: Option<(AuthorityState, AuthorityState, AuthorityEvent)>,
}

impl AuthoritySupervisor {
    #[must_use]
    pub fn new(params: AuthorityParams) -> Self {
        Self::with_calibration_validator(params, |_| true)
    }

    #[must_use]
    pub fn with_calibration_validator<F>(params: AuthorityParams, validator: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        Self {
            params,
            state: AuthorityState::Disarmed,
            armed: false,
            validator: Box::new(validator),
            last_sequence: None,
            lease_deadline_ns: None,
            g2: false,
            g3: false,
            warning_since_ns: None,
            latched_since_ns: None,
            caution_clear_since_ns: None,
            pending_arm_edge: false,
            transition: None,
        }
    }

    #[must_use]
    pub const fn state(&self) -> AuthorityState {
        self.state
    }

    #[must_use]
    pub fn output(&self) -> AuthorityOutput {
        let (state_authority, alarm) = self.state.output();
        AuthorityOutput {
            steering_authorised: state_authority
                && self.params.is_complete()
                && self.armed
                && self.g2
                && self.g3
                && self.lease_deadline_ns.is_some(),
            state: self.state,
            alarm,
            transition: self.transition,
        }
    }

    pub fn acknowledge(&mut self, now_ns: u64) {
        self.apply(AuthorityEvent::Acknowledge, now_ns, false);
    }

    fn apply(&mut self, event: AuthorityEvent, now_ns: u64, arm_guard: bool) {
        let old = self.state;
        let new = matrix_successor(old, event, arm_guard);
        if new != old {
            self.state = new;
            self.transition = Some((old, new, event));
            if new == AuthorityState::Warning {
                self.armed = false;
                self.warning_since_ns = Some(now_ns);
            }
            if new == AuthorityState::LatchedSafe {
                self.latched_since_ns = Some(now_ns);
            }
            if new == AuthorityState::Disarmed {
                self.lease_deadline_ns = None;
            }
        }
    }

    fn seconds_ns(value: f64) -> u64 {
        std::time::Duration::try_from_secs_f64(value)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_nanos()).ok())
            .unwrap_or(u64::MAX)
    }

    #[allow(clippy::too_many_lines)]
    fn evaluate_solution(&mut self, solution: AuthoritySolution<'_>, now_ns: u64) {
        self.transition = None;
        let limits = match solution.profile {
            AuthorityProfile::Aided => self.params.aided,
            AuthorityProfile::Denied => self.params.denied,
        };
        let heading_accuracy = solution
            .state
            .covariance
            .get(6 * solution.state.covariance_dimension + 6)
            .copied()
            .filter(|value| value.is_finite() && *value >= 0.0)
            .map(f64::sqrt);
        let age_ok = match (solution.last_absolute_observation_ns, self.params.t_dr_s) {
            (Some(last), Some(limit)) => now_ns.saturating_sub(last) <= Self::seconds_ns(limit),
            _ => false,
        };
        let eph_ok = match (solution.ephemeris_age_s, self.params.t_eph_s) {
            (Some(age), Some(limit)) => age.is_finite() && age >= 0.0 && age <= limit,
            _ => false,
        };
        let metric = solution.state.horizontal_accuracy_m();
        let new_g2 = self.params.is_complete()
            && limits
                .horizontal_position_m
                .is_some_and(|limit| solution.state.horizontal_accuracy_m() <= limit)
            && limits
                .horizontal_velocity_mps
                .is_some_and(|limit| solution.state.speed_accuracy_mps() <= limit)
            && limits
                .heading_rad
                .zip(heading_accuracy)
                .is_some_and(|(limit, value)| value <= limit)
            && age_ok
            && eph_ok
            && self
                .params
                .revoke_threshold
                .is_some_and(|limit| metric < limit);
        let new_g3 = solution
            .calibration_id
            .is_some_and(|id| !id.is_empty() && (self.validator)(id));
        let advanced = self
            .last_sequence
            .is_none_or(|last| solution.sequence > last);
        self.last_sequence = Some(solution.sequence);
        if advanced && new_g2 && new_g3 {
            if let Some(seconds) = self.params.t_lease_s {
                self.lease_deadline_ns = Some(now_ns.saturating_add(Self::seconds_ns(seconds)));
            }
        }
        let fault = if self.g3 && !new_g3 {
            Some(AuthorityEvent::G3Fall)
        } else if self.g2 && !new_g2 {
            Some(AuthorityEvent::G2Fall)
        } else {
            None
        };
        self.g2 = new_g2;
        self.g3 = new_g3;
        if matches!(
            self.state,
            AuthorityState::Nominal | AuthorityState::Caution
        ) && (!new_g3 || !new_g2)
        {
            self.apply(
                fault.unwrap_or(if new_g3 {
                    AuthorityEvent::G2Fall
                } else {
                    AuthorityEvent::G3Fall
                }),
                now_ns,
                false,
            );
        } else if self.pending_arm_edge {
            let rearm_elapsed = self
                .latched_since_ns
                .zip(self.params.dwell_rearm_s)
                .is_none_or(|(start, dwell)| {
                    now_ns.saturating_sub(start) >= Self::seconds_ns(dwell)
                });
            let before = self.state;
            self.apply(
                AuthorityEvent::Arm,
                now_ns,
                advanced && new_g2 && new_g3 && rearm_elapsed,
            );
            if self.state != before {
                self.pending_arm_edge = false;
            }
        }
        if self.state == AuthorityState::Nominal
            && self.params.caution_enter.is_some_and(|v| metric >= v)
        {
            self.apply(AuthorityEvent::CautionEnter, now_ns, false);
        } else if self.state == AuthorityState::Caution
            && self.params.caution_clear.is_some_and(|v| metric <= v)
        {
            let start = *self.caution_clear_since_ns.get_or_insert(now_ns);
            if self
                .params
                .dwell_clear_s
                .is_some_and(|d| now_ns.saturating_sub(start) >= Self::seconds_ns(d))
            {
                self.apply(AuthorityEvent::CautionClear, now_ns, false);
                self.caution_clear_since_ns = None;
            }
        } else {
            self.caution_clear_since_ns = None;
        }
    }

    fn tick(&mut self, now_ns: u64) {
        if matches!(
            self.state,
            AuthorityState::Nominal | AuthorityState::Caution
        ) && self
            .lease_deadline_ns
            .is_some_and(|deadline| now_ns >= deadline)
        {
            self.lease_deadline_ns = None;
            self.apply(AuthorityEvent::LeaseExpiry, now_ns, false);
        }
        if self.state == AuthorityState::Warning
            && self
                .warning_since_ns
                .zip(self.params.t_ack_s)
                .is_some_and(|(start, limit)| {
                    now_ns.saturating_sub(start) >= Self::seconds_ns(limit)
                })
        {
            self.apply(AuthorityEvent::AckTimeout, now_ns, false);
        }
    }
}

impl IntegrityAuthorityGate for AuthoritySupervisor {
    fn solution(&mut self, solution: AuthoritySolution<'_>, monotonic_ns: u64) {
        self.evaluate_solution(solution, monotonic_ns);
    }

    fn steering_authorised(&mut self, _state: &FilterState, monotonic_ns: u64) -> bool {
        self.tick(monotonic_ns);
        let mut output = self.output();
        if output.steering_authorised
            && self
                .lease_deadline_ns
                .is_some_and(|deadline| monotonic_ns >= deadline)
        {
            output.steering_authorised = false;
        }
        output.steering_authorised
    }

    fn arm_command(&mut self, command: &ArmCommand) {
        match command.action {
            ArmAction::Disarm => {
                self.armed = false;
                self.pending_arm_edge = false;
                self.apply(AuthorityEvent::Disarm, command.host_monotonic_ns, false);
            }
            ArmAction::Arm if !self.armed => {
                self.armed = true;
                self.pending_arm_edge = true;
            }
            ArmAction::Arm => {}
        }
    }
}
