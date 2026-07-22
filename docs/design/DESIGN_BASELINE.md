# Design Baseline

Status: **normative**
Contract: v1 (2026-07-22)

This is the single normative design document for the LEO signals-of-opportunity maritime
PNT system. Every other project document, including `ARCHITECTURE.md`, is subordinate to
this baseline. If another document conflicts with it, this document governs until this
baseline is explicitly revised; superseded documents must be marked in their own text.

## Mission and operating assumptions

The system shall estimate position, velocity, heading, receiver clock bias and receiver
clock drift on a moving, **manned displacement-hull vessel**, without GNSS, and provide a
navigation solution to ArduPilot Rover in boat frame (`FRAME_CLASS=2`). A qualified person
shall remain aboard with a physical, controller-independent manual override.

The intended operating area is assumed to be the Danish straits. This is a **working
assumption**, not a verified deployment decision. The displacement-hull assumption is also
load-bearing: if the vessel planes or slams, the frequency-reference mounting and dynamics
analysis shall be revisited.

The system is both a research instrument and an operational navigator. It shall resolve
that tension with one executable code path and one configuration key, `gnss_authority`:

- `production`: GNSS measurements may enter fusion and are also journalled as truth.
- `recorded_only`: GNSS measurements go only to the truth journal and never enter fusion.
- `off`: GNSS is neither fused nor recorded as truth.
- Any other value shall raise an error; it shall never fall back to `production`.

Mode selection changes measurement authority, not implementation. The **aided** acceptance
profile applies to `production`; the **denied** profile applies to `recorded_only` and
`off`. Headline LEO performance statistics shall be computed offline from GNSS-withheld
runs, never from a state estimate driven by GNSS.

## Vessel equipment and sensor baseline

The baseline sensor and interface set is:

| Source | Required role | Baseline |
|---|---|---|
| Nuand bladeRF 2.0 micro xA4 or xA9 | Coherent RF sampling | Two coherent RX channels on one external reference. RX allocation is survey-dependent. |
| Ku LNB and antenna | Starlink and conditional OneWeb reception | 10.7--12.75 GHz downconverted to approximately 950--2150 MHz IF. Track beacon/PSS-SSS correlation Doppler at 2.5--5 MHz bandwidth; do not depend on the degraded tone comb. |
| L-band antenna | Iridium reception | Direct 1616--1626.5 MHz reception, bypassing the Ku LNB. |
| 137 MHz antenna/front end and independent low-cost receiver | Orbcomm reception | A separate, non-coherent receiver provides continuous constellation/front-end diversity without consuming either bladeRF coherent RX channel; exact receiver is a BOM dependency `[UNVERIFIED]`. |
| Free-running rubidium (FE-5680A class) or good OCXO | SDR frequency reference | 10 MHz external reference, calibrated only before deployment; no GNSS discipline. Mount its resultant acceleration-sensitivity vector vertically under the displacement-hull assumption. |
| IMU | Propagation and turn dynamics | Drives every estimator propagation; it does not carry passage-scale position unaided. |
| Two calibrated magnetometers | Heading measurement and redundancy | Calibration includes a propulsion-current deviation term. |
| Speed log | Speed through water | Compared with LEO-derived speed over ground to estimate current set and drift. |
| GNSS receiver | Aiding or truth, by explicit authority | Its path is controlled only by `gnss_authority`. |
| Optional solar/sky-polarisation compass | Independent absolute heading | Candidate mitigation for magnetic heading weakness; not required until selected and verified. |
| Companion computer monotonic clock | Runtime ordering and watchdogs | Time ownership is defined in the subordinate architecture. |
| ArduPilot Rover autopilot | Navigation consumer | Receives MAVLink `GPS_INPUT` (message 232, `GPS1_TYPE=14`). |

OneWeb tracker implementation is forbidden until a 24-hour channel-occupancy survey at the
actual operating location demonstrates useful correlation gain. The survey shall decide
whether the second coherent receiver is Ku + L-band (diverse simultaneous tracking) or
dual Ku (alternating OneWeb channel coverage, with Iridium time-shared). Interferometric
use remains an open research option and is not baseline functionality.

Orbcomm shall remain simultaneous through its separate receiver in either bladeRF allocation.
Its samples are not coherent with the bladeRF channels; this is intentional because it
preserves both bladeRF channels and decorrelates receiver-clock and front-end failure modes.

Ephemerides shall be cached CelesTrak supplemental ephemerides (SupGP), not plain TLEs,
with an explicit age gate. The provisional maximum age shall be 6 hours, the first supplied
error-growth datum (approximately 0.94 km orbit error), rather than one day or seven days;
the mapping from orbit error to navigation integrity remains `[UNVERIFIED]`, so this limit
shall be validated or tightened before steering trials.

## Rate contract

