//! IQ capture sources and a deterministic, hardware-free raw-IQ replay format.
//!
//! The file representation is little-endian IEEE-754 `f32`, ordered by sample instant,
//! then channel, then I/Q: `s0c0i,s0c0q,s0c1i,s0c1q,...`. Channels at a sample instant
//! are coherent. Values are converted to [`Complex64`] for `pnt-tracker`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)] // Bounds and finiteness are checked at each file/timestamp representation boundary.

use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

pub const FORMAT_VERSION: u16 = 1;
pub const REPRESENTATION: &str = "complex-f32";
pub const ENDIANNESS: &str = "little";

/// Static capture metadata. All frequencies and rates are SI units.
///
/// `first_sample_monotonic_ns` is the time of sample zero in the clock-service monotonic
/// domain. `gaps` describe samples absent from the file; an event at `sample_index` is
/// surfaced on the first following block. `overrun` distinguishes a device/host loss from
/// another known discontinuity. Gain is recorded, not interpreted by capture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaptureMetadata {
    pub format_version: u16,
    pub representation: String,
    pub endianness: String,
    pub sample_rate_hz: f64,
    pub centre_frequency_hz: f64,
    pub bandwidth_hz: f64,
    pub gain_db: Vec<f64>,
    pub channels: Vec<u32>,
    pub external_reference: bool,
    pub first_sample_monotonic_ns: u64,
    pub first_sample_utc_rfc3339: Option<String>,
    pub sample_count_per_channel: u64,
    pub gaps: Vec<Gap>,
    pub calibration_id: String,
    pub configuration_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gap {
    /// Index of the first stored sample after the discontinuity.
    pub sample_index: u64,
    pub missing_samples: u64,
    pub overrun: bool,
}

/// Optional expectations used to reject a capture before samples reach a tracker.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MetadataRequirements {
    pub sample_rate_hz: Option<f64>,
    pub centre_frequency_hz: Option<f64>,
    pub channels: Option<Vec<u32>>,
    pub calibration_id: Option<String>,
    pub configuration_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IqBlock {
    pub first_sample_monotonic_ns: u64,
    pub sample_rate_hz: f64,
    pub centre_frequency_hz: f64,
    pub channels: Vec<u32>,
    /// Channel-major block data. Every channel has the same number of samples.
    pub samples: Vec<Vec<Complex64>>,
    pub discontinuities_before: Vec<Gap>,
}

/// A pull-based source for timestamped coherent complex-baseband blocks.
///
/// Implementations must preserve channel alignment, report discontinuities rather than
/// silently closing them, and return monotonically increasing first-sample timestamps.
/// `Ok(None)` is permanent end-of-source. A tracker consumes one selected channel from
/// `IqBlock::samples` and the block's `first_sample_monotonic_ns`.
pub trait IqSource {
    fn metadata(&self) -> &CaptureMetadata;

    /// Returns the next block, or permanent end-of-source.
    ///
    /// # Errors
    ///
    /// Returns [`CaptureError`] on an I/O failure, malformed metadata/data, or a live-source
    /// failure.
    fn next_block(&mut self) -> Result<Option<IqBlock>, CaptureError>;
}

#[derive(Debug)]
pub enum CaptureError {
    Io(io::Error),
    Metadata(String),
    Codec(String),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Metadata(reason) => write!(formatter, "metadata error: {reason}"),
            Self::Codec(reason) => write!(formatter, "capture codec error: {reason}"),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<io::Error> for CaptureError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct FileIqSource {
    metadata: CaptureMetadata,
    reader: BufReader<File>,
    block_size: usize,
    stored_sample_index: u64,
    elapsed_sample_index: u64,
    next_gap: usize,
    path: PathBuf,
}

impl FileIqSource {
    /// Opens `iq_path` and its JSON sidecar (`<iq_path>.json`).
    ///
    /// # Errors
    ///
    /// Returns [`CaptureError`] if either file cannot be read, the sidecar or byte length is
    /// invalid, the block size is zero, or a metadata requirement does not match.
    pub fn open(
        iq_path: impl AsRef<Path>,
        block_size: usize,
        requirements: &MetadataRequirements,
    ) -> Result<Self, CaptureError> {
        let iq_path = iq_path.as_ref();
        if block_size == 0 {
            return Err(CaptureError::Metadata(
                "block size must be positive".to_owned(),
            ));
        }
        let sidecar = sidecar_path(iq_path);
        let metadata: CaptureMetadata = serde_json::from_reader(BufReader::new(
            File::open(&sidecar).map_err(CaptureError::Io)?,
        ))
        .map_err(|error| CaptureError::Codec(format!("{}: {error}", sidecar.display())))?;
        validate_metadata(&metadata, requirements)?;
        let channel_count = metadata.channels.len() as u64;
        let expected_bytes = metadata
            .sample_count_per_channel
            .checked_mul(channel_count)
            .and_then(|value| value.checked_mul(8))
            .ok_or_else(|| CaptureError::Metadata("capture byte length overflows".to_owned()))?;
        let actual_bytes = fs::metadata(iq_path)?.len();
        if actual_bytes != expected_bytes {
            return Err(CaptureError::Codec(format!(
                "IQ length is {actual_bytes} bytes; metadata requires {expected_bytes}"
            )));
        }
        Ok(Self {
            metadata,
            reader: BufReader::new(File::open(iq_path)?),
            block_size,
            stored_sample_index: 0,
            elapsed_sample_index: 0,
            next_gap: 0,
            path: iq_path.to_owned(),
        })
    }
}

impl IqSource for FileIqSource {
    fn metadata(&self) -> &CaptureMetadata {
        &self.metadata
    }

