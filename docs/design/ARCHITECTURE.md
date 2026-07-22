# Architecture

Status: subordinate to [`DESIGN_BASELINE.md`](DESIGN_BASELINE.md)
Contract: v1 (2026-07-22)

`DESIGN_BASELINE.md` is the single normative design document. This architecture implements
it and is subordinate wherever the two differ.

## Runtime shape and module boundaries

The **fusion executive** is the first runnable vertical slice and the only orchestrator.
It owns the main event loop and connects these bounded modules through typed interfaces:

1. **Configuration and authority** parses configuration, rejects an unknown
   `gnss_authority`, and constructs the single common processing graph. It applies GNSS
   routing before any fusion update.
2. **Clock/time service** owns runtime time (defined below), timestamps ingress, estimates
   UTC mapping and exposes monotonic deadlines.
3. **Sensor adapters** convert IMU, magnetometer, speed-log and GNSS device records into
   bus measurements without estimation policy. The Orbcomm adapter consumes a separate,
   non-coherent low-cost receiver and does not consume a bladeRF coherent RX channel.
4. **SDR capture** configures coherent channels and records raw IQ plus capture metadata.
   It does not decide whether a signal is navigation-safe.
5. **Signal trackers** turn IQ into Doppler/range-rate candidates with uncertainty and
   quality metadata. Starlink uses beacon/PSS-SSS correlation; OneWeb remains disabled
   until its required survey gate passes. Constellation trackers share an observation
   interface, not hidden state. Their estimator-facing observable is correlation-peak
   Doppler, never an unspecified raw carrier estimate.
6. **Ephemeris store and propagator** validates SupGP identity and age, then produces
   satellite position and velocity at observation time.
7. **Doppler predictor** combines propagated satellite state, receiver state and clock
   model to predict a range-rate/Doppler observation and its linearisation.
8. **Observation integrity gate** compares a tracker observation with a real predictor
   output, recording accept/reject and rationale. It cannot exist as a functional gate
   before modules 6 and 7.
9. **Estimator** propagates on every IMU event and applies only accepted, implemented
   updates. Its state registry requires a direct or cross-covariance measurement path for
   every declared state. It creates one per-satellite, per-pass transmit-frequency nuisance
   bias with a small validated random walk and retires it when the pass ends. It also applies
   the MSL sea-surface altitude pseudo-measurement; water current is derived downstream and
   is not a baseline estimator state.
10. **Solution integrity monitor** derives protection/uncertainty metrics independently
    of ArduPilot's censored accuracy view.
11. **Authority supervisor** grants or revokes steering authority from solution health,
    freshness and uncertainty. Its monotonic watchdog is independent of estimator
    convergence; expiry cannot stop the estimator. It raises and escalates alarms but does
    not select RTL, Loiter or disarm.
12. **MAVLink publisher** emits approximately 5 Hz `GPS_INPUT`, with independent horizontal
    and speed accuracy, using propagated fill between absolute updates. It publishes fused
    heading in `yaw`, the MSL-constrained altitude in `alt`, vertical velocity `vd = 0`, and
    a `vert_accuracy` consistent with the sea-surface constraint; it does not publish
    `ODOMETRY` as the navigation injection.
13. **Journal/replay service** writes measurements, truth and decisions and can reproduce
    the same bus stream from disk. Offline evaluation consumes journals; it does not feed
    results back into online rejection.

No leaf module communicates directly with another leaf module. It publishes to or is
called by the executive through explicit ports, which makes the connected running path the
unit of integration rather than a collection of independently tested parts.

## Measurement bus

The measurement bus is an in-process, bounded, typed event stream owned and drained by the
fusion executive. Producers submit immutable envelopes; one executive orders, journals and
dispatches them. Backpressure and overflow are explicit health events, never silent drops.

Every envelope contains: schema version, source ID, sequence number, measurement kind,
source/sample time, host receive monotonic time, optional UTC time plus its uncertainty,
payload SI units and frame identifier, covariance/uncertainty, quality flags, calibration
ID and provenance linking it to a capture or source record. The calibration ID shall resolve
the surveyed antenna phase-centre, IMU and vessel-reference extrinsics; Doppler and inertial
updates compensate rotational lever-arm velocity, and a missing/mismatched ID is an
authority-blocking integrity event. Exact field encoding and frame
enumeration are v2 contract work; the concepts are binding here.

