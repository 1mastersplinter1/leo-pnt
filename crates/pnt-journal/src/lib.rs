//! Deterministic on-disk measurement and physically separate truth journals.
#![allow(
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    clippy::semicolon_if_nothing_returned,
    clippy::unnecessary_semicolon
)]

use pnt_types::{
    ArmAction, ArmCommand, Constellation, Frame, GnssFix, Heading, ImuSample, MeasurementEnvelope,
    MeasurementPayload, Provenance, QualityFlags, SourceId, SpeedThroughWater, TimeTag,
    TrackerDoppler, UtcTime, SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

const MAGIC: &[u8; 4] = b"PNTJ";
const HEADER_LEN: u64 = 7;
const MAX_RECORD_LEN: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct IntegrityEvent {
    pub monotonic_ns: u64,
    pub source_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MonotonicEpoch {
    pub epoch_id: String,
    pub start_monotonic_ns: u64,
    pub utc_rfc3339: Option<String>,
    pub utc_uncertainty_ns: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct RunMetadata {
    pub run_uuid: String,
    pub created_utc_rfc3339: Option<String>,
    pub monotonic_epochs: Vec<MonotonicEpoch>,
    pub config_hash: String,
    pub calibration_ids: Vec<String>,
    pub software_revision: String,
    pub hardware_setup: String,
    pub ephemeris_snapshot_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub stream: String,
    pub byte_length: u64,
    pub crc32: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunManifest {
    pub run_uuid: String,
    pub measurement_schema_version: u16,
    pub truth_schema_version: u16,
    pub created_utc_rfc3339: Option<String>,
    pub monotonic_epochs: Vec<MonotonicEpoch>,
    pub config_hash: String,
    pub calibration_ids: Vec<String>,
    pub software_revision: String,
    pub hardware_setup: String,
    pub ephemeris_snapshot_id: String,
    pub files: Vec<ManifestFile>,
}

impl From<RunMetadata> for RunManifest {
    fn from(value: RunMetadata) -> Self {
        Self {
            run_uuid: value.run_uuid,
            measurement_schema_version: SCHEMA_VERSION,
            truth_schema_version: SCHEMA_VERSION,
            created_utc_rfc3339: value.created_utc_rfc3339,
            monotonic_epochs: value.monotonic_epochs,
            config_hash: value.config_hash,
            calibration_ids: value.calibration_ids,
            software_revision: value.software_revision,
            hardware_setup: value.hardware_setup,
            ephemeris_snapshot_id: value.ephemeris_snapshot_id,
            files: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamKind {
    Measurement = 1,
    Truth = 2,
}

impl StreamKind {
    fn label(self) -> &'static str {
        match self {
            Self::Measurement => "measurement",
            Self::Truth => "truth",
        }
    }
    fn prefix(self) -> &'static str {
        match self {
            Self::Measurement => "measurements",
            Self::Truth => "truth",
        }
    }
}

#[derive(Debug)]
pub enum JournalError {
    Io(io::Error),
    Manifest(String),
    InvalidHeader(PathBuf),
    WrongStream {
        path: PathBuf,
        expected: StreamKind,
    },
    UnknownSchemaVersion {
        path: PathBuf,
        version: u16,
    },
    CorruptRecord {
        path: PathBuf,
        offset: u64,
        reason: &'static str,
    },
    Codec(&'static str),
    WriterFailed,
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for JournalError {}
impl From<io::Error> for JournalError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryReason {
    TruncatedLength,
    TruncatedPayload,
    ChecksumMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    pub path: PathBuf,
    pub original_len: u64,
    pub recovered_len: u64,
    pub reason: RecoveryReason,
}

pub trait JournalSinks {
    fn write_measurement(&mut self, envelope: &MeasurementEnvelope);
    fn write_truth(&mut self, envelope: &MeasurementEnvelope);
    fn write_integrity(&mut self, event: IntegrityEvent);
}

#[derive(Debug, Default)]
pub struct MemoryJournals {
    measurement_records: Vec<MeasurementEnvelope>,
    truth_records: Vec<MeasurementEnvelope>,
    integrity_events: Vec<IntegrityEvent>,
}
impl MemoryJournals {
    #[must_use]
    pub fn measurement_records(&self) -> &[MeasurementEnvelope] {
        &self.measurement_records
    }
    #[must_use]
    pub fn truth_records(&self) -> &[MeasurementEnvelope] {
        &self.truth_records
    }
    #[must_use]
    pub fn integrity_events(&self) -> &[IntegrityEvent] {
        &self.integrity_events
    }
}
impl JournalSinks for MemoryJournals {
    fn write_measurement(&mut self, envelope: &MeasurementEnvelope) {
        self.measurement_records.push(envelope.clone());
    }
    fn write_truth(&mut self, envelope: &MeasurementEnvelope) {
        self.truth_records.push(envelope.clone());
    }
    fn write_integrity(&mut self, event: IntegrityEvent) {
        self.integrity_events.push(event);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MeasurementJournalRecord {
    Envelope(MeasurementEnvelope),
    Integrity(IntegrityEvent),
    Epoch(MonotonicEpoch),
}
#[derive(Clone, Debug, PartialEq)]
pub enum TruthJournalRecord {
    Envelope(MeasurementEnvelope),
    Epoch(MonotonicEpoch),
}

struct SegmentWriter {
    root: PathBuf,
    kind: StreamKind,
    index: u32,
    max_bytes: u64,
    file: File,
    tmp_path: PathBuf,
    bytes: u64,
}
impl SegmentWriter {
    fn create(
        root: &Path,
        kind: StreamKind,
        index: u32,
        max_bytes: u64,
    ) -> Result<Self, JournalError> {
        let tmp_path = root.join(format!("{}-{index:06}.tmp", kind.prefix()));
        let mut file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&tmp_path)?;
        file.write_all(MAGIC)?;
        file.write_all(&[kind as u8])?;
        file.write_all(&SCHEMA_VERSION.to_le_bytes())?;
        Ok(Self {
            root: root.to_owned(),
            kind,
            index,
            max_bytes,
            file,
            tmp_path,
            bytes: HEADER_LEN,
        })
    }
    fn append(&mut self, payload: &[u8]) -> Result<bool, JournalError> {
        let record_len = 8_u64
            + u64::try_from(payload.len()).map_err(|_| JournalError::Codec("record too large"))?;
        if self.bytes > HEADER_LEN && self.bytes + record_len > self.max_bytes {
            return Ok(false);
        }
        let len =
            u32::try_from(payload.len()).map_err(|_| JournalError::Codec("record too large"))?;
        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(payload)?;
        self.file.write_all(&crc32(payload).to_le_bytes())?;
        self.bytes += record_len;
        Ok(true)
    }
    fn finish(mut self) -> Result<(ManifestFile, u32), JournalError> {
        self.file.flush()?;
        self.file.sync_all()?;
        self.file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        self.file.read_to_end(&mut bytes)?;
        let final_name = format!("{}-{:06}.seg", self.kind.prefix(), self.index);
        fs::rename(&self.tmp_path, self.root.join(&final_name))?;
        sync_dir(&self.root)?;
        Ok((
            ManifestFile {
                path: final_name,
                stream: self.kind.label().into(),
                byte_length: self.bytes,
                crc32: format!("{:08x}", crc32(&bytes)),
            },
            self.index + 1,
        ))
    }
}

pub struct FileJournals {
    root: PathBuf,
    manifest: RunManifest,
    measurement: Option<SegmentWriter>,
    truth: Option<SegmentWriter>,
    max_segment_bytes: u64,
    next_measurement: u32,
    next_truth: u32,
    latched_error: Option<JournalError>,
}
impl FileJournals {
    pub fn create(
        path: impl AsRef<Path>,
        metadata: RunMetadata,
        max_segment_bytes: u64,
    ) -> Result<Self, JournalError> {
        fs::create_dir_all(path.as_ref())?;
        if fs::read_dir(path.as_ref())?.next().is_some() {
            return Err(JournalError::Manifest("run directory is not empty".into()));
        }
        let manifest = RunManifest::from(metadata);
        write_manifest(path.as_ref(), &manifest)?;
        Self::from_parts(path.as_ref(), manifest, max_segment_bytes, Vec::new())
    }
    pub fn open(
        path: impl AsRef<Path>,
        max_segment_bytes: u64,
    ) -> Result<(Self, Vec<RecoveryReport>), JournalError> {
        let mut manifest: RunManifest =
            serde_json::from_reader(File::open(path.as_ref().join("manifest.json"))?)
                .map_err(|e| JournalError::Manifest(e.to_string()))?;
        if manifest.measurement_schema_version != SCHEMA_VERSION {
            return Err(JournalError::UnknownSchemaVersion {
                path: path.as_ref().join("manifest.json"),
                version: manifest.measurement_schema_version,
            });
        }
        if manifest.truth_schema_version != SCHEMA_VERSION {
            return Err(JournalError::UnknownSchemaVersion {
                path: path.as_ref().join("manifest.json"),
                version: manifest.truth_schema_version,
            });
        }
        let mut reports = Vec::new();
        for entry in fs::read_dir(path.as_ref())? {
            let entry = entry?;
            let p = entry.path();
            if p.extension().and_then(|x| x.to_str()) == Some("tmp")
                && p.file_name().and_then(|x| x.to_str()) != Some("manifest.json.tmp")
            {
                let report = recover_tail(&p)?;
                let mut bytes = Vec::new();
                File::open(&p)?.read_to_end(&mut bytes)?;
                let final_path = p.with_extension("seg");
                fs::rename(&p, &final_path)?;
                sync_dir(path.as_ref())?;
                let stream = if p
                    .file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .starts_with(StreamKind::Measurement.prefix())
                {
                    StreamKind::Measurement.label()
                } else {
                    StreamKind::Truth.label()
                };
                let name = final_path
                    .file_name()
                    .and_then(|x| x.to_str())
                    .ok_or(JournalError::Codec("invalid segment name"))?
                    .to_owned();
                manifest.files.retain(|x| x.path != name);
                manifest.files.push(ManifestFile {
                    path: name,
                    stream: stream.into(),
                    byte_length: bytes.len() as u64,
                    crc32: format!("{:08x}", crc32(&bytes)),
                });
                reports.push(report);
            }
        }
        manifest.files.sort_by(|a, b| a.path.cmp(&b.path));
        write_manifest(path.as_ref(), &manifest)?;
        let this = Self::from_parts(path.as_ref(), manifest, max_segment_bytes, reports.clone())?;
        Ok((this, reports))
    }
    fn from_parts(
        root: &Path,
        manifest: RunManifest,
        max_segment_bytes: u64,
        _: Vec<RecoveryReport>,
    ) -> Result<Self, JournalError> {
        let next_measurement = next_index(root, StreamKind::Measurement)?;
        let next_truth = next_index(root, StreamKind::Truth)?;
        Ok(Self {
            root: root.into(),
            manifest,
            measurement: None,
            truth: None,
            max_segment_bytes: max_segment_bytes.max(HEADER_LEN + 9),
            next_measurement,
            next_truth,
            latched_error: None,
        })
    }
    pub fn try_write_measurement(
        &mut self,
        value: &MeasurementEnvelope,
    ) -> Result<(), JournalError> {
        self.append(
            StreamKind::Measurement,
            encode_measurement(&MeasurementJournalRecord::Envelope(value.clone()))?,
        )
    }
    pub fn try_write_truth(&mut self, value: &MeasurementEnvelope) -> Result<(), JournalError> {
        self.append(
            StreamKind::Truth,
            encode_truth(&TruthJournalRecord::Envelope(value.clone()))?,
        )
    }
    pub fn try_write_integrity(&mut self, value: IntegrityEvent) -> Result<(), JournalError> {
        self.append(
            StreamKind::Measurement,
            encode_measurement(&MeasurementJournalRecord::Integrity(value))?,
        )
    }
    pub fn write_epoch(&mut self, epoch: MonotonicEpoch) -> Result<(), JournalError> {
        self.append(
            StreamKind::Measurement,
            encode_measurement(&MeasurementJournalRecord::Epoch(epoch.clone()))?,
        )?;
        self.append(
            StreamKind::Truth,
            encode_truth(&TruthJournalRecord::Epoch(epoch))?,
        )
    }
    fn append(&mut self, kind: StreamKind, payload: Vec<u8>) -> Result<(), JournalError> {
        if self.latched_error.is_some() {
            return Err(JournalError::WriterFailed);
        }
        let slot = match kind {
            StreamKind::Measurement => &mut self.measurement,
            StreamKind::Truth => &mut self.truth,
        };
        let next = match kind {
            StreamKind::Measurement => &mut self.next_measurement,
            StreamKind::Truth => &mut self.next_truth,
        };
        if slot.is_none() {
            *slot = Some(SegmentWriter::create(
                &self.root,
                kind,
                *next,
                self.max_segment_bytes,
            )?);
        }
        if !slot.as_mut().expect("created").append(&payload)? {
            let old = slot.take().expect("present");
            let (file, index) = old.finish()?;
            self.manifest.files.push(file);
            *next = index;
            write_manifest(&self.root, &self.manifest)?;
            *slot = Some(SegmentWriter::create(
                &self.root,
                kind,
                *next,
                self.max_segment_bytes,
            )?);
            slot.as_mut().expect("created").append(&payload)?;
        }
        Ok(())
    }
    pub fn finalize(mut self) -> Result<RunManifest, JournalError> {
        for slot in [&mut self.measurement, &mut self.truth] {
            if let Some(writer) = slot.take() {
                let (file, _) = writer.finish()?;
                self.manifest.files.push(file);
            }
        }
        self.manifest.files.sort_by(|a, b| a.path.cmp(&b.path));
        write_manifest(&self.root, &self.manifest)?;
        Ok(self.manifest)
    }
    #[must_use]
    pub fn latched_error(&self) -> Option<&JournalError> {
        self.latched_error.as_ref()
    }
}
impl JournalSinks for FileJournals {
    fn write_measurement(&mut self, e: &MeasurementEnvelope) {
        if let Err(err) = self.try_write_measurement(e) {
            self.latched_error = Some(err);
        }
    }
    fn write_truth(&mut self, e: &MeasurementEnvelope) {
        if let Err(err) = self.try_write_truth(e) {
            self.latched_error = Some(err);
        }
    }
    fn write_integrity(&mut self, e: IntegrityEvent) {
        if let Err(err) = self.try_write_integrity(e) {
            self.latched_error = Some(err);
        }
    }
}

pub struct MeasurementReader {
    inner: SegmentReader,
}
impl MeasurementReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        Ok(Self {
            inner: SegmentReader::open(path.as_ref(), StreamKind::Measurement)?,
        })
    }
}
impl Iterator for MeasurementReader {
    type Item = Result<MeasurementJournalRecord, JournalError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next_payload()
            .map(|r| r.and_then(|p| decode_measurement(&p)))
    }
}
pub struct TruthReader {
    inner: SegmentReader,
}
impl TruthReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        Ok(Self {
            inner: SegmentReader::open(path.as_ref(), StreamKind::Truth)?,
        })
    }
}
impl Iterator for TruthReader {
    type Item = Result<TruthJournalRecord, JournalError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next_payload()
            .map(|r| r.and_then(|p| decode_truth(&p)))
    }
}

struct SegmentReader {
    files: Vec<PathBuf>,
    position: usize,
    current: Option<File>,
    offset: u64,
    kind: StreamKind,
}
impl SegmentReader {
    fn open(root: &Path, kind: StreamKind) -> Result<Self, JournalError> {
        let mut files = segment_paths(root, kind, "seg")?;
        files.sort();
        Ok(Self {
            files,
            position: 0,
            current: None,
            offset: HEADER_LEN,
            kind,
        })
    }
    fn next_payload(&mut self) -> Option<Result<Vec<u8>, JournalError>> {
        loop {
            if self.current.is_none() {
                let path = self.files.get(self.position)?.clone();
                self.position += 1;
                match open_checked(&path, self.kind) {
                    Ok(f) => {
                        self.current = Some(f);
                        self.offset = HEADER_LEN;
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
            let path = self.files[self.position - 1].clone();
            let file = self.current.as_mut().expect("set");
            let mut len = [0; 4];
            match file.read(&mut len) {
                Ok(0) => {
                    self.current = None;
                    continue;
                }
                Ok(4) => {}
                Ok(_) => {
                    return Some(Err(JournalError::CorruptRecord {
                        path,
                        offset: self.offset,
                        reason: "truncated length",
                    }))
                }
                Err(e) => return Some(Err(e.into())),
            }
            let n = u32::from_le_bytes(len) as usize;
            if n > MAX_RECORD_LEN {
                return Some(Err(JournalError::CorruptRecord {
                    path,
                    offset: self.offset,
                    reason: "invalid length",
                }));
            }
            let mut payload = vec![0; n];
            if file.read_exact(&mut payload).is_err() {
                return Some(Err(JournalError::CorruptRecord {
                    path,
                    offset: self.offset,
                    reason: "truncated payload",
                }));
            }
            let mut sum = [0; 4];
            if file.read_exact(&mut sum).is_err() {
                return Some(Err(JournalError::CorruptRecord {
                    path,
                    offset: self.offset,
                    reason: "truncated checksum",
                }));
            }
            if crc32(&payload) != u32::from_le_bytes(sum) {
                return Some(Err(JournalError::CorruptRecord {
                    path,
                    offset: self.offset,
                    reason: "checksum mismatch",
                }));
            }
            self.offset += 8 + n as u64;
            return Some(Ok(payload));
        }
    }
}

fn open_checked(path: &Path, kind: StreamKind) -> Result<File, JournalError> {
    let mut f = File::open(path)?;
    let mut h = [0; 7];
    f.read_exact(&mut h)
        .map_err(|_| JournalError::InvalidHeader(path.into()))?;
    if &h[..4] != MAGIC {
        return Err(JournalError::InvalidHeader(path.into()));
    }
    if h[4] != kind as u8 {
        return Err(JournalError::WrongStream {
            path: path.into(),
            expected: kind,
        });
    }
    let version = u16::from_le_bytes([h[5], h[6]]);
    if version != SCHEMA_VERSION {
        return Err(JournalError::UnknownSchemaVersion {
            path: path.into(),
            version,
        });
    }
    Ok(f)
}

fn recover_tail(path: &Path) -> Result<RecoveryReport, JournalError> {
    let mut f = OpenOptions::new().read(true).write(true).open(path)?;
    let original_len = f.metadata()?.len();
    let mut h = [0; 7];
    f.read_exact(&mut h)
        .map_err(|_| JournalError::InvalidHeader(path.into()))?;
    if &h[..4] != MAGIC || !matches!(h[4], 1 | 2) {
        return Err(JournalError::InvalidHeader(path.into()));
    }
    let version = u16::from_le_bytes([h[5], h[6]]);
    if version != SCHEMA_VERSION {
        return Err(JournalError::UnknownSchemaVersion {
            path: path.into(),
            version,
        });
    }
    let mut good = HEADER_LEN;
    let reason = loop {
        let mut len = [0; 4];
        match f.read(&mut len)? {
            0 => break RecoveryReason::TruncatedLength,
            4 => {}
            _ => break RecoveryReason::TruncatedLength,
        }
        let n = u32::from_le_bytes(len) as usize;
        if n > MAX_RECORD_LEN {
            break RecoveryReason::TruncatedPayload;
        }
        let mut payload = vec![0; n];
        if f.read_exact(&mut payload).is_err() {
            break RecoveryReason::TruncatedPayload;
        }
        let mut sum = [0; 4];
        if f.read_exact(&mut sum).is_err() {
            break RecoveryReason::TruncatedPayload;
        }
        if crc32(&payload) != u32::from_le_bytes(sum) {
            break RecoveryReason::ChecksumMismatch;
        }
        good += 8 + n as u64;
    };
    if good == original_len {
        return Ok(RecoveryReport {
            path: path.into(),
            original_len,
            recovered_len: good,
            reason: RecoveryReason::TruncatedLength,
        });
    }
    f.set_len(good)?;
    f.sync_all()?;
    Ok(RecoveryReport {
        path: path.into(),
        original_len,
        recovered_len: good,
        reason,
    })
}

fn next_index(root: &Path, kind: StreamKind) -> Result<u32, JournalError> {
    let mut max = None;
    for ext in ["seg", "tmp"] {
        for p in segment_paths(root, kind, ext)? {
            if let Some(stem) = p.file_stem().and_then(|x| x.to_str()) {
                if let Some(v) = stem.rsplit('-').next().and_then(|x| x.parse::<u32>().ok()) {
                    max = Some(max.map_or(v, |m: u32| m.max(v)));
                }
            }
        }
    }
    Ok(max.map_or(0, |v| v + 1))
}
fn segment_paths(root: &Path, kind: StreamKind, ext: &str) -> Result<Vec<PathBuf>, JournalError> {
    let mut out = Vec::new();
    for e in fs::read_dir(root)? {
        let p = e?.path();
        let name = p.file_name().and_then(|x| x.to_str()).unwrap_or("");
        if name.starts_with(kind.prefix()) && p.extension().and_then(|x| x.to_str()) == Some(ext) {
            out.push(p);
        }
    }
    Ok(out)
}
fn write_manifest(root: &Path, manifest: &RunManifest) -> Result<(), JournalError> {
    let temp = root.join("manifest.json.tmp");
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp)?;
    serde_json::to_writer_pretty(&mut f, manifest)
        .map_err(|e| JournalError::Manifest(e.to_string()))?;
    f.write_all(b"\n")?;
    f.sync_all()?;
    fs::rename(temp, root.join("manifest.json"))?;
    sync_dir(root)
}
fn sync_dir(path: &Path) -> Result<(), JournalError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

struct Encoder(Vec<u8>);
impl Encoder {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend(v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend(v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend(v.to_le_bytes());
    }
    fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }
    fn string(&mut self, v: &str) -> Result<(), JournalError> {
        self.u32(u32::try_from(v.len()).map_err(|_| JournalError::Codec("string too long"))?);
        self.0.extend(v.as_bytes());
        Ok(())
    }
}
struct Decoder<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Decoder<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], JournalError> {
        let end = self
            .at
            .checked_add(n)
            .ok_or(JournalError::Codec("overflow"))?;
        let v = self
            .bytes
            .get(self.at..end)
            .ok_or(JournalError::Codec("truncated value"))?;
        self.at = end;
        Ok(v)
    }
    fn u8(&mut self) -> Result<u8, JournalError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, JournalError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("length"),
        ))
    }
    fn u32(&mut self) -> Result<u32, JournalError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("length"),
        ))
    }
    fn u64(&mut self) -> Result<u64, JournalError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("length"),
        ))
    }
    fn f64(&mut self) -> Result<f64, JournalError> {
        Ok(f64::from_bits(self.u64()?))
    }
    fn string(&mut self) -> Result<String, JournalError> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| JournalError::Codec("invalid utf8"))
    }
    fn done(&self) -> Result<(), JournalError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(JournalError::Codec("trailing bytes"))
        }
    }
}

