# U-H2.1 fix-round report

The prior closed-form D46 harness was removed. The committed D47 artifact is produced by the real
`pnt-mission` generator and routed through `fusion-executive::Executive` with the real
`pnt_estimator::FilterStub` error-state EKF. Every reported position/velocity value is the EKF state
compared with the generator truth stream.

## D47 scenario of record

Each shared-seed tier runs aided for 300 s. The study records the real converged position error,
velocity error, covariance trace, and covariance dimension at GNSS loss, then continues the same EKF
denied for a constant 100 km at 7, 20, and exploratory 30 kn. Wave/slam and speed-scaled IMU are on.
The executive uses U-P1 graduated ephemeris aging and reports age and margin to its 30 h ceiling.

The study is explicitly a **SYNTHETIC [UNVERIFIED] capability/plumbing demonstration**, not a
navigation-performance or authority claim. It is consistent with D50: 20 kn denied is not supported
on present evidence, and 30 kn remains aided/manual-only, exploratory, with no denied autonomous
authority.

## Review dispositions

- **F1 — fixed.** No closed-form position, velocity, or convergence model remains. Generated
  measurement journals drive the Executive and EKF; filter state is measured against generated
  truth.
- **F2 — fixed.** The route description now matches the generator's actual single coordinated turn.
  Reconvergence time is searched from real post-turn error samples. Distance is independently summed
  from successive truth positions; a non-recovery is reported honestly rather than fabricated.
- **F3 — fixed.** The legacy post-turn branch again returns exactly `FRAC_PI_2` for the default path,
  including missions shorter than 10 s. A 4 s test compares manifest, measurement-segment, and
  truth-segment fingerprints against main's committed behavior, rather than new-vs-new only.
- **F4 — fixed.** Slam acceleration is a bounded full-cycle cosine with zero mean. R5 supplies the
  100–450 ms duration and 0.44 g RMS anchor. The selected 0.25 s duration, event rate, pitch coupling,
  sinusoidal mapping, and 30 kn scaling are explicitly `[UNVERIFIED]`.
- **F5 — fixed.** Disturbance acceleration is integrated into disturbance velocity and truth
  position. The same total acceleration is emitted by the IMU, so integrating noise/bias-free IMU
  samples recovers truth plus disturbance. Local vertical is mapped to ECEF up rather than ECEF X.
- **F6 — fixed.** Behavioral tests construct `SpeedScaledImuConfig` and verify both noise and bias
  scaling at reference and doubled speed. A separate integration test verifies a sampled full slam
  cycle returns disturbance velocity to zero.
- **F7 — fixed with F5.** Local up is transformed explicitly into ECEF.
- **F8 — fixed by removal.** The speed-dependent closed-form phase offset no longer exists. All tiers
  use the same seed; speed changes generator configuration, not an additive result offset.
- **F9 — fixed.** Study text says precisely what ran and retains `[UNVERIFIED]`/D50 caveats. Process
  noise remains config-driven. No hardcoded endpoint-error law remains.

## Additional integration fixes

Long-duration propagation exposed two estimator issues. GNSS NED velocity is now rotated at the GNSS
fix position (not an ill-conditioned cold-start prior position), and covariance is explicitly
re-symmetrized after Joseph updates and propagation to control floating-point skew over hundreds of
thousands of steps.

The generator accepts the D46/D47 exploratory envelope through 15.5 m/s. Its default one-second path
and RNG sequence remain unchanged. Decimated endurance tracker observations reacquire independently
around the current prediction, and mission truth propagation permits the 30 h graduated-aging
envelope; authority/inflation remains an Executive decision.

## Verification

- Targeted mission and high-speed integration tests pass.
- `PATH="$HOME/.cargo/bin:$PATH" cargo test` — pass.
- `PATH="$HOME/.cargo/bin:$PATH" cargo clippy --all-targets -- -D warnings` — pass.
- `PATH="$HOME/.cargo/bin:$PATH" cargo fmt --all -- --check` — pass.
