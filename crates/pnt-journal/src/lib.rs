//! Measurement and physically separate truth-journal ports.

use pnt_types::MeasurementEnvelope;

pub trait JournalSinks {
    fn write_measurement(&mut self, envelope: &MeasurementEnvelope);
    fn write_truth(&mut self, envelope: &MeasurementEnvelope);
}

#[derive(Debug, Default)]
pub struct MemoryJournals {
    measurement_records: Vec<MeasurementEnvelope>,
    truth_records: Vec<MeasurementEnvelope>,
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
}

impl JournalSinks for MemoryJournals {
    fn write_measurement(&mut self, envelope: &MeasurementEnvelope) {
        self.measurement_records.push(envelope.clone());
    }

    fn write_truth(&mut self, envelope: &MeasurementEnvelope) {
        self.truth_records.push(envelope.clone());
    }
}
