# Brief U-R4 — LEO downlink signal-structure research for the tracker (live web)

Contract version: v5.1. Read `.orchestration/CONTRACTS.md` v1 report rules first.

## Goal
Live web research consolidating what is PUBLISHED about the signal structures the tracker
(crates/pnt-tracker) must eventually correlate against. For each, cite source URL + access
date; split VERIFIED (read it) vs ASSUMED; note licensing/legal constraints on use:
1. **Starlink Ku downlink sync**: the published PSS/SSS structure (Humphreys et al. and
   successors) — frame period, symbol rate, sequence construction (exact generator if
   published), center frequencies/channelization, bandwidth needed (does 2.5-5 MHz capture
   suffice and why), any post-2023 changes reported.
2. **Iridium**: ring-alert / simplex / broadcast burst structure usable for Doppler (which
   channels are always-on, burst timing, published decoders like gr-iridium — what they
   reveal about tone/preamble structure usable for correlation).
3. **Orbcomm**: downlink format at 137 MHz (symbol rate, packet structure, published SoOP
   trackers) — what reference waveform a correlator would use.
4. **OneWeb**: any published beacon/repetition structure details beyond the demand-dependent
   10 ms repetition already known.
For each: state exactly what a correlator reference generator needs (sequence bits/tones,
rate, period) and whether that is fully published or requires capture-based reverse
engineering; flag anything uncertain.

## Output format (REPLACES file writing)
Do NOT write or edit any files. Deliver everything as your final answer text:
line `===R4-DOC===`, then the research document, then `===R4-REPORT===`, then the unit
report (summary, VERIFIED/ASSUMED, dead ends, contract version).
