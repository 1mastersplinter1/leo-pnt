# CONTRACTS.md — v1

Workers state the version they built against. Changes land here FIRST, with a DECISIONS.md line.

## v1 (2026-07-22)

- Repo layout: `docs/design/` (normative docs), `docs/research/` (research outputs), `docs/HANDOFF_PROMPT_BLADERF.md` (source brief), `.orchestration/` (plan/briefs/reports), code in a Rust workspace at repo root (crates under `crates/`, defined in v2 by U-C1).
- Language: English; SI units; all timestamps UTC; absolute dates only (no "yesterday").
- Report contract: every unit writes `.orchestration/reports/<unit>.md` — what changed, evidence (commands run + output), assumptions, open uncertainties. Grok reports additionally split VERIFIED (ran/read it) vs ASSUMED.
- `gnss_authority` config key: `production | recorded_only | off`; `recorded_only` routes GNSS to the truth journal only; unrecognised value raises, never defaults. Same code path in every mode. (Fixed by handoff; restated here as binding.)
- Acceptance criteria are split into `aided` and `denied` profiles — no single position limit applies to both.
- DR timeout governs steering authority only, never estimator execution.
- To be fixed in v2 (by U-C1 from the reviewed U-D1 baseline): measurement-bus message schema, coordinate frames, on-disk formats, module-owns-time statement, rate contract.

## v2 (2026-07-22)

v2 retains every v1 rule and resolves the five v1 deferrals from the reviewed design. Where
the design explicitly leaves a physical value or codec `[UNVERIFIED]`, v2 preserves that
status instead of inventing one.

### Measurement-bus schema (resolves v1 schema deferral)

The in-process bus uses the Rust types in `pnt-types`. The v2 envelope is immutable after
ingress stamping and has this exact logical schema (Rust field types are normative):

```rust
struct MeasurementEnvelope {
    schema_version: u16,                 // exactly 2
    source_id: SourceId,                 // struct SourceId(String)
    sequence: u64,
    sample_time: TimeTag,                // DeviceNanoseconds(u64) | HostMonotonicNanoseconds(u64)
    host_receive_monotonic_ns: u64,      // overwritten by ClockService at ingress
    utc: Option<UtcTime>,                // { rfc3339: String, uncertainty_ns: u64 }
    payload: MeasurementPayload,
    frame: Frame,
    covariance: Vec<f64>,                // row-major SI covariance for payload components
    quality: QualityFlags,               // struct QualityFlags(u32); bit 0 means valid
    calibration_id: String,
    provenance: Provenance,              // CaptureRecord(String) | SourceRecord(String) |
                                         // DerivedRecord(String)
}

enum MeasurementPayload {
    Imu(ImuSample),                      // acceleration_mps2: [f64; 3], angular_rate_rps: [f64; 3]
    Heading(Heading),                    // radians: f64
    SpeedThroughWater(SpeedThroughWater),// metres_per_second: f64
    Gnss(GnssFix),                       // position_ecef_m, velocity_ned_mps: [f64; 3]
    TrackerDoppler(TrackerDoppler),      // constellation, correlation_peak_hz,
                                         // nominal_carrier_hz
}
```

`Constellation` is `Starlink | Iridium | OneWeb | Orbcomm`. OneWeb remains disabled until
its survey gate passes. Per D10, `Orbcomm` observations are rejected at executive ingress
and **must not enter fusion** until the estimator implements either a second receiver-clock
state or an explicit per-receiver clock nuisance term. The bus is bounded and executive-
drained; overflow, lateness, sequence gaps and clock resets are integrity events rather
than silent drops. Calibration IDs are mandatory for affected measurements, and missing
or mismatched extrinsics block steering authority.

### Coordinate frames (resolves v1 frame deferral)

`Frame` is the closed enumeration `EarthCenteredEarthFixed | LocalNorthEastDown |
VesselReference | Sensor | AntennaPhaseCenter | FrameIndependent`. ECEF is used for global
position; NED is the local navigation frame and orders velocity components north, east,
down; `VesselReference` identifies the surveyed vessel reference point; `Sensor` and
`AntennaPhaseCenter` identify calibrated physical origins; `FrameIndependent` is for scalar
observables such as frequency. A `calibration_id` resolves the full 3-D orientation and
lever arm between sensor/antenna and vessel reference. The reviewed design does not define
vessel-axis signs, so calibration metadata, not an assumed convention, is authoritative
until that physical convention is verified.

### On-disk formats (resolves v1 format deferral)

Every run is an append-only, schema-versioned directory. Its manifest records run UUID,
schema versions, optional absolute RFC 3339 UTC creation time, monotonic epoch metadata,
configuration hash, calibration IDs, software revision, hardware/channel setup, ephemeris
snapshot identity and file hashes. Segments are atomically finalised and recoverable after
an unclean stop.

