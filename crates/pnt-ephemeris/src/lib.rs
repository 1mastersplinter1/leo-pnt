#![doc = "Local-file OMM/TLE storage, age gating, SGP4 propagation, and TEME-to-ECEF conversion."]

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use sgp4::{Constants, Elements};
use std::{collections::HashMap, fs, path::Path};

pub const DEFAULT_MAX_AGE: Duration = Duration::hours(6);
const EARTH_ROTATION_RAD_S: f64 = 7.292_115_0e-5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TemeState {
    pub position_km: [f64; 3],
    pub velocity_kmps: [f64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EcefState {
    pub position_m: [f64; 3],
    pub velocity_mps: [f64; 3],
}

#[derive(Debug, thiserror::Error)]
pub enum EphemerisError {
    #[error("failed to read local ephemeris: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid OMM JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid ephemeris record: {0}")]
    Parse(String),
    #[error("satellite {0} is not in the ephemeris store")]
    Missing(u64),
    #[error("ephemeris for satellite {norad_id} is too old: {age_seconds:.3}s exceeds {limit_seconds:.3}s")]
    TooOld {
        norad_id: u64,
        age_seconds: f64,
        limit_seconds: f64,
    },
    #[error("SGP4 propagation failed: {0}")]
    Propagation(String),
}

struct Entry {
    elements: Elements,
    constants: Constants,
}
pub struct EphemerisStore {
    entries: HashMap<u64, Entry>,
    max_age: Duration,
}

impl EphemerisStore {
    /// Reads and parses one or more named or unnamed TLE records from a local file.
    ///
    /// # Errors
    /// Returns an I/O or typed parse error; this function performs no network access.
    pub fn from_tle_file(path: impl AsRef<Path>) -> Result<Self, EphemerisError> {
        Self::from_tle_str(&fs::read_to_string(path)?)
    }

    /// Parses one or more named or unnamed TLE records.
    ///
    /// # Errors
    /// Returns [`EphemerisError::Parse`] for malformed or incomplete records.
    pub fn from_tle_str(text: &str) -> Result<Self, EphemerisError> {
        let records: Vec<&str> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        let mut elements = Vec::new();
        let mut index = 0;
        while index < records.len() {
            let (name, row_a, row_b, consumed) = if records[index].starts_with("1 ") {
                (None, records[index], records.get(index + 1), 2)
            } else {
                (
                    Some(records[index].trim_start_matches("0 ").to_owned()),
                    *records
                        .get(index + 1)
                        .ok_or_else(|| EphemerisError::Parse("TLE line 1 missing".into()))?,
                    records.get(index + 2),
                    3,
                )
            };
            let row_b = row_b.ok_or_else(|| EphemerisError::Parse("TLE line 2 missing".into()))?;
            elements.push(
                Elements::from_tle(name, row_a.as_bytes(), row_b.as_bytes())
                    .map_err(|e| EphemerisError::Parse(e.to_string()))?,
            );
            index += consumed;
        }
        Self::from_elements(elements)
    }

    /// Reads a `CelesTrak` OMM JSON object or array from a local file.
    ///
    /// # Errors
    /// Returns an I/O, JSON, or orbital-element validation error.
    pub fn from_omm_json_file(path: impl AsRef<Path>) -> Result<Self, EphemerisError> {
        let text = fs::read_to_string(path)?;
        let elements = serde_json::from_str::<Vec<Elements>>(&text)
            .or_else(|_| serde_json::from_str::<Elements>(&text).map(|one| vec![one]))?;
        Self::from_elements(elements)
    }

    fn from_elements(elements: Vec<Elements>) -> Result<Self, EphemerisError> {
        let mut entries = HashMap::new();
        for element in elements {
            // Use sgp4's recommended improved mode (WGS-84 and IAU sidereal time) for
            // production accuracy. AFSPC compatibility is confined to reference tests.
            // [UNVERIFIED] Improved-mode production propagation lacks a project-local,
            // literature-anchored numerical validation fixture.
            let constants = Constants::from_elements(&element)
                .map_err(|e| EphemerisError::Parse(e.to_string()))?;
            entries.insert(
                element.norad_id,
                Entry {
                    elements: element,
                    constants,
                },
            );
        }
        Ok(Self {
            entries,
            max_age: DEFAULT_MAX_AGE,
        })
    }

    #[must_use]
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }
    #[must_use]
    pub fn contains(&self, norad_id: u64) -> bool {
        self.entries.contains_key(&norad_id)
    }
    #[must_use]
    pub fn epoch(&self, norad_id: u64) -> Option<DateTime<Utc>> {
        self.entries
            .get(&norad_id)
            .map(|entry| DateTime::from_naive_utc_and_offset(entry.elements.datetime, Utc))
    }

    /// Propagates the requested satellite to a TEME state after enforcing the age gate.
    ///
    /// # Errors
    /// Returns a missing-satellite, stale-ephemeris, time-conversion, or SGP4 error.
    pub fn propagate_teme(
        &self,
        norad_id: u64,
        query: DateTime<Utc>,
    ) -> Result<TemeState, EphemerisError> {
        let entry = self
            .entries
            .get(&norad_id)
            .ok_or(EphemerisError::Missing(norad_id))?;
        self.check_age(norad_id, entry.elements.datetime, query.naive_utc())?;
        let minutes = entry
            .elements
            .datetime_to_minutes_since_epoch(&query.naive_utc())
            .map_err(|e| EphemerisError::Propagation(e.to_string()))?;
        let prediction = entry
            .constants
            .propagate(minutes)
            .map_err(|e| EphemerisError::Propagation(e.to_string()))?;
        Ok(TemeState {
            position_km: prediction.position,
            velocity_kmps: prediction.velocity,
        })
    }

    /// Propagates the requested satellite and converts its state to ECEF.
    ///
    /// # Errors
    /// Returns the same typed errors as [`Self::propagate_teme`].
    pub fn propagate_ecef(
        &self,
        norad_id: u64,
        query: DateTime<Utc>,
    ) -> Result<EcefState, EphemerisError> {
        let teme = self.propagate_teme(norad_id, query)?;
        Ok(teme_to_ecef_at_gmst(
            teme.position_km,
            teme.velocity_kmps,
            gmst_rad(query),
        ))
    }

    fn check_age(
        &self,
        norad_id: u64,
        epoch: NaiveDateTime,
        query: NaiveDateTime,
    ) -> Result<(), EphemerisError> {
        let age = query.signed_duration_since(epoch).abs();
        if age > self.max_age {
            let age_seconds = age
                .to_std()
                .map_or(f64::INFINITY, |value| value.as_secs_f64());
            let limit_seconds = self
                .max_age
                .to_std()
                .map_or(0.0, |value| value.as_secs_f64());
            return Err(EphemerisError::TooOld {
                norad_id,
                age_seconds,
                limit_seconds,
            });
        }
        Ok(())
    }
}

/// Rotates TEME into an Earth-fixed frame using the IAU-1982 GMST expression and constant
/// Earth angular velocity, following Vallado's TEME-to-PEF method. `[UNVERIFIED]` Polar
/// motion, DUT1, precession/nutation corrections, and length-of-day variation are omitted.
/// With UTC used as UT1, the IERS bound |UT1-UTC| < 0.9 s implies <420 m equatorial rotation
/// error. `[UNVERIFIED]` Omitted polar motion is expected to be metre-scale, but this project
/// has no EOP-aware validation fixture. This is therefore not a precision EOP transform.
#[must_use]
pub fn teme_to_ecef_at_gmst(
    position_km: [f64; 3],
    velocity_kmps: [f64; 3],
    gmst: f64,
) -> EcefState {
    let (sin, cos) = gmst.sin_cos();
    let rotate = |v: [f64; 3]| [cos * v[0] + sin * v[1], -sin * v[0] + cos * v[1], v[2]];
    let r = rotate(position_km).map(|x| x * 1000.0);
    let inertial_v = rotate(velocity_kmps).map(|x| x * 1000.0);
    let omega_cross_r = [
        -EARTH_ROTATION_RAD_S * r[1],
        EARTH_ROTATION_RAD_S * r[0],
        0.0,
    ];
    EcefState {
        position_m: r,
        velocity_mps: [
            inertial_v[0] - omega_cross_r[0],
            inertial_v[1] - omega_cross_r[1],
            inertial_v[2],
        ],
    }
}

/// Computes IAU-1982 Greenwich mean sidereal time, using UTC as an approximation to UT1.
///
/// # Panics
///
/// Panics if `time` lies outside the range whose Unix-day count fits in an `i32`.
#[must_use]
pub fn gmst_rad(time: DateTime<Utc>) -> f64 {
    let whole_days = i32::try_from(time.timestamp().div_euclid(86_400))
        .expect("supported chrono dates fit in i32 Unix days");
    let day_seconds = i32::try_from(time.timestamp().rem_euclid(86_400))
        .expect("seconds within a day fit in i32");
    let unix_days = f64::from(whole_days)
        + (f64::from(day_seconds) + f64::from(time.timestamp_subsec_nanos()) / 1e9) / 86_400.0;
    let jd = 2_440_587.5 + unix_days;
    let t = (jd - 2_451_545.0) / 36_525.0;
    let degrees = 280.460_618_37 + 360.985_647_366_29 * (jd - 2_451_545.0) + 0.000_387_933 * t * t
        - t * t * t / 38_710_000.0;
    degrees.to_radians().rem_euclid(core::f64::consts::TAU)
}
