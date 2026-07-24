# Brief U-CAP1 — Real-hardware capture/ingest front-end (hardware-day de-risk)

Contract v5.1. Worktree branch `unit/U-CAP1`. Commit there; never merge to main; NO trailers.
Read first: DECISIONS.md D62, docs/research/R1-bladerf-market.md (the libbladeRF-Rust vs
SoapySDR maturity findings — you resolve this decision), docs/design/ARCHITECTURE.md
(module 4 SDR capture, on-disk raw-IQ format), .orchestration/CONTRACTS.md (raw-IQ container),
crates/pnt-tracker (synth + Tracker API — the consumer), crates/pnt-journal (on-disk format
precedent), crates/pnt-types (Frame/envelope).

## Goal
New crate `crates/pnt-capture` — the IQ source abstraction that feeds the tracker, testable
WITHOUT hardware:
1. **`IqSource` trait**: yields timestamped complex-baseband blocks (block size, sample rate,
   centre freq, channel) — the exact shape `pnt-tracker::Tracker` consumes. Document the
   metadata contract (matches ARCHITECTURE raw-IQ sidecar: representation, endianness, rate,
   centre freq, bandwidth, gain, ext-ref state, first-sample monotonic time, gaps/overruns).
2. **`FileIqSource`** (the testable path): reads a raw-IQ capture file + sidecar metadata,
   yields blocks deterministically; handles the interleaved coherent 2-channel layout from
   ARCHITECTURE. Round-trips with a `write` path so tests can synth->write->read->track.
3. **Binding DECISION (resolve from R1)**: choose libbladeRF-Rust vs SoapySDR for the live
   backend; write the rationale (maturity, 2-ch coherent support, sustained-throughput risk
   from R1) into the crate README + report. Implement the live `BladerfIqSource` behind a
   `hardware` cargo feature (OFF by default) so the crate builds and tests with zero hardware
   and no libbladeRF/SoapySDR system dep in the default build. The live path may be a
   documented, feature-gated skeleton calling the chosen binding's API [UNVERIFIED — not run
   against hardware]; do NOT fake a hardware run.
4. **End-to-end (no hardware)**: a test that generates IQ via pnt-tracker::synth, writes it
   through FileIqSource's writer, reads it back, runs the Tracker over the blocks, and
   asserts detections match — proving the capture->tracker seam is correct and ready for a
   real file. Overrun/gap and metadata-mismatch handling tested.

## Method
TDD. Gate: PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings
&& cargo fmt --all -- --check (default features; the hardware feature need not build without
the system lib, but MUST NOT break the default gate).

## Files owned
crates/pnt-capture/**, root Cargo.toml member line, .orchestration/reports/U-CAP1.md.

## Report
The binding decision + rationale, the IqSource contract, what the file path proves, exactly
what remains for real hardware (the [UNVERIFIED] live-backend surface), integration notes.