- Raw IQ consists of independently recoverable fixed-duration segments of interleaved,
  coherent, lossless RX-channel samples plus sidecar/header metadata: representation,
  endianness, sample rate, centre frequency, bandwidth, gain, allocation, external-reference
  state, first-sample monotonic and optional UTC time, sample count, gaps/overruns and
  calibration/configuration identity. ADC packing remains `[UNVERIFIED]`.
- The measurement journal is a checksummed, length-delimited binary stream of exact v2 bus
  envelopes, segmented and indexed by monotonic time, source and message kind. It includes
  inputs, decisions, propagation/update records, solutions, integrity/authority events,
  alarms and publication outcomes. Unknown required schema versions are hard errors.
- The truth journal is a physically and logically separate versioned, length-delimited
  record stream containing source ID, sample time, host receipt time, optional UTC,
  position/velocity/heading truth, covariance, quality and provenance. Online estimator
  and rejection-gate APIs have no read edge to it.

The reviewed design leaves all three binary container codecs `[UNVERIFIED]`; v2 therefore
fixes record boundaries, required metadata, separation, version/error behaviour and recovery
semantics, but deliberately does not name a codec.

### Time ownership (resolves v1 ownership deferral)

The clock/time service is the sole owner of runtime time. It establishes the process
monotonic epoch and alone stamps ingress and defines ordering, freshness, deadlines,
authority leases and watchdogs. Adapters may supply device counters or UTC labels only as
measurements. The service maintains and journals an uncertainty-bearing device/monotonic/UTC
mapping. Persistent records carry nanoseconds from the run monotonic epoch and optional
absolute RFC 3339 UTC. Estimator receiver-clock bias/drift are filter states, not system
time, and GNSS must never silently discipline runtime time or the SDR reference.

### Rate contract (resolves v1 rate deferral)

| Interface | v2 contract |
|---|---|
| SDR processing bandwidth | Tracker-selected 2.5--5 MHz per active correlation channel; sample rate `[UNVERIFIED]` |
| IMU to executive | 100 Hz nominal (**estimate**); every accepted sample propagates state and covariance |
| Each magnetometer | 10 Hz nominal (**estimate**) |
| Speed log | 5 Hz nominal (**estimate**) |
| LEO Doppler | Event-driven per valid correlation result; target 1 Hz per signal (**estimate**); never synthesize observations to meet a rate |
| GNSS input | Native rate, at least 1 Hz (**estimate**), routed at ingress by `gnss_authority` |
| Navigation/integrity solution | 5 Hz nominal (**estimate**) with propagated fill |
| MAVLink `GPS_INPUT` | 5 Hz nominal while healthy and subject to steering-authority gating |
| Journals | Every input, decision and output at native rate; batching only with configured crash-loss bounds (**estimate**) |

Measurement arrival is never the propagation trigger: accepted IMU ticks are. A dead-
reckoning timeout affects steering authority only and must not stop propagation, updates,
journalling or convergence.

## v3 (2026-07-22)

v3 retains v1 and v2 and resolves D10, D13, and review findings F3/F4.

### Estimator state and epoch uncertainty

`FilterState` carries ECEF position and velocity, heading, primary receiver-clock bias and
drift, and the estimator's complete row-major covariance together with its dimension. The
nine core slots are ordered position ECEF (0--2), velocity ECEF (3--5), heading (6), clock
bias metres (7), and clock drift metres/second (8). Dynamically registered states follow.
`SolutionEpoch` exposes horizontal-position, horizontal-speed, and vertical one-sigma
accuracies in SI units, derived from that epoch's covariance. For source compatibility with
the v2 executive these are accessors rather than additional stored fields; U-I2 shall move
epoch creation to a constructor before any wire schema represents them as stored fields.

### Helm arm command (resolves D13)

`ArmCommand` is a measurement-bus payload with `action: Arm | Disarm`, the clock-service
stamped `host_monotonic_ns: u64`, and `source_id: SourceId`. Receipt is not itself an
authority grant: the executive must journal and route it to the authority supervisor, which
applies freshness, source, health, and revocation policy. U-I2 owns that routing.

### Independent receiver-clock registry (resolves D10)

The estimator owns a registry from opaque `ReceiverClockId` to `ReceiverClockSlot {
bias_index, drift_index }`. Reserving a receiver dynamically augments the state and full
covariance with bias (metres) and drift (metres/second); propagation couples its bias to its
drift, and receiver-specific Doppler updates linearise against that slot instead of the
primary clock. Retirement/reindexing must preserve registry validity. Orbcomm remains
rejected at ingress until U-I2 explicitly provisions its receiver clock and routes accepted
predictor output through this receiver-specific update path.
