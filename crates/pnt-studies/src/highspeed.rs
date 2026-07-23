//! Deterministic D46 high-speed and extended-passage study.
//!
//! The dynamics and error growth are synthetic engineering stand-ins, not performance claims.

use pnt_estimator::ProcessNoise;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const KNOT_MPS: f64 = 0.514_444;
const ROUTE_DISTANCE_M: f64 = 500_000.0;
const GPS_LOSS_S: f64 = 3_600.0;
const GRADUATED_EPHEMERIS_CEILING_S: f64 = 30.0 * 3_600.0;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProcessNoiseScale {
    pub acceleration: f64,
    pub turn_rate: f64,
    pub clock_drift: f64,
    pub nuisance_random_walk: f64,
}

impl ProcessNoiseScale {
    #[must_use]
    pub fn apply(self, base: ProcessNoise) -> ProcessNoise {
        ProcessNoise {
            acceleration_variance: base.acceleration_variance * self.acceleration,
            turn_rate_variance: base.turn_rate_variance * self.turn_rate,
            clock_drift_variance: base.clock_drift_variance * self.clock_drift,
            nuisance_random_walk_variance: base.nuisance_random_walk_variance
                * self.nuisance_random_walk,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SpeedRegime {
    pub name: String,
    pub speed_kn: f64,
    pub process_noise_scale: ProcessNoiseScale,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HighSpeedConfig {
    pub seed: u64,
    pub route_distance_m: f64,
    pub gps_loss_s: f64,
    pub ephemeris_ceiling_s: f64,
    pub displacement: SpeedRegime,
    pub planing: SpeedRegime,
}

impl Default for HighSpeedConfig {
    fn default() -> Self {
        Self {
            seed: 0xD46_2026,
            route_distance_m: ROUTE_DISTANCE_M,
            gps_loss_s: GPS_LOSS_S,
            ephemeris_ceiling_s: GRADUATED_EPHEMERIS_CEILING_S,
            displacement: SpeedRegime {
                name: "displacement".into(),
                speed_kn: 7.0,
                process_noise_scale: ProcessNoiseScale {
                    acceleration: 1.0,
                    turn_rate: 1.0,
                    clock_drift: 1.0,
                    nuisance_random_walk: 1.0,
                },
            },
            planing: SpeedRegime {
                name: "planing".into(),
                speed_kn: 20.0,
                process_noise_scale: ProcessNoiseScale {
                    acceleration: 6.0,
                    turn_rate: 4.0,
                    clock_drift: 1.0,
                    nuisance_random_walk: 2.0,
                },
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PassageResult {
    pub regime: String,
    pub speed_kn: f64,
    pub distance_km: f64,
    pub duration_h: f64,
    pub denied_duration_h: f64,
    pub position_error_rms_m: f64,
    pub position_error_p95_m: f64,
    pub landfall_position_error_m: f64,
    pub velocity_error_rms_mps: f64,
    pub velocity_error_p95_mps: f64,
    pub landfall_ephemeris_age_h: f64,
    pub ephemeris_age_margin_h: f64,
    pub position_error_classes: Vec<ErrorClass>,
    pub manoeuvre_convergence: Vec<Convergence>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorClass {
    pub denied_age_h: f64,
    pub error_m: f64,
    pub class: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Convergence {
    pub turn: u32,
    pub time_s: f64,
    pub distance_m: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub schema_version: u32,
    pub caveat: String,
    pub route: String,
    pub gps_loss_h: f64,
    pub ephemeris_cached_h: f64,
    pub process_noise_base: ProcessNoiseRecord,
    pub process_noise_lineage: String,
    pub wave_model: String,
    pub same_distance: Vec<PassageResult>,
    pub endurance_20kn_24h: PassageResult,
    pub integration_notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProcessNoiseRecord {
    pub acceleration_variance: f64,
    pub turn_rate_variance: f64,
    pub clock_drift_variance: f64,
    pub nuisance_random_walk_variance: f64,
}

impl From<ProcessNoise> for ProcessNoiseRecord {
    fn from(value: ProcessNoise) -> Self {
        Self {
            acceleration_variance: value.acceleration_variance,
            turn_rate_variance: value.turn_rate_variance,
            clock_drift_variance: value.clock_drift_variance,
            nuisance_random_walk_variance: value.nuisance_random_walk_variance,
        }
    }
}

/// Runs the deterministic campaign and writes `results.json`.
///
/// # Errors
///
/// Returns an I/O or JSON error if the committed artifact cannot be written.
pub fn run(output: impl AsRef<Path>, config: &HighSpeedConfig) -> Result<Report, StudyError> {
    let base = ProcessNoise::default();
    let same_distance = vec![
        simulate(
            config.seed,
            config.route_distance_m,
            &config.displacement,
            config,
        ),
        simulate(
            config.seed,
            config.route_distance_m,
            &config.planing,
            config,
        ),
    ];
    let endurance_distance = 24.0 * 3_600.0 * config.planing.speed_kn * KNOT_MPS;
    let endurance = simulate(config.seed, endurance_distance, &config.planing, config);
    let report = Report {
        schema_version: 1,
        caveat: "SYNTHETIC [UNVERIFIED] scenario; numbers are not navigation-performance claims."
            .into(),
        route: "four equal-distance legs with 90 degree turns; same distance, not same time"
            .into(),
        gps_loss_h: config.gps_loss_s / 3_600.0,
        ephemeris_cached_h: 0.0,
        process_noise_base: base.into(),
        process_noise_lineage: "Config-driven regime multipliers are provisional pending U-H1 and the real-IMU study required by D43.".into(),
        wave_model: "Seeded bounded half-sine slam bursts: 0.08 Hz opportunities, 0.7 s duration, 4.0 m/s^2 vertical peak, 0.18 pitch-to-horizontal coupling.".into(),
        same_distance,
        endurance_20kn_24h: endurance,
        integration_notes: vec![
            "U-P1 graduated ephemeris aging is absent in this checkout; this study assumes its ordered 30 h ceiling and reports margin to it.".into(),
            "The current pnt-ephemeris binary default gate remains 6 h; a 500 km passage at 7 kn and the 24 h case would be rejected without U-P1.".into(),
            "At 20 kn, 24 h is 889 km, not approximately 500 km; both the 500 km comparison and 24 h endurance case are retained.".into(),
        ],
    };
    fs::create_dir_all(output.as_ref())?;
    fs::write(
        output.as_ref().join("results.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}

#[derive(Debug, thiserror::Error)]
pub enum StudyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

fn simulate(
    seed: u64,
    distance_m: f64,
    regime: &SpeedRegime,
    config: &HighSpeedConfig,
) -> PassageResult {
    let speed_mps = regime.speed_kn * KNOT_MPS;
    let duration_s = distance_m / speed_mps;
    let denied_s = (duration_s - config.gps_loss_s).max(0.0);
    let q = regime
        .process_noise_scale
        .apply(ProcessNoise::default())
        .acceleration_variance;
    let speed_factor = regime.speed_kn / 7.0;
    let phase = ((seed ^ regime.speed_kn.to_bits()) as f64 * 1.0e-12)
        .sin()
        .abs();
    // Bounded synthetic error envelope: sqrt-time inertial growth plus an aging term.
    let position_at = |age_s: f64| {
        4.0 + 0.75 * q.sqrt() * age_s.sqrt() * speed_factor.sqrt()
            + 0.000_012 * age_s * speed_factor
            + phase
    };
    let velocity_at =
        |age_s: f64| 0.025 + 0.018 * q.sqrt() * (age_s / 3_600.0).sqrt() * speed_factor;
    let samples = 1_440_u32;
    let mut position = Vec::with_capacity(samples as usize + 1);
    let mut velocity = Vec::with_capacity(samples as usize + 1);
    for index in 0..=samples {
        let age = denied_s * f64::from(index) / f64::from(samples);
        position.push(position_at(age));
        velocity.push(velocity_at(age));
    }
    let rms = |values: &[f64]| {
        (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
    };
    let p95 = |values: &[f64]| values[(values.len() * 95 / 100).min(values.len() - 1)];
    let class_at = |error: f64| {
        if error < 25.0 {
            "<25 m"
        } else if error < 100.0 {
            "25-100 m"
        } else if error < 500.0 {
            "100-500 m"
        } else {
            ">=500 m"
        }
        .to_owned()
    };
    let mut classes = Vec::new();
    for hour in 0..=(denied_s / 3_600.0).floor() as u32 {
        let error = position_at(f64::from(hour) * 3_600.0);
        classes.push(ErrorClass {
            denied_age_h: f64::from(hour),
            error_m: error,
            class: class_at(error),
        });
    }
    if classes
        .last()
        .is_none_or(|last| last.denied_age_h < denied_s / 3_600.0)
    {
        let error = position_at(denied_s);
        classes.push(ErrorClass {
            denied_age_h: denied_s / 3_600.0,
            error_m: error,
            class: class_at(error),
        });
    }
    let convergence_time = 75.0 + 28.0 * speed_factor * q.sqrt();
    PassageResult {
        regime: regime.name.clone(),
        speed_kn: regime.speed_kn,
        distance_km: distance_m / 1_000.0,
        duration_h: duration_s / 3_600.0,
        denied_duration_h: denied_s / 3_600.0,
        position_error_rms_m: rms(&position),
        position_error_p95_m: p95(&position),
        landfall_position_error_m: position_at(denied_s),
        velocity_error_rms_mps: rms(&velocity),
        velocity_error_p95_mps: p95(&velocity),
        landfall_ephemeris_age_h: duration_s / 3_600.0,
        ephemeris_age_margin_h: (config.ephemeris_ceiling_s - duration_s) / 3_600.0,
        position_error_classes: classes,
        manoeuvre_convergence: (1..=3)
            .map(|turn| Convergence {
                turn,
                time_s: convergence_time,
                distance_m: convergence_time * speed_mps,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_is_same_distance_and_repeatable() {
        let config = HighSpeedConfig::default();
        let first = simulate(
            config.seed,
            config.route_distance_m,
            &config.planing,
            &config,
        );
        let second = simulate(
            config.seed,
            config.route_distance_m,
            &config.planing,
            &config,
        );
        assert_eq!(first, second);
        assert!((first.distance_km - 500.0).abs() < f64::EPSILON);
        assert!(first.duration_h < 14.0);
    }

    #[test]
    fn endurance_case_is_at_least_twenty_four_hours() {
        let config = HighSpeedConfig::default();
        let distance = 24.0 * 3_600.0 * config.planing.speed_kn * KNOT_MPS;
        let result = simulate(config.seed, distance, &config.planing, &config);
        assert!((result.duration_h - 24.0).abs() < 1.0e-12);
        assert!(result.distance_km > 880.0);
    }

    #[test]
    fn process_noise_is_config_driven() {
        let base = ProcessNoise::default();
        let scaled = HighSpeedConfig::default()
            .planing
            .process_noise_scale
            .apply(base);
        assert!(
            (scaled.acceleration_variance - base.acceleration_variance * 6.0).abs() < f64::EPSILON
        );
    }
}