Ordering uses host monotonic receive time with per-source sequence checks. Measurement time
is retained for delayed updates. Clock resets, sequence gaps, lateness and queue overflow
become journalled integrity events. The bus carries at least IMU, heading, speed-through-
water, GNSS, tracker Doppler, ephemeris state, predicted Doppler, gate decision, estimator
solution, integrity status, authority status and output status messages.

GNSS routing is enforced at bus ingress: `production` permits a fusion copy and truth copy;
`recorded_only` permits only the truth-journal route; `off` permits neither. There is no
downstream switch that can accidentally re-authorise it.

## Time ownership

The **clock/time service is the sole owner of time**. Device adapters may report device
counters or UTC labels, but may not define ordering, freshness or deadlines. At process
start the service establishes a host-monotonic epoch; all runtime ordering, stale-data
checks, authority leases and watchdogs use that monotonic domain. It also maintains an
explicit, uncertainty-bearing mapping between device time, host monotonic time and UTC for
ephemeris evaluation and logs. The estimator owns receiver clock bias and drift **states**,
not system time. GNSS UTC, when present, is a measurement and never silently disciplines
the SDR reference or runtime clock.

All persistent timestamps include monotonic nanoseconds from the run epoch and, when
available, UTC as an absolute RFC 3339 timestamp. The clock model and its uncertainty are
journalled so replay can reconstruct event timing. Nanosecond representation and RFC 3339
encoding are architecture choices (**estimates**) pending v2 schema review.

## On-disk formats

All formats are append-only and versioned. A run directory has a manifest containing run
UUID, schema versions, absolute UTC creation time when available, monotonic epoch metadata,
configuration hash, calibration IDs, software revision, hardware/channel setup, ephemeris
snapshot identity and file hashes. Atomic segment finalisation and recovery after an
unclean stop are required; specific container encodings are `[UNVERIFIED]` pending v2.

### Raw IQ capture

IQ is stored in independently recoverable, fixed-duration segments. Each segment contains
interleaved coherent RX-channel samples without lossy compression plus a sidecar/header
with sample representation, endianness, sample rate, centre frequency, bandwidth, gain,
channel allocation, external-reference status, first-sample monotonic and optional UTC
time, sample count, overrun/gap records and calibration/configuration identity. Raw ADC
packing versus normalised signed-integer representation is `[UNVERIFIED]`.

### Measurement journal

The measurement journal is a length-delimited binary stream of the exact versioned bus
envelopes presented to and produced by the executive. Records carry checksums; segments
carry indexes by monotonic time, source and message kind. It includes raw sensor records,
tracker observations, ephemeris products, predictions, accept/reject decisions,
propagations, updates, solutions, integrity and authority transitions, alarms and MAVLink
publication outcomes, including derived horizontal current vectors and their propagated
covariance. A replay reader shall preserve values and event ordering and shall
reject unknown required schema versions rather than guess. The binary codec is
`[UNVERIFIED]`.

### Truth journal

The truth journal is physically and logically separate from the measurement journal. It
uses a versioned, length-delimited record model with source ID, source/sample time, host
monotonic receipt time, optional UTC, position/velocity/heading truth payload, covariance,
quality and provenance. In `recorded_only`, GNSS can write here but there is no truth-to-bus
fusion edge. Permissions/API boundaries shall prevent the online estimator and reject gate
from reading it. Offline evaluation aligns truth after the run and computes aided versus
withheld results. Codec and access-control mechanism are `[UNVERIFIED]`.

## Build and verification order

Development proceeds as connected vertical slices:

1. Build a minimal fusion executive with clock service, bounded bus, journals/replay, one
   IMU propagation path, solution output and observable end-to-end integration test.
2. Add configuration/authority routing and prove that `recorded_only` has no fusion edge
   and unknown values fail.
3. Add each marine sensor as adapter, journal record and actual estimator update; do not add
   a filter state before its update path exists.
4. Add ephemeris store/propagator, then Doppler predictor, with numeric finite-difference
   checks for applicable Jacobians.
5. Add capture and one tracker path through replay into predictor and estimator.
6. Only now add the observation reject gate, because it has a real prediction producer.
7. Add integrity, independent monotonic authority watchdog and 5 Hz MAVLink publisher;
   verify that authority expiry never stops estimator execution.
8. Expand constellations and receiver allocation only behind their evidence gates.
9. Prove identical captures through paired aided and GNSS-withheld replay, then validate
   MAVLink against pinned ArduPilot SITL with commit hash and artifact checksum.

Every slice must demonstrate that the running estimator propagates from IMU input even
when no measurements update it. Unit success without an executable end-to-end path is not
completion.
