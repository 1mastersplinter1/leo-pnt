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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    pub gnss_authority: GnssAuthority,
}

impl Config {
    /// Parses the deliberately small v2 configuration surface.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed input, an unknown key, or any authority value other
    /// than `production`, `recorded_only`, or `off`.
    pub fn parse(input: &str) -> Result<Self, ConfigError> {
        let (key, value) = input.split_once('=').ok_or(ConfigError::Malformed)?;
        if key.trim() != "gnss_authority" {
            return Err(ConfigError::UnknownKey(key.trim().to_owned()));
        }
        Ok(Self {
            gnss_authority: value.trim().parse()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    Malformed,
    UnknownKey(String),
    UnknownGnssAuthority(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => formatter.write_str("expected `gnss_authority = <value>`"),
            Self::UnknownKey(key) => write!(formatter, "unknown configuration key `{key}`"),
            Self::UnknownGnssAuthority(value) => {
                write!(formatter, "unknown gnss_authority `{value}`")
            }
        }
    }
}

impl Error for ConfigError {}
