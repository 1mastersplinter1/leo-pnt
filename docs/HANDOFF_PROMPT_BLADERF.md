# Handoff prompt — GPS-denied maritime PNT from LEO signals of opportunity (bladeRF)



---

You are designing and building a **GPS-denied maritime positioning system** that navigates
from **LEO satellite signals of opportunity** (Starlink, OneWeb, Iridium, Orbcomm), using
**Nuand bladeRF 2.0 micro** SDRs. Work to production engineering standards: this steers a
boat.

## Mission

Measure position, velocity and heading on a moving vessel **without GNSS**, from Doppler
(range-rate) observations of LEO downlinks, and fuse them with inertial, speed-log and
heading sensors into a navigation solution good enough to feed an autopilot.

The honest framing: this is simultaneously a **research instrument** (can LEO SoOP navigate
a boat without GNSS, and how well?) and an **operational navigator**. Those two goals fight
each other. Resolve that tension explicitly and early — see "Failure modes" below.

## Fixed constraints

- **SDR: Nuand bladeRF 2.0 micro (xA4 or xA9).** Verify current specs, price and EU
  availability yourself — do not trust my numbers. The properties that matter here:
  - **2×2 coherent MIMO.** Two RX channels on a shared clock. This is the reason to pick
    this radio: RX0 can take the Ku IF (Starlink/OneWeb behind an LNB) while RX1 takes
    L-band (Iridium 1616–1626.5 MHz) **coherently**, eliminating the multi-dongle,
    multi-clock mess that plagues cheaper builds. Exploit this deliberately.
  - **Native 10 MHz external reference input.** Critical — see the clock constraint below.
  - 47 MHz–6 GHz tuning, 12-bit ADC, up to ~61.44 MSPS, USB 3.0, Cyclone V FPGA.
  - Ku band (10.7–12.75 GHz) still requires an LNB to downconvert to the ~950–2150 MHz
    L-band IF. That IF is comfortably inside bladeRF range.
- **Frequency reference: free-running, no GNSS discipline.** A GPSDO defeats the entire
  premise. Use a surplus rubidium (FE-5680A class) or a good OCXO, calibrated
  pre-deployment only. The position solver must carry receiver clock bias and drift as
  estimated states.
- **The vessel is manned.** A qualified person is aboard with a physical manual override.
  This is not a detail — it dissolves the COLREGs Rule 5 lookout obligation that makes an
  unmanned build legally intractable, and it is what makes sea trials possible at all.
- **Autopilot: ArduPilot Rover, boat frame** (`FRAME_CLASS=2`). PX4 has no boat frame and
  its EKF2 external-vision path demands 30–50 Hz or it will not fuse.

## What is already known — do not rediscover these

These were established by prior work at real cost. Verify anything you intend to rely on,
but start from here rather than from zero.

**Signal layer**
- The Starlink Ku **tone comb degraded fleet-wide after 2023** (~50 → ~20 dB-Hz). Do not
  architect around tones. The tractable observable is **beacon / PSS-SSS correlation
  Doppler** at 2.5–5 MHz bandwidth. The sync sequences are published.
- Use **CelesTrak supplemental ephemerides (SupGP)**, not plain TLEs. Confirmed full-fleet
  for Starlink and OneWeb. SGP4 error grows roughly 0.94 km at 6 h, ~2.6 km at 1 day,
  38.5 km at 7 days — cache before going offline and gate on age.
- **Orbcomm (137 MHz) is the best value per dollar** in the constellation set: continuous,
  cheap to receive, deep published SoOP pedigree, and it decorrelates a front end that is
  otherwise entirely dependent on one LNB.
- **OneWeb earns its place on geometry, not on signal count — but it is conditional.**
  At high latitude Starlink's 53° shells produce mostly east–west passes, so the east error
  axis stays poorly observed however many Starlink satellites you track. OneWeb's near-polar
  87.9° orbits run within ~4° of due north–south, near-orthogonal to Starlink: published
  OneWeb-only east sigma of 636.7 m collapses to 3.5 m fused, and single-constellation 2-D
  errors of 30–35 m each fuse to ~5 m with eight satellites (a truth-aided ceiling, but the
  direction is real). Marginal hardware cost is ~zero — same Ku band, same LNB, same IF;
  active channels 11.075/11.325 GHz land at 1325/1575 MHz, and unlike a 1700 MHz-limited
  receiver the bladeRF also reaches 11.575 GHz (IF 1825 MHz).
  **The catch:** there is no demand-proof narrowband OneWeb observable. The trackable 10 ms
  beacon is the demand-dependent default-payload repetition, and beam activity at a fixed
  site pulses in roughly 20-second windows on alternating channels. Energy detection is
  near-certain; continuous trackability is not. **Gate all OneWeb tracker work behind a
  24-hour channel-occupancy survey at the actual operating location** — measure real
  correlation gain before writing a tracker.
