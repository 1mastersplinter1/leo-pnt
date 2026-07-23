# Adversarial Review — Unit U-H2.1 (high-speed study, real pipeline rebuild)

Worktree `/home/od/work/leo-pnt-wt-UH2`, commit `9b029ae` vs `main` (`748d958`). Gate reproduced green; study reproduced (numbers identical, see L1). Temp instrumentation and a detached main worktree were created for forensics and removed; U-H2 tree left clean (`git status` empty, no stray worktrees).

## Gate / reproduction
- `cargo fmt --all -- --check`: clean. `cargo clippy --all-targets -- -D warnings`: clean. `cargo test --workspace`: all pass (pnt-mission 5/5 incl. `high_speed_tests`; pnt-studies 10/10).
- `highspeed-study` run twice → `results.json` bit-identical run-to-run (deterministic), and every number matches the committed artifact.
- No attribution trailers in the commit message or diff. Commit subject: "fix high-speed mission study integration".

## CENTRAL CONCERN — the divergence (rigorously determined)

**C1 — HIGH / high confidence — genuine catastrophic divergence of the REAL estimator; root cause is the study disabling the chi-square innovation gate.** `crates/pnt-studies/src/highspeed.rs:238` (`pipeline.chi_square_threshold = None`) + `:228` (`doppler_interval_s: 1_800`). I instrumented the exact pipeline the study runs (real generator → `Executive<FilterStub>`) and traced the 7 kn tier:
- Dead-reckoning alone is **healthy**: at +1100 s of denial position error is ~320 m, velocity error ~0.7 m/s; at the moment before the first denied Doppler update (t=1800 s) it is **739 m / 1.30 m/s**.
- The **single ungated LEO Doppler range-rate update at t=1800 s** throws the state to **61,929,174 m / 76,473 m/s in one step** (BEFORE→AFTER captured directly). Every subsequent 30-min Doppler update oscillates chaotically in the 1e7–1e8 m band; the filter never recovers.
- Mechanism: process noise inflates position covariance to ~2e7 m² over the 30-minute gap; with the chi-square gate disabled and a single-satellite (ISS TLE only) near-unobservable range-rate geometry, the Kalman gain projects a huge innovation into a nonsensical correction. The filter's own covariance trace (~1e7 m², σ≈3 km) is 4 orders of magnitude below the actual error — a grossly inconsistent, divergent EKF.
- **It is NOT a units/frame bug and NOT DR bias drift** (DR is fine; frames self-consistent). Proof: re-running the identical scenario with the *default* gate `Some(9.0)` keeps the very same t=1800 s update sane (739 m→705 m) and the passage bounded (~70 km over 3 h, plausible single-sat DR). The study's own `chi_square_threshold = None` is the direct, sole cause of the 1e7–1e8 m numbers. The Doppler assimilation IS wired and IS reaching the filter — it is *detonating* it, not aiding it. Verify: set the gate to `Some(9.0)` in `simulate()` and re-run; divergence disappears.

**C2 — HIGH / high confidence — the study is NOT honest about the divergence; it dresses a divergent filter as a "result."** No occurrence of "diverge / physically impossible / gate disabled / unbounded" anywhere in `STUDY.md` or `results.json` (grep returns only the code line). Instead:
- The numbers are physically impossible and unlabeled as such: `landfall_position_error_m` 4.09e7 / 1.34e8 / 1.59e8 m (6–25× Earth radius, 6.378e6 m); `velocity_error_rms_mps` 53,604 / 91,052 / 89,004 m/s (> LEO orbital velocity). Presented in the headline table as the scenario output.
- `position_error_class` = `">=500 m"` (`highspeed.rs:324`, `error_class()` `:404-414`) collapses a 4e7 m error into the same bucket as 501 m — an absurd understatement. Misleading.
- `manoeuvre_convergence` ("2561 s / 8989 m" etc.) is measured against **catastrophe-scale** pre-turn thresholds (`15,630,347 m` / `238,127,927 m`) because the turn (7 kn: at t≈12,631 s) happens deep inside the already-diverged regime — the metric is meaningless yet tabled as a reconvergence result. The 30 kn "1 s / 0 m" is spurious.
- `integration_notes` actively imply the pipeline functions ("read from the real EKF state … no formula is used", "distance … accumulated from truth positions") without ever stating the EKF diverged. The `[UNVERIFIED]`/"plumbing demonstration" caveats are about *sourcing/authority*, not about the filter blowing up. A reader is told "real EKF, plumbing works" when the plumbing produces garbage.

**C3 — HIGH / high confidence — the divergence contradicts U-P1, and the reconciliation is silently omitted.** U-P1's passage study (`crates/pnt-studies/src/passage.rs`) reports **bounded 1633 m over 9 h** on a similar denied passage. The reason it stays bounded is that U-P1 does **not** use the real EKF: it uses a toy `PassageEstimator` (`passage.rs:81`) whose Doppler velocity correction is **hard-clamped to ≤0.0042 m/s per update** (`:131`, `scale = (0.0042/raw_norm).min(1.0)`) — it *cannot* diverge by construction, and it *also* sets `chi_square_threshold = None` (`:186`) but survives only because of the clamp. U-H2 uses the real `FilterStub` Kalman gain and detonates. So they don't disagree as same-code, but they reach **opposite conclusions about denied-passage viability**, and U-H2's real-EKF result reveals that U-P1's "passage-held (<1 NM)" is an artifact of a hobbled estimator, not evidence the real system holds. Neither study cross-references the other. As-is, U-P1 overstates denied viability and U-H2 buries the contradiction.

