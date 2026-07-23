# Brief U-T1 — Correlation Doppler tracker (synthetic-IQ validated)

Contract version: v5.1. Workspace: dedicated worktree, branch `unit/U-T1`. Commit here;
never merge to main. Read first: `docs/design/DESIGN_BASELINE.md` (observable definition:
correlation-peak Doppler, 2.5–5 MHz bandwidth, never synthesize observations),
`.orchestration/CONTRACTS.md` v2 (TrackerDoppler envelope) + v4 (offset-from-nominal
convention, NORAD source_id), `docs/design/ARCHITECTURE.md` module 5.

## Goal
New crate `crates/pnt-tracker`:
1. **Synthesis module** (`synth`): generate complex-baseband IQ containing a repeating
   known reference sequence (configurable PN/BPSK burst) with injected frequency offset,
   offset ramp (Doppler drift), delay, and AWGN at configurable C/N0. Deterministic from a
   seed (no OS randomness). This is test infrastructure AND the future capture-simulator.
2. **Acquisition + tracking**: FFT-based search (you MAY use `rustfft`; record version)
   over frequency bins × time for the reference sequence; correlation-peak detection with
   a quality metric (peak-to-noise-floor ratio); block-to-block tracking that refines the
   frequency hypothesis and follows a ramp. Output per block: correlation-peak offset (Hz,
   offset-from-nominal per v4), quality, timestamp — as data the executive can wrap into
   TrackerDoppler envelopes (provide the constructor; do not modify the executive).
3. **Honesty rule (baseline: never synthesize to meet a rate)**: below the detection
   threshold the tracker emits NOTHING (typed NoDetection, never a fabricated observation);
   the threshold is explicit and configurable, default justified in comments [UNVERIFIED
   pending link-budget work].
4. **Real-signal plug-in point**: the reference sequence is a parameter. Document (README)
   exactly where a real Starlink PSS/SSS or Iridium/Orbcomm reference plugs in, and mark
   the real sequences [UNVERIFIED — not in this unit; requires verified published
   sequences and real captures].

## Tests (TDD; analytic values derived in comments, never from the code under test)
- Injected constant offset recovered within a stated tolerance (derive tolerance from bin
  width + interpolation method) at high C/N0; at moderate C/N0; multiple offsets incl.
  negative and near-Nyquist-bin edges.
- Doppler ramp tracked across ≥10 blocks with per-block error bound; no cycle-slip-style
  jump between adjacent blocks (smoothness bound).
- Pure-noise input at the same threshold → NoDetection every block (zero false observations
  across the test's blocks); signal restored → reacquisition.
- Determinism: same seed + params → bit-identical outputs.
- Quality metric monotone in C/N0 across ≥3 levels.
Gate: PATH="$HOME/.cargo/bin:$PATH" cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check.

## Files owned
`crates/pnt-tracker/**`, root `Cargo.toml` member line, `.orchestration/reports/U-T1.md`.
Nothing else — no executive edits (integration wires it later).

## Report
Evidence (real gate output), tolerance derivations, dependency versions, [UNVERIFIED] list,
integration notes for wiring into the executive.
