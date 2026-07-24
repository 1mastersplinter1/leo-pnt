# U-CAP1 report

## Delivered

Added `pnt-capture`, a default-hardware-free IQ ingress crate. `IqSource` yields timestamped,
coherent channel blocks carrying sample rate, centre frequency, channel allocation, and
explicit discontinuities. Its static metadata covers the architecture sidecar contract:
representation, endianness, rate, tuning/bandwidth/gain, channel allocation, external
reference state, first-sample monotonic and optional UTC, sample count, gaps/overruns, and
calibration/configuration identity.

`write_capture` and `FileIqSource` implement deterministic little-endian complex-f32
sample-major/channel-interleaved segments. The reader validates byte length and requested
metadata before yielding data. Gaps and overruns are surfaced on the following block and
advance the monotonic sample timeline.

## Binding decision

Selected system libbladeRF C with thin Rust FFI for the eventual live backend, rather than
SoapySDR. R1 reports that the active pure-Rust driver is BladeRF1-only, the existing
libbladeRF Rust wrapper is thin/WIP, and the C library remains the production-proven route.
It exposes the bladeRF-specific coherent `RX_X2`, buffer/transfer, timestamp, and reference
controls needed here. SoapyBladeRF is useful and packaged, but its extra abstraction/plugin
layer does not remove the target-host validation needed for coherent timing, overrun
semantics, or sustained dual-channel throughput. This follows R1's recommendation:
libbladeRF for production; Soapy for tools.

The `hardware` feature contains only `BladerfIqSource`, a non-operational, explicitly
**[UNVERIFIED]** skeleton. No device was opened and no hardware run is claimed.

## Hardware-free proof

The end-to-end test uses `pnt-tracker::synth`, writes two coherently interleaved channels,
replays blocks, and compares tracker detections against direct synthetic processing.
Additional tests cover channel layout, gap/overrun timing, exact metadata mismatch, and
truncated IQ rejection.

## Remaining real-hardware work [UNVERIFIED]

Pin libbladeRF and generate/own its FFI; implement device configuration and synchronous
`RX_X2`; verify ADC representation; map device timestamps to the process monotonic epoch;
handle timeout/short read/overrun/reset/shutdown; add atomic segment lifecycle/manifests;
and run sustained dual-channel tests on the target host/USB controller. Raw ADC packing and
the broader run-container codec remain **[UNVERIFIED]**.

## Integration notes

Consumers select a channel from `IqBlock::samples`, reject or reset tracking on a non-empty
`discontinuities_before`, and pass `first_sample_monotonic_ns` to
`CorrelationTracker::process_block`. `MetadataRequirements` should be populated from the
tracker and calibration configuration so mismatches fail before processing.

## Evidence

- `PATH="$HOME/.cargo/bin:$PATH" cargo test` — PASS (workspace; 3 new capture tests).
- `PATH="$HOME/.cargo/bin:$PATH" cargo clippy --all-targets -- -D warnings` — PASS.
- `PATH="$HOME/.cargo/bin:$PATH" cargo fmt --all -- --check` — PASS.
- `PATH="$HOME/.cargo/bin:$PATH" cargo check -p pnt-capture --features hardware` — PASS
  (compile check of the non-operational skeleton only; no device access).
