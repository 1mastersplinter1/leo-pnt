//! Deterministic synthetic passage-endurance comparison.

use serde::{Deserialize, Serialize};
use std::path::Path;

const HOUR_S: u64 = 3600;
const DURATION_S: u64 = 9 * HOUR_S;
const SPEED_MPS: f64 = 6.0 * 0.514_444;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PassageStudy {
    pub schema_version: u16,
    pub synthetic_only: bool,
    pub duration_h: f64,
    pub speed_kn: f64,
    pub distance_km: f64,
    pub gps_loss_h: f64,
    pub ephemeris_cache_h: f64,
    pub hard_6h: Outcome,
    pub graduated_30h: Outcome,
    pub caveat: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    pub doppler_available_until_h: f64,
    pub final_position_error_m: f64,
    pub position_class: String,
}

#[must_use]
pub fn simulate() -> PassageStudy {
    // [UNVERIFIED] Synthetic error law: Doppler bounds the passage error at 350 m; after
    // Doppler loss, a 0.25 m/s current/DR velocity error integrates linearly.
    let bounded_error_m = 350.0;
    let dr_error_rate_mps = 0.25;
    let hard_error = bounded_error_m + dr_error_rate_mps * 3.0 * HOUR_S as f64;
    PassageStudy {
        schema_version: 1,
        synthetic_only: true,
        duration_h: 9.0,
        speed_kn: 6.0,
        distance_km: SPEED_MPS * DURATION_S as f64 / 1000.0,
        gps_loss_h: 2.0,
        ephemeris_cache_h: 0.0,
        hard_6h: Outcome {
            doppler_available_until_h: 6.0,
            final_position_error_m: hard_error,
            position_class: "dead-reckoning (>1 NM error)".into(),
        },
        graduated_30h: Outcome {
            doppler_available_until_h: 9.0,
            final_position_error_m: bounded_error_m,
            position_class: "passage-held (<1 NM error)".into(),
        },
        caveat: "D43 applies: synthetic epoch aging aliases orbital phase and is a stand-in, not validation of real SupGP error growth.".into(),
    }
}

/// Writes the deterministic JSON and Markdown passage artifacts.
///
/// # Errors
///
/// Returns filesystem or JSON serialization errors.
pub fn write(output: impl AsRef<Path>) -> Result<PassageStudy, Box<dyn std::error::Error>> {
    let output = output.as_ref();
    std::fs::create_dir_all(output)?;
    let study = simulate();
    std::fs::write(
        output.join("results.json"),
        serde_json::to_vec_pretty(&study)?,
    )?;
    std::fs::write(output.join("STUDY.md"), markdown(&study))?;
    Ok(study)
}

fn markdown(study: &PassageStudy) -> String {
    format!(
        "# Passage endurance study\n\n**SYNTHETIC ONLY.** Nine hours at 6 kn covers {:.2} km; GNSS is lost at hour 2 and ephemeris is cached at departure.\n\n## Result\n\n| handling | Doppler through | final position error | position class |\n|---|---:|---:|---|\n| hard 6 h | {:.1} h | {:.0} m | {} |\n| graduated, 30 h ceiling | {:.1} h | {:.0} m | {} |\n\nThe binary gate kills Doppler three hours before arrival and the solution degrades to DR. Graduated weighting retains Doppler through the passage and holds the synthetic position class.\n\n## D43 caveat\n\n{}\n\nThe 350 m aided bound, 0.25 m/s DR error, SGP4 error curve, LOS-rate mapping, and 30 h ceiling are `[UNVERIFIED]` pending real-SupGP aging and at-sea replay.\n",
        study.distance_km,
        study.hard_6h.doppler_available_until_h,
        study.hard_6h.final_position_error_m,
        study.hard_6h.position_class,
        study.graduated_30h.doppler_available_until_h,
        study.graduated_30h.final_position_error_m,
        study.graduated_30h.position_class,
        study.caveat
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passage_is_deterministic_and_meets_distance_contract() {
        assert_eq!(
            serde_json::to_vec(&simulate()).unwrap(),
            serde_json::to_vec(&simulate()).unwrap()
        );
        let study = simulate();
        assert!(study.distance_km >= 100.0);
        assert!(study.hard_6h.final_position_error_m > 1852.0);
        assert!(study.graduated_30h.final_position_error_m < 1852.0);
    }
}
