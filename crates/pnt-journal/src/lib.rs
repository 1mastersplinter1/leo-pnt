//! Measurement and physically separate truth-journal ports.

use pnt_types::MeasurementEnvelope;

#[derive(Clone, Debug, PartialEq)]
pub struct IntegrityEvent {
    pub monotonic_ns: u64,
    pub source_id: String,
    pub reason: String,
}

pub trait JournalSinks {
    fn write_measurement(&mut self, envelope: &MeasurementEnvelope);
    fn write_truth(&mut self, envelope: &MeasurementEnvelope);
    fn write_integrity(&mut self, event: IntegrityEvent);
}

#[derive(Debug, Default)]
pub struct MemoryJournals {
    measurement_records: Vec<MeasurementEnvelope>,
    truth_records: Vec<MeasurementEnvelope>,
    integrity_events: Vec<IntegrityEvent>,
}

impl MemoryJournals {
    #[must_use]
    pub fn measurement_records(&self) -> &[MeasurementEnvelope] {
        &self.measurement_records
    }

    #[must_use]
    pub fn truth_records(&self) -> &[MeasurementEnvelope] {
        &self.truth_records
    }
    #[must_use]
    pub fn integrity_events(&self) -> &[IntegrityEvent] {
        &self.integrity_events
    }
}

impl JournalSinks for MemoryJournals {
    fn write_measurement(&mut self, envelope: &MeasurementEnvelope) {
        self.measurement_records.push(envelope.clone());
    }

    fn write_truth(&mut self, envelope: &MeasurementEnvelope) {
        self.truth_records.push(envelope.clone());
    }
    fn write_integrity(&mut self, event: IntegrityEvent) {
        self.integrity_events.push(event);
    }
}
