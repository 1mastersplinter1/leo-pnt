# Estimator Covariance-Consistency Fix — Design Spec (U-FS1)

Status: actionable design spec for the estimator-consistency fix. This document is
the implementation plan handed to whoever lands the fix (possibly a different
contributor). It is grounded in the verified U-CD1 diagnosis and the U-CF1
disambiguation; it does not itself modify the estimator.

Authoritative rulings referenced: **D68**, **D72**, **D74** (see
`.orchestration/DECISIONS.md`). Empirical inflation targets: the U-FS1 correction
study, `crates/pnt-studies/src/correction.rs` (`STUDY.md` / `results.json`).

Synthetic-derived numbers are marked **[UNVERIFIED]** — they are the correct
initial calibration and validation target, to be re-measured against real SoOP
data before flight.

---

## 1. Problem statement

Over long GPS-denied LEO-Doppler legs the production `Executive` + `FilterStub`
EKF is **covariance-inconsistent**, and the inconsistency is **state-group-specific**:

- **D68** (verified, the crux result): position covariance converges and stays
  bounded (~50–160 m reported sigma) while the true horizontal error runs 7–70×
  larger. The km-scale true error is therefore **not** a physics/geometry/clock
  floor — a real floor would present km-scale *covariance*. It is filter
  **overconfidence**: an estimation-consistency defect, software-fixable in the
  estimator. The point estimate is fine; the covariance is miscalibrated.
- **D72** (verified NEES decomposition, the diagnosis): the overconfidence is
  **localized to POSITION (~55× denied-late NEES/dof) and VELOCITY (~47×)**,
  while **clock-drift and heading are PESSIMISTIC** (NEES/dof < 1 — the opposite
  problem). The overconfidence grows smoothly with denial time. The
  position–clock cross-term is ruled out (|corr| ≈ 0.04); the defect lives in the
  position/velocity diagonal blocks. Because two groups are over- and two are
  under-confident, **the fix must be group-specific, not a global covariance
  scale.**
- **D74** (verified, decisive negative): per-SV bias retirement across handover
  was tested end-to-end. It bounds the augmented state dimension exactly as
  designed (nuisance count 8→70 OFF becomes 8→11 ON) but leaves the
  position/velocity overconfidence **bit-identical** (pos NEES 165.716 both arms,
  true error 3239 m both, sigma 326 m both). The D72 0.99 bias-count↔NEES
  collinearity was denial-time confounding, not causation. **Retirement is ruled
  out as the causal fix** — see §3(c).

This is a **software calibration defect in the estimator**, not hardware,
geometry, clock physics, or the Doppler measurement model. It is the project's
gating problem (D68/D69): the mechanism of GPS-denied LEO-Doppler navigation
works (~100 m observable), but the EKF reports a covariance far tighter than the
error it actually carries on long denied legs.

---

## 2. What the fix must achieve (acceptance criterion)

The **U-CD1 consistency study (`crates/pnt-studies/src/consistency.rs`) is the
acceptance-test harness.** It drives the real production `Executive` +
`FilterStub` through the public API and computes per-group NEES vs generator
truth over the denied leg. The fix is accepted when, re-running U-CD1 before/after:

1. **Position and velocity denied-late NEES/dof return into the U-CD1 two-sided
   95% consistency band** (`GroupSummary.consistency_lower/upper`), i.e. their
   `denied_late_overconfidence_factor` moves from ~55×/~47× to ≈ 1 and the
   `verdict` reads `CONSISTENT`.
2. **The point estimate is not degraded.** True denied horizontal error
   (`EpochAggregate.mean_horizontal_error_m`) is unchanged within noise. This is a
   covariance-calibration fix, not an estimate change — a correct fix touches only
   `P`, never the state trajectory `x`.
3. **Clock-drift and heading are not pushed further out of band.** They are
   already pessimistic (NEES/dof < 1); the correction must leave them essentially
   untouched. A global inflation is therefore explicitly disqualified.

