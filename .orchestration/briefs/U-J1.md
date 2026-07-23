# Brief U-J1 — On-disk journals (replace MemoryJournals)

Contract version: v4.1 (do NOT touch CONTRACTS.md — v5 is owned by a concurrent unit; your
codec decision goes in the crate README + report, contracts folding happens at integration).
Workspace: dedicated worktree, branch `unit/U-J1`. Commit here; never merge to main.
Read first: `docs/design/ARCHITECTURE.md` (On-disk formats section), `.orchestration/CONTRACTS.md`
v2 on-disk formats (record boundaries, manifest, separation, version/error behaviour, recovery
semantics — all binding; the codec itself is yours to choose), `crates/pnt-journal/src/lib.rs`
(the existing sink trait and MemoryJournals).

## Goal
`crates/pnt-journal`: a `FileJournals` implementation behind the SAME trait surface as
MemoryJournals (executive untouched — verify the workspace builds with zero changes outside
this crate):
1. Run directory with manifest (run UUID, schema versions, monotonic epoch metadata, optional
   RFC3339 UTC creation time, config hash + calibration IDs + software revision as caller-
   supplied strings, file list with checksums on finalisation).
2. Measurement journal and truth journal as PHYSICALLY separate, length-delimited,
   per-record-checksummed (CRC32 or better), schema-versioned binary streams of the v2 bus
   envelopes / truth records + integrity events + epochs. Choose the codec (serde/postcard,
   bincode, or hand-rolled TLV), justify in README (determinism matters: same input bytes →
   same output bytes, no HashMap iteration order), record the decision + rationale.
3. Segmenting: fixed-duration or fixed-size segments, atomic finalisation (write temp,
   fsync, rename), unclean-stop recovery: on open, a truncated/corrupt tail record is
   detected via length+checksum and the segment is recovered to the last good record —
   never a crash, never silent acceptance of a corrupt record (typed error surface).
4. Replay reader: iterate records in order with schema-version check (unknown REQUIRED
   version = hard error per contract); bit-exact roundtrip guarantee.
5. Truth separation: reader API for truth records is a separate type from the measurement
   reader; nothing in the estimator/gate crates can obtain it via this crate's public API
   without explicitly naming truth (compile-time separation, matching the architecture's
   no-online-read-edge rule).

## Tests (TDD)
Roundtrip bit-exactness (write N mixed records, read back, byte-compare re-serialisation);
truncated-tail recovery (chop file mid-record, reopen, assert last-good recovery + typed
report); corrupt-checksum detection; segment finalisation atomicity (temp file never visible
as final); unknown-schema-version hard error; separation (truth records never appear in
measurement reader); manifest contents. Use tempdir; zero wall-clock dependence in record
content (timestamps injected).
Gate: PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check.

## Files owned
`crates/pnt-journal/**` ONLY (plus its Cargo.toml deps). `.orchestration/reports/U-J1.md`.

## Report
Codec decision + determinism rationale, recovery semantics, evidence (real gate output),
[UNVERIFIED] items (e.g. crash-loss bounds pending config), what integration must wire later.