fn encode_measurement(v: &MeasurementJournalRecord) -> Result<Vec<u8>, JournalError> {
    let mut e = Encoder(Vec::new());
    match v {
        MeasurementJournalRecord::Envelope(x) => {
            e.u8(1);
            enc_envelope(&mut e, x)?
        }
        MeasurementJournalRecord::Integrity(x) => {
            e.u8(2);
            e.u64(x.monotonic_ns);
            e.string(&x.source_id)?;
            e.string(&x.reason)?
        }
        MeasurementJournalRecord::Epoch(x) => {
            e.u8(3);
            enc_epoch(&mut e, x)?
        }
    }
    Ok(e.0)
}
fn decode_measurement(b: &[u8]) -> Result<MeasurementJournalRecord, JournalError> {
    let mut d = Decoder { bytes: b, at: 0 };
    let v = match d.u8()? {
        1 => MeasurementJournalRecord::Envelope(dec_envelope(&mut d)?),
        2 => MeasurementJournalRecord::Integrity(IntegrityEvent {
            monotonic_ns: d.u64()?,
            source_id: d.string()?,
            reason: d.string()?,
        }),
        3 => MeasurementJournalRecord::Epoch(dec_epoch(&mut d)?),
        _ => return Err(JournalError::Codec("unknown measurement record")),
    };
    d.done()?;
    Ok(v)
}
fn encode_truth(v: &TruthJournalRecord) -> Result<Vec<u8>, JournalError> {
    let mut e = Encoder(Vec::new());
    match v {
        TruthJournalRecord::Envelope(x) => {
            e.u8(1);
            enc_envelope(&mut e, x)?
        }
        TruthJournalRecord::Epoch(x) => {
            e.u8(2);
            enc_epoch(&mut e, x)?
        }
    }
    Ok(e.0)
}
fn decode_truth(b: &[u8]) -> Result<TruthJournalRecord, JournalError> {
    let mut d = Decoder { bytes: b, at: 0 };
    let v = match d.u8()? {
        1 => TruthJournalRecord::Envelope(dec_envelope(&mut d)?),
        2 => TruthJournalRecord::Epoch(dec_epoch(&mut d)?),
        _ => return Err(JournalError::Codec("unknown truth record")),
    };
    d.done()?;
    Ok(v)
}
fn enc_epoch(e: &mut Encoder, x: &MonotonicEpoch) -> Result<(), JournalError> {
    e.string(&x.epoch_id)?;
    e.u64(x.start_monotonic_ns);
    enc_opt_string(e, x.utc_rfc3339.as_deref())?;
    match x.utc_uncertainty_ns {
        Some(v) => {
            e.u8(1);
            e.u64(v)
        }
        None => e.u8(0),
    }
    Ok(())
}
fn dec_epoch(d: &mut Decoder<'_>) -> Result<MonotonicEpoch, JournalError> {
    Ok(MonotonicEpoch {
        epoch_id: d.string()?,
        start_monotonic_ns: d.u64()?,
        utc_rfc3339: dec_opt_string(d)?,
        utc_uncertainty_ns: if d.u8()? == 0 { None } else { Some(d.u64()?) },
    })
}
fn enc_opt_string(e: &mut Encoder, v: Option<&str>) -> Result<(), JournalError> {
    match v {
        Some(x) => {
            e.u8(1);
            e.string(x)?
        }
        None => e.u8(0),
    }
    Ok(())
}
fn dec_opt_string(d: &mut Decoder<'_>) -> Result<Option<String>, JournalError> {
    Ok(if d.u8()? == 0 {
        None
    } else {
        Some(d.string()?)
    })
}

