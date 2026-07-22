//! Solution-integrity and steering-authority ports.

use pnt_types::{ArmCommand, FilterState};

pub trait IntegrityAuthorityGate {
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