All rates below are interface contracts. Values absent from the handoff are deliberately
marked **estimate** and shall be validated against sensor capability and replay data.

| Interface | Required rate/behaviour |
|---|---|
| SDR sample stream | Tracker-selected 2.5--5 MHz processing bandwidth per active correlation channel (stated expectation; sample rate itself remains `[UNVERIFIED]`). |
| IMU to fusion executive | 100 Hz nominal (**estimate**), with every accepted sample causing time propagation; measurement arrivals shall never be the sole trigger for propagation. |
| Dual magnetometers to fusion executive | 10 Hz each nominal (**estimate**). |
| Speed log to fusion executive | 5 Hz nominal (**estimate**). |
| LEO Doppler observations | Event-driven at each valid correlation solution; target 1 Hz per tracked signal (**estimate**). No synthetic observation is emitted to satisfy a rate. |
| GNSS measurement/truth input | Native receiver rate, at least 1 Hz (**estimate**); routing is determined before fusion by `gnss_authority`. |
| Navigation/integrity solution | 5 Hz nominal (**estimate**), using propagated dead-reckoned fill between absolute observations. |
| MAVLink `GPS_INPUT` output | 5 Hz nominal (handoff requirement: approximately 5 Hz), continuously while the process is healthy, subject to steering-authority gating. |
| Journal writes | Every input, decision and output event, preserving native rate; batched disk flush is allowed only if crash-loss bounds are explicitly configured (**estimate**). |

The four-second `GPS_TIMEOUT_MS` motivates continuous output but does not define estimator
validity. Any dead-reckoning timeout shall govern **steering authority only**. It shall never
stop propagation, observation processing, journalling, or the 10--20 minute position
convergence process.

## Estimator and degradation contract

LEO Doppler shall be treated primarily as a velocity/range-rate observation. Position is
recovered through the evolution of line-of-sight geometry over 10--20 minute,
constant-heading legs, and manoeuvres reset convergence expectations. The estimator shall
carry only states with implemented measurement paths. At minimum these are position,
horizontal velocity, attitude/heading, receiver clock bias and receiver clock drift;
position is 3-D with the vertical component constrained as specified below, and vertical
velocity is not a baseline state. Additional
states may be enabled only with a documented direct or cross-covariance measurement path;
this permits observable IMU bias states but forbids decorative unobservable states.

The normative LEO observable at the estimator interface is **correlation-peak Doppler**
from the beacon/PSS-SSS tracker, converted to range rate with its stated nominal carrier;
raw carrier-frequency Doppler is not the interface observable. Each satellite pass shall
have a separate transmit-frequency nuisance-bias state, constant over the pass with a
validated small random-walk model, so a satellite oscillator/carrier offset cannot be
misinterpreted as receiver velocity or clock drift. The state shall be retired at pass end.

The online estimator shall not carry a water-current state in the baseline. The solution
module shall derive and journal the horizontal current vector as LEO-derived velocity over
ground minus the heading-rotated speed-log velocity through water, with covariance
propagated from both inputs. This makes current observable without enlarging the core state;
a future first-class current state requires an explicit update model and baseline revision.

Altitude shall be a vertical position state constrained to local mean sea level by a
sea-surface pseudo-measurement. Its variance shall cover chart/geoid uncertainty plus local
tide, wave and vessel-motion scale; the numeric model is `[UNVERIFIED]` and shall be frozen
for the operating area before steering trials. Vertical velocity is excluded from steering
use: `GPS_INPUT.alt` shall publish the MSL-constrained estimate, `vd` shall be 0, and
`vert_accuracy` shall report a bound consistent with that pseudo-measurement rather than 0
or an unconstrained filter covariance.

The antenna phase centres, IMU origin and vessel reference point, including their full
3-D lever arms and orientations, shall be surveyed calibration inputs. Doppler and inertial
updates shall compensate rotational lever-arm velocity before fusion. Every affected
measurement envelope shall reference the calibration ID for these extrinsics; missing or
mismatched calibration is an integrity fault that forbids steering authority.

Expected degradation is explicit:

| Loss or condition | What remains / required response |
|---|---|
| GNSS absent or forbidden | Normal denied operation: IMU propagation, measured heading, speed-through-water and accepted LEO Doppler remain. GNSS cannot quietly re-enter fusion. |
| One LEO constellation or RF front end lost | Continue with remaining accepted constellations and inertial/marine sensors; geometry and uncertainty shall degrade honestly. Loss of Ku shall not remove independently received Orbcomm, and loss of the LNB shall not remove direct L-band reception when the bladeRF allocation includes it. |
| All LEO observations lost | Continue IMU propagation plus magnetometer and speed-log updates; covariance grows and current/ground-speed observability degrades. Steering authority expires by the independently enforced authority policy, but the estimator keeps running. |
| One magnetometer lost/rejected | Continue with the other calibrated magnetometer, IMU turn dynamics and any selected non-magnetic heading sensor; inflate heading uncertainty. |
| Both magnetometers lost/rejected | Continue short-term attitude propagation and any selected non-magnetic heading measurement. Heading is not passage-capable by inertial integration alone; revoke authority when its bound is exceeded. |
| Speed log lost | LEO ground velocity and heading/IMU remain; direct water-current separation is unavailable and uncertainty shall reflect it. |
| IMU stream stale | Time propagation is no longer trustworthy. The supervisor shall revoke steering authority; the estimator shall remain running but shall not fabricate propagation samples, and journalling and recovery may continue. |
| Frequency reference degrades or sustained heel invalidates its model | Clock bias/drift states may absorb only modelled behaviour. Reject biased Doppler as integrity dictates and revoke authority when solution limits are exceeded. |
| Sea-surface multipath biases a tracker | Correlation quality and innovation consistency shall drive rejection or uncertainty inflation; persistent constellation-correlated residuals revoke authority when protection limits are exceeded. |
| Ephemeris missing or too old | Do not form or accept the affected Doppler prediction. Continue with other measurements; never bypass the age gate. |
| Companion process stalls | Its independent monotonic watchdog shall make steering authority expire. Alarm escalation and the physical override remain available; software shall not autonomously select RTL, Loiter or disarm. |

## Acceptance profiles

These are system-level trial limits, evaluated against an independent truth journal over
declared test segments. They do not replace per-epoch integrity and authority checks.
Velocity means horizontal per-axis error; position means horizontal error. Exact percentile,
window and confidence definitions remain `[UNVERIFIED]` and must be frozen before trials.

| Criterion | Aided (`production`) | Denied (`recorded_only` or `off`) |
|---|---:|---:|
| Horizontal position error | <= 25 m (**estimate**, operational-grade target inferred from the handoff's example of a correct operational limit) | <= 200 m after an uninterrupted 20-minute constant-heading convergence leg (**estimate**, conservative edge of the stated approximately 100--200 m expectation) |
| Horizontal velocity error, each axis | <= 0.02 m/s (**estimate**, operational target within the handoff's 0.007--0.04 m/s expectation) | <= 0.04 m/s (**estimate**, upper edge of the stated 0.7--4 cm/s expectation) |
| Heading error | <= 2 degrees (**estimate**) | <= 5 degrees (**estimate**) |
| Horizontal current-vector error | Recorded and scored, but no pass/fail threshold until local truth instrumentation and current variability are characterised (`[UNVERIFIED]`) | Same; omission of a threshold does not make publication optional |
| Output continuity | Valid `GPS_INPUT` at nominal 5 Hz while healthy; no gap >= 4 s (**estimate** derived from the stated timeout) | Same; dead-reckoned fill is explicitly permitted, with reported uncertainty and authority enforced upstream |
| GNSS isolation | GNSS fusion is allowed and auditable | Zero GNSS measurements delivered to fusion; `recorded_only` may write truth only |
| Replay determinism | Same raw log, config, software and deterministic execution settings produces bit-exact journal outputs; explicitly non-deterministic backends require a frozen numeric tolerance `[UNVERIFIED]` | Same, including a paired GNSS-withheld replay used for headline results |

The aided numbers are engineering estimates, not literature-derived guarantees. Here,
“operational-grade” means satisfying the estimated 25 m position, 0.02 m/s per-axis
velocity, and 2 degree heading trial targets while integrity permits steering. It does not
mean certified safety performance. The denied limit intentionally does not inherit the
aided 25 m position target, which would make the research result fail by construction.
The 2/5 degree heading gates are provisional engineering targets chosen to bound transverse
velocity error from heading-rotated speed-log measurements while allowing degraded magnetic
operation; the handoff supplies no validated heading limit, so both require trial validation
and are `[UNVERIFIED]`.

The real horizontal and speed uncertainty monitor and every steering gate shall live in
the companion process upstream of MAVLink, because the handoff states that ArduPilot
internally clamps reported horizontal accuracy at 100 m. Report `horiz_accuracy` and
`speed_accuracy` independently in `GPS_INPUT`. Publish fused heading through the
`GPS_INPUT.yaw` field with its validity governed by the companion integrity gate. `ODOMETRY`
shall not be used for this navigation injection because its ArduPilot path cannot transport
the required per-epoch velocity uncertainty.

## Verification obligations

Before steering trials, tests shall demonstrate end-to-end execution from real-capture
replay through propagation, measurement updates, integrity, authority decision and
`GPS_INPUT`. Jacobians shall be checked against finite differences. The same raw log shall
be replayed aided and GNSS-withheld. MAVLink integration shall be checked in ArduPilot SITL
against a pinned firmware commit and artifact checksum. Measurements, inferences and
estimates shall remain distinguishable in resulting evidence.
