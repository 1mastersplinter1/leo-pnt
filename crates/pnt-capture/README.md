# pnt-capture

`pnt-capture` is the IQ-source boundary between capture/replay and `pnt-tracker`. Its
default build has no SDR library or hardware dependency. `FileIqSource` reads a binary
segment plus `<segment>.json`, preserves coherent channel alignment, timestamps each block,
and surfaces gaps/overruns. `write_capture` provides the inverse path used by synthetic
end-to-end tests.

## File and metadata contract

Format version 1 stores little-endian IEEE-754 complex `f32`, interleaved by sample then
channel then I/Q. This is a normalized replay representation, not a claim about bladeRF ADC
packing; raw ADC packing remains **[UNVERIFIED]**. The JSON sidecar records representation,
endianness, sample rate, centre frequency, bandwidth, per-channel gain, channel allocation,
external-reference state, first-sample monotonic time and optional UTC, per-channel sample
count, gap/overrun records, calibration ID, and configuration ID.

`IqSource` yields channel-major `Complex64` blocks with block size selected by the caller.
All channels in a block cover the same sample instants. A gap is attached to the first block
after the missing samples and advances its timestamp; consumers must not silently track
across that discontinuity. Exact metadata requirements can reject a file before processing.

This is one independently replayable segment codec. Run manifests, hashing, atomic segment
finalisation, and crash recovery remain integration work; the architecture's overall
raw-IQ container choice remains **[UNVERIFIED]**.

## Live binding decision

The planned live backend uses the system **libbladeRF C API through thin Rust FFI**, not
SoapySDR. R1 found the pure-Rust `libbladerf-rs` driver does not support bladeRF 2.0, the
available Rust wrapper is thin/WIP, and libbladeRF is the production-proven API with
explicit synchronous `RX_X2`, buffer/transfer, timestamp, and clock-reference surfaces.
SoapyBladeRF is mature for interoperability, but adds a plugin layer where coherent
two-channel layout, timestamps, overrun reporting, and sustained throughput still need
target-host measurement. R1 recommends libbladeRF for production and Soapy for tools.

The `hardware` feature exposes `BladerfIqSource` only as a documented **[UNVERIFIED]**
non-operational skeleton. It has not opened a device or captured RF. Completing it requires:

- project-owned/generated bindings to the pinned target system libbladeRF;
- device selection and `RX_X2` channel/rate/frequency/bandwidth/gain/ref-in configuration;
- synchronous stream lifecycle and conversion from the verified native ADC format;
- device/host timestamp mapping into clock-service monotonic time;
- explicit timeout, short-read, overrun, USB reset, and shutdown handling;
- sustained dual-channel soak tests on the target USB controller and comparison against a
  known capture.

No hardware run is claimed.
