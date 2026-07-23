//! Deterministic reference-correlation Doppler acquisition and tracking.
//!
//! The reported frequency is the correlation peak's offset from the configured nominal
//! carrier, not an RF carrier estimate. A failed threshold produces [`NoDetection`]; this
//! crate never fills a requested output rate with synthetic observations.

#![allow(clippy::cast_precision_loss)] // Sample counters are far below f64's exact-integer limit.

pub mod synth;

use std::f64::consts::TAU;
use std::fmt;
use std::sync::Arc;

use num_complex::Complex64;
use pnt_types::{
    Constellation, Frame, MeasurementEnvelope, MeasurementPayload, Provenance, QualityFlags,
    SourceId, TimeTag, TrackerDoppler, UtcTime, SCHEMA_VERSION,
};
use rustfft::{Fft, FftPlanner};

/// Configuration for acquisition and block-to-block tracking.
#[derive(Clone, Debug)]
pub struct TrackerConfig {
    pub sample_rate_hz: f64,
    pub min_frequency_hz: f64,
    pub max_frequency_hz: f64,
    pub frequency_bin_hz: f64,
    /// Minimum peak-to-noise-floor power ratio.
    ///
    /// The default of 32 (15.1 dB) is an engineering starting point accounting for the
    /// multiple-cell frequency × delay search **[UNVERIFIED — pending link-budget,
    /// false-alarm analysis, and real-capture tuning]**. It is deliberately explicit.
    pub detection_threshold: f64,
    /// Half-width of the search about the extrapolated tracked frequency.
    pub tracking_half_width_hz: f64,
}

impl TrackerConfig {
    pub const DEFAULT_DETECTION_THRESHOLD: f64 = 32.0;

    /// Validates configuration and constructs a tracker.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when rates, bounds, thresholds, or reference samples are invalid.
    pub fn build(self, reference: Vec<Complex64>) -> Result<CorrelationTracker, ConfigError> {
        CorrelationTracker::new(self, reference)
    }
}

/// A detected correlation peak for one block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Detection {
    pub correlation_peak_hz: f64,
    pub quality: f64,
    pub sample_time_ns: u64,
    pub delay_samples: usize,
}

