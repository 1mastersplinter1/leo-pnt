//! Band-aware fusion trust (code-change plan U1).
//!
//! Each LEO Doppler observation carries a [`Band`] (derived from its constellation). Jamming
//! susceptibility differs sharply by band — a contested theatre jams Ku (Starlink/OneWeb)
//! alongside GNSS while VHF (Orbcomm) and L (Iridium) survive — so the executive down-weights
//! a band's Doppler observations when that band's interference estimate rises.
//!
//! Design constraints (from the reviewed plan):
//! - **Inflate measurement variance only.** Never also tighten the chi-square gate here;
//!   inflating `R` already shrinks the normalised innovation, so gating too would
//!   double-penalise and could drop good VHF/L observations while Ku is merely de-weighted.
//! - **Trust is floored above zero** so `scale_variance` never divides by zero / returns
//!   non-finite values.
//! - **Hysteresis** so a noisy interference estimate cannot chatter the trust.
//! - **Unknown ⇒ full trust.** An absent or under-sampled interference estimate maps to
//!   `trust = 1.0`, never to "kill the band".
//! - **Down-weight only.** A hard exclusion of a confirmed-jammed band is a policy that lives
//!   in the integrity supervisor / executive, not in this component.

use pnt_types::Band;

/// Lower bound on trust, so `scale_variance` stays finite and bounded.
const TRUST_FLOOR: f64 = 0.05;
/// Interference→trust shaping constant: `trust = 1 / (1 + K * interference)`.
const SHAPING_K: f64 = 4.0;
/// Fraction of the new target folded in per observation (exponential smoothing / hysteresis).
const SMOOTHING: f64 = 0.25;

/// Adaptive per-band trust in `(TRUST_FLOOR, 1]`, one entry per [`Band`].
#[derive(Clone, Copy, Debug)]
pub struct BandTrust {
    vhf: f64,
    l: f64,
    ku: f64,
}

impl Default for BandTrust {
    fn default() -> Self {
        Self {
            vhf: 1.0,
            l: 1.0,
            ku: 1.0,
        }
    }
}

impl BandTrust {
    /// A fresh instance with full trust on every band.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn slot(&mut self, band: Band) -> &mut f64 {
        match band {
            Band::Vhf => &mut self.vhf,
            Band::L => &mut self.l,
            Band::Ku => &mut self.ku,
        }
    }

    /// Current trust for a band, in `(TRUST_FLOOR, 1]`.
    #[must_use]
    pub fn trust(&self, band: Band) -> f64 {
        match band {
            Band::Vhf => self.vhf,
            Band::L => self.l,
            Band::Ku => self.ku,
        }
    }

    /// Folds a new interference observation into a band's trust with hysteresis.
    ///
    /// `interference` is a non-negative, defined statistic (e.g. per-band residual / noise-floor
    /// inflation versus a clear-sky baseline). Higher means more jamming. A negative or
    /// non-finite input is treated as "unknown" and leaves trust unchanged (it does **not**
    /// reset toward 1.0 — an established low-trust state persists until a clean observation
    /// raises it), so callers must pass `0.0` for "measured, no interference".
    pub fn observe(&mut self, band: Band, interference: f64) {
        if !interference.is_finite() || interference < 0.0 {
            return;
        }
        let target = (1.0 / (1.0 + SHAPING_K * interference)).clamp(TRUST_FLOOR, 1.0);
        let slot = self.slot(band);
        *slot += SMOOTHING * (target - *slot);
        *slot = slot.clamp(TRUST_FLOOR, 1.0);
    }

    /// Scales a base measurement variance by the band's trust: lower trust ⇒ larger effective
    /// variance ⇒ the observation is down-weighted. Always finite and `>= base_variance`.
    #[must_use]
    pub fn scale_variance(&self, band: Band, base_variance: f64) -> f64 {
        base_variance / self.trust(band)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_full_trust_on_every_band() {
        let trust = BandTrust::new();
        for band in [Band::Vhf, Band::L, Band::Ku] {
            assert!((trust.trust(band) - 1.0).abs() < f64::EPSILON);
            assert!((trust.scale_variance(band, 4.0) - 4.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn rising_interference_lowers_trust_monotonically_and_stays_bounded() {
        let mut low = BandTrust::new();
        let mut high = BandTrust::new();
        // Fold each level in repeatedly so the hysteresis settles near its target.
        for _ in 0..64 {
            low.observe(Band::Ku, 0.5);
            high.observe(Band::Ku, 5.0);
        }
        assert!(high.trust(Band::Ku) < low.trust(Band::Ku));
        assert!(low.trust(Band::Ku) < 1.0);
        for value in [low.trust(Band::Ku), high.trust(Band::Ku)] {
            assert!(value > 0.0 && value <= 1.0, "trust {value} out of (0,1]");
        }
    }

    #[test]
    fn down_weighting_one_band_leaves_others_untouched() {
        let mut trust = BandTrust::new();
        for _ in 0..64 {
            trust.observe(Band::Ku, 5.0);
        }
        assert!(trust.trust(Band::Ku) < 1.0);
        assert!((trust.trust(Band::Vhf) - 1.0).abs() < f64::EPSILON);
        assert!((trust.trust(Band::L) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scale_variance_is_finite_and_inflating_even_at_floor() {
        let mut trust = BandTrust::new();
        for _ in 0..1000 {
            trust.observe(Band::Ku, 1.0e9);
        }
        let scaled = trust.scale_variance(Band::Ku, 4.0);
        assert!(scaled.is_finite(), "scaled variance must be finite");
        assert!(scaled >= 4.0, "down-weighting must not shrink variance");
        assert!(
            scaled <= 4.0 / TRUST_FLOOR + 1.0,
            "bounded by the trust floor"
        );
    }

    #[test]
    fn unknown_interference_leaves_trust_unchanged() {
        let mut trust = BandTrust::new();
        trust.observe(Band::Ku, f64::NAN);
        trust.observe(Band::Ku, -1.0);
        assert!((trust.trust(Band::Ku) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hysteresis_prevents_single_sample_chatter() {
        let mut trust = BandTrust::new();
        // A single high-interference sample must not slam trust to the floor.
        trust.observe(Band::Ku, 100.0);
        assert!(
            trust.trust(Band::Ku) > 0.5,
            "one noisy sample over-reacted: {}",
            trust.trust(Band::Ku)
        );
    }

    #[test]
    fn clean_observation_recovers_trust() {
        let mut trust = BandTrust::new();
        for _ in 0..64 {
            trust.observe(Band::Ku, 5.0);
        }
        let jammed = trust.trust(Band::Ku);
        for _ in 0..64 {
            trust.observe(Band::Ku, 0.0);
        }
        assert!(
            trust.trust(Band::Ku) > jammed,
            "trust should recover when clean"
        );
        assert!((trust.trust(Band::Ku) - 1.0).abs() < 1.0e-6);
    }
}
