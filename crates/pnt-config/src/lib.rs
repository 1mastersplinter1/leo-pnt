//! Configuration and measurement-authority policy.

use std::{error::Error, fmt, str::FromStr};

/// The only supported authority modes for GNSS ingress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GnssAuthority {
    Production,
    RecordedOnly,
    Off,
}

impl FromStr for GnssAuthority {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "production" => Ok(Self::Production),
            "recorded_only" => Ok(Self::RecordedOnly),
            "off" => Ok(Self::Off),
            unknown => Err(ConfigError::UnknownGnssAuthority(unknown.to_owned())),
        }
    }
}

/// `[UNVERIFIED]` defaults for graduated ephemeris measurement weighting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EphemerisAgingConfig {
    pub fresh_age_s: f64,
    pub ceiling_age_s: f64,
    pub orbit_error_intercept_km: f64,
    pub orbit_error_slope_km_per_h: f64,
    pub los_rate_rad_s: f64,
}

impl Default for EphemerisAgingConfig {
    fn default() -> Self {
        Self {
            fresh_age_s: 21_600.0,
            ceiling_age_s: 108_000.0,
            orbit_error_intercept_km: 0.386_666_666_666_666_7,
            orbit_error_slope_km_per_h: 0.092_222_222_222_222_22,
            los_rate_rad_s: 0.0076,
        }
    }
}

impl EphemerisAgingConfig {
    /// Validates the ordering and physical domain of the aging parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when a value is non-finite, negative, or the fresh boundary is
    /// above the hard ceiling.
    pub fn validate(self) -> Result<(), ConfigError> {
        let valid = self.fresh_age_s.is_finite()
            && self.ceiling_age_s.is_finite()
            && self.orbit_error_intercept_km.is_finite()
            && self.orbit_error_slope_km_per_h.is_finite()
            && self.los_rate_rad_s.is_finite()
            && self.fresh_age_s >= 0.0
            && self.ceiling_age_s >= self.fresh_age_s
            && self.orbit_error_intercept_km >= 0.0
            && self.orbit_error_slope_km_per_h >= 0.0
            && self.los_rate_rad_s >= 0.0;
        if valid {
            Ok(())
        } else {
            Err(ConfigError::InvalidEphemerisAging)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub gnss_authority: GnssAuthority,
    pub oneweb_enabled: bool,
    pub ephemeris_aging: EphemerisAgingConfig,
}

impl Config {
    /// Parses the deliberately small v2 configuration surface.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed input, an unknown key, or any authority value other
    /// than `production`, `recorded_only`, or `off`.
    pub fn parse(input: &str) -> Result<Self, ConfigError> {
        let mut gnss_authority = None;
        let mut oneweb_enabled = false;
        let mut ephemeris_aging = EphemerisAgingConfig::default();
        for line in input.lines().filter(|line| !line.trim().is_empty()) {
            let (key, value) = line.split_once('=').ok_or(ConfigError::Malformed)?;
            match key.trim() {
                "gnss_authority" => gnss_authority = Some(value.trim().parse()?),
                "oneweb_enabled" => {
                    oneweb_enabled = value
                        .trim()
                        .parse()
                        .map_err(|_| ConfigError::InvalidBoolean(value.trim().to_owned()))?;
                }
                "ephemeris_fresh_age_s" => {
                    ephemeris_aging.fresh_age_s = parse_number(value)?;
                }
                "ephemeris_ceiling_age_s" => {
                    ephemeris_aging.ceiling_age_s = parse_number(value)?;
                }
                "ephemeris_orbit_error_intercept_km" => {
                    ephemeris_aging.orbit_error_intercept_km = parse_number(value)?;
                }
                "ephemeris_orbit_error_slope_km_per_h" => {
                    ephemeris_aging.orbit_error_slope_km_per_h = parse_number(value)?;
                }
                "ephemeris_los_rate_rad_s" => {
                    ephemeris_aging.los_rate_rad_s = parse_number(value)?;
                }
                unknown => return Err(ConfigError::UnknownKey(unknown.to_owned())),
            }
        }
        ephemeris_aging.validate()?;
        Ok(Self {
            gnss_authority: gnss_authority.ok_or(ConfigError::Malformed)?,
            oneweb_enabled,
            ephemeris_aging,
        })
    }
}

fn parse_number(value: &str) -> Result<f64, ConfigError> {
    value
        .trim()
        .parse()
        .map_err(|_| ConfigError::InvalidNumber(value.trim().to_owned()))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    Malformed,
    UnknownKey(String),
    UnknownGnssAuthority(String),
    InvalidBoolean(String),
    InvalidNumber(String),
    InvalidEphemerisAging,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => formatter.write_str("expected `gnss_authority = <value>`"),
            Self::UnknownKey(key) => write!(formatter, "unknown configuration key `{key}`"),
            Self::UnknownGnssAuthority(value) => {
                write!(formatter, "unknown gnss_authority `{value}`")
            }
            Self::InvalidBoolean(value) => write!(formatter, "invalid boolean `{value}`"),
            Self::InvalidNumber(value) => write!(formatter, "invalid number `{value}`"),
            Self::InvalidEphemerisAging => formatter.write_str(
                "ephemeris aging values must be finite and non-negative with fresh_age <= ceiling_age",
            ),
        }
    }
}

impl Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ephemeris_aging_and_rejects_fail_open_ordering() {
        let config = Config::parse(
            "gnss_authority = off\n\
             ephemeris_fresh_age_s = 42\n\
             ephemeris_ceiling_age_s = 84\n\
             ephemeris_los_rate_rad_s = 0.5",
        )
        .unwrap();
        assert!((config.ephemeris_aging.fresh_age_s - 42.0).abs() < f64::EPSILON);
        assert!((config.ephemeris_aging.ceiling_age_s - 84.0).abs() < f64::EPSILON);
        assert!((config.ephemeris_aging.los_rate_rad_s - 0.5).abs() < f64::EPSILON);

        assert!(matches!(
            Config::parse(
                "gnss_authority = off\n\
                 ephemeris_fresh_age_s = 85\n\
                 ephemeris_ceiling_age_s = 84"
            ),
            Err(ConfigError::InvalidEphemerisAging)
        ));
    }
}