    fn next_block(&mut self) -> Result<Option<IqBlock>, CaptureError> {
        if self.stored_sample_index == self.metadata.sample_count_per_channel {
            return Ok(None);
        }
        let mut discontinuities = Vec::new();
        while let Some(gap) = self.metadata.gaps.get(self.next_gap) {
            if gap.sample_index != self.stored_sample_index {
                break;
            }
            self.elapsed_sample_index = self
                .elapsed_sample_index
                .checked_add(gap.missing_samples)
                .ok_or_else(|| CaptureError::Metadata("sample timeline overflows".to_owned()))?;
            discontinuities.push(gap.clone());
            self.next_gap += 1;
        }
        let remaining = self.metadata.sample_count_per_channel - self.stored_sample_index;
        let until_gap = self
            .metadata
            .gaps
            .get(self.next_gap)
            .map_or(remaining, |gap| gap.sample_index - self.stored_sample_index);
        let count = remaining.min(until_gap).min(self.block_size as u64) as usize;

        let mut samples = vec![Vec::with_capacity(count); self.metadata.channels.len()];
        let mut bytes = [0_u8; 4];
        for _ in 0..count {
            for channel in &mut samples {
                self.reader.read_exact(&mut bytes).map_err(|error| {
                    CaptureError::Codec(format!("{}: {error}", self.path.display()))
                })?;
                let i = f32::from_le_bytes(bytes);
                self.reader.read_exact(&mut bytes).map_err(|error| {
                    CaptureError::Codec(format!("{}: {error}", self.path.display()))
                })?;
                let q = f32::from_le_bytes(bytes);
                channel.push(Complex64::new(f64::from(i), f64::from(q)));
            }
        }
        let first_sample_monotonic_ns = sample_time_ns(&self.metadata, self.elapsed_sample_index)?;
        self.stored_sample_index += count as u64;
        self.elapsed_sample_index += count as u64;
        Ok(Some(IqBlock {
            first_sample_monotonic_ns,
            sample_rate_hz: self.metadata.sample_rate_hz,
            centre_frequency_hz: self.metadata.centre_frequency_hz,
            channels: self.metadata.channels.clone(),
            samples,
            discontinuities_before: discontinuities,
        }))
    }
}

/// Writes one complete independently replayable segment and its JSON sidecar.
///
/// `channels` is channel-major and all channel lengths must match metadata exactly.
///
/// # Errors
///
/// Returns [`CaptureError`] for invalid metadata/dimensions/samples or an I/O/JSON failure.
pub fn write_capture(
    iq_path: impl AsRef<Path>,
    metadata: &CaptureMetadata,
    channels: &[Vec<Complex64>],
) -> Result<(), CaptureError> {
    validate_metadata(metadata, &MetadataRequirements::default())?;
    if channels.len() != metadata.channels.len()
        || channels
            .iter()
            .any(|channel| channel.len() as u64 != metadata.sample_count_per_channel)
    {
        return Err(CaptureError::Metadata(
            "channel data dimensions do not match metadata".to_owned(),
        ));
    }
    let iq_path = iq_path.as_ref();
    let mut writer = BufWriter::new(File::create(iq_path)?);
    for sample in 0..metadata.sample_count_per_channel as usize {
        for channel in channels {
            let value = channel[sample];
            if !value.re.is_finite()
                || !value.im.is_finite()
                || value.re < f64::from(f32::MIN)
                || value.re > f64::from(f32::MAX)
                || value.im < f64::from(f32::MIN)
                || value.im > f64::from(f32::MAX)
            {
                return Err(CaptureError::Codec(
                    "IQ samples must be finite and representable as f32".to_owned(),
                ));
            }
            writer.write_all(&(value.re as f32).to_le_bytes())?;
            writer.write_all(&(value.im as f32).to_le_bytes())?;
        }
    }
    writer.flush()?;
    let sidecar = sidecar_path(iq_path);
    let mut sidecar_writer = BufWriter::new(File::create(sidecar)?);
    serde_json::to_writer_pretty(&mut sidecar_writer, metadata)
        .map_err(|error| CaptureError::Codec(error.to_string()))?;
    sidecar_writer.flush()?;
    Ok(())
}

fn validate_metadata(
    metadata: &CaptureMetadata,
    requirements: &MetadataRequirements,
) -> Result<(), CaptureError> {
    if metadata.format_version != FORMAT_VERSION {
        return Err(CaptureError::Metadata(format!(
            "unsupported format version {}",
            metadata.format_version
        )));
    }
    if metadata.representation != REPRESENTATION || metadata.endianness != ENDIANNESS {
        return Err(CaptureError::Metadata(
            "only little-endian complex-f32 is supported".to_owned(),
        ));
    }
    if !metadata.sample_rate_hz.is_finite()
        || metadata.sample_rate_hz <= 0.0
        || !metadata.centre_frequency_hz.is_finite()
        || metadata.centre_frequency_hz < 0.0
        || !metadata.bandwidth_hz.is_finite()
        || metadata.bandwidth_hz <= 0.0
        || metadata.channels.is_empty()
        || metadata.gain_db.len() != metadata.channels.len()
        || metadata.gain_db.iter().any(|gain| !gain.is_finite())
        || metadata.calibration_id.is_empty()
        || metadata.configuration_id.is_empty()
    {
        return Err(CaptureError::Metadata(
            "invalid rate/frequency/channel/gain/identity metadata".to_owned(),
        ));
    }
    let mut previous = None;
    for gap in &metadata.gaps {
        if gap.missing_samples == 0
            || gap.sample_index >= metadata.sample_count_per_channel
            || previous.is_some_and(|value| gap.sample_index <= value)
        {
            return Err(CaptureError::Metadata(
                "gaps must be nonzero, ordered, unique, and in range".to_owned(),
            ));
        }
        previous = Some(gap.sample_index);
    }
    check_f64(
        "sample rate",
        requirements.sample_rate_hz,
        metadata.sample_rate_hz,
    )?;
    check_f64(
        "centre frequency",
        requirements.centre_frequency_hz,
        metadata.centre_frequency_hz,
    )?;
    check_eq(
        "channels",
        requirements.channels.as_ref(),
        &metadata.channels,
    )?;
    check_eq(
        "calibration ID",
        requirements.calibration_id.as_ref(),
        &metadata.calibration_id,
    )?;
    check_eq(
        "configuration ID",
        requirements.configuration_id.as_ref(),
        &metadata.configuration_id,
    )
}

fn check_f64(label: &str, expected: Option<f64>, actual: f64) -> Result<(), CaptureError> {
    if expected.is_some_and(|value| value.to_bits() != actual.to_bits()) {
        return Err(CaptureError::Metadata(format!("{label} mismatch")));
    }
    Ok(())
}

fn check_eq<T: PartialEq + ?Sized>(
    label: &str,
    expected: Option<&T>,
    actual: &T,
) -> Result<(), CaptureError> {
    if expected.is_some_and(|value| value != actual) {
        return Err(CaptureError::Metadata(format!("{label} mismatch")));
    }
    Ok(())
}

fn sample_time_ns(metadata: &CaptureMetadata, sample_index: u64) -> Result<u64, CaptureError> {
    let offset = (sample_index as f64 * 1e9 / metadata.sample_rate_hz).round();
    if !offset.is_finite() || offset > u64::MAX as f64 {
        return Err(CaptureError::Metadata(
            "sample timestamp overflows".to_owned(),
        ));
    }
    metadata
        .first_sample_monotonic_ns
        .checked_add(offset as u64)
        .ok_or_else(|| CaptureError::Metadata("sample timestamp overflows".to_owned()))
}

fn sidecar_path(iq_path: &Path) -> PathBuf {
    let mut name = iq_path.as_os_str().to_owned();
    name.push(".json");
    PathBuf::from(name)
}

/// Live bladeRF boundary, excluded from default builds.
///
/// This is deliberately a non-operational skeleton until target-host libbladeRF integration
/// and hardware validation are performed. See the crate README.
#[cfg(feature = "hardware")]
pub mod hardware {
    use super::{CaptureError, CaptureMetadata, IqBlock, IqSource};

    /// Planned libbladeRF synchronous `RX_X2` source **[UNVERIFIED]**.
    pub struct BladerfIqSource {
        metadata: CaptureMetadata,
    }

    impl BladerfIqSource {
        /// No live constructor is shipped: opening hardware without tested FFI lifecycle,
        /// stream configuration, timestamp, and overrun handling would be misleading.
        #[must_use]
        pub fn unverified_skeleton(metadata: CaptureMetadata) -> Self {
            Self { metadata }
        }
    }

    impl IqSource for BladerfIqSource {
        fn metadata(&self) -> &CaptureMetadata {
            &self.metadata
        }

        fn next_block(&mut self) -> Result<Option<IqBlock>, CaptureError> {
            Err(CaptureError::Metadata(
                "libbladeRF live RX_X2 backend is [UNVERIFIED] and not implemented".to_owned(),
            ))
        }
    }
}
