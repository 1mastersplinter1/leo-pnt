# U-V2 report — estimator validation and D39 closure

## Consistency

The 96-run, 57,600-epoch synthetic campaign says the stub covariance is **pessimistic**,
not honest at the nominal scale: six-state NEES mean is 0.870 versus the expected 6.0 and
95% coverage is 100.0%. Doppler NIS is closer but still broad (mean 0.738, 97.83%
coverage versus 95%). This detects over-stated covariance/process noise in the tested
truth model; it is not evidence that the filter is conservative for real sensors.

## D39 answer

D39 has two evidenced mechanisms.

First, the existing replay's disclosed `ReceiverPrior` is not an initialization.
`pnt-replay` feeds it through `update_gnss`; both the zero-state filter covariance and
configured prior variance are 1, so the Kalman gain is 0.5. The supposedly truth-equivalent
ECEF position retains **3,189,068.5 m radial error**. Horizontal replay scoring at the
equator does not see this radial component, but Doppler geometry does. A 20-minute
controlled replay of that path produces 3,835 m/s velocity RMS. The named structural fix
is a future pnt-replay/estimator integration unit: add an atomic state-and-covariance
initialization API and regression-test radial ECEF error.

Second, after replacing that confounded path with an effectively exact prior, default
acceleration process variance is 0.04 while injected acceleration-error variance is
approximately 0.0004. The filter therefore gives excessive gain to a noisier scalar
range-rate observation. Prior-only velocity RMS is **0.2506 m/s**; default Doppler raises
it to **0.3753 m/s**. The decomposition identifies the affected direction: along-LOS RMS
rises **0.1116 → 0.3075 m/s**, while across-LOS RMS improves **0.2243 → 0.2153 m/s**.
This is not a general vector-velocity loss; it is range-rate noise injected along the
observable LOS.

The controls agree:

- Fed measurement variance 0.01/0.14/1/10 m²/s² gives velocity RMS
  0.8483/0.3753/0.3272/0.2487 m/s with the generated noise held fixed.
- Acceleration Q 0.0004/0.004/0.04/0.4 gives
  0.2365/0.3084/0.3753/0.6643 m/s.
- Four TLE offsets give 0.3753–0.3947 m/s; geometry changes magnitude but not mechanism.
- Observation periods 1/2/5/10 s give 0.3753/0.3693/0.4168/0.6814 m/s; simple
  decimation is not the cure.
- Fixing or loosening the nuisance bias does not remove the default-Q degradation.

A demonstrated non-degrading tuning exists: acceleration variance **0.0004** with the
nominal fed Doppler variance gives **0.2365 m/s**, better than prior-only. This is a
synthetic-stack tuning, not a value to freeze. The replay initialization defect must be
fixed before production tuning.

## Position observability

The stub reproduces only the relative part of the 10–20 minute claim. Doppler is worse at
2 minutes (5.74 versus 0.81 m RMS), crosses into benefit near 20 minutes (86.95 versus
88.84 m), and is better at 30 minutes (149.59 versus 199.81 m). Absolute RMS grows rather
than converges. The inserted turn does not reset and reconverge: 28.52 m before the turn,
43.87 m just after, and 125.24 m at the end. The stub has no heading-to-velocity coupling
or manoeuvre covariance reset, so it cannot express the handoff's proposed reset mechanism.

## Stale ephemeris

With Doppler generated from epoch offsets 0/1/6/24 h and predicted from the fresh epoch,
innovation RMS is 0/5413.71/4395.29/3825.29 m/s. Threshold 9 rejects
0/99.83/100/100%. This synthetic phase-shift experiment says 6 h is certainly not too
tight for this fixture, but it does **not** validate actual SupGP error growth or prove
that 6 h is the correct boundary; the first tested nonzero offset already rejects.

## [UNVERIFIED]

- Real-signal residual distributions, oscillator behavior, and correct production R/Q.
- Multi-satellite and real pass availability; the campaign uses one ISS fixture.
- SupGP age-error behavior; epoch shifting is deliberately adversarial, not an aged-TLE corpus.
- Long debug builds hit the stub's absolute `1e-8` covariance-symmetry assertion; the full
  production-math campaign was generated in release mode.