impl Detection {
    /// Wraps a detection in the v2/v4.1 measurement envelope.
    ///
    /// `norad_catalog_id` must be the satellite's decimal NORAD catalogue ID. UTC is
    /// required by contract v4.1 and is therefore not optional here.
    #[must_use]
    pub fn into_envelope(self, metadata: EnvelopeMetadata<'_>) -> MeasurementEnvelope {
        MeasurementEnvelope {
            schema_version: SCHEMA_VERSION,
            source_id: SourceId(metadata.norad_catalog_id.to_owned()),
            sequence: metadata.sequence,
            sample_time: TimeTag::HostMonotonicNanoseconds(self.sample_time_ns),
            host_receive_monotonic_ns: metadata.host_receive_monotonic_ns,
            utc: Some(metadata.utc),
            payload: MeasurementPayload::TrackerDoppler(TrackerDoppler {
                constellation: metadata.constellation,
                correlation_peak_hz: self.correlation_peak_hz,
                nominal_carrier_hz: metadata.nominal_carrier_hz,
            }),
            frame: Frame::FrameIndependent,
            covariance: vec![metadata.frequency_variance_hz2],
            quality: QualityFlags::VALID,
            calibration_id: metadata.calibration_id.to_owned(),
            provenance: Provenance::CaptureRecord(metadata.capture_record.to_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnvelopeMetadata<'a> {
    pub norad_catalog_id: &'a str,
    pub sequence: u64,
    pub host_receive_monotonic_ns: u64,
    pub utc: UtcTime,
    pub constellation: Constellation,
    pub nominal_carrier_hz: f64,
    pub frequency_variance_hz2: f64,
    pub calibration_id: &'a str,
    pub capture_record: &'a str,
}

/// Typed absence of an observation. No envelope constructor exists for this value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoDetection {
    pub best_quality: f64,
    pub threshold: f64,
    pub sample_time_ns: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackOutcome {
    Detection(Detection),
    NoDetection(NoDetection),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError(&'static str);

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for ConfigError {}

/// Stateful FFT correlation tracker. State is used only to narrow and extrapolate search;
/// every emitted frequency remains a measurement from the current IQ block.
pub struct CorrelationTracker {
    config: TrackerConfig,
    reference: Vec<Complex64>,
    reference_spectrum_conjugate: Vec<Complex64>,
    fft: Arc<dyn Fft<f64>>,
    inverse_fft: Arc<dyn Fft<f64>>,
    previous: Option<(f64, u64)>,
    drift_hz_per_s: f64,
}

impl CorrelationTracker {
    fn new(config: TrackerConfig, reference: Vec<Complex64>) -> Result<Self, ConfigError> {
        if reference.len() < 4 || !reference.len().is_power_of_two() {
            return Err(ConfigError(
                "reference length must be a power of two and at least four",
            ));
        }
        if !config.sample_rate_hz.is_finite() || config.sample_rate_hz <= 0.0 {
            return Err(ConfigError("sample rate must be finite and positive"));
        }
        if !config.frequency_bin_hz.is_finite() || config.frequency_bin_hz <= 0.0 {
            return Err(ConfigError(
                "frequency bin width must be finite and positive",
            ));
        }
        if !config.min_frequency_hz.is_finite()
            || !config.max_frequency_hz.is_finite()
            || config.min_frequency_hz > config.max_frequency_hz
            || config.min_frequency_hz < -config.sample_rate_hz / 2.0
            || config.max_frequency_hz >= config.sample_rate_hz / 2.0
        {
            return Err(ConfigError(
                "frequency search must be ordered and within Nyquist",
            ));
        }
        if !config.detection_threshold.is_finite() || config.detection_threshold <= 0.0 {
            return Err(ConfigError(
                "detection threshold must be finite and positive",
            ));
        }
        if !config.tracking_half_width_hz.is_finite() || config.tracking_half_width_hz < 0.0 {
            return Err(ConfigError(
                "tracking half width must be finite and non-negative",
            ));
        }
        if reference
            .iter()
            .any(|sample| !sample.re.is_finite() || !sample.im.is_finite())
        {
            return Err(ConfigError("reference samples must be finite"));
        }

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(reference.len());
        let inverse_fft = planner.plan_fft_inverse(reference.len());
        let mut reference_spectrum_conjugate = reference.clone();
        fft.process(&mut reference_spectrum_conjugate);
        for value in &mut reference_spectrum_conjugate {
            *value = value.conj();
        }

        Ok(Self {
            config,
            reference,
            reference_spectrum_conjugate,
            fft,
            inverse_fft,
            previous: None,
            drift_hz_per_s: 0.0,
        })
    }

    /// Searches frequency bins and all circular time offsets using FFT correlation.
    pub fn process_block(&mut self, iq: &[Complex64], sample_time_ns: u64) -> TrackOutcome {
        if iq.len() != self.reference.len()
            || iq
                .iter()
                .any(|sample| !sample.re.is_finite() || !sample.im.is_finite())
        {
            return TrackOutcome::NoDetection(NoDetection {
                best_quality: 0.0,
                threshold: self.config.detection_threshold,
                sample_time_ns,
            });
        }

        let (search_min, search_max) = self.search_bounds(sample_time_ns);
        let bins = frequency_bins(search_min, search_max, self.config.frequency_bin_hz);
        let mut best = SearchPeak {
            power: 0.0,
            noise_floor: f64::INFINITY,
            frequency_hz: 0.0,
            delay: 0,
        };
        let mut work = vec![Complex64::new(0.0, 0.0); iq.len()];

        for frequency_hz in bins {
            for (index, (output, input)) in work.iter_mut().zip(iq).enumerate() {
                let phase = -TAU * frequency_hz * index as f64 / self.config.sample_rate_hz;
                *output = *input * Complex64::from_polar(1.0, phase);
            }
            self.fft.process(&mut work);
            for (value, reference) in work.iter_mut().zip(&self.reference_spectrum_conjugate) {
                *value *= reference;
            }
            self.inverse_fft.process(&mut work);

            let (delay, power) = work
                .iter()
                .map(Complex64::norm_sqr)
                .enumerate()
                .max_by(|left, right| left.1.total_cmp(&right.1))
                .unwrap_or((0, 0.0));
            let floor = ((work.iter().map(Complex64::norm_sqr).sum::<f64>() - power)
                / (iq.len() - 1) as f64)
                .max(f64::MIN_POSITIVE);
            if power / floor > best.power / best.noise_floor {
                best = SearchPeak {
                    power,
                    noise_floor: floor,
                    frequency_hz,
                    delay,
                };
            }
        }

        let quality = best.power / best.noise_floor;
        if quality < self.config.detection_threshold {
            self.previous = None;
            self.drift_hz_per_s = 0.0;
            return TrackOutcome::NoDetection(NoDetection {
                best_quality: quality,
                threshold: self.config.detection_threshold,
                sample_time_ns,
            });
        }

        let refined = self.refine_frequency(iq, best.delay, best.frequency_hz);
        if let Some((previous_hz, previous_ns)) = self.previous {
            let elapsed = (sample_time_ns.saturating_sub(previous_ns) as f64) * 1e-9;
            if elapsed > 0.0 {
                self.drift_hz_per_s = (refined - previous_hz) / elapsed;
            }
        }
        self.previous = Some((refined, sample_time_ns));
        TrackOutcome::Detection(Detection {
            correlation_peak_hz: refined,
            quality,
            sample_time_ns,
            delay_samples: best.delay,
        })
    }

    fn search_bounds(&self, sample_time_ns: u64) -> (f64, f64) {
        self.previous.map_or(
            (self.config.min_frequency_hz, self.config.max_frequency_hz),
            |(previous_hz, previous_ns)| {
                let elapsed = (sample_time_ns.saturating_sub(previous_ns) as f64) * 1e-9;
                let predicted = previous_hz + self.drift_hz_per_s * elapsed;
                (
                    (predicted - self.config.tracking_half_width_hz)
                        .max(self.config.min_frequency_hz),
                    (predicted + self.config.tracking_half_width_hz)
                        .min(self.config.max_frequency_hz),
                )
            },
        )
    }

    // The coarse bin selects/unwinds aliases. The circularly aligned, despread adjacent-
    // sample phase gives a sub-bin mean frequency at the block midpoint. For a linear ramp,
    // averaging adjacent phases analytically equals the midpoint instantaneous frequency.
    fn refine_frequency(&self, iq: &[Complex64], delay: usize, coarse_hz: f64) -> f64 {
        let mut phase_vector = Complex64::new(0.0, 0.0);
        let mut previous: Option<Complex64> = None;
        for index in 0..iq.len() {
            let reference_index = (index + iq.len() - delay) % iq.len();
            let phase = -TAU * coarse_hz * index as f64 / self.config.sample_rate_hz;
            let despread = iq[index]
                * Complex64::from_polar(1.0, phase)
                * self.reference[reference_index].conj();
            if let Some(last) = previous {
                phase_vector += despread * last.conj();
            }
            previous = Some(despread);
        }
        coarse_hz + phase_vector.arg() * self.config.sample_rate_hz / TAU
    }
}

#[derive(Clone, Copy)]
struct SearchPeak {
    power: f64,
    noise_floor: f64,
    frequency_hz: f64,
    delay: usize,
}

fn frequency_bins(minimum: f64, maximum: f64, width: f64) -> Vec<f64> {
    let mut bins = Vec::new();
    let mut frequency = minimum;
    while frequency <= maximum {
        bins.push(frequency);
        frequency += width;
    }
    if bins.last().is_none_or(|last| maximum - last > width * 1e-9) {
        bins.push(maximum);
    }
    bins
}
