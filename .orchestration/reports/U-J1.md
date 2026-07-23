# U-J1 report — on-disk journals

## Outcome

Implemented `FileJournals` in `pnt-journal` without changing the executive or the existing
`JournalSinks`/`MemoryJournals` surface. The crate now provides a run manifest, physically
separate measurement/truth streams, fixed-size segmentation, atomic segment and manifest
finalisation, typed recovery/corruption errors, and separate replay-reader types.

## Codec decision and determinism

Selected a hand-rolled binary codec. Segment headers contain magic, stream kind and required
schema version. Every record is `u32 little-endian length | payload | u32 CRC-32(payload)`.
Payloads encode enums with fixed discriminants, strings/vectors with explicit `u32` lengths,
integers little-endian, and `f64` as its exact IEEE-754 bits. The format contains no maps,
native-width numbers, derived field ordering, or runtime timestamps. Therefore equal input
values produce equal bytes, including signed zero and NaN payload bits. The crate README is
the format decision record.

## Recovery semantics

The only mutable segment has a `.tmp` suffix. Finalisation flushes and fsyncs the file,
renames it to `.seg`, syncs the directory, and then atomically rewrites/fsyncs the manifest.
Opening a run validates required manifest/stream schema versions and scans active segments.
A truncated length, truncated payload/checksum, or bad tail CRC truncates only that active
segment to its last verified record boundary and returns a typed `RecoveryReport`. The
recovered segment is then atomically promoted to immutable `.seg` history and entered in the
manifest. Corruption in a final `.seg` is never repaired or accepted: replay returns a typed
`JournalError::CorruptRecord`.

The inherited trait cannot return I/O errors. `FileJournals` therefore exposes fallible
`try_write_measurement`, `try_write_truth`, and `try_write_integrity` APIs; calls through the
unchanged trait latch the first typed error, queryable through `latched_error()`.

## Evidence

Executed from the workspace on 2026-07-23 (offline because the environment has no DNS):

```text
cargo test --offline
  fusion-executive: 15 passed
  pnt-ephemeris: 6 passed
  pnt-estimator: 13 passed
  pnt-journal: 5 passed
  pnt-predictor: 4 passed
  pnt-types: 3 passed
  all unit/doc-test suites passed; 0 failed

cargo clippy --all-targets --offline -- -D warnings
  Finished dev profile successfully

cargo fmt --all -- --check
  exited 0
```

Journal tests cover deterministic mixed-record re-serialization, physical truth separation,
truncated active-tail recovery, final-segment checksum corruption, atomic temp/final
visibility, manifest metadata/checksums, fixed-size segment rollover, and unknown required
stream-schema rejection.

## Integration and unverified items

- Integration must construct `RunMetadata` from the configuration, calibration registry,
  software build revision, hardware setup and ephemeris snapshot, then select an operational
  segment-size/crash-loss bound.
- Integration must replace the executive's in-memory journal construction with
  `FileJournals` and treat a latched sink error as an integrity/operational failure.
- UTC creation time and monotonic epoch mappings remain caller-injected; the journal never
  reads wall-clock time.
- `[UNVERIFIED]`: the appropriate production segment size and resulting crash-loss/storage
  bounds require deployment measurements.
- `[UNVERIFIED]`: CRC-32 is contract-compliant for record/file integrity but is not a
  cryptographic authenticity mechanism; deployments requiring tamper evidence should add a
  signed manifest or cryptographic digest in a later format version.
