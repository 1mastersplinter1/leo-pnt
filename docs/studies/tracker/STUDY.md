# Tracker synthetic-IQ stress-envelope study

Date: 2026-07-23. Harness: `pnt-studies` schema 1. Fixture: Fs = 8192 Hz, 256-sample
PN/BPSK reference, 256 frequency rows at 32 Hz spacing, ±4080 Hz acquisition band,
threshold Q = 32, and ±128 Hz tracking window. Full command:

```sh
cargo run -p pnt-studies --release --bin tracker-study -- --out docs/studies/tracker
```

The final correction run used Rayon, 500 deterministic seeds per C/N0 level, 1,000,000
noise-only blocks, and 200 trials per stochastic impairment point. Harness wall time was
202.974 s (false-alarm tail 198.364 s). Earlier complete runs took 201.083, 207.97, and
215.77 s.
JSON files beside this report are the authoritative full-precision results.

## Headline findings

- At the measured 8.5 Hz sub-bin position, the threshold-32 detection knee is 32 dB-Hz:
  Pdet = 0.240/0.546/0.848 at 31/32/33 dB-Hz. This is a favorable -1.0 dB scalloping
  point; the half-bin worst case is -3.9 dB, so the disclosed position-dependent knee is
  32–35 dB-Hz. Detection is not accuracy: accepted weak locks have hundreds of hertz
  error. Error sigma falls below 4 Hz only around 52 dB-Hz and reaches 0.482 Hz at
  62 dB-Hz.
- The archived 4000-block probe is reproduced (median 11.50, p99 15.77). At one million
  blocks the maximum is 24.78 and Q=32 has zero exceedances. The Fisher/independent-row
  union prediction is conservative across the reachable tail by roughly 1.5–2.4×, but
  zero events only gives an approximate 95% empirical upper bound of 3e-6. It cannot
  validate the model's 5.30e-9 value.
- At this fixture, all 16 blocks track through 4000 Hz/s; only 8/16 survive at 8000 Hz/s.
  Real worst-case Ku drift from the stated circular-orbit model is 3718 Hz/s, barely
  inside that measured envelope, but real Ku Doppler magnitude is ±270 kHz while the
  fixture acquisition band is only ±4.08 kHz. Production therefore requires ephemeris
  wipe-off or a much wider acquisition architecture.
- CW behavior is not uniformly fail-loud: at J/S=+10 dB, the outcome depends on CW
  placement, ranging from all NoDetection to 20.5% or 100% false locks. Clock error maps
  directly into Doppler bias (f_carrier × fractional error). Every tested 1–16 block
  outage reacquired on the first restored block. The original two-signal false-capture
  headline was a harness construction artifact: both paths had the same PN code, delay,
  and phase, leaving a clean two-tone after despreading whose Kay discriminator returned
  the power-weighted frequency centroid. With a distinct PN seed, all tested outcomes are
  direct-signal locks or NoDetection. The genuine same-code/different-delay multipath case
  instead selects the stronger delay, splitting roughly evenly at equal power.
- Quality is useful below saturation but is not a globally invertible variance proxy.
  For Q<180, a log-log fit gives `ln(var_Hz2) = 27.222 - 4.279 ln(Q)`, RMS log residual
  0.571. Above about Q=180, Q saturates near 191 while error variance continues improving.

## 1. Detection and accuracy versus C/N0

Each point is 500 independently seeded one-block trials at an injected 487.5 Hz offset.
Quality quantiles include both detected and rejected trials; error statistics cover emitted
detections. Pdet intervals are two-sided 95% Wilson binomial intervals.

| C/N0 (dB-Hz) | Pdet | 95% CI | error mean (Hz) | error sigma (Hz) | max abs (Hz) | median Q |
|---:|---:|---:|---:|---:|---:|---:|
| 30 | 0.072 | 0.052–0.098 | -92.11 | 411.39 | 1783.32 | 22.33 |
| 31 | 0.240 | 0.205–0.279 | 23.16 | 336.39 | 1083.05 | 26.92 |
| 32 | 0.546 | 0.502–0.589 | -29.93 | 272.90 | 956.11 | 32.89 |
| 33 | 0.848 | 0.814–0.877 | 5.71 | 229.75 | 904.44 | 39.52 |
| 34 | 0.988 | 0.974–0.994 | 4.32 | 195.44 | 660.73 | 47.85 |
| 40 | 1.000 | 0.992–1.000 | 0.26 | 46.64 | 152.10 | 108.89 |
| 46 | 1.000 | 0.992–1.000 | -0.44 | 11.93 | 32.54 | 160.91 |
| 52 | 1.000 | 0.992–1.000 | 0.03 | 3.26 | 15.42 | 182.69 |
| 58 | 1.000 | 0.992–1.000 | -0.03 | 0.955 | 2.99 | 189.22 |
| 62 | 1.000 | 0.992–1.000 | -0.006 | 0.482 | 2.06 | 190.63 |
| 70 | 1.000 | 0.992–1.000 | -0.004 | 0.155 | 0.52 | 191.49 |
| 78 | 1.000 | 0.992–1.000 | -0.003 | 0.059 | 0.20 | 191.63 |