A single global covariance scale cannot satisfy (1) and (3) simultaneously — this
is the core reason the fix is group-specific.

---

## 3. Candidate corrections, trade-offs, recommendation

### (a) Group-specific process-noise (Q) retuning — *the principled fix*

Increase the process-noise fed into the position/velocity blocks during
propagation so the predicted covariance grows to honestly reflect the
weakly-observable Doppler-only regime, instead of collapsing tighter than the
true error.

- **Where:** `ProcessNoise.acceleration_variance` (`pnt-estimator`), which drives
  the position/velocity `Q` block in `FilterStub::propagate` (see §4).
- **Pro:** principled — it fixes the *mechanism* (the prior decays and the
  propagation under-feeds `Q` as observability weakens), reshapes the covariance
  *over time* (not just at one operating point), and is the standard EKF-tuning
  remedy. It naturally addresses the smooth NEES-vs-time growth D72 measured.
- **Con:** a scalar bump to `acceleration_variance` is coarse; the required
  inflation grows with denial time, so a constant `Q` bump may over-inflate early
  and under-inflate late. It also couples position and velocity through the
  kinematic `Q` structure (dt³/3, dt²/2, dt), so the two groups cannot be tuned
  fully independently by this lever alone. Requires iteration against U-CD1.
- **Risk to (2):** low — larger `Q` changes only the covariance and the Kalman
  gain, not the truth-tracking of the estimate in the mean.

### (b) NEES-consistency covariance correction — *empirical, group-specific inflation*

Apply a direct multiplicative inflation to the position and velocity covariance
sub-blocks, keyed to the measured overconfidence factor. This exploits the exact
NEES identity `NEES = eᵀP⁻¹e`: scaling a group's covariance sub-block by scalar
`s` divides that group's NEES by `s`, so the inflation that restores calibration
in the mean is exactly `s_g = (denied-late mean group NEES)/dof`.

