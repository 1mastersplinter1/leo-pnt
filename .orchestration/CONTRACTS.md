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
Position covariance is rotated from ECEF into ENU at the epoch position. Horizontal
accuracy is 2-D DRMS, `sqrt(P_ENU[E,E] + P_ENU[N,N])`; because the complete rotation is
applied, ECEF cross-covariances contribute. Vertical accuracy is the ENU up-axis one-sigma,
`sqrt(P_ENU[U,U])`. Speed accuracy applies the same 2-D DRMS convention to the ECEF
velocity covariance. `horizontal_velocity_ned_mps` is the north/east projection of ECEF
velocity at the epoch position, and the speed-through-water model is its horizontal norm.

The primary clock-bias state is retained for future pseudorange/STL observability and to
preserve the baseline's required state surface. Doppler currently observes drift, not bias,
so propagation applies the standard two-state integrated drift-noise covariance and caps
clock-bias variance at `1e8 m^2` for the primary and registered receiver clocks. That cap and
the process-noise coefficient are engineering bounds pending replay tuning `[UNVERIFIED]`.

GNSS aiding uses six sequential scalar, one-degree-of-freedom updates (three ECEF position,
then three ECEF velocity). A supplied chi-square threshold is therefore a per-component
1-DOF NIS threshold, not a joint 6-DOF gate. Acceptance or rejection is independent for each
component. Callers requiring a joint gate must perform it before invoking this API.

In GNSS-denied operation, ECEF vertical velocity is only weakly observable through the
local-MSL and horizontal-speed constraints. U-M1 must publish `vd = 0` with a consistent
nonzero vertical-accuracy bound as required by the baseline; stronger vertical dynamics and
noise tuning remain `[UNVERIFIED]`.

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

## v4 (2026-07-22)

v4 retains v1--v3 and resolves D15 and D22 integration seams.

### Doppler observable and clock convention

`DopplerRangeRateUpdate::predicted_range_rate_mps` is exclusively the pure geometric
satellite-to-receiver `d(range)/dt` produced by `pnt-predictor`; it contains no receiver
clock term and no satellite nuisance term. The predictor may compute correlation frequency
using a clock argument for tracker-facing uses, but its `Prediction::range_rate_mps` remains
geometric. The estimator observation model supplies all receiver-clock terms through H·x:
the primary path uses core clock-drift slot 8, while `update_doppler_for_receiver` remaps
that coefficient to the registered independent receiver's drift slot. Both add the
pass-scoped satellite nuisance in H·x. Measured correlation offset is converted to range
rate as `-correlation_peak_hz * c / nominal_carrier_hz`.

Orbcomm remains rejected and journalled at executive ingress. Having a receiver-specific
estimator API alone does not lift D10: allocation and a verified source-to-clock mapping
must be explicitly configured before fusion is enabled.

### Routing and constellation gate

`ArmCommand` is journalled and routed only to the authority/integrity port. It never enters
the estimator update surface and receipt alone never grants authority. Configuration has
`oneweb_enabled: bool`, defaulting to `false`; false causes every OneWeb tracker observation
to become a journalled integrity rejection. GNSS disabled by `off`, stale/missing
ephemeris, prediction/elevation failures, chi-square failures, Orbcomm, and all other
policy rejects are likewise journalled and never silently dropped.

### Solution-epoch NDJSON output

The executive owns a line-oriented output seam. Each line is one finite-number JSON object
with exactly the bridge input shape below (field names and nesting are wire normative):

```json
{"monotonic_ns":123456789000,"state":{"position_ecef_m":[-4479000.0,2670000.0,-3660000.0],"horizontal_velocity_ned_mps":[1.0,0.0],"heading_rad":0.0,"receiver_clock_bias_m":0.0,"receiver_clock_drift_mps":0.0},"steering_authorised":true,"horiz_accuracy_m":0.8,"speed_accuracy_mps":0.1,"vert_accuracy_m":1.5,"msl_alt_m":584.0}
```