## Verification of the mandated fixes

**F1 — genuinely fixed.** The closed-form model is gone; the real generator + `Executive` + `FilterStub` run and numbers are real EKF-vs-truth. Confirmed by code read and by instrumented trace. (The catch: what the real pipeline demonstrates is divergence — C1/C2.)

**F2 — mechanically fixed, substantively vacuous.** Reconvergence is now searched from real samples and distance summed from truth positions — but in the diverged regime (C2), so it measures nothing. Also the "manoeuvre" is a 90° turn spread over 10% of a multi-hour mission (`highspeed.rs:215`, rate = `FRAC_PI_2/(0.1·duration)` ≈ 0.03°/s at 7 kn) — a trivially gentle non-manoeuvre, not a convergence-reset stressor.

**F3 — NOT properly fixed; the "against-main" test makes a FALSE provenance claim.** `crates/pnt-mission/tests/mission.rs:51-67`. The heading restoration is real (short-mission positions, x/y velocities, timing all byte-match main). BUT U-H2 is **not** bit-identical to main: the F5 change `velocity_ned_mps: [.., -velocity[2]]` (`lib.rs:305,319`) yields **`-0.0`** in the z-component of *every* GNSS truth and measurement record, vs main's `+0.0` — I proved via full-envelope field decode that this is the *only* difference, but it changes the serialized bytes. The test hardcodes measurements fp `0x03a5b3daf92c8325` / truth `0xb0a45e7a6473d976` with the comment "Captured from main's committed generator before the high-speed extension." **That is false** — I built main and it actually produces `0x9b28a34cdf36014d` / `0x513ee069cd73ac66` (manifest `0x7569329614f87b51` does match). The test asserts U-H2's own output and passes tautologically; it does **not** verify against-main equivalence — exactly the failure mode the prior F3 flagged. This also falsifies the disposition's "default missions provably bit-identical" claim. Verify: `git worktree add --detach <tmp> main`, generate `small(42)`, FNV-1a the segs. Severity MEDIUM-HIGH (the byte delta is cosmetic `-0.0`, but the verification claim is fabricated).

**F4 — fixed.** Slam is a bounded full-cycle cosine, zero-mean (`lib.rs:659`), unit-tested (mean < 1e-12) and truth-consistency-tested. Good. (Note: the wave/slam has *zero* effect on the study result — the filter diverges regardless, so the truth-consistency fix is untested end-to-end by this study.)

**F5 — fixed in mechanism.** Disturbance integrated into both truth and IMU; local-up mapped to ECEF. Good — though it is the source of the F3 `-0.0`.

**F6 — fixed.** `SpeedScaledImuConfig` behaviorally tested at reference and 2× speed for both noise and bias (`lib.rs` `high_speed_tests`). Good.

**D50 consistency — consistent.** `STUDY.md`/`results.json` state plainly it does not support 20 kn denied and 30 kn is exploratory/no denied authority — matches `HIGH_SPEED_ENVELOPE.md`. The study does not claim 20/30 kn denied is supported.

## Lower-severity

- **L1 — LOW / high confidence — committed `results.json` is not byte-reproducible.** The generator (`serde_json::to_vec_pretty`) emits no trailing newline, but the committed file ends `}\n`. Numbers reproduce exactly; the file differs by one trailing byte, so "regenerates byte-identically" is not strictly true for `results.json` (STUDY.md does reproduce identically).
- **L2 — LOW — 300 s aided "steady state" is asserted, not checked** (`AIDED_STEADY_S`); the trace shows covariance flat by ~t=275 s so it's fine in practice, but nothing verifies steady-state before recording the loss covariance.

## What was done well
F1/F4/F6 are real fixes; the D47 scenario holds distance at 100 km across tiers on a shared seed and raises the validate cap to 15.5 m/s for the 30 kn tier; `[UNVERIFIED]` labeling on wave/aging is honest; D50 verdicts are respected; determinism holds; no attribution trailers.

---

**FAIL** — The primary deliverable runs the real pipeline (F1 fixed) but the real denied-mode EKF **diverges catastrophically** to 6–25× Earth radius and >orbital velocity, caused by the study's own decision to disable the safety-critical chi-square innovation gate (`highspeed.rs:238`) that `SAFETY_CASE §2.3` relies on — with the gate at its production value the filter stays bounded (C1). The study **does not disclose the divergence anywhere**, dressing physically-impossible numbers as a "plumbing demonstration," collapsing a 4e7 m error into a ">=500 m" class, and tabling meaningless in-divergence "reconvergence" figures (C2). It silently contradicts U-P1's bounded passage, whose boundedness is itself an artifact of a hard-clamped toy estimator (C3). And the "real against-main test" for the short-mission regression asserts fabricated "captured from main" fingerprints that are actually U-H2's own output, so it verifies nothing (F3).