- **Empirical targets (from the U-FS1 correction study, default 60-min/8-seed)
  [UNVERIFIED]:**
  - Position: denied-late NEES/dof ≈ **55.2×** → covariance ×**55.2**, i.e.
    **sigma ×7.43**.
  - Velocity: denied-late NEES/dof ≈ **46.9×** → covariance ×**46.9**, i.e.
    **sigma ×6.85**.
  - Clock-drift (factor ≈ 0.24) and heading (factor ≈ 0.004): pessimistic —
    **do not inflate**.
  (See the correction study's `recommended_inflation`; full table in §6.)
- **Pro:** mechanism-agnostic, directly and exactly restores the denied-late
  *mean* NEES to the dof by construction, group-specific by design (leaves
  clock/heading alone), and gives the implementer a concrete number to apply
  immediately.
- **Con:** a *static* scalar restores the mean but — per the correction study's
  distribution-restoration test — does **not** necessarily restore the full
  per-sample NEES *distribution* (the defect has a time/shape component, not pure
  scale). It is a calibration patch, not a mechanism fix; if applied as a blanket
  inflation it can leave the covariance mis-shaped over the leg.
- **Risk to (2):** low — it scales `P` only.

### (c) Per-SV bias retirement — *documented and REJECTED as the primary lever*

Call `retire_satellite_bias` in the running pipeline when an SV sets / at
handover, to bound the never-retired augmented nuisance-bias null-space (count
grows 8→70 over a denied leg).

- **Rejected because (D74):** tested end-to-end, it bounds the state dimension
  (8→11) but leaves position/velocity NEES **bit-identical** — the marginalized
  per-SV Doppler biases carry negligible cross-covariance with the core
  position/velocity block. It does **not** move consistency at all.
- **Retain anyway, config-gated, default-off:** it is a free
  numerical/compute/endurance benefit (bounds state growth at zero
  accuracy/consistency cost). It is simply **not** the consistency fix.

### Recommendation

**Combine (a) and (b): Q-retuning (a) as the principled fix, calibrated and
validated by the empirical inflation factors from (b).**

1. Use the correction study's group inflation factors (§6) as the **initial
   calibration target**: they say exactly how much the position/velocity
   covariance must grow in the denied-late steady state.
2. Implement the growth via **group-specific process-noise retuning** on the
   position/velocity blocks (principled, time-shaped) rather than a static
   post-hoc inflation, because the correction study shows a static scalar restores
   the *mean* but not the *distribution* — a time-varying `Q` that reshapes the
   covariance across the leg is needed to also fix the shape.
3. Optionally ship the exact NEES-consistency inflation (b) as an interim/backstop
   correction if a clean `Q` retune proves hard to converge — it guarantees the
   mean-NEES acceptance criterion even if the distribution is only partially
   restored.
4. Do **not** rely on retirement (c) for consistency; keep it as the endurance
   nicety it is.

Validate every iteration against U-CD1 (§2) and confirm true error unchanged.

---

## 4. Exact touch-points (map for the implementer — do NOT edit these here)

Error-state layout (both files assume it): position 0–2, velocity 3–5, heading 6,
clock bias 7, clock drift 8; per-SV transmit-frequency nuisance biases augmented
after index 8.

**`crates/pnt-estimator/src/lib.rs`:**

- `struct ProcessNoise` (fields `acceleration_variance` = 0.04,
  `turn_rate_variance` = 1e-4, `clock_drift_variance` = 1e-4,
  `nuisance_random_walk_variance` = 1e-6). The position/velocity lever is
  **`acceleration_variance`**. For a group-specific retune, this is the field to
  raise — and, if position and velocity need decoupled tuning, the struct is where
  a separate position/velocity process-noise term would be added.
- `impl Estimator for FilterStub::propagate` — the covariance propagation
  `self.covariance = &f * &self.covariance * f.transpose() + q`. The `q` block for
  position/velocity is built from `acceleration_variance` as
  `q[(POS+a,POS+a)] = accel·dt³/3`, `q[(POS+a,VEL+a)] = accel·dt²/2`,
  `q[(VEL+a,VEL+a)] = accel·dt`. **This is where Q-retuning (a) lands**, and where
  a group-specific NEES-consistency inflation (b) would be applied to the
  position/velocity diagonal sub-blocks of `self.covariance` (post-propagation,
  before/after the symmetrization `(&cov + cov.T)*0.5`).
- `FilterStub::covariance()` / `state()` — public read API the U-CD1 harness uses;
  keep them intact so the acceptance test keeps working.
- `retire_satellite_bias` (exists, **uncalled** in the production pipeline) and
  `augment_satellite_bias` — the rejected lever (c). If wired, gate it config-off.
- `cap_clock_bias_variance` / `CLOCK_BIAS_VARIANCE_CAP_M2` — clock block; **out of
  scope** for this fix (clock-drift is pessimistic, not overconfident).

**`crates/fusion-executive/src/lib.rs`:**

- The Doppler update construction sets **`satellite_bias_variance_mps2: 100.0`**
  (hardcoded) in the `DopplerRangeRateUpdate`. This is the initial per-SV nuisance
  variance; relevant to the augmentation null-space (c) but D74 showed it is not
  the consistency lever. Note it here so the implementer does not chase it.
- `Executive::new(... FilterStub::new(1.0, ProcessNoise::default()) ...)`
  construction path — if the fix introduces new/retuned `ProcessNoise` fields or a
  config-gated consistency-correction flag, this is where the executive would pass
  them through.

No other crates are touched by the consistency fix itself.

---

## 5. Coordination note (serial landing, one-owner-per-file)

This fix edits `pnt-estimator` and `fusion-executive`, which are **shared with an
external contributor's open PRs** (PR #1 Huber robust cost; PR #2 smoother /
executive guard reseed). Per the one-owner-per-file discipline (D71/D75/D77) this
work **must land serially after those PRs merge, not in parallel** — the
consistency fix operates in the same estimator/executive layer as PR #2's smoother
and reseed guard. Sequencing/ownership is a user + external-contributor decision.
The U-CD1 harness and this spec are the non-colliding deliverables produced ahead
of that serial landing; they do not touch the shared files.