The three accuracy values use the v3 definitions and are materialised when the epoch is
constructed. `heading_rad` may be JSON null only when unavailable. `msl_alt_m` is the
separate MSL-constrained altitude, not ECEF ellipsoid height. Until that estimator surface
is connected, the executive's zero MSL value is `[UNVERIFIED]` and must not be treated as a
validated publication value.

## v4.1 (2026-07-22)

v4.1 amends, but does not replace, v4. For every `TrackerDoppler` envelope, `source_id`
is the satellite's decimal NORAD catalogue ID string and `utc` is required with a valid
RFC3339 timestamp used for ephemeris propagation. Missing or malformed values are
journalled rejects.

The production Doppler pipeline applies a 5-degree elevation mask, converted to radians
at its degrees-safe API boundary. The value is `[UNVERIFIED]` pending link-budget and
replay tuning; callers may explicitly disable the mask for geometry-independent tests.

Correction to v4 wording: horizontal, speed, and vertical accuracy values are derived by
the `SolutionEpoch` accessors at emission, not materialised when the epoch is constructed.
An epoch containing any non-finite state, covariance, or derived accuracy value is not
written to NDJSON and produces a journalled integrity event instead.

## v5 (2026-07-23)

v5 retains v1--v4.1 and replaces the fail-open integrity placeholder with the authority
supervisor contract from `SAFETY_CASE.md` §1--§3.

`AuthorityParams` carries separate aided and denied `ProtectionLimits` for horizontal
position (m), horizontal velocity (m/s), and heading (rad), plus `Option<f64>` fields for
`t_lease`, `t_dr`, `t_eph`, `dwell_clear`, `dwell_rearm`, `caution_enter`,
`caution_clear`, `revoke_threshold`, and `T_ack` (seconds for all time values). Every
numeric must be present, finite, and non-negative. If any field is `None` or invalid, the
supervisor can never grant steering authority. This is the literal D17 fail-closed gate;
there are no numeric defaults.

The supervisor consumes clock-service monotonic nanoseconds only, `ArmCommand`, solution
sequence and covariance-derived horizontal-position/horizontal-velocity/heading accuracies,
active profile, last-absolute-observation time, ephemeris age/integrity, and calibration ID.
Calibration identity is checked by an injected validator; absence is always rejection and
the default validator accepts any present, non-empty ID. A renewing frame is exactly
`sequence advanced && G2 && G3`, independent of current lease state. Deadline equality is
expiry. Arm defaults false, disarm withdraws G1, and every fault revocation clears the arm
latch so a fresh arm command is required.

Outputs are `AuthorityOutput { steering_authorised, state, alarm, transition }`. `transition`
identifies only supervisor state transitions. The output type deliberately has no vehicle
mode, manoeuvre, RTL, Loiter, or disarm command. The six states, eleven events, guarded arm
cells, destination-state authority/annunciation, simultaneous priority, dwells, ACK timeout,
and latched recovery are exactly the total matrix in `SAFETY_CASE.md` §2.2.

The executive routes `ArmCommand` only to integrity and supplies every emitted solution to
the supervisor before deriving `SolutionEpoch.steering_authorised`. Its production-default
skeleton constructor installs the real supervisor with an incomplete parameter register and
is therefore fail-closed. `test_stub` remains explicitly stub-backed for focused legacy
executive tests.

## v5.1 (2026-07-23)

v5.1 amends, but does not replace, v5. `AckCommand` is the helm acknowledgement sibling of
`ArmCommand` on the measurement bus. It carries clock-service-stamped
`host_monotonic_ns: u64` and `source_id: SourceId`, is journalled, and is routed by the
executive only to `IntegrityAuthorityGate::acknowledge`; it never enters the estimator.
Acknowledgement changes annunciation/latch state only as specified by the §2.2 matrix and
does not itself restore steering authority.

Every accepted IMU propagation emits a DR-fill solution epoch through both
`IntegrityAuthorityGate::solution` and the NDJSON seam. This intentionally uses the IMU
propagation cadence rather than an unverified decimator: lease renewal remains conditional
on sequence advance plus G2/G3, while absolute-observation age independently revokes at
`t_dr`. A grant-capable supervisor can be constructed only with complete parameters and an
explicit calibration validator; the incomplete-register constructor is a hard error if
given complete parameters.
