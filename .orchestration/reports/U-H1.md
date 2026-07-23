# U-H1 — 20-knot / extended-passage envelope analysis

Unit: U-H1 (Opus seat, deep engineering analysis) · 2026-07-23 · requirements `DECISIONS.md` D46 (20 kn) + D47 (30 kn exploratory)
Deliverable: `docs/design/HIGH_SPEED_ENVELOPE.md` (subordinate to `DESIGN_BASELINE.md`).
Files owned: `docs/design/HIGH_SPEED_ENVELOPE.md`, `.orchestration/reports/U-H1.md` (these two only).
No code, no commit, no web research (gaps marked `[UNVERIFIED]`).

## Fix-round history

- **Draft 1** (D46) — 20 kn envelope. **Draft 2** (D47) — added 30 kn §6.
- **Draft 3 (this) — fix round after dual review FAIL** (`U-H1-review-sol.md` codex deep seat;
  `U-H1-triage.md` grok, advisory) **plus REQUIRED incorporation of `docs/research/R5-highspeed-dynamics.md`**
  (grok sourced research). The document was **rewritten wholesale** to replace assumed constants with R5's
  sourced values, add an R5 reconciliation table (§0.1), scope every conclusion as evidence-supported vs
  judgment, and re-derive the verdicts. Each review finding is addressed inline (finding numbers cited).

## What changed in the fix round (the load-bearing corrections)

Both reviews were substantially correct; I verified each against R5 and the repo studies before adopting.

1. **Vibration-rectification "isolation MANDATORY" — WITHDRAWN [SOFTENED].** Draft rested on assumed
   `a_rms = 3 g`. R5's *measured* RMS is **0.44 g** (MK V head seas); 3 g is a peak/A₁-class statistic, not
   RMS. At 0.44 g the rectified bias (hypothetical Γ₂=1e-11/g²) is **5.8e-4 m/s = ~48× UNDER** the denied PL.
   Rectification is now a *measure-it hypothesis*, not a proven integrity breach. (sol #1, triage F1 —
   confirmed by recompute.)