---

## 6. Validation plan

**Harness:** U-CD1 consistency study (`consistency::run`) — run it **before** and
**after** the fix.

**Empirical targets & post-hoc validation:** the U-FS1 correction study
(`crates/pnt-studies/src/correction.rs`, `correction::run`) computes the
group-specific inflation the fix must produce, and validates that a scalar group
inflation restores the denied-late mean by the NEES identity. Its outputs:

- `recommended_inflation`: the concrete position/velocity covariance scale `s_g`
  and sigma scale `√s_g` to hit.
- `corrected_mean_in_band`: confirms the recommended inflation lands the corrected
  denied-late **mean** NEES inside the 95% band (in-band by construction of the
  mean — the identity `NEES/s_g` divides the mean by `s_g = mean/dof`).
- `corrected_in_interval_fraction` vs the nominal 0.95: the
  **distribution-restoration** check — whether a single scalar also fixes the
  per-sample distribution shape or only the mean.

**Empirical numbers [UNVERIFIED]** (from the correction study default 60-min,
8-seed config, denied-late window, 320 pooled per-sample points; see its
`STUDY.md` / `results.json` for the authoritative values):

| group | dof | denied-late NEES/dof (factor) | covariance ×s_g | sigma ×√s_g | corrected per-sample coverage (nominal 0.95) |
|---|---:|---:|---:|---:|---:|
| position | 3 | 55.24× | 55.24 | 7.43 | 0.80 (raw 0.14) |
| velocity | 3 | 46.92× | 46.92 | 6.85 | 0.78 (raw 0.16) |
| clock-drift | 1 | 0.24 (pessimistic) | — do not inflate — | — | 0.97 |
| heading | 1 | 0.004 (pessimistic) | — do not inflate — | — | 1.00 |

**Honest read of the distribution test:** the recommended scalar inflation lands
the denied-late **mean** NEES exactly in-band (by construction of the identity),
but the **distribution is only partially restored** — corrected per-sample
coverage rises to ~0.80 (position) / ~0.78 (velocity), still well below the
nominal 0.95. `scalar_restores_distribution = false`. The defect is therefore
**not pure scale**; it has a time/shape component, which is precisely why the
recommendation (§3) is Q-retuning (time-shaped) calibrated to these factors rather
than a static blanket inflation. Inflating clock-drift or heading would push their
already-pessimistic coverage further from nominal — confirming the correction must
be group-specific.

**Acceptance run sequence:**

1. Baseline: `consistency::run` → record position/velocity
   `denied_late_overconfidence_factor` (expect ~55×/~47×) and
   `mean_horizontal_error_m`.
2. Apply the group-specific Q-retune (§3a), calibrated to the correction study's
   `recommended_inflation` (§3b targets).
3. Re-run `consistency::run` → confirm (i) position/velocity `verdict` =
   `CONSISTENT` (factor ≈ 1, within the 95% band), (ii) `mean_horizontal_error_m`
   unchanged within noise, (iii) clock-drift/heading not pushed further out of
   band.
4. If the correction study reports `scalar_restores_distribution = false`, a
   static inflation is insufficient for the *distribution*; prefer the
   time-shaped Q-retune and re-check the per-sample coverage, not just the mean.

**Honesty discipline:** report whatever U-CD1 shows after the fix. Do not clamp,
target-fit, or tune to the acceptance number by editing the study. The point
estimate must be demonstrably unchanged; if it moves, the "fix" changed `x` and is
wrong.
