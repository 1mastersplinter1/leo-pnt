# Brief U-V1 — Tracker stress-envelope study (synthetic IQ)

Contract version: v5.1. Workspace: dedicated worktree, branch `unit/U-V1`. Commit here; never
merge to main. Read first: `crates/pnt-tracker/**` (lib, synth, README, tests),
`docs/design/PARAMS_PROPOSAL.md` §4 (the PFA model this study must test empirically),
`.orchestration/reports/U-T1-review-opus-measurements.md` (prior measurements to reproduce —
making them finally REPRODUCIBLE is one of your jobs), `docs/research/R4-signal-structures.md`
(real signal parameter ranges), `.orchestration/DECISIONS.md` D36/D38.

## Goal
New crate `crates/pnt-studies` (lib + `tracker-study` binary) producing a quantitative
characterization of the tracker far beyond the 6 shipped tests. Studies (each a seeded,
deterministic, JSON-emitting module + a section in the study report):

1. **Detection & accuracy vs C/N0**: sweep C/N0 from well above threshold down through
   failure (e.g. 30–80 dB-Hz, fine steps near the knee): detection probability, frequency
   error mean/σ/max per level (≥500 seeds/level), quality distribution. Deliverable: the
   knee location and the accuracy curve, replacing single-point evidence.
2. **False-alarm tail at scale**: ≥10^5 (aim 10^6 if runtime allows — use rayon, record
   wall time) noise-only blocks at the fixture config: empirical quality-max distribution
   tail vs the PARAMS_PROPOSAL Fisher-g model prediction (overlay predicted vs observed
   exceedance at multiple thresholds), threshold-32 exceedances counted. This directly
   tests the proposal's PFA(32)≈5.3e-9 model in the regime the data can reach, and
   supersedes the irreproducible review-probe numbers with committed, reproducible code.
3. **Doppler dynamics envelope**: compute (in the report, from orbital mechanics — show the
   derivation) the real LEO Doppler and drift extremes for Ku (11.325/11.575 GHz), L-band
   (1.616 GHz), and VHF (137 MHz) at 550–1200 km altitudes; then measure the tracker's
   actual limits with synthetic IQ: max trackable ramp before loss (sweep drift up to and
   beyond the real extremes at the fixture sample rate), search-bound escape behavior
   (offset walking out of ±band mid-pass — does it fail loud or lock wrong), block-length
   sensitivity. State plainly where the current fixture config falls short of real
   dynamics [UNVERIFIED items for the real-signal unit].
4. **Impairments**: (a) CW interferer swept across the band at various J/S — false-lock
   probability and accuracy impact; (b) reference/clock frequency offset between synth and
   tracker reference (simulating rubidium calibration error, e.g. 1e-9..1e-7 fractional) —
   bias in reported Doppler; (c) reacquisition latency after N-block outages; (d) two
   overlapping signals at different offsets — capture behavior.
5. **Quality→variance mapping evidence**: empirical frequency-error variance vs quality
   metric across the sweep — the data the [UNVERIFIED] `frequency_variance_hz2` mapping
   needs (fit + residuals, saturation region flagged).

## Method
Study harness code TDD-light (determinism + schema tests; the studies themselves are the
product). Everything seeded/deterministic. Binary: `cargo run -p pnt-studies --release --bin
tracker-study -- --out DIR [--quick]` (--quick for CI-scale, full for the real numbers).
Run the FULL study yourself; commit the JSON outputs under `docs/studies/tracker/` plus
`docs/studies/tracker/STUDY.md` (tables, derivations, honest interpretation, [UNVERIFIED]
markers). Gate: cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all
-- --check (PATH="$HOME/.cargo/bin:$PATH").

## Files owned
`crates/pnt-studies/**`, root `Cargo.toml` member line, `docs/studies/tracker/**`,
`.orchestration/reports/U-V1.md`.

## Report
Headline findings (knee, PFA-model verdict, dynamics gap, impairment sensitivities),
evidence pointers, runtime notes, [UNVERIFIED] list.
