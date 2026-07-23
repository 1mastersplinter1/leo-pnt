//! Deterministic synthetic-IQ test and capture-simulator infrastructure.

#![allow(clippy::cast_precision_loss)] // Counters/mantissas are intentionally bounded to 53 bits.

use std::f64::consts::TAU;

use num_complex::Complex64;

/// A deterministic configurable PN/BPSK reference burst.
#[derive(Clone, Debug, PartialEq)]
pub struct BpskReference {
    pub samples: Vec<Complex64>,
}

impl BpskReference {
    /// Generates BPSK chips from a seeded xorshift64* stream (no OS randomness).
    #[must_use]
    pub fn pn(length: usize, seed: u64) -> Self {
        let mut rng = DeterministicRng::new(seed);
        let samples = (0..length)
            .map(|_| Complex64::new(if rng.next_u64() & 1 == 0 { -1.0 } else { 1.0 }, 0.0))
            .collect();
        Self { samples }
    }

    #[must_use]
    pub fn from_chips(chips: &[i8]) -> Self {
        Self {
            samples: chips
                .iter()
                .map(|chip| Complex64::new(if *chip < 0 { -1.0 } else { 1.0 }, 0.0))
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SynthConfig {
    pub sample_rate_hz: f64,
    pub initial_offset_hz: f64,
    pub offset_ramp_hz_per_s: f64,
    pub delay_samples: usize,
    /// Carrier-to-noise-density ratio in dB-Hz. Per-sample SNR is C/N0 divided by sample rate.
    pub cn0_db_hz: f64,
    pub seed: u64,
}

/// Stateful generator whose phase, ramp, and deterministic noise continue across blocks.
pub struct Synthesizer {
    config: SynthConfig,
    reference: BpskReference,
    rng: DeterministicRng,
    sample_index: u64,
}

impl Synthesizer {
    #[must_use]
    pub fn new(config: SynthConfig, reference: BpskReference) -> Self {
        let rng = DeterministicRng::new(config.seed);
        Self {
            config,
            reference,
            rng,
            sample_index: 0,
        }
    }

    /// Generates one reference-length block. The reference repeats circularly.
    #[must_use]
    pub fn next_block(&mut self) -> Vec<Complex64> {
        let sample_rate = self.config.sample_rate_hz;
        let linear_cn0 = 10.0_f64.powf(self.config.cn0_db_hz / 10.0);
        let noise_standard_deviation = (sample_rate / (2.0 * linear_cn0)).sqrt();
        let length = self.reference.samples.len();
        (0..length)
            .map(|block_index| {
                let absolute_index = self.sample_index + block_index as u64;
                let time = absolute_index as f64 / sample_rate;
                let phase = TAU
                    * (self.config.initial_offset_hz * time
                        + 0.5 * self.config.offset_ramp_hz_per_s * time * time);
                let reference_index =
                    (block_index + length - self.config.delay_samples % length) % length;
                let signal =
                    self.reference.samples[reference_index] * Complex64::from_polar(1.0, phase);
                let (noise_i, noise_q) = self.rng.normal_pair();
                signal + Complex64::new(noise_i, noise_q) * noise_standard_deviation
            })
            .collect::<Vec<_>>()
            .tap(|_| self.sample_index += length as u64)
    }

    /// Generates an AWGN-only block with the same noise density and deterministic stream.
    /// This is an explicit test/capture-simulator mode; it does not create observations.
    #[must_use]
    pub fn next_noise_block(&mut self) -> Vec<Complex64> {
        let linear_cn0 = 10.0_f64.powf(self.config.cn0_db_hz / 10.0);
        let deviation = (self.config.sample_rate_hz / (2.0 * linear_cn0)).sqrt();
        let length = self.reference.samples.len();
        (0..length)
            .map(|_| {
                let (noise_i, noise_q) = self.rng.normal_pair();
                Complex64::new(noise_i, noise_q) * deviation
            })
            .collect::<Vec<_>>()
            .tap(|_| self.sample_index += length as u64)
    }
}

trait Tap: Sized {
    fn tap(mut self, operation: impl FnOnce(&mut Self)) -> Self {
        operation(&mut self);
        self
    }
}
impl<T> Tap for T {}

#[derive(Clone, Debug)]
struct DeterministicRng(u64);

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0x9e37_79b9_7f4a_7c15
        } else {
            seed
        })
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.0 = value;
        value.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn uniform_open(&mut self) -> f64 {
        let mantissa = (self.next_u64() >> 11).max(1);
        mantissa as f64 / ((1_u64 << 53) as f64)
    }

    fn normal_pair(&mut self) -> (f64, f64) {
        let radius = (-2.0 * self.uniform_open().ln()).sqrt();
        let angle = TAU * self.uniform_open();
        (radius * angle.cos(), radius * angle.sin())
    }
}