The 62/70/78 sigma values reproduce the prior uncommitted review probe
(0.481/0.155/0.059 Hz) to rounding. The sharp Pdet knee and the much slower accuracy
transition show why threshold crossing alone is insufficient evidence for a covariance.
The injected tone occupies a favorable 8.5 Hz position inside a 32 Hz bin (-1.0 dB
scalloping). A half-bin review probe (-3.9 dB scalloping) measured Pdet=0.232 at
34 dB-Hz versus 0.988 here, shifting the practical knee by about 2–3 dB. The supported
claim is therefore a 32–35 dB-Hz position-dependent range, not a universal 32 dB-Hz
threshold. See `detection-accuracy.json`.

## 2. False-alarm tail and Fisher-g model

For one frequency row with N=256 i.i.d. exponential delay powers, let
`g=Pmax/S`. The tracker statistic is

```text
Q = (N-1) g / (1-g),       g = Q / (Q+N-1).
```

The exact row exceedance is Fisher's sum

```text
P(g>x) = Σ[j=1..floor(1/x)] (-1)^(j-1) C(N,j) (1-jx)^(N-1).
```

The reported prediction conservatively unions this over 256 frequency rows. Observations:

| Q threshold | observed count | observed P | Fisher union P | model/observed |
|---:|---:|---:|---:|---:|
| 14 | 56,513 | 5.6513e-2 | 7.8952e-2 | 1.40 |
| 16 | 7,960 | 7.960e-3 | 1.1941e-2 | 1.50 |
| 18 | 1,091 | 1.091e-3 | 1.8313e-3 | 1.68 |
| 20 | 131 | 1.31e-4 | 2.8470e-4 | 2.17 |
| 24 | 3 | 3.0e-6 | 7.1640e-6 | 2.39 |
| 28 | 0 | 0 | 1.8997e-7 | — |
| 32 | 0 | 0 | 5.3008e-9 | — |

The model gets the tail scale right and remains conservative for this white-noise fixture;
positive dependence between frequency rows reduces the effective trial count. It is not
empirically tested at 5.3e-9: a useful zero-event bound needs about 5.7e8 blocks, so one
million samples are about 2.8 orders of magnitude short. Coloured/non-Gaussian real noise,
interference, ADC effects, real sequences, and production search geometry remain
**[UNVERIFIED]**. See `false-alarm-tail.json`.

## 3. Doppler dynamics

### Circular-orbit derivation

Use Earth equatorial radius `R=6,378,137 m`, gravitational parameter
`mu=3.986004418e14 m^3/s^2`, orbital radius `r=R+h`, and `c=299,792,458 m/s`.
Circular speed is `v=sqrt(mu/r)`. At the geometric horizon the limiting line-of-sight
speed magnitude is `v R/r`, hence

```text
|fD|max = (fc/c) sqrt(mu/r) R/r.
```

At overhead the range curve has maximum curvature
`|rho_ddot| = mu R/(r^2 h)`, giving

```text
|fD_dot|max = (fc/c) mu R/(r^2 h).
```

These are ideal spherical-Earth, circular-orbit, stationary-receiver extremes. Receiver
motion, Earth rotation, eccentricity, refraction, oscillator error, and actual elevation
masks are **[UNVERIFIED]**.

| Band | fc | altitude | max |Doppler| | overhead max drift |
|---|---:|---:|---:|---:|
| Ku low | 11.325 GHz | 550 / 1200 km | 263.8 / 230.6 kHz | 3638 / 1394 Hz/s |
| Ku high | 11.575 GHz | 550 / 1200 km | 269.6 / 235.7 kHz | 3718 / 1424 Hz/s |
| L | 1.616 GHz | 550 / 1200 km | 37.64 / 32.90 kHz | 519 / 199 Hz/s |
| VHF | 137 MHz | 550 / 1200 km | 3.191 / 2.789 kHz | 44.0 / 16.9 Hz/s |

The 256-sample fixture tracked all blocks at 4000 Hz/s and half at 8000 Hz/s. Block-length
sweeps found largest all-detected grid points of 4000, 8000, 4000, and 2000 Hz/s for
64, 128, 256, and 512 samples respectively; these non-monotone coarse-grid results show
interaction among coherent smear, bin spacing, and extrapolation and are not a closed-form
limit.

