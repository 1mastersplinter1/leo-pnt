//! Deterministic validation campaigns for the PNT research stack:
//! synthetic-IQ tracker stress studies and estimator consistency studies.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::naive_bytecount
)]

pub mod consistency;
pub mod correction;
pub mod endurance;
pub mod estimator;
pub mod highspeed;
pub mod maneuver;
pub mod multisat;

use std::f64::consts::TAU;
use std::fs;
use std::path::Path;
use std::time::Instant;

use num_complex::Complex64;
use pnt_tracker::synth::{BpskReference, SynthConfig, Synthesizer};
use pnt_tracker::{CorrelationTracker, TrackOutcome, TrackerConfig};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub const SAMPLE_RATE_HZ: f64 = 8_192.0;
pub const LENGTH: usize = 256;
pub const BLOCK_SECONDS: f64 = LENGTH as f64 / SAMPLE_RATE_HZ;
const BLOCK_NS: u64 = 31_250_000;
const OFFSET_HZ: f64 = 487.5;
const REFERENCE_SEED: u64 = 0x1234_5678;
const INTERFERER_REFERENCE_SEED: u64 = REFERENCE_SEED ^ 0x5555;

#[derive(Clone, Copy, Debug)]
pub struct StudySize {
    pub seeds_per_cn0: u64,
    pub noise_blocks: u64,
    pub impairment_trials: u64,
}

impl StudySize {
    #[must_use]
    pub const fn quick() -> Self {
        Self {
            seeds_per_cn0: 8,
            noise_blocks: 64,
            impairment_trials: 8,
        }
    }