2. **Slam model corrected.** R5: impact durations **100–450 ms** (not 5–50 ms) → energy **< ~10 Hz**. This
   *inverts* two sub-claims: (a) low-freq slam gets *less* 1/(πfτ) suppression, not more (sol #3: my
   "sub-cm/s" was 1.9 cm/s and misapplied the sinusoid model); (b) isolation is *harder* — energy sits below
   light-isolator resonances, needs low-fn + large stroke (sol #2, triage F2).
3. **The real, sourced reference concern is the LINEAR g-sensitivity** (Γ≈1e-9/g, Wenzel/NIST via R5):
   per-slam transient velocity excursions 0.1–0.8 m/s (innovation-gated → availability/cycle-slip) + phase
   noise via R5's sourced L(f_v) formula. Isolation downgraded REQUIRED → **STRONGLY RECOMMENDED-and-hard**,
   and **split**: isolate the oscillator, keep the IMU **rigidly mounted** (R5/VN-100 vendor guidance;
   co-isolation degrades AHRS) — reversing draft's shared plate (sol #4, triage F9).
4. **Tracker "no block survives / structural" — WITHDRAWN [SOFTENED].** Study explicitly says the block
   sweep is "not a closed-form limit" on a ±4.08 kHz fixture band. Re-cast the heave+satellite stack as a
   conservative **screening bound**; with R5-sourced g (4.3–8.6 g) the 20 kn combined rate 5300–6900 Hz/s is
   *within* the 128-block tested point (8000), so shorter blocks plausibly cope (sol #6/#7, triage F3).
5. **Isolation dB corrected 10–15 → ~5–7 dB** (bias ∝ a², so 10·log₁₀(R)) (sol #5, triage F4).
6. **Timers: `1/v` rescale REPLACED by explicit budget.** R5 human-response floor **3–5 s**. 20 kn's implied
   3.5 s is already at/below the floor; the safe-speed cap is **NOT located at 20–30 kn** — on a 5 s floor it
   is ~14 kn, on 3 s ~23 kn. Supported claim narrowed to "no 30 kn denied authority; derive cap from budget,
   may be < 20 kn" (sol #8/#19, triage F5-adjacent). D46 dwell order now *analysed* via state-machine
   revocation-independence, not asserted (sol #9).
7. **Heading/position limits relabelled** acceptance → per-epoch authority PLs (100 m/2.5° denied, 12 m/1°
   aided); times unchanged by coincidence of halving (sol #10, triage F5).
8. **Good-fix timeline reconciled with `t_dr`** — authority expires at `t_dr` (~28–120 s), NOT the 2.5 min
   heading time; good fix buys start-accuracy + headroom, not authority duration. Added the open definitional
   question "which LEO obs resets t_dr" routed to contracts (sol #11, triage — and finding 21).
9. **"Speed improves velocity conditioning" — WITHDRAWN.** LOS Jacobian is speed-independent; larger Doppler
   ≠ more Fisher info (sol #15).
10. **Q "10²–10⁴×" removed** — direction/magnitude ungrounded; planing Q must come from selected-IMU residual
    spectrum, no multiplier asserted (sol #17).
11. **"One measurement clears all six" — WITHDRAWN**; §5.7 evidence matrix (necessary-not-sufficient) (sol #12).
12. **Added §5.6 missing high-speed consequences** (antenna/attitude/spray fades, RF multipath, power/EMI,
    crew factors, Hs coupling, speed-log ventilation) and **§5.5 architecture/contracts routing** (sol #20/#21,
    triage F15).
13. **30 kn §6 corrected throughout**: rectification "3–5× over PL" WITHDRAWN (sourced RMS → ~10× under);
    "no block survives" → screening bound exceeds *tested* envelope; isolation 5–7 dB; cap not at 20–30 kn.

## R5 disagreements reconciled (per D5; non-Grok arithmetic confirmation done)

- **Adopted from R5:** trim 2–4° (not 3–6°); RMS 0.44 g; peak 8.62 g / A₁/₁₀ 4.3 g (not "bow 10–20+ g");
  duration 100–450 ms; Γ≈1e-9/g linear; vib phase-noise formula; VN-100 rigid-mount + 4.5 g RMS saturation +
  VRE-unspecified; human floor 3–5 s; no small-craft crash-stop table.
- **Rejected from R5:** its §6.3 translational-Doppler figure ("~1.5 Hz Ku at 30 kn") — arithmetic slip;
  correct is 583 Hz Ku (v/c·f). Kept R5's *qualitative* point (attitude jitter/spray dominate). Confirmed by
  recompute.
- **R5 gaps (kept [UNVERIFIED]):** oscillator Γ₂ (not in R5; FE-5680A datasheet has no g-sens row);
  small-craft 20–30 kn slam-vs-Hs matrix; planing-spray satcom trial; VN VRE number.

## Key corrected numbers (all python-verified this round)

- Rectification at R5 0.44 g RMS: δ_DC=1.9e-12 → **5.8e-4 m/s (48× under PL)**; needs a_rms≈3 g RMS to reach PL.
- Linear-term per-slam residual (Γ=1e-9/g): 8.62 g inst 2.58 m/s → @1 Hz 0.82, @2 Hz 0.41, @5 Hz 0.16 m/s.
- cot(θ/2) at R5 trim: 2°→57×, 4°→29× (stronger than draft's 3–6°).
- Heave stack (sourced g): 4.3 g→5310 Hz/s, 8.62 g→6909 Hz/s (both +3718 sat); within 128-block 8000 tested pt.
- Isolation dB: R=3.3–4.9 → 10·log₁₀(R) = **5.1–6.9 dB**.
- Human-floor cap: D=v7·10s=36 m; floor 5 s→14.0 kn, floor 3 s→23.3 kn. Tack@20kn=3.5 s, @30kn=2.3 s.
- Authority-PL time (100 m/2.5°): 7 kn 637 s, 20 kn 223 s, 30 kn 149 s. (aided 12 m/1°: 191/67/45 s.)
- 30 kn: vessel Doppler Ku 583 Hz; scaled peaks ~16–19 g; heave stack ~9600–10900 Hz/s (exceeds 8000 tested);
  rectification (scaled 0.8–1 g RMS) ~0.002–0.003 m/s (~10× under PL).
- Passage: 100 km@30 kn=1.80 h; 500 km@20 kn=13.5 h; 20-min leg@20 kn=12.35 km; 24 h denied → 6 h to 30 h ceiling.

## Weakest-evidence list (most to least load-bearing)

1. **No measured hull slam/vibration/trim/manoeuvre environment** on the selected craft. R5's best data is an
   82 ft SOC at Hs 0.9 m, not a 6–12 m RIB at 20–30 kn; R5 found no small-craft speed-vs-Hs matrix, no
   crash-stop table, no planing-spray satcom trial. Gates most rows; necessary but not sufficient for any.
2. **Oscillator Γ, Γ₂** — not on FE-5680A datasheet; Γ₂ absent from R5 entirely. Needs shaker test.
3. **Human-response floor and craft manoeuvre budget** — R5's 3–5 s is mixed VERIFIED/ASSUMED; the cap
   location (14–23 kn) swings on it. No small-craft stopping trial exists.
4. **IMU VRE / gyro-g / hull a_rms vs 4.5 g saturation** — VN VRE unspecified by vendor.
5. **Slam-scaling exponent (v^1.5–v^2)** — judgment, not in R5; swings 30 kn peaks.
6. **Planing Q** — direction and magnitude both unestablished; needs selected-IMU residual spectrum.
7. **"Which LEO obs resets t_dr"** — open contracts definition; governs the good-fix scenario.
8. **U-P1 dependency** — 30 h ceiling / aging fit read from an in-progress unit's log.

## Routing summary (candidates only; U-H1 edits none of these files)

Baseline B-1..B-6; BOM M-1a/M-1b (split: isolate oscillator, rigid IMU)/M-2/M-3/M-4; Safety S-1..S-4; Params
P-1..P-3; **Architecture/contracts A-1..A-5** (t_dr-reset definition, class selector, IMU heave feed-forward,
isolated-frame extrinsics, clock-leakage monitor); **missing-consequence E-1..E-7** (antenna/spray/RF/power/
crew/Hs/speed-log); 30 kn H-1..H-6. All `[UNVERIFIED]`, fail-closed per SAFETY_CASE §1. Implementation +
synthetic 20 kn/24 h/500 km + 30 kn study is U-H2's scope.

## Verdicts (one paragraph each — see the doc for full text)

**20 kn:** NOT supportable on present evidence, but for narrower, better-sourced reasons than draft 1. The
frequency-reference case *softens* — vertical mounting is kept/stronger (29–57× at 2–4°) and the
rectification "mandatory isolation" claim is withdrawn (48× under PL at R5's 0.44 g); the real sourced concern
is linear-g availability/cycle-slip, isolation recommended-and-hard (oscillator only, rigid IMU). Genuine
blockers: unmeasured hull environment; IMU a_rms-vs-4.5 g-saturation + VRE/gyro-g unknown; collision timers
already at the 3–5 s human floor at 20 kn (cap may be < 20 kn); continuous denied position breached ~3.7 min
after each manoeuvre reset; planing Q retune. Extended passage is the easy part (500 km@20 kn=13.5 h; 24 h
denied fits 30 h ceiling with 6 h margin iff cached at departure). Fail closed to displacement; 20 kn
aided-only/unproven.

**30 kn (exploratory):** Do not grant denied autonomous authority; aided/manual-only. Corrected against R5 it
does NOT change the estimator/passage *class* (margins 1.5× tighter, helped by good-fix start) and is NOT the
hardware class-change draft 1 claimed (rectification ~10× under PL at both speeds; extra isolation ~5–7 dB not
10–15; tracker "no block survives" withdrawn → screening bound exceeds *tested* envelope, needs heave-rate
aiding + replay). The one firm 30 kn result: distance-preserving T_ack≈2.3 s is below the 3–5 s floor. The
analysis does NOT locate the cap at 20–30 kn (could be ~14 kn). Scoping only, gated behind 20 kn clearance +
measured slam spectrum + H-1..H-6.