When a 2000 Hz/s signal walked out of the acquisition band, the first NoDetection followed
one block after first crossing, but later aliased/wrong detections occurred (9 blocks).
At 8000 Hz/s loss occurred before midpoint crossing and 9 wrong-lock blocks followed.
Thus band escape is not reliably fail-loud. Real-sequence behavior, production bandwidth,
front-end filtering, ephemeris wipe-off, and constellation-specific block lengths are
**[UNVERIFIED]**. See `dynamics.json`.

## 4. Impairments

The CW sweep used a 62 dB-Hz desired signal. At J/S=0 dB every point detected, but mean
bias ranged from -15.70 to +9.63 Hz. At +10 dB, CW at -3000 or +480 Hz caused 200/200
NoDetection; CW at 0 Hz emitted 200/200 with 41/200 false locks; CW at +2000 Hz emitted
200/200 with 200/200 false locks. At +20 dB all points were rejected. These results
quantify the proposal's warning that narrowband interference can produce heavier,
non-Gaussian failure modes.

Clock/reference mismatch was injected as an equivalent carrier term. Measured bias follows
`fc epsilon`: at epsilon=1e-7 it was 13.663 Hz (137 MHz), 161.644 Hz (1.616 GHz), and
1157.541 Hz (11.575 GHz). At Ku even 1e-9 produces about 11.6 Hz bias. A distribution of
actual rubidium calibration errors is **[UNVERIFIED]**.

After 1, 2, 4, 8, and 16 noise-only outage blocks, every case reacquired on the first
restored block. This is deterministic one-seed evidence, not a probability or a real burst
schedule.

The original two-signal result was a **construction artifact**, not tracker evidence:
`reference()` fixed both synthesizers to PN seed `0x12345678` and delay 37, with equal
carrier phase. Despreading therefore removed the shared code and left a clean two-tone;
the Kay frequency discriminator correctly returned its power-weighted centroid, which is
neither injected frequency. The corrected co-channel harness uses distinct PN seed
`0x12345678 XOR 0x5555`. Across offsets -1000/0/+1500 Hz and powers -10/0/+10 dB
(200 trials each), all -10 and 0 dB cases locked the direct signal (1200/1200), while all
+10 dB cases returned NoDetection (600/600); there were zero secondary or other locks.

The genuine same-code analog was measured separately as specular multipath: direct delay
37, echo delays 45/69/101 samples, common frequency and phase, and echo/direct powers
-10/0/+10 dB. Every one of 1800 trials detected with no other-delay selection. The direct
path won 600/600 at -10 dB, the echo won 600/600 at +10 dB, and equal-power selection
split 321 direct versus 279 echo. Mean frequency error across the nine points stayed
between -0.523 and +0.033 Hz. This fixture demonstrates delay selection, not the old
frequency-centroid artifact; fractional delays, phase distributions, fading, and real
channel codes remain **[UNVERIFIED]**. See `impairments.json`.

## 5. Quality to variance

Emitted detections from the C/N0 sweep were binned by Q. Variance falls from
80,240 Hz² at mean Q=35.99 to 65.09 Hz² at Q=171.36. A least-squares fit below Q=180 is

```text
variance_hz2 = exp(27.2216) * Q^-4.2795
```

with RMS residual 0.571 in natural-log variance. Per-bin fitted values and residuals are in
`quality-variance.json`. Above Q≈180 the statistic saturates (median Q is already 190.63 at
62 dB-Hz and 191.63 at 80 dB-Hz), while sigma improves from 0.482 to 0.049 Hz. Therefore
the fitted mapping must not be extrapolated through saturation. A production covariance
mapping conditioned on real sequence, bandwidth, dynamics, interference, and calibration
is **[UNVERIFIED]**.

## Reproducibility and limits

All random streams use committed xorshift64* synthesis and parameter-derived seeds.
Consecutive xorshift64* seeds have a lag-1 correlation of 0.194 in the first draw, confined
to draw 0; the measured block-statistic lag-1 correlation is 0.0025. This is a harness
note, not evidence of a material result bias, and a splitmix64 seed hash would be reasonable
future hardening. The tracker implementation itself processes every trial. Only wall-time
fields are nondeterministic. This is synthetic white-noise fixture evidence, not a production
threshold freeze. In particular, real signal sequences, capture C/N0 distributions,
coloured noise tails, oscillator statistics, multi-signal delay/phase populations, and
2.5–5 MHz production geometry remain **[UNVERIFIED]**.