- **Spend the second coherent RX channel deliberately, and let that survey decide it.**
  Ku + L-band (Starlink/OneWeb on RX0, Iridium on RX1) maximises constellation diversity and
  decorrelates the front end, since Iridium bypasses the LNB entirely — no shared bias tee,
  no common rain-fade mode. Dual-Ku (two OneWeb channels at once) instead defeats the
  alternating-channel pulsing and roughly doubles OneWeb's usable duty cycle. If OneWeb
  proves reliably present on one channel, take the first option; if it pulses across
  channels as the evidence suggests, take the second and time-share Iridium.

**Estimation layer — the single most important fact**
- **On a moving receiver, LEO Doppler is a velocity instrument first and a position
  instrument only by accumulation.** Receiver velocity enters range-rate at unit magnitude
  every sample; position enters only through line-of-sight *direction*, suppressed by
  1/range (550–1200 km), and is recoverable only from how the Doppler curve evolves over
  **10–20 minutes**. Every manoeuvre resets convergence. Design the test campaign around
  long constant-heading legs or you will never observe position at all.
- Expect **0.7–4 cm/s per-axis velocity** accuracy and, satellite-only, **~100–200 m
  position** — not the metre-level figures in the literature, which were all obtained with
  **GPS-disciplined** lab receivers.
- **Marine dead-reckoning error is dominated by current set and drift** (0.2–1.0 kn
  typical, 2–4 kn in Danish straits), which is 1.5–6× the heading term. A speed log
  measures speed *through water*; Doppler measures speed *over ground*; differencing them
  yields the current vector directly. This is the strongest argument for LEO Doppler on a
  boat, and it is a **velocity** argument.
- **Heading is the weakest link**, and GNSS compasses are excluded by the premise. Budget
  for it properly: calibrated dual magnetometers including a propulsion-current deviation
  term, and consider a solar/sky-polarisation compass (absolute, non-magnetic, passive).
- **Do not buy a tactical IMU expecting it to carry a passage.** Even navigation-grade
  drifts ~1 km/hour free-inertial. A boat *measures* heading rather than integrating it,
  which breaks the classic inertial divergence mode. The IMU keeps the dynamics model
  honest through turns; it does not carry position.

**Autopilot integration**
- Inject the solution as **`GPS_INPUT` (MAVLink msg 232, `GPS1_TYPE=14`) at ~5 Hz**, not
  `ODOMETRY`. Reason, verified in ArduPilot source: `handle_odometry()` hard-codes the
  velocity error argument to literal `0` and the decoded velocity covariance is floored to
  a constant, so ODOMETRY **cannot** transport per-epoch velocity uncertainty. Since your
  solution is good in velocity and poor in position, that distinction is decisive.
  `GPS_INPUT` carries `horiz_accuracy` and `speed_accuracy` independently.
- **ArduPilot clamps reported horizontal accuracy at 100 m internally.** Its EKF variance
  is therefore a censored view. The real uncertainty monitor and the steering gate must
  live in your companion process, upstream of MAVLink. This is not tunable away.
- Publish continuously with dead-reckoned fill between absolute fixes; `GPS_TIMEOUT_MS` is
  4000 ms, so fixes arriving minutes apart cannot be published raw.

**Hardware**
- The frequency reference's enemy at sea is **sustained mean tilt, not wave motion**.
  Oscillatory FM is suppressed as 1/(π·f·τ); a sustained heel is a *step* the linear drift
  model cannot absorb. Free ~11× mitigation: mount the resultant g-sensitivity vector
  vertical. If the hull planes and slams, re-derive this — the analysis above assumed a
  displacement hull.

## Required deliverables

1. **A normative design baseline** — one document, one language, stating the vessel, the
   sensors, the rate contract, the degradation model and the acceptance criteria. Everything
   else is subordinate to it and says so.
2. **A decision log** recording every load-bearing constraint change with its rationale.
3. **Architecture** — module boundaries, the measurement bus, on-disk formats, and an
   explicit statement of which module owns time.
4. **Implementation**, with the estimator actually wired end to end (see failure modes).
5. **A bill of materials** with live-verified EU pricing and availability.
6. **A safety case** — what grants steering authority, what revokes it, and what the
   backstop is when the human does not respond.

## Verification discipline

