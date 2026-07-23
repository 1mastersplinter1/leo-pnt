# pnt-journal on-disk format

The journal uses a hand-rolled, deterministic little-endian binary codec. Each segment
starts with `PNTJ`, a stream kind, and a required `u16` schema version. Records are
`u32 payload length | payload | u32 CRC-32`. The CRC covers the payload. Strings and
vectors have explicit `u32` lengths, enums have fixed `u8` discriminants, and floating
point values are stored as their exact IEEE-754 bits. There are no maps or platform-sized
fields in the encoding, so identical values always produce identical bytes on every
supported platform. This was chosen over a derived codec because `pnt-types` deliberately
has no serialization dependency and because an explicit encoding makes compatibility
decisions reviewable.

Measurement and truth data use different stream-kind headers, filenames, writer paths,
and reader types. `MeasurementReader` cannot return truth records; reading truth requires
explicitly constructing a `TruthReader`.

The active segment has a `.tmp` suffix. Finalisation flushes and `fsync`s it, renames it
to `.seg`, syncs the run directory, then atomically rewrites the manifest. Opening a run
scans active tails. A partial header, partial payload, or bad final-record CRC is truncated
to the preceding valid boundary and returned as a typed `RecoveryReport`; corruption in
a finalised segment is a hard `JournalError`. This deliberately treats only the active
tail as recoverable—immutable finalised history is never silently repaired.
