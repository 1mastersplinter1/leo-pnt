//! Empirical group-specific covariance-CORRECTION study for the denied-leg EKF
//! (U-FS1, backing `docs/design/CONSISTENCY_FIX_SPEC.md`).
//!
//! The U-CD1 diagnosis (`consistency.rs`, D68/D72) proved the production
//! `Executive` + `FilterStub` EKF is covariance-INCONSISTENT over long denied
//! LEO-Doppler legs: the position and velocity state groups are OVERCONFIDENT
//! (true error many times larger than the reported sigma) while clock-drift and
//! heading are PESSIMISTIC. D74 RULED OUT per-SV bias retirement as the causal
//! lever (bit-identical NEES with retirement on/off). The remaining fix is a
//! GROUP-SPECIFIC covariance correction / Q-retuning on the position and velocity
//! blocks. This study computes the CONCRETE multiplicative inflation the fix
//! needs, and validates -- honestly -- how much of the defect a single scalar
//! per group actually repairs.
//!
//! Method (no EKF is re-implemented or edited):
//! - Reuse the ONE simulation in `consistency.rs` via
//!   [`consistency::collect_seed_traces`], obtaining the per-seed per-epoch
//!   [`NeesSample`] traces of the real production filter versus generator truth.
//! - Exploit the NEES identity `NEES = e^T P^-1 e`: scaling a group's covariance
//!   sub-block by a scalar `s` divides that group's NEES by `s`. Hence the
//!   inflation that restores calibration IN THE MEAN is exactly the measured
//!   overconfidence factor `s_g = (denied-late mean group NEES) / dof`, and the
//!   equivalent sigma scale is `sqrt(s_g)`.
//! - Validate two ways over the SAME denied-late window `consistency.rs` uses:
//!   (i) the corrected mean NEES (`raw / s_g`) versus the dof and the two-sided
//!   95% band -- in-band by construction of the mean; and (ii) the fraction of
//!   INDIVIDUAL corrected samples inside the per-sample two-sided 95% chi-square
//!   interval for that dof -- which tests whether one scalar fixes the
//!   DISTRIBUTION shape, not merely the mean. A shortfall in (ii) is reported as
//!   an honest limitation implying time-varying Q-retuning is needed beyond a
//!   static inflation.
//!
//! All inputs are synthetic and labelled [UNVERIFIED]. No number is clamped,
//! target-fitted, or formula-generated; the inflation factors are read directly
//! off the real filter's measured NEES.

use std::{collections::BTreeSet, fmt::Write, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::consistency::{
    self, chi_square_quantile, ConsistencyConfig, NeesSample, StudyError, Z_LOWER, Z_UPPER,
};

/// Per-sample two-sided confidence level for the distribution-restoration test.
const PER_SAMPLE_CONFIDENCE: f64 = 0.95;
/// A recommended group's corrected per-sample coverage at or above this fraction
/// is treated as "distribution restored by a scalar"; below it, only the mean is
/// restored (shape defect remains -> Q-retuning indicated).
const DISTRIBUTION_RESTORED_COVERAGE: f64 = 0.90;

/// Configuration: the study runs entirely off the U-CD1 consistency simulation,
/// so it simply carries a [`ConsistencyConfig`]. The `Default` uses the same
/// >= 8-seed, 60-minute denied leg as `consistency.rs`.
#[derive(Clone, Debug, Default)]
pub struct CorrectionConfig {
    pub consistency: ConsistencyConfig,
}

/// One state group's measured overconfidence and the validated correction.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GroupCorrection {
    pub group: String,
    pub dof: usize,
    /// Number of individual denied-late per-seed samples with an invertible
    /// sub-block for this group.
    pub denied_late_samples: usize,
    pub denied_late_mean_nees: Option<f64>,
    /// `s_g = mean NEES / dof`. `> 1` overconfident (inflate covariance by this),
    /// `< 1` pessimistic (inflating would WORSEN it).
    pub overconfidence_factor: Option<f64>,
    /// Equivalent multiplicative SIGMA scale `sqrt(s_g)` the implementer applies
    /// to the group covariance sub-block.
    pub sigma_scale: Option<f64>,
    /// Corrected denied-late mean NEES `= mean / s_g` (equals `dof` by the NEES
    /// scaling identity).
    pub corrected_mean_nees: Option<f64>,
    /// Two-sided 95% band for the MEAN of `denied_late_samples` chi-square(dof)
    /// draws.
    pub mean_band_lower: f64,
    pub mean_band_upper: f64,
    pub corrected_mean_in_band: bool,
    /// Per-sample two-sided 95% chi-square(dof) interval (lower clamped to the
    /// chi-square support at 0).
    pub per_sample_interval_lower: f64,
    pub per_sample_interval_upper: f64,
    /// Fraction of RAW samples already inside the per-sample interval (baseline).
    pub raw_in_interval_fraction: Option<f64>,
    /// Fraction of CORRECTED (`raw / s_g`) samples inside the per-sample interval
    /// -- the distribution-restoration test. Nominal for a true chi-square(dof)
    /// is 0.95.
    pub corrected_in_interval_fraction: Option<f64>,
    pub verdict: String,
}

