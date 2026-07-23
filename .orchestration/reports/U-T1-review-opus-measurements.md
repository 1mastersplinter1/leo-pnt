# U-T1 Opus review — archived Monte-Carlo measurements (2026-07-23)

Provenance: produced by the U-T1 deep-review seat's independent probes against pnt-tracker
commit 54005dd (probes run in-worktree, then removed; full review in session transcript,
summary in DECISIONS D36). Archived here as citable evidence for downstream analysis
(PARAMS_PROPOSAL). These are REVIEW measurements, not shipped-test evidence: the committed
suite's noise test covers 24 blocks only.

- Discriminator frequency-error 1σ: 0.059 / 0.155 / 0.481 Hz at C/N0 = 78 / 70 / 62 dB-Hz
  (2000 seeds each; max |error| 1.6 Hz at 62 dB-Hz).
- Coarse-bin-only error for the four fixture offsets: 6.75–14 Hz (all > the 4 Hz tolerance,
  proving the tolerance discriminates a broken interpolator).
- Noise-only quality statistic (256×256 search, fixture config): median 11.5, p99 15.7,
  max 20.0 over 4000 blocks; zero false alarms at threshold 32 in 4000 blocks.
- Signal quality at C/N0 62/65 dB-Hz: 186–191 (saturation ~190 for C/N0 ≥ ~62 dB-Hz).
- Second-seat spot-check (60 trials, 65 dB-Hz): noise-only max-quality range ~9.7–15.9,
  consistent with the above.
