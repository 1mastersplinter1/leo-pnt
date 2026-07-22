//! The sole runtime orchestrator.

use pnt_config::{Config, GnssAuthority};
use pnt_estimator::{Estimator, FilterStub};
use pnt_integrity::{IntegrityAuthorityGate, IntegrityStub};
use pnt_journal::{JournalSinks, MemoryJournals};
use pnt_time::{ClockService, ManualClock};
use pnt_types::{Constellation, MeasurementEnvelope, MeasurementPayload, SolutionEpoch};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingDestination {
    Fusion,
    TruthJournal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingTable {
    pub gnss: Vec<RoutingDestination>,
    pub non_gnss: Vec<RoutingDestination>,
}

pub struct Executive<C, E, I, J> {
    config: Config,
    clock: C,
    filter: E,
    integrity: I,
    journals: J,
    solution_epochs: Vec<SolutionEpoch>,
}

impl<C, E, I, J> Executive<C, E, I, J>
where
    C: ClockService,
    E: Estimator,
    I: IntegrityAuthorityGate,
    J: JournalSinks,
{
    pub fn new(config: Config, clock: C, filter: E, integrity: I, journals: J) -> Self {
        Self {
            config,
            clock,
            filter,
            integrity,
            journals,
            solution_epochs: Vec::new(),
        }
    }

    #[must_use]
    pub fn routing_table(authority: GnssAuthority) -> RoutingTable {
        let gnss = match authority {
            GnssAuthority::Production => {
                vec![RoutingDestination::Fusion, RoutingDestination::TruthJournal]
            }
            GnssAuthority::RecordedOnly => vec![RoutingDestination::TruthJournal],
            GnssAuthority::Off => Vec::new(),
        };
        RoutingTable {
            gnss,
            non_gnss: vec![RoutingDestination::Fusion],
        }
    }

    pub fn process(&mut self, mut envelope: MeasurementEnvelope) -> Vec<RoutingDestination> {
        envelope.host_receive_monotonic_ns = self.clock.ingress_monotonic_ns();
        let routes = self.routes_for(&envelope.payload);
        for route in &routes {
            match route {
                RoutingDestination::Fusion => self.dispatch_to_fusion(&envelope),
                RoutingDestination::TruthJournal => self.journals.write_truth(&envelope),
            }
        }
        routes
    }

    fn routes_for(&self, payload: &MeasurementPayload) -> Vec<RoutingDestination> {
        match payload {
            MeasurementPayload::Gnss(_) => Self::routing_table(self.config.gnss_authority).gnss,
            MeasurementPayload::TrackerDoppler(observation)
                if observation.constellation == Constellation::Orbcomm =>
            {
                Vec::new()
            }
            _ => Self::routing_table(self.config.gnss_authority).non_gnss,
        }
    }

    fn dispatch_to_fusion(&mut self, envelope: &MeasurementEnvelope) {
        self.journals.write_measurement(envelope);
        if let MeasurementPayload::Imu(imu) = &envelope.payload {
            self.filter.propagate(*imu);
            return;
        }
        self.filter.update(envelope);
        let state = self.filter.state();
        let monotonic_ns = envelope.host_receive_monotonic_ns;
        let steering_authorised = self.integrity.steering_authorised(&state, monotonic_ns);
        self.solution_epochs.push(SolutionEpoch {
            monotonic_ns,
            state,
            steering_authorised,
        });
    }

    #[must_use]
    pub const fn filter(&self) -> &E {
        &self.filter
    }

    #[must_use]
    pub const fn journals(&self) -> &J {
        &self.journals
    }

    pub fn take_solution_epochs(&mut self) -> Vec<SolutionEpoch> {
        std::mem::take(&mut self.solution_epochs)
    }
}

impl Executive<ManualClock, FilterStub, IntegrityStub, MemoryJournals> {
    #[must_use]
    pub fn test_default(gnss_authority: GnssAuthority) -> Self {
        Self::new(
            Config { gnss_authority },
            ManualClock::default(),
            FilterStub::default(),
            IntegrityStub,
            MemoryJournals::default(),
        )
    }
}