fn enc_envelope(e: &mut Encoder, x: &MeasurementEnvelope) -> Result<(), JournalError> {
    e.u16(x.schema_version);
    e.string(&x.source_id.0)?;
    e.u64(x.sequence);
    match x.sample_time {
        TimeTag::DeviceNanoseconds(v) => {
            e.u8(1);
            e.u64(v)
        }
        TimeTag::HostMonotonicNanoseconds(v) => {
            e.u8(2);
            e.u64(v)
        }
    }
    e.u64(x.host_receive_monotonic_ns);
    match &x.utc {
        Some(v) => {
            e.u8(1);
            e.string(&v.rfc3339)?;
            e.u64(v.uncertainty_ns)
        }
        None => e.u8(0),
    };
    enc_payload(e, &x.payload)?;
    e.u8(frame_to_u8(x.frame));
    e.u32(u32::try_from(x.covariance.len()).map_err(|_| JournalError::Codec("vector too long"))?);
    for v in &x.covariance {
        e.f64(*v)
    }
    e.u32(x.quality.0);
    e.string(&x.calibration_id)?;
    match &x.provenance {
        Provenance::CaptureRecord(v) => {
            e.u8(1);
            e.string(v)?
        }
        Provenance::SourceRecord(v) => {
            e.u8(2);
            e.string(v)?
        }
        Provenance::DerivedRecord(v) => {
            e.u8(3);
            e.string(v)?
        }
    }
    Ok(())
}
fn dec_envelope(d: &mut Decoder<'_>) -> Result<MeasurementEnvelope, JournalError> {
    let schema_version = d.u16()?;
    if schema_version != SCHEMA_VERSION {
        return Err(JournalError::UnknownSchemaVersion {
            path: PathBuf::from("record"),
            version: schema_version,
        });
    }
    let source_id = SourceId(d.string()?);
    let sequence = d.u64()?;
    let sample_time = match d.u8()? {
        1 => TimeTag::DeviceNanoseconds(d.u64()?),
        2 => TimeTag::HostMonotonicNanoseconds(d.u64()?),
        _ => return Err(JournalError::Codec("time tag")),
    };
    let host_receive_monotonic_ns = d.u64()?;
    let utc = if d.u8()? == 0 {
        None
    } else {
        Some(UtcTime {
            rfc3339: d.string()?,
            uncertainty_ns: d.u64()?,
        })
    };
    let payload = dec_payload(d)?;
    let frame = u8_to_frame(d.u8()?)?;
    let n = d.u32()? as usize;
    if n > MAX_RECORD_LEN / 8 {
        return Err(JournalError::Codec("vector too long"));
    }
    let mut covariance = Vec::with_capacity(n);
    for _ in 0..n {
        covariance.push(d.f64()?)
    }
    let quality = QualityFlags(d.u32()?);
    let calibration_id = d.string()?;
    let provenance = match d.u8()? {
        1 => Provenance::CaptureRecord(d.string()?),
        2 => Provenance::SourceRecord(d.string()?),
        3 => Provenance::DerivedRecord(d.string()?),
        _ => return Err(JournalError::Codec("provenance")),
    };
    Ok(MeasurementEnvelope {
        schema_version,
        source_id,
        sequence,
        sample_time,
        host_receive_monotonic_ns,
        utc,
        payload,
        frame,
        covariance,
        quality,
        calibration_id,
        provenance,
    })
}
fn enc_payload(e: &mut Encoder, p: &MeasurementPayload) -> Result<(), JournalError> {
    match p {
        MeasurementPayload::Imu(v) => {
            e.u8(1);
            for x in v.acceleration_mps2 {
                e.f64(x)
            }
            for x in v.angular_rate_rps {
                e.f64(x)
            }
        }
        MeasurementPayload::Heading(v) => {
            e.u8(2);
            e.f64(v.radians)
        }
        MeasurementPayload::SpeedThroughWater(v) => {
            e.u8(3);
            e.f64(v.metres_per_second)
        }
        MeasurementPayload::Gnss(v) => {
            e.u8(4);
            for x in v.position_ecef_m {
                e.f64(x)
            }
            for x in v.velocity_ned_mps {
                e.f64(x)
            }
        }
        MeasurementPayload::TrackerDoppler(v) => {
            e.u8(5);
            e.u8(match v.constellation {
                Constellation::Starlink => 1,
                Constellation::Iridium => 2,
                Constellation::OneWeb => 3,
                Constellation::Orbcomm => 4,
            });
            e.f64(v.correlation_peak_hz);
            e.f64(v.nominal_carrier_hz)
        }
        MeasurementPayload::ArmCommand(v) => {
            e.u8(6);
            e.u8(match v.action {
                ArmAction::Arm => 1,
                ArmAction::Disarm => 2,
            });
            e.u64(v.host_monotonic_ns);
            e.string(&v.source_id.0)?
        }
    }
    Ok(())
}
fn dec_payload(d: &mut Decoder<'_>) -> Result<MeasurementPayload, JournalError> {
    Ok(match d.u8()? {
        1 => {
            let mut a = [0.; 3];
            let mut g = [0.; 3];
            for x in &mut a {
                *x = d.f64()?
            }
            for x in &mut g {
                *x = d.f64()?
            }
            MeasurementPayload::Imu(ImuSample {
                acceleration_mps2: a,
                angular_rate_rps: g,
            })
        }
        2 => MeasurementPayload::Heading(Heading { radians: d.f64()? }),
        3 => MeasurementPayload::SpeedThroughWater(SpeedThroughWater {
            metres_per_second: d.f64()?,
        }),
        4 => {
            let mut p = [0.; 3];
            let mut v = [0.; 3];
            for x in &mut p {
                *x = d.f64()?
            }
            for x in &mut v {
                *x = d.f64()?
            }
            MeasurementPayload::Gnss(GnssFix {
                position_ecef_m: p,
                velocity_ned_mps: v,
            })
        }
        5 => MeasurementPayload::TrackerDoppler(TrackerDoppler {
            constellation: match d.u8()? {
                1 => Constellation::Starlink,
                2 => Constellation::Iridium,
                3 => Constellation::OneWeb,
                4 => Constellation::Orbcomm,
                _ => return Err(JournalError::Codec("constellation")),
            },
            correlation_peak_hz: d.f64()?,
            nominal_carrier_hz: d.f64()?,
        }),
        6 => MeasurementPayload::ArmCommand(ArmCommand {
            action: match d.u8()? {
                1 => ArmAction::Arm,
                2 => ArmAction::Disarm,
                _ => return Err(JournalError::Codec("arm action")),
            },
            host_monotonic_ns: d.u64()?,
            source_id: SourceId(d.string()?),
        }),
        _ => return Err(JournalError::Codec("payload")),
    })
}
fn frame_to_u8(v: Frame) -> u8 {
    match v {
        Frame::EarthCenteredEarthFixed => 1,
        Frame::LocalNorthEastDown => 2,
        Frame::VesselReference => 3,
        Frame::Sensor => 4,
        Frame::AntennaPhaseCenter => 5,
        Frame::FrameIndependent => 6,
    }
}
fn u8_to_frame(v: u8) -> Result<Frame, JournalError> {
    Ok(match v {
        1 => Frame::EarthCenteredEarthFixed,
        2 => Frame::LocalNorthEastDown,
        3 => Frame::VesselReference,
        4 => Frame::Sensor,
        5 => Frame::AntennaPhaseCenter,
        6 => Frame::FrameIndependent,
        _ => return Err(JournalError::Codec("frame")),
    })
}

#[cfg(test)]
mod tests;