/// A concrete inflation the estimator-fixer applies to an overconfident group.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RecommendedInflation {
    pub group: String,
    pub dof: usize,
    pub covariance_scale: f64,
    pub sigma_scale: f64,
    pub corrected_in_interval_fraction: Option<f64>,
    pub scalar_restores_distribution: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CorrectionControls {
    pub seed_count: usize,
    pub seed_values: Vec<u64>,
    pub denied_min: u64,
    pub doppler_interval_s: u64,
    pub sample_interval_s: u64,
    pub per_sample_confidence: f64,
    pub denied_late_window: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CorrectionReport {
    pub schema_version: u16,
    pub caveat: String,
    pub controls: CorrectionControls,
    /// Elapsed seconds included in the denied-late window (last third of denied
    /// epochs), matching `consistency.rs`.
    pub denied_late_epochs: Vec<u64>,
    pub groups: Vec<GroupCorrection>,
    /// The overconfident groups (position, velocity) with their applied scale.
    pub recommended_inflation: Vec<RecommendedInflation>,
    /// True only if EVERY recommended group's corrected per-sample coverage meets
    /// [`DISTRIBUTION_RESTORED_COVERAGE`]; false means a scalar restores the MEAN
    /// but not the distribution shape (Q-retuning indicated).
    pub scalar_restores_distribution: bool,
    pub conclusions: Vec<String>,
    pub unverified: Vec<String>,
}

/// Runs the empirical correction-factor study and writes `results.json` +
/// `STUDY.md` under `output`.
///
/// # Errors
///
/// Returns a mission, journal, ephemeris, prediction, I/O, or JSON error from
/// the underlying consistency simulation, or an I/O/JSON error while writing.
///
/// # Panics
///
/// Panics if fewer than eight seeds are configured (mirrors `consistency::run`).
pub fn run(
    output: impl AsRef<Path>,
    config: &CorrectionConfig,
) -> Result<CorrectionReport, StudyError> {
    assert!(
        config.consistency.seeds.len() >= 8,
        "at least eight seeds required"
    );
    let traces = consistency::collect_seed_traces(&config.consistency)?;
    let (denied_late_epochs, late_samples) = denied_late_window(&traces);

    let groups = vec![
        group_correction("position", 3, &late_samples, |s| s.position_nees),
        group_correction("velocity", 3, &late_samples, |s| s.velocity_nees),
        group_correction("heading", 1, &late_samples, |s| s.heading_nees),
        group_correction("clock-drift", 1, &late_samples, |s| s.clock_drift_nees),
        group_correction("aggregate", 8, &late_samples, |s| s.aggregate_nees),
    ];

    let recommended_inflation: Vec<RecommendedInflation> = groups
        .iter()
        .filter(|group| group.group == "position" || group.group == "velocity")
        .filter_map(|group| {
            let scale = group.overconfidence_factor?;
            let sigma_scale = group.sigma_scale?;
            Some(RecommendedInflation {
                group: group.group.clone(),
                dof: group.dof,
                covariance_scale: scale,
                sigma_scale,
                corrected_in_interval_fraction: group.corrected_in_interval_fraction,
                scalar_restores_distribution: group
                    .corrected_in_interval_fraction
                    .is_some_and(|fraction| fraction >= DISTRIBUTION_RESTORED_COVERAGE),
            })
        })
        .collect();
    let scalar_restores_distribution = !recommended_inflation.is_empty()
        && recommended_inflation
            .iter()
            .all(|inflation| inflation.scalar_restores_distribution);

    let conclusions = conclusions(
        &groups,
        &recommended_inflation,
        scalar_restores_distribution,
    );

    let report = CorrectionReport {
        schema_version: 1,
        caveat: "SYNTHETIC EMPIRICAL COVARIANCE-CORRECTION STUDY [UNVERIFIED]. Group inflation factors are read directly off the real production Executive + FilterStub EKF NEES (via the U-CD1 consistency simulation) versus generator truth; no value is clamped, target-fitted, or formula-generated. This computes and validates the group-specific correction the fix needs; it does NOT modify or test the estimator (the fix lands serially in pnt-estimator/fusion-executive).".into(),
        controls: CorrectionControls {
            seed_count: config.consistency.seeds.len(),
            seed_values: config.consistency.seeds.clone(),
            denied_min: config.consistency.denied_min,
            doppler_interval_s: config.consistency.doppler_interval_s,
            sample_interval_s: config.consistency.sample_interval_s,
            per_sample_confidence: PER_SAMPLE_CONFIDENCE,
            denied_late_window: "Last third of denied epochs (identical windowing to consistency.rs), pooled across all seeds at the per-sample level.".into(),
        },
        denied_late_epochs,
        groups,
        recommended_inflation,
        scalar_restores_distribution,
        conclusions,
        unverified: vec![
            "All inputs inherited from the U-CD1 consistency simulation: synthetic 960-satellite three-shell LEO Walker grid, sticky best-eight-visible handover, constant-heading maritime leg, injected clock/SV biases and deterministic measurement noise [UNVERIFIED].".into(),
            "Inflation factors are computed on synthetic truth; they are the correct INITIAL calibration and validation target for the estimator fix, to be re-measured against real SoOP data before flight.".into(),
            "The per-sample chi-square interval uses the Wilson-Hilferty quantile approximation (shared with consistency.rs), lower-clamped to the chi-square support at 0; it is a calibration diagnostic, not an exact test.".into(),
        ],
    };

    fs::create_dir_all(output.as_ref())?;
    let mut json = serde_json::to_vec_pretty(&report)?;
    json.push(b'\n');
    fs::write(output.as_ref().join("results.json"), json)?;
    fs::write(output.as_ref().join("STUDY.md"), markdown(&report))?;
    Ok(report)
}

/// Selects the denied-late window (last third of denied epochs, as in
/// `consistency.rs`) and returns those epoch elapsed-seconds plus every per-seed
/// sample falling in them.
fn denied_late_window(traces: &[Vec<NeesSample>]) -> (Vec<u64>, Vec<NeesSample>) {
    let epochs: Vec<u64> = traces
        .iter()
        .flatten()
        .filter(|sample| sample.denied)
        .map(|sample| sample.elapsed_s)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let third = epochs.len() / 3;
    let late: BTreeSet<u64> = if third > 0 {
        epochs[epochs.len() - third..].iter().copied().collect()
    } else {
        epochs.iter().copied().collect()
    };
    let samples: Vec<NeesSample> = traces
        .iter()
        .flatten()
        .filter(|sample| sample.denied && late.contains(&sample.elapsed_s))
        .cloned()
        .collect();
    (late.into_iter().collect(), samples)
}

fn group_correction(
    group: &str,
    dof: usize,
    late_samples: &[NeesSample],
    select: impl Fn(&NeesSample) -> Option<f64>,
) -> GroupCorrection {
    let raw: Vec<f64> = late_samples.iter().filter_map(&select).collect();
    let count = raw.len();

    // Per-sample two-sided 95% chi-square(dof) interval (support is non-negative,
    // so clamp the Wilson-Hilferty lower tail at 0).
    let per_sample_interval_lower = chi_square_quantile(dof as f64, Z_LOWER).max(0.0);
    let per_sample_interval_upper = chi_square_quantile(dof as f64, Z_UPPER);

    // Two-sided 95% band for the MEAN of `count` chi-square(dof) draws.
    let (mean_band_lower, mean_band_upper) = if count > 0 {
        let n = count as f64;
        let nd = n * dof as f64;
        (
            chi_square_quantile(nd, Z_LOWER) / n,
            chi_square_quantile(nd, Z_UPPER) / n,
        )
    } else {
        (f64::NAN, f64::NAN)
    };

    if raw.is_empty() {
        return GroupCorrection {
            group: group.into(),
            dof,
            denied_late_samples: 0,
            denied_late_mean_nees: None,
            overconfidence_factor: None,
            sigma_scale: None,
            corrected_mean_nees: None,
            mean_band_lower,
            mean_band_upper,
            corrected_mean_in_band: false,
            per_sample_interval_lower,
            per_sample_interval_upper,
            raw_in_interval_fraction: None,
            corrected_in_interval_fraction: None,
            verdict: "No invertible sub-block sampled for this group.".into(),
        };
    }

    let mean_nees = mean(&raw);
    let factor = mean_nees / dof as f64;
    let sigma_scale = factor.sqrt();
    // NEES scaling identity: corrected = raw / s_g. The corrected MEAN is dof by
    // construction; compute it from the data rather than asserting it.
    let corrected_mean_nees = mean_nees / factor;
    let corrected_mean_in_band =
        corrected_mean_nees >= mean_band_lower && corrected_mean_nees <= mean_band_upper;

    let in_interval =
        |value: f64| value >= per_sample_interval_lower && value <= per_sample_interval_upper;
    let raw_in_interval_fraction =
        raw.iter().filter(|&&v| in_interval(v)).count() as f64 / count as f64;
    let corrected_in_interval_fraction =
        raw.iter().filter(|&&v| in_interval(v / factor)).count() as f64 / count as f64;

    let verdict = if factor > mean_band_upper / dof as f64 {
        format!(
            "OVERCONFIDENT: denied-late mean NEES {mean_nees:.1} (dof {dof}) => inflate covariance by {factor:.1}x (sigma x{sigma_scale:.2}). Corrected mean {corrected_mean_nees:.2} lands in [{mean_band_lower:.2}, {mean_band_upper:.2}]. Per-sample coverage rises from {raw_in_interval_fraction:.2} to {corrected_in_interval_fraction:.2} (nominal {PER_SAMPLE_CONFIDENCE:.2})."
        )
    } else if factor < mean_band_lower / dof as f64 {
        format!(
            "PESSIMISTIC: denied-late mean NEES {mean_nees:.1} (dof {dof}), factor {factor:.2}x < 1. Inflating this group would WORSEN consistency; the correction must be group-specific and must NOT touch it."
        )
    } else {
        format!(
            "CONSISTENT: denied-late mean NEES {mean_nees:.1} (dof {dof}) already within [{mean_band_lower:.2}, {mean_band_upper:.2}]; no inflation needed (factor {factor:.2}x)."
        )
    };

    GroupCorrection {
        group: group.into(),
        dof,
        denied_late_samples: count,
        denied_late_mean_nees: Some(mean_nees),
        overconfidence_factor: Some(factor),
        sigma_scale: Some(sigma_scale),
        corrected_mean_nees: Some(corrected_mean_nees),
        mean_band_lower,
        mean_band_upper,
        corrected_mean_in_band,
        per_sample_interval_lower,
        per_sample_interval_upper,
        raw_in_interval_fraction: Some(raw_in_interval_fraction),
        corrected_in_interval_fraction: Some(corrected_in_interval_fraction),
        verdict,
    }
}

fn conclusions(
    groups: &[GroupCorrection],
    recommended: &[RecommendedInflation],
    scalar_restores_distribution: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    out.push(
        "Group-specific denied-late covariance-correction factors (NEES identity NEES = e^T P^-1 e => scaling a group sub-block by s divides its NEES by s):".into(),
    );
    for group in groups {
        out.push(format!("  - {}: {}", group.group, group.verdict));
    }
    if recommended.is_empty() {
        out.push(
            "No overconfident observable group measured at the 95% band in this window.".into(),
        );
    } else {
        let list: Vec<String> = recommended
            .iter()
            .map(|inflation| {
                format!(
                    "{} x{:.1} (sigma x{:.2})",
                    inflation.group, inflation.covariance_scale, inflation.sigma_scale
                )
            })
            .collect();
        out.push(format!(
            "RECOMMENDED group inflation for the estimator fix (position/velocity blocks only): {}. Clock-drift and heading are PESSIMISTIC and must be left untouched -- a global scale is wrong.",
            list.join(", ")
        ));
    }
    if scalar_restores_distribution {
        out.push(
            "A single static scalar per group restores BOTH the mean and the per-sample distribution (corrected coverage >= 0.90) -- a static group inflation is sufficient.".into(),
        );
    } else {
        out.push(
            "HONEST LIMITATION: a single static scalar restores the denied-late MEAN NEES (in-band by construction) but does NOT fully restore the per-sample DISTRIBUTION (corrected coverage stays below 0.95). The defect is therefore not pure scale -- it has a shape/time component -- so a time-varying process-noise (Q) retuning that reshapes the covariance over the leg is needed beyond a static inflation; the static factors here are the correct initial calibration and the Q-retuning validation target.".into(),
        );
    }
    out
}

// ---------------------------------------------------------------------------
// Markdown rendering (mirrors consistency.rs style)
// ---------------------------------------------------------------------------

fn markdown(report: &CorrectionReport) -> String {
    let mut text = format!(
        "# Empirical group covariance-correction factors (U-FS1)\n\n**{}**\n\nBacks `docs/design/CONSISTENCY_FIX_SPEC.md`. Reuses the U-CD1 consistency simulation (`consistency::collect_seed_traces`) so there is ONE EKF-driving implementation. Uses the NEES identity `NEES = e^T P^-1 e`: scaling a group covariance sub-block by scalar `s` divides that group's NEES by `s`, so the inflation restoring calibration in the mean is the measured overconfidence factor `s_g = denied-late mean group NEES / dof`, sigma scale `sqrt(s_g)`.\n\n",
        report.caveat,
    );

    text.push_str("## 1. Per-group correction and validation\n\nDenied-late window (last third of denied epochs, pooled per-sample across seeds). `factor` is the covariance inflation `s_g`; `sigma x` is `sqrt(s_g)`. `corrected mean` is `raw/s_g` (dof by construction); `cover raw`->`cover corr` is the fraction of individual samples inside the per-sample two-sided 95% chi-square interval (nominal 0.95).\n\n| group | dof | samples | mean NEES | factor | sigma x | corrected mean | mean 95% band | cover raw | cover corr |\n|---|---:|---:|---:|---:|---:|---:|---|---:|---:|\n");
    for group in &report.groups {
        let _ = writeln!(
            text,
            "| {} | {} | {} | {} | {} | {} | {} | [{:.2}, {:.2}] | {} | {} |",
            group.group,
            group.dof,
            group.denied_late_samples,
            optional(group.denied_late_mean_nees),
            group
                .overconfidence_factor
                .map_or_else(|| "n/a".into(), |factor| format!("{factor:.1}x")),
            group
                .sigma_scale
                .map_or_else(|| "n/a".into(), |scale| format!("{scale:.2}")),
            optional(group.corrected_mean_nees),
            group.mean_band_lower,
            group.mean_band_upper,
            optional(group.raw_in_interval_fraction),
            optional(group.corrected_in_interval_fraction),
        );
    }

    text.push_str("\n## 2. Recommended inflation for the estimator fix\n\nApply ONLY to the overconfident observable groups; clock-drift and heading are pessimistic and must be left alone.\n\n");
    if report.recommended_inflation.is_empty() {
        text.push_str("- No overconfident observable group measured at the 95% band.\n");
    } else {
        for inflation in &report.recommended_inflation {
            let _ = writeln!(
                text,
                "- **{}** (dof {}): covariance x{:.2}, sigma x{:.2}; corrected per-sample coverage {} -> scalar {} the distribution.",
                inflation.group,
                inflation.dof,
                inflation.covariance_scale,
                inflation.sigma_scale,
                optional(inflation.corrected_in_interval_fraction),
                if inflation.scalar_restores_distribution {
                    "restores"
                } else {
                    "does NOT fully restore"
                },
            );
        }
    }
    let _ = writeln!(
        text,
        "\nScalar-per-group restores the full distribution: **{}**.",
        report.scalar_restores_distribution
    );

    text.push_str("\n## 3. Conclusions\n\n");
    for conclusion in &report.conclusions {
        let _ = writeln!(text, "- {conclusion}");
    }

    let _ = write!(
        text,
        "\n## Controls\n\n- Seeds: {:?}.\n- Denied leg: {} min; Doppler cadence {}s; NEES sampling {}s.\n- Denied-late window: {}\n- Per-sample confidence: {:.2}.\n- Source: real production `Executive` + `FilterStub` EKF NEES via `consistency::collect_seed_traces` (single shared simulation).\n\n## [UNVERIFIED] inputs\n\n",
        report.controls.seed_values,
        report.controls.denied_min,
        report.controls.doppler_interval_s,
        report.controls.sample_interval_s,
        report.controls.denied_late_window,
        report.controls.per_sample_confidence,
    );
    for item in &report.unverified {
        let _ = writeln!(text, "- {item}");
    }
    text.push_str("\n## Denied-late epochs (elapsed s)\n\n");
    let _ = writeln!(text, "{:?}\n", report.denied_late_epochs);
    text
}

fn optional(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".into(), |number| format!("{number:.2}"))
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn short_config() -> CorrectionConfig {
        CorrectionConfig {
            consistency: ConsistencyConfig {
                denied_min: 10,
                doppler_interval_s: 30,
                sample_interval_s: 30,
                seeds: (0..8).map(|index| 0xE11D_2026_u64 + index).collect(),
            },
        }
    }

    #[test]
    fn nees_scaling_identity_holds() {
        // The whole method rests on: scaling a group's covariance by s divides
        // its NEES by s. Corrected = raw / s exactly.
        for &(raw, s) in &[(165.716_f64, 55.238_f64), (9.0, 3.0), (1.0, 0.5)] {
            let corrected = raw / s;
            assert!((corrected * s - raw).abs() < 1.0e-9);
        }
        // s_g chosen as mean/dof makes the corrected mean equal dof.
        let raw = [30.0_f64, 60.0, 90.0]; // mean 60, dof 3 -> s = 20
        let dof = 3.0;
        let s = mean(&raw) / dof;
        let corrected_mean = mean(&raw) / s;
        assert!((corrected_mean - dof).abs() < 1.0e-9);
    }

    #[test]
    fn group_correction_recenters_the_mean_into_band() {
        // Synthetic overconfident group: NEES far above dof. The recommended
        // inflation must land the corrected mean inside the reported mean band.
        let samples: Vec<NeesSample> = (0..24)
            .map(|index| sample_with_position_nees(30.0 + f64::from(index)))
            .collect();
        let correction = group_correction("position", 3, &samples, |s| s.position_nees);
        assert!(correction.overconfidence_factor.unwrap() > 1.0);
        assert!(
            correction.corrected_mean_in_band,
            "corrected mean {:?} must fall in [{}, {}]",
            correction.corrected_mean_nees, correction.mean_band_lower, correction.mean_band_upper
        );
    }

    #[test]
    fn recommended_inflation_makes_denied_late_mean_consistent() {
        let temp = tempfile::TempDir::new().unwrap();
        let report = run(temp.path(), &short_config()).unwrap();
        for group in report
            .groups
            .iter()
            .filter(|g| g.group == "position" || g.group == "velocity")
        {
            assert!(
                group.overconfidence_factor.unwrap() > 1.0,
                "{} must be overconfident",
                group.group
            );
            assert!(
                group.corrected_mean_in_band,
                "{} corrected mean {:?} not in band [{}, {}]",
                group.group,
                group.corrected_mean_nees,
                group.mean_band_lower,
                group.mean_band_upper
            );
        }
        // Clock-drift is pessimistic -> not a recommended inflation target.
        assert!(report
            .recommended_inflation
            .iter()
            .all(|inflation| inflation.group != "clock-drift"));
    }

    #[test]
    fn run_produces_well_formed_report() {
        let temp = tempfile::TempDir::new().unwrap();
        let report = run(temp.path(), &short_config()).unwrap();
        assert_eq!(report.schema_version, 1);
        assert!(report.caveat.contains("[UNVERIFIED]"));
        assert_eq!(report.groups.len(), 5);
        assert!(!report.denied_late_epochs.is_empty());
        assert!(!report.conclusions.is_empty());
        assert!(temp.path().join("results.json").exists());
        assert!(temp.path().join("STUDY.md").exists());
        // Determinism: same inputs, identical report.
        let again = run(temp.path(), &short_config()).unwrap();
        assert_eq!(report, again);
    }

    fn sample_with_position_nees(nees: f64) -> NeesSample {
        NeesSample {
            elapsed_s: 0,
            denied: true,
            handover: false,
            nuisance_count: 0,
            position_nees: Some(nees),
            velocity_nees: None,
            heading_nees: None,
            clock_drift_nees: None,
            aggregate_nees: None,
            horizontal_error_m: 0.0,
            sigma_horizontal_m: 0.0,
            clock_drift_sigma_mps: 0.0,
            clock_bias_variance_m2: 0.0,
            position_clock_drift_correlation: 0.0,
            nuisance_variance_max: 0.0,
            nuisance_variance_mean: 0.0,
        }
    }
}