- Write tests first; loop on `cargo test`/`pytest` and the linter until both pass cleanly.
- **Verify Jacobians numerically against finite differences.** A wrong Jacobian is silent —
  the filter still runs and still produces a covariance, just the wrong one, and that
  covariance gates steering authority.
- Prove the estimator on **replayed real captures** before trusting it, and run the same
  raw log twice — once GNSS-aided, once GNSS-withheld. That delta is the headline result.
- Use **ArduPilot SITL** to validate the MAVLink path against pinned firmware, recording
  the commit hash and artifact checksum.
- Cite sources for factual claims. Mark estimates as estimates. Separate what you measured
  from what you inferred.

## Failure modes to actively avoid

These are the specific ways the prior attempt went wrong. Guard against each explicitly.

1. **GNSS quietly absorbing the project.** If a GNSS receiver is present as a production
   source, the GNSS-denied path becomes a fallback nobody validates, and the research
   question is never answered. Fix: a single `gnss_authority` config key with
   `production | recorded_only | off`, enforced in the fusion path, where `recorded_only`
   routes GNSS to the truth journal and nothing else. Same code in every mode — a research
   mode running different code does not validate the operational mode. An unrecognised
   value must raise, never default to `production`.
2. **Acceptance limits that make the research result fail by construction.** A single 25 m
   position limit is correct operationally and impossible for satellite-only navigation.
   Split acceptance into **aided** and **denied** profiles with limits drawn from the
   literature for each.
3. **A dead-reckoning timeout that forbids convergence.** A 30-second DR gate is right for
   *authority* and fatal for an estimator that needs 10–20 minutes to observe position.
   Let the gate govern authority only, never estimator execution.
4. **Unit-tested modules that are never connected.** The prior build had a filter, a bus, an
   integrity monitor and a supervisor, each with passing tests, and **nothing wired them
   together** — no fusion executive existed. Build the executive first and grow modules into
   it, not the other way round.
5. **Forgetting to propagate.** Related and worse: ensure the filter is actually
   time-propagated from IMU input in the running loop, not only updated when measurements
   arrive. Otherwise the covariance never grows with motion and every uncertainty gate
   reads optimistically.
6. **Building the reject gate before the producer.** A gate that scores observations against
   a prediction nobody produces is decoration. Build the ephemeris propagator and Doppler
   predictor before the gate that consumes them.
7. **Circular validation.** If the LEO residual is scored against a state estimate driven by
   GNSS, accepted residuals measure agreement-with-GNSS, and exactly the measurements that
   would reveal a LEO bias get rejected as outliers. Compute LEO performance statistics only
   from GNSS-withheld runs, offline.
8. **Declaring states you never observe.** Do not carry filter states with no measurement
   path — they accrue process noise unbounded and inflate the state vector without
   informing it. Either implement the update or delete the state.
9. **A safety inversion without a backstop.** On a manned fast boat it is correct that
   software never auto-selects RTL/Loiter/disarm — an unannounced manoeuvre at speed is
   itself the hazard. But "never manoeuvre" must not become "no backstop": give the
   supervisor its own monotonic watchdog so authority cannot outlive the solution, make
   un-acknowledged alarms escalate, and ensure a physical, controller-independent override
   exists (a helm kill-cord is cheap and standard).
10. **Documentation drift.** When the design changes, mark the superseded documents as
    superseded *in the documents themselves*. A demotion recorded only in the new file's
    preamble is invisible to anyone reading top-down — including your future self.

## Open questions to resolve with live research

- bladeRF 2.0 micro: current price, EU availability, xA4 vs xA9 FPGA sizing for on-board
  channelisation, sustained USB 3.0 throughput on your chosen host, and the maturity of
  Rust bindings versus going through SoapySDR.
- Whether the two coherent RX channels are better spent on Ku + L-band simultaneity, on two
  Ku channels for OneWeb's alternating allocation, or on an interferometric baseline.
- **Iridium STL (Satelles)** — a commercial receive-only GNSS-alternative PNT service on
  Iridium, ~20 m and sub-100 ns timing. If purchasable at a sane price it changes the
  build-versus-buy calculation substantially. Get a quote.
- Whether your intended operating area has terrestrial signals of opportunity worth using
  (DVB-T2, cellular, R-Mode) — they are static emitters at known positions, so they yield
  fixes in seconds rather than the minutes LEO needs. Note the trap: DVB-T single-frequency
  networks are usually GNSS-disciplined, so they may degrade under the very jamming that
  motivated the project.
- Local regulations for autonomous or semi-autonomous vessel operation, and what a manned
  test platform is permitted to do.

## Working style

Research before designing; design before implementing. Where evidence contradicts an
assumption in this brief, say so directly and recommend the change rather than complying
silently. State what you did not verify.