    #[must_use]
    pub const fn full() -> Self {
        Self {
            seeds_per_cn0: 500,
            noise_blocks: 1_000_000,
            impairment_trials: 200,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub schema_version: u32,
    pub deterministic_seed_scheme: String,
    pub fixture: Fixture,
    pub requested: Counts,
    pub wall_time_seconds: WallTimes,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Fixture {
    pub sample_rate_hz: f64,
    pub reference_length: usize,
    pub search_min_hz: f64,
    pub search_max_hz: f64,
    pub frequency_bin_hz: f64,
    pub detection_threshold: f64,
    pub tracking_half_width_hz: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Counts {
    pub seeds_per_cn0: u64,
    pub noise_blocks: u64,
    pub impairment_trials: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct WallTimes {
    pub detection_accuracy: f64,
    pub false_alarm_tail: f64,
    pub dynamics: f64,
    pub impairments: f64,
    pub total: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StudyResults {
    pub manifest: Manifest,
    pub detection_accuracy: DetectionStudy,
    pub false_alarm_tail: FalseAlarmStudy,
    pub dynamics: DynamicsStudy,
    pub impairments: ImpairmentStudy,
    pub quality_variance: QualityVarianceStudy,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DetectionStudy {
    pub levels: Vec<DetectionLevel>,
    pub knee_cn0_db_hz_at_p50: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DetectionLevel {
    pub cn0_db_hz: f64,
    pub trials: u64,
    pub detections: u64,
    pub detection_probability: f64,
    pub detection_probability_ci95_low: f64,
    pub detection_probability_ci95_high: f64,
    pub error_mean_hz: Option<f64>,
    pub error_sigma_hz: Option<f64>,
    pub error_max_abs_hz: Option<f64>,
    pub quality_p10: f64,
    pub quality_median: f64,
    pub quality_p90: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FalseAlarmStudy {
    pub blocks: u64,
    pub threshold_32_exceedances: u64,
    pub observed_quantiles: Vec<Quantile>,
    pub exceedance: Vec<Exceedance>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Quantile {
    pub probability: f64,
    pub quality: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Exceedance {
    pub threshold: f64,
    pub observed_count: u64,
    pub observed_probability: f64,
    pub fisher_union_prediction: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DynamicsStudy {
    pub orbital_extremes: Vec<OrbitalExtreme>,
    pub ramp_sweeps: Vec<RampResult>,
    pub escape: Vec<EscapeResult>,
    pub block_length_sensitivity: Vec<BlockSensitivity>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OrbitalExtreme {
    pub band: String,
    pub carrier_hz: f64,
    pub altitude_km: f64,
    pub orbital_speed_m_s: f64,
    pub max_doppler_hz: f64,
    pub overhead_max_drift_hz_s: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RampResult {
    pub ramp_hz_s: f64,
    pub detected_blocks: u64,
    pub total_blocks: u64,
    pub max_abs_error_hz: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EscapeResult {
    pub initial_offset_hz: f64,
    pub ramp_hz_s: f64,
    pub first_outside_block: Option<u64>,
    pub first_no_detection_block: Option<u64>,
    pub wrong_lock_blocks: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockSensitivity {
    pub reference_length: usize,
    pub block_seconds: f64,
    pub largest_all_detected_ramp_hz_s: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ImpairmentStudy {
    pub cw: Vec<CwResult>,
    pub clock: Vec<ClockResult>,
    pub reacquisition: Vec<ReacquisitionResult>,
    pub two_signal: Vec<TwoSignalResult>,
    pub multipath: Vec<MultipathResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CwResult {
    pub interferer_offset_hz: f64,
    pub js_db: f64,
    pub trials: u64,
    pub detections: u64,
    pub false_locks: u64,
    pub mean_error_hz: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClockResult {
    pub carrier_hz: f64,
    pub fractional_error: f64,
    pub expected_bias_hz: f64,
    pub measured_bias_hz: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReacquisitionResult {
    pub outage_blocks: u64,
    pub latency_blocks: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TwoSignalResult {
    pub second_offset_hz: f64,
    pub second_to_primary_db: f64,
    pub trials: u64,
    pub primary_captures: u64,
    pub secondary_captures: u64,
    pub other_locks: u64,
    pub no_detections: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MultipathResult {
    pub echo_delay_samples: usize,
    pub echo_to_direct_db: f64,
    pub trials: u64,
    pub detections: u64,
    pub direct_delay_selections: u64,
    pub echo_delay_selections: u64,
    pub other_delay_selections: u64,
    pub no_detections: u64,
    pub mean_frequency_error_hz: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct QualityVarianceStudy {
    pub bins: Vec<QualityVarianceBin>,
    pub log_log_fit_intercept: Option<f64>,
    pub log_log_fit_slope: Option<f64>,
    pub residual_rms_log_variance: Option<f64>,
    pub saturation_quality_above: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct QualityVarianceBin {
    pub quality_min: f64,
    pub quality_max: f64,
    pub samples: u64,
    pub mean_quality: f64,
    pub error_variance_hz2: f64,
    pub fitted_variance_hz2: Option<f64>,
    pub residual_log_variance: Option<f64>,
}

#[derive(Clone, Copy)]
struct Trial {
    detected: bool,
    quality: f64,
    error: Option<f64>,
}

fn reference(length: usize) -> BpskReference {
    BpskReference::pn(length, REFERENCE_SEED)
}

fn tracker(length: usize, threshold: f64) -> CorrelationTracker {
    TrackerConfig {
        sample_rate_hz: SAMPLE_RATE_HZ,
        min_frequency_hz: -4_080.0,
        max_frequency_hz: 4_080.0,
        frequency_bin_hz: SAMPLE_RATE_HZ / length as f64,
        detection_threshold: threshold,
        tracking_half_width_hz: 128.0,
    }
    .build(reference(length).samples)
    .expect("study fixture is valid")
}

fn synth(length: usize, offset: f64, ramp: f64, cn0: f64, seed: u64) -> Synthesizer {
    synth_with_reference(length, offset, ramp, cn0, seed, REFERENCE_SEED, 37 % length)
}

fn synth_with_reference(
    length: usize,
    offset: f64,
    ramp: f64,
    cn0: f64,
    noise_seed: u64,
    reference_seed: u64,
    delay_samples: usize,
) -> Synthesizer {
    Synthesizer::new(
        SynthConfig {
            sample_rate_hz: SAMPLE_RATE_HZ,
            initial_offset_hz: offset,
            offset_ramp_hz_per_s: ramp,
            delay_samples: delay_samples % length,
            cn0_db_hz: cn0,
            seed: noise_seed,
        },
        BpskReference::pn(length, reference_seed),
    )
}

fn signal_trial(cn0: f64, seed: u64) -> Trial {
    let mut generator = synth(LENGTH, OFFSET_HZ, 0.0, cn0, seed);
    match tracker(LENGTH, 32.0).process_block(&generator.next_block(), 0) {
        TrackOutcome::Detection(value) => Trial {
            detected: true,
            quality: value.quality,
            error: Some(value.correlation_peak_hz - OFFSET_HZ),
        },
        TrackOutcome::NoDetection(value) => Trial {
            detected: false,
            quality: value.best_quality,
            error: None,
        },
    }
}

fn detection_levels() -> Vec<f64> {
    let mut levels: Vec<f64> = (30..=40).map(f64::from).collect();
    levels.extend((42..=64).step_by(2).map(f64::from));
    levels.extend([68.0, 70.0, 72.0, 76.0, 78.0, 80.0]);
    levels
}

fn detection_study(count: u64) -> (DetectionStudy, Vec<(f64, f64)>) {
    let raw: Vec<(DetectionLevel, Vec<(f64, f64)>)> = detection_levels()
        .into_iter()
        .map(|cn0| {
            let trials: Vec<Trial> = (1..=count)
                .into_par_iter()
                .map(|seed| signal_trial(cn0, seed ^ cn0.to_bits()))
                .collect();
            let errors: Vec<f64> = trials.iter().filter_map(|trial| trial.error).collect();
            let mut qualities: Vec<f64> = trials.iter().map(|trial| trial.quality).collect();
            qualities.sort_by(f64::total_cmp);
            let detections = trials.iter().filter(|trial| trial.detected).count() as u64;
            let (ci95_low, ci95_high) = wilson_interval_95(detections, count);
            let pairs = trials
                .iter()
                .filter_map(|trial| trial.error.map(|error| (trial.quality, error)))
                .collect();
            (
                DetectionLevel {
                    cn0_db_hz: cn0,
                    trials: count,
                    detections,
                    detection_probability: detections as f64 / count as f64,
                    detection_probability_ci95_low: ci95_low,
                    detection_probability_ci95_high: ci95_high,
                    error_mean_hz: mean(&errors),
                    error_sigma_hz: sample_sigma(&errors),
                    error_max_abs_hz: errors.iter().map(|value| value.abs()).reduce(f64::max),
                    quality_p10: quantile(&qualities, 0.1),
                    quality_median: quantile(&qualities, 0.5),
                    quality_p90: quantile(&qualities, 0.9),
                },
                pairs,
            )
        })
        .collect();
    let knee = raw
        .iter()
        .filter(|(level, _)| level.detection_probability >= 0.5)
        .map(|(level, _)| level.cn0_db_hz)
        .next();
    let pairs = raw.iter().flat_map(|(_, pairs)| pairs.clone()).collect();
    (
        DetectionStudy {
            levels: raw.into_iter().map(|(level, _)| level).collect(),
            knee_cn0_db_hz_at_p50: knee,
        },
        pairs,
    )
}

fn noise_quality(seed: u64) -> f64 {
    let mut generator = synth(LENGTH, 0.0, 0.0, 60.0, seed);
    match tracker(LENGTH, f64::MAX).process_block(&generator.next_noise_block(), 0) {
        TrackOutcome::NoDetection(value) => value.best_quality,
        TrackOutcome::Detection(_) => unreachable!("infinite threshold"),
    }
}

fn false_alarm_study(blocks: u64) -> FalseAlarmStudy {
    let mut qualities: Vec<f64> = (1..=blocks).into_par_iter().map(noise_quality).collect();
    qualities.sort_by(f64::total_cmp);
    let exceedance = [10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0]
        .into_iter()
        .map(|threshold| {
            let observed_count = qualities
                .iter()
                .filter(|quality| **quality >= threshold)
                .count() as u64;
            Exceedance {
                threshold,
                observed_count,
                observed_probability: observed_count as f64 / blocks as f64,
                fisher_union_prediction: fisher_block_exceedance(threshold, LENGTH, LENGTH),
            }
        })
        .collect();
    FalseAlarmStudy {
        blocks,
        threshold_32_exceedances: qualities.iter().filter(|quality| **quality >= 32.0).count()
            as u64,
        observed_quantiles: [0.5, 0.9, 0.99, 0.999, 1.0]
            .into_iter()
            .map(|probability| Quantile {
                probability,
                quality: quantile(&qualities, probability),
            })
            .collect(),
        exceedance,
    }
}

/// Fisher-g row exceedance with a conservative union over frequency rows.
#[must_use]
pub fn fisher_block_exceedance(quality: f64, delay_bins: usize, frequency_rows: usize) -> f64 {
    let n = delay_bins as f64;
    let g = quality / (quality + n - 1.0);
    let terms = (1.0 / g).floor().min(n) as usize;
    let mut row = 0.0;
    let mut combination = 1.0;
    for j in 1..=terms {
        combination *= (n - (j - 1) as f64) / j as f64;
        let term = combination * (1.0 - j as f64 * g).max(0.0).powf(n - 1.0);
        row += if j % 2 == 1 { term } else { -term };
    }
    (row * frequency_rows as f64).clamp(0.0, 1.0)
}

fn orbital_extremes() -> Vec<OrbitalExtreme> {
    const MU: f64 = 3.986_004_418e14;
    const EARTH_RADIUS: f64 = 6_378_137.0;
    const C: f64 = 299_792_458.0;
    let bands = [
        ("Ku-low", 11.325e9),
        ("Ku-high", 11.575e9),
        ("L-band", 1.616e9),
        ("VHF", 137e6),
    ];
    bands
        .into_iter()
        .flat_map(|(band, carrier)| {
            [550.0, 1_200.0].into_iter().map(move |altitude_km| {
                let altitude = altitude_km * 1_000.0;
                let radius = EARTH_RADIUS + altitude;
                let speed = (MU / radius).sqrt();
                // At the geometric horizon LOS tends to the satellite velocity component
                // v*R/r. At overhead the range curvature is mu*R/(r^2*h).
                let max_range_rate = speed * EARTH_RADIUS / radius;
                let overhead_range_acceleration = MU * EARTH_RADIUS / (radius * radius * altitude);
                OrbitalExtreme {
                    band: band.to_owned(),
                    carrier_hz: carrier,
                    altitude_km,
                    orbital_speed_m_s: speed,
                    max_doppler_hz: carrier * max_range_rate / C,
                    overhead_max_drift_hz_s: carrier * overhead_range_acceleration / C,
                }
            })
        })
        .collect()
}

fn ramp_run(length: usize, ramp: f64) -> RampResult {
    let mut generator = synth(length, -900.0, ramp, 70.0, 0x44);
    let mut receiver = tracker(length, 32.0);
    let mut detected = 0;
    let mut max_error: Option<f64> = None;
    for block in 0..16_u64 {
        let timestamp = (block as f64 * length as f64 / SAMPLE_RATE_HZ * 1e9) as u64;
        if let TrackOutcome::Detection(value) =
            receiver.process_block(&generator.next_block(), timestamp)
        {
            detected += 1;
            let midpoint = (block * length as u64) as f64 / SAMPLE_RATE_HZ
                + (length - 1) as f64 / (2.0 * SAMPLE_RATE_HZ);
            let error = (value.correlation_peak_hz - (-900.0 + ramp * midpoint)).abs();
            max_error = Some(max_error.map_or(error, |prior| prior.max(error)));
        }
    }
    RampResult {
        ramp_hz_s: ramp,
        detected_blocks: detected,
        total_blocks: 16,
        max_abs_error_hz: max_error,
    }
}

fn dynamics_study() -> DynamicsStudy {
    let ramps = [
        0.0, 75.0, 250.0, 500.0, 1_000.0, 2_000.0, 4_000.0, 8_000.0, 16_000.0, 32_000.0,
    ];
    let ramp_sweeps: Vec<_> = ramps
        .into_iter()
        .map(|ramp| ramp_run(LENGTH, ramp))
        .collect();
    let escape = [2_000.0, 8_000.0].into_iter().map(escape_run).collect();
    let block_length_sensitivity = [64, 128, 256, 512]
        .into_iter()
        .map(|length| {
            let largest = ramps
                .iter()
                .copied()
                .map(|ramp| ramp_run(length, ramp))
                .take_while(|result| result.detected_blocks == result.total_blocks)
                .last()
                .map(|result| result.ramp_hz_s);
            BlockSensitivity {
                reference_length: length,
                block_seconds: length as f64 / SAMPLE_RATE_HZ,
                largest_all_detected_ramp_hz_s: largest,
            }
        })
        .collect();
    DynamicsStudy {
        orbital_extremes: orbital_extremes(),
        ramp_sweeps,
        escape,
        block_length_sensitivity,
    }
}

fn escape_run(ramp: f64) -> EscapeResult {
    let initial = 3_500.0;
    let mut generator = synth(LENGTH, initial, ramp, 75.0, 0xeeee);
    let mut receiver = tracker(LENGTH, 32.0);
    let mut first_outside = None;
    let mut first_no_detection = None;
    let mut wrong = 0;
    for block in 0..20_u64 {
        let midpoint = block as f64 * BLOCK_SECONDS + (LENGTH - 1) as f64 / (2.0 * SAMPLE_RATE_HZ);
        let expected = initial + ramp * midpoint;
        if expected > 4_080.0 && first_outside.is_none() {
            first_outside = Some(block);
        }
        match receiver.process_block(&generator.next_block(), block * BLOCK_NS) {
            TrackOutcome::Detection(value) => {
                if (value.correlation_peak_hz - expected).abs() > 64.0 {
                    wrong += 1;
                }
            }
            TrackOutcome::NoDetection(_) => {
                first_no_detection.get_or_insert(block);
            }
        }
    }
    EscapeResult {
        initial_offset_hz: initial,
        ramp_hz_s: ramp,
        first_outside_block: first_outside,
        first_no_detection_block: first_no_detection,
        wrong_lock_blocks: wrong,
    }
}

fn add_cw(block: &mut [Complex64], offset: f64, amplitude: f64) {
    for (index, value) in block.iter_mut().enumerate() {
        let phase = TAU * offset * index as f64 / SAMPLE_RATE_HZ;
        *value += Complex64::from_polar(amplitude, phase);
    }
}

fn cw_study(trials: u64) -> Vec<CwResult> {
    [-3_000.0, 0.0, 480.0, 2_000.0]
        .into_iter()
        .flat_map(|interferer| {
            [-20.0, -10.0, 0.0, 10.0, 20.0]
                .into_iter()
                .map(move |js_db| {
                    let outcomes: Vec<Option<f64>> = (1..=trials)
                        .into_par_iter()
                        .map(|seed| {
                            let mut generator = synth(LENGTH, OFFSET_HZ, 0.0, 62.0, seed);
                            let mut block = generator.next_block();
                            add_cw(&mut block, interferer, 10.0_f64.powf(js_db / 20.0));
                            match tracker(LENGTH, 32.0).process_block(&block, 0) {
                                TrackOutcome::Detection(value) => {
                                    Some(value.correlation_peak_hz - OFFSET_HZ)
                                }
                                TrackOutcome::NoDetection(_) => None,
                            }
                        })
                        .collect();
                    let errors: Vec<f64> = outcomes.iter().flatten().copied().collect();
                    CwResult {
                        interferer_offset_hz: interferer,
                        js_db,
                        trials,
                        detections: errors.len() as u64,
                        false_locks: errors.iter().filter(|error| error.abs() > 64.0).count()
                            as u64,
                        mean_error_hz: mean(&errors),
                    }
                })
        })
        .collect()
}

fn clock_study() -> Vec<ClockResult> {
    [137e6, 1.616e9, 11.575e9]
        .into_iter()
        .flat_map(|carrier| {
            [1e-9, 1e-8, 1e-7].into_iter().map(move |fractional| {
                let expected = carrier * fractional;
                let mut generator = synth(LENGTH, OFFSET_HZ + expected, 0.0, 78.0, 0xc10c);
                let measured = match tracker(LENGTH, 2.0).process_block(&generator.next_block(), 0)
                {
                    TrackOutcome::Detection(value) => value.correlation_peak_hz - OFFSET_HZ,
                    TrackOutcome::NoDetection(_) => f64::NAN,
                };
                ClockResult {
                    carrier_hz: carrier,
                    fractional_error: fractional,
                    expected_bias_hz: expected,
                    measured_bias_hz: measured,
                }
            })
        })
        .collect()
}

fn reacquisition_study() -> Vec<ReacquisitionResult> {
    [1, 2, 4, 8, 16]
        .into_iter()
        .map(|outage| {
            let mut generator = synth(LENGTH, 600.0, 75.0, 70.0, 0xac01);
            let mut receiver = tracker(LENGTH, 32.0);
            let _ = receiver.process_block(&generator.next_block(), 0);
            for block in 1..=outage {
                let _ = receiver.process_block(&generator.next_noise_block(), block * BLOCK_NS);
            }
            let latency = (1..=4_u64).find(|after| {
                matches!(
                    receiver.process_block(&generator.next_block(), (outage + after) * BLOCK_NS),
                    TrackOutcome::Detection(_)
                )
            });
            ReacquisitionResult {
                outage_blocks: outage,
                latency_blocks: latency,
            }
        })
        .collect()
}

fn two_signal_study(trials: u64) -> Vec<TwoSignalResult> {
    [-1_000.0, 0.0, 1_500.0]
        .into_iter()
        .flat_map(|offset| {
            [-10.0, 0.0, 10.0].into_iter().map(move |relative_db| {
                let captures: Vec<u8> = (1..=trials)
                    .into_par_iter()
                    .map(|seed| {
                        let mut primary = synth(LENGTH, OFFSET_HZ, 0.0, 70.0, seed);
                        let mut secondary = synth_with_reference(
                            LENGTH,
                            offset,
                            0.0,
                            70.0,
                            seed ^ 0x5555,
                            INTERFERER_REFERENCE_SEED,
                            37,
                        );
                        let mut block = primary.next_block();
                        let scale = 10.0_f64.powf(relative_db / 20.0);
                        for (left, right) in block.iter_mut().zip(secondary.next_block()) {
                            *left += right * scale;
                        }
                        match tracker(LENGTH, 32.0).process_block(&block, 0) {
                            TrackOutcome::Detection(value)
                                if (value.correlation_peak_hz - OFFSET_HZ).abs() < 32.0 =>
                            {
                                1
                            }
                            TrackOutcome::Detection(value)
                                if (value.correlation_peak_hz - offset).abs() < 32.0 =>
                            {
                                2
                            }
                            TrackOutcome::Detection(_) => 3,
                            TrackOutcome::NoDetection(_) => 0,
                        }
                    })
                    .collect();
                TwoSignalResult {
                    second_offset_hz: offset,
                    second_to_primary_db: relative_db,
                    trials,
                    primary_captures: captures.iter().filter(|value| **value == 1).count() as u64,
                    secondary_captures: captures.iter().filter(|value| **value == 2).count() as u64,
                    other_locks: captures.iter().filter(|value| **value == 3).count() as u64,
                    no_detections: captures.iter().filter(|value| **value == 0).count() as u64,
                }
            })
        })
        .collect()
}

fn multipath_study(trials: u64) -> Vec<MultipathResult> {
    [45, 69, 101]
        .into_iter()
        .flat_map(|echo_delay| {
            [-10.0, 0.0, 10.0].into_iter().map(move |relative_db| {
                let outcomes: Vec<Option<(usize, f64)>> = (1..=trials)
                    .into_par_iter()
                    .map(|seed| {
                        let mut direct = synth(LENGTH, OFFSET_HZ, 0.0, 70.0, seed);
                        let mut echo = synth_with_reference(
                            LENGTH,
                            OFFSET_HZ,
                            0.0,
                            70.0,
                            seed ^ 0xa5a5,
                            REFERENCE_SEED,
                            echo_delay,
                        );
                        let mut block = direct.next_block();
                        let scale = 10.0_f64.powf(relative_db / 20.0);
                        for (left, right) in block.iter_mut().zip(echo.next_block()) {
                            *left += right * scale;
                        }
                        match tracker(LENGTH, 32.0).process_block(&block, 0) {
                            TrackOutcome::Detection(value) => {
                                Some((value.delay_samples, value.correlation_peak_hz - OFFSET_HZ))
                            }
                            TrackOutcome::NoDetection(_) => None,
                        }
                    })
                    .collect();
                let frequency_errors: Vec<f64> =
                    outcomes.iter().flatten().map(|(_, error)| *error).collect();
                MultipathResult {
                    echo_delay_samples: echo_delay,
                    echo_to_direct_db: relative_db,
                    trials,
                    detections: frequency_errors.len() as u64,
                    direct_delay_selections: outcomes
                        .iter()
                        .flatten()
                        .filter(|(delay, _)| *delay == 37)
                        .count() as u64,
                    echo_delay_selections: outcomes
                        .iter()
                        .flatten()
                        .filter(|(delay, _)| *delay == echo_delay)
                        .count() as u64,
                    other_delay_selections: outcomes
                        .iter()
                        .flatten()
                        .filter(|(delay, _)| *delay != 37 && *delay != echo_delay)
                        .count() as u64,
                    no_detections: outcomes.iter().filter(|outcome| outcome.is_none()).count()
                        as u64,
                    mean_frequency_error_hz: mean(&frequency_errors),
                }
            })
        })
        .collect()
}

fn quality_variance(pairs: &[(f64, f64)]) -> QualityVarianceStudy {
    let edges = [
        32.0, 40.0, 55.0, 75.0, 100.0, 130.0, 160.0, 180.0, 195.0, 220.0,
    ];
    let mut bins: Vec<_> = edges
        .windows(2)
        .filter_map(|edge| {
            let values: Vec<_> = pairs
                .iter()
                .filter(|(quality, _)| *quality >= edge[0] && *quality < edge[1])
                .copied()
                .collect();
            if values.len() < 2 {
                return None;
            }
            let errors: Vec<_> = values.iter().map(|(_, error)| *error).collect();
            Some(QualityVarianceBin {
                quality_min: edge[0],
                quality_max: edge[1],
                samples: values.len() as u64,
                mean_quality: values.iter().map(|(quality, _)| quality).sum::<f64>()
                    / values.len() as f64,
                error_variance_hz2: sample_variance(&errors).unwrap_or(0.0),
                fitted_variance_hz2: None,
                residual_log_variance: None,
            })
        })
        .collect();
    let fit_points: Vec<_> = bins
        .iter()
        .filter(|bin| bin.error_variance_hz2 > 0.0 && bin.mean_quality < 180.0)
        .map(|bin| (bin.mean_quality.ln(), bin.error_variance_hz2.ln()))
        .collect();
    let (intercept, slope, residual) = linear_fit(&fit_points);
    if let (Some(intercept), Some(slope)) = (intercept, slope) {
        for bin in &mut bins {
            if bin.mean_quality < 180.0 {
                let fitted = (intercept + slope * bin.mean_quality.ln()).exp();
                bin.fitted_variance_hz2 = Some(fitted);
                bin.residual_log_variance = Some(bin.error_variance_hz2.ln() - fitted.ln());
            }
        }
    }
    QualityVarianceStudy {
        bins,
        log_log_fit_intercept: intercept,
        log_log_fit_slope: slope,
        residual_rms_log_variance: residual,
        saturation_quality_above: Some(180.0),
    }
}

#[must_use]
pub fn run(size: StudySize) -> StudyResults {
    let total_start = Instant::now();
    let start = Instant::now();
    let (detection_accuracy, pairs) = detection_study(size.seeds_per_cn0);
    let detection_seconds = start.elapsed().as_secs_f64();
    let start = Instant::now();
    let false_alarm_tail = false_alarm_study(size.noise_blocks);
    let false_alarm_seconds = start.elapsed().as_secs_f64();
    let start = Instant::now();
    let dynamics = dynamics_study();
    let dynamics_seconds = start.elapsed().as_secs_f64();
    let start = Instant::now();
    let impairments = ImpairmentStudy {
        cw: cw_study(size.impairment_trials),
        clock: clock_study(),
        reacquisition: reacquisition_study(),
        two_signal: two_signal_study(size.impairment_trials),
        multipath: multipath_study(size.impairment_trials),
    };
    let impairments_seconds = start.elapsed().as_secs_f64();
    let quality_variance = quality_variance(&pairs);
    StudyResults {
        manifest: Manifest {
            schema_version: 1,
            deterministic_seed_scheme: "xorshift64*: trial index XOR parameter bits".to_owned(),
            fixture: Fixture {
                sample_rate_hz: SAMPLE_RATE_HZ,
                reference_length: LENGTH,
                search_min_hz: -4_080.0,
                search_max_hz: 4_080.0,
                frequency_bin_hz: 32.0,
                detection_threshold: 32.0,
                tracking_half_width_hz: 128.0,
            },
            requested: Counts {
                seeds_per_cn0: size.seeds_per_cn0,
                noise_blocks: size.noise_blocks,
                impairment_trials: size.impairment_trials,
            },
            wall_time_seconds: WallTimes {
                detection_accuracy: detection_seconds,
                false_alarm_tail: false_alarm_seconds,
                dynamics: dynamics_seconds,
                impairments: impairments_seconds,
                total: total_start.elapsed().as_secs_f64(),
            },
        },
        detection_accuracy,
        false_alarm_tail,
        dynamics,
        impairments,
        quality_variance,
    }
}

/// Writes one stable, independently consumable JSON file per study plus the manifest.
///
/// # Errors
///
/// Returns an I/O error if the output directory or a JSON file cannot be written.
pub fn write_results(results: &StudyResults, directory: &Path) -> std::io::Result<()> {
    fs::create_dir_all(directory)?;
    write_json(directory.join("manifest.json"), &results.manifest)?;
    write_json(
        directory.join("detection-accuracy.json"),
        &results.detection_accuracy,
    )?;
    write_json(
        directory.join("false-alarm-tail.json"),
        &results.false_alarm_tail,
    )?;
    write_json(directory.join("dynamics.json"), &results.dynamics)?;
    write_json(directory.join("impairments.json"), &results.impairments)?;
    write_json(
        directory.join("quality-variance.json"),
        &results.quality_variance,
    )
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> std::io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value).expect("serializable study schema");
    fs::write(path, bytes)
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn wilson_interval_95(successes: u64, trials: u64) -> (f64, f64) {
    const Z: f64 = 1.959_963_984_540_054;
    if trials == 0 {
        return (0.0, 1.0);
    }
    let n = trials as f64;
    let probability = successes as f64 / n;
    let z_squared = Z * Z;
    let denominator = 1.0 + z_squared / n;
    let center = (probability + z_squared / (2.0 * n)) / denominator;
    let half_width =
        Z * ((probability * (1.0 - probability) + z_squared / (4.0 * n)) / n).sqrt() / denominator;
    (center - half_width, center + half_width)
}

fn sample_variance(values: &[f64]) -> Option<f64> {
    let average = mean(values)?;
    (values.len() > 1).then(|| {
        values
            .iter()
            .map(|value| (value - average).powi(2))
            .sum::<f64>()
            / (values.len() - 1) as f64
    })
}

fn sample_sigma(values: &[f64]) -> Option<f64> {
    sample_variance(values).map(f64::sqrt)
}

fn quantile(sorted: &[f64], probability: f64) -> f64 {
    let index = ((sorted.len() - 1) as f64 * probability).round() as usize;
    sorted[index]
}

fn linear_fit(points: &[(f64, f64)]) -> (Option<f64>, Option<f64>, Option<f64>) {
    if points.len() < 2 {
        return (None, None, None);
    }
    let x_mean = points.iter().map(|point| point.0).sum::<f64>() / points.len() as f64;
    let y_mean = points.iter().map(|point| point.1).sum::<f64>() / points.len() as f64;
    let denominator = points
        .iter()
        .map(|point| (point.0 - x_mean).powi(2))
        .sum::<f64>();
    if denominator == 0.0 {
        return (None, None, None);
    }
    let slope = points
        .iter()
        .map(|point| (point.0 - x_mean) * (point.1 - y_mean))
        .sum::<f64>()
        / denominator;
    let intercept = y_mean - slope * x_mean;
    let residual = (points
        .iter()
        .map(|point| (point.1 - (intercept + slope * point.0)).powi(2))
        .sum::<f64>()
        / points.len() as f64)
        .sqrt();
    (Some(intercept), Some(slope), Some(residual))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_study_is_deterministic_except_wall_clock() {
        let mut first = run(StudySize {
            seeds_per_cn0: 2,
            noise_blocks: 2,
            impairment_trials: 2,
        });
        let mut second = run(StudySize {
            seeds_per_cn0: 2,
            noise_blocks: 2,
            impairment_trials: 2,
        });
        first.manifest.wall_time_seconds = WallTimes::default();
        second.manifest.wall_time_seconds = WallTimes::default();
        assert_eq!(first, second);
    }

    #[test]
    fn schema_serializes_all_studies() {
        let result = run(StudySize {
            seeds_per_cn0: 1,
            noise_blocks: 1,
            impairment_trials: 1,
        });
        let value = serde_json::to_value(result).expect("study serializes");
        for key in [
            "manifest",
            "detection_accuracy",
            "false_alarm_tail",
            "dynamics",
            "impairments",
            "quality_variance",
        ] {
            assert!(value.get(key).is_some(), "missing {key}");
        }
    }

    #[test]
    fn fisher_model_matches_threshold_32_proposal() {
        let probability = fisher_block_exceedance(32.0, 256, 256);
        assert!((probability - 5.30e-9).abs() < 0.02e-9, "{probability}");
    }

    #[test]
    fn distinct_code_interferer_does_not_create_centroid_locks() {
        let results = two_signal_study(200);
        assert!(
            results.iter().all(|result| result.other_locks == 0),
            "{results:#?}"
        );
    }
}
pub mod passage;
pub mod realtle;
