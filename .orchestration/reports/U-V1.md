# U-V1 report — tracker stress envelope

Status: complete. Branch: `unit/U-V1`.

## Headline findings

- **Knee:** threshold-32 Pdet crosses 50% at 32 dB-Hz (0.240/0.546/0.848 at
  31/32/33). Weak accepted detections remain grossly inaccurate; sigma is 272.9 Hz at
  32 dB-Hz and falls below 4 Hz around 52 dB-Hz.
- **PFA model:** 1,000,000 deterministic noise blocks reproduce the archived median/p99
  (11.50/15.77), extend max Q to 24.78, and produce zero Q>=32 events. Fisher's row-union
  model is conservative by about 1.5–2.4× over the measurable tail. The claimed
  5.30e-9 model value is not empirically reachable here; zero events imply only an
  approximate 95% upper bound of 3e-6.
- **Dynamics gap:** the fixture sustains all 16 blocks through 4000 Hz/s and only 8/16 at
  8000 Hz/s. Ideal 550 km Ku overhead drift reaches 3718 Hz/s, leaving little fixture
  margin, while real Ku Doppler magnitude reaches 270 kHz versus ±4.08 kHz acquisition.
  Walking out of band can later alias/wrong-lock and is not reliably fail-loud.
- **Impairments:** +10 dB CW produces outcomes from all rejection to 100% false locks,
  depending on placement. Frequency-reference error biases Doppler by `fc*epsilon`
  (11.6 Hz at Ku for 1e-9). Tested outages reacquire in one restored block. Every tested
  two-copy trial emitted a lock at neither true frequency.
- **Variance mapping:** below Q=180,
  `ln(var_Hz2)=27.2216-4.2795 ln(Q)` with RMS log residual 0.571. Q then saturates near
  191 while accuracy continues improving, so no global quality-only variance mapping is
  justified.

## Evidence and runtime

- Narrative, derivation, tables, caveats: `docs/studies/tracker/STUDY.md`
- Full JSON: `detection-accuracy.json`, `false-alarm-tail.json`, `dynamics.json`,
  `impairments.json`, `quality-variance.json`, and `manifest.json` in the same directory.
- Reproducible harness: `crates/pnt-studies`; full and quick commands in its README.
- Final full run: harness 201.083 s; `/usr/bin/time`: 202.55 s real, 2671.85 s user,
  1.23 s system. Rayon used all available worker threads.

## [UNVERIFIED] carried forward

Production 2.5–5 MHz geometry and sequences; real C/N0/link budgets; coloured and
non-Gaussian capture tails; ADC/front-end effects; constellation-specific coherent block
lengths; ephemeris wipe-off; orbital-model corrections and receiver motion; oscillator
error distributions; real interference populations; multi-signal delay/phase populations;
and the production `frequency_variance_hz2` mapping.
