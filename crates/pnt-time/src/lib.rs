//! Runtime clock ownership and timestamping port.

pub trait ClockService {
    fn ingress_monotonic_ns(&mut self) -> u64;
}

#[derive(Debug, Default)]
pub struct ManualClock {
    monotonic_ns: u64,
}

impl ClockService for ManualClock {
    fn ingress_monotonic_ns(&mut self) -> u64 {
        self.monotonic_ns += 1;
        self.monotonic_ns
    }
}
