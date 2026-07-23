## Adversarial review: U-MS1 multi-satellite fixture study — FAIL

Worktree `/home/od/work/leo-pnt-wt-UMS1`, branch `unit/U-MS1`, commit `58c8834`. Worktree left **clean** (`git status` empty, HEAD 58c8834), no stray processes. Study reproduced from source.

### The good news first (mechanically honest — these all hold)
The pipeline itself is clean, and this matters:
- **No truth leakage.** The Doppler EKF update predicts range-rate and the Jacobian from the filter's *own* estimated state (`fusion-executive/src/lib.rs:273-306`), and the measured range-rate is derived from the observation's `correlation_peak_hz` (`:293-294`). The measurement is a legitimate function of truth+noise; the innovation is real. Noise draws (`rng.symmetric()*0.5`) are independent per call.
- **No clamped estimator, no formula outputs.** Real `FilterStub` (constant-velocity + IMU integration, `pnt-estimator/src/lib.rs:414-470`), same one as the highspeed study. Every error is `norm(filter_state, truth)` (`multisat.rs:333-336`).
- **Production gate genuinely on.** `chi_square_threshold = Some(9.0)` (`multisat.rs:194`); rejections are nonzero in every scenario (2–16). Verified against journal `IntegrityEvent` reasons.
- **Real elevation mask.** 5° mask via `ecef_to_enu_rotation` (`multisat.rs:403-412`); 45–54 of 960 sats visible ≈ 5%, geometrically plausible; below-horizon excluded. Walker-like fixture with genuine RAAN/anomaly diversity, `[UNVERIFIED]`-marked, shells cited.
- **Determinism & reproducibility: exact.** Two fresh runs byte-identical; both byte-identical to committed `results.json` *and* `STUDY.md`. Every headline number (33.8 / 6.1 / 2.4 / 11.4 m, etc.) reproduces exactly.
- **No attribution trailers** anywhere (commit author `halo24-worker`, clean body); `error_class` honestly includes DIVERGED.

### Why it still FAILs

**F1 — Gate is RED (HIGH / certain).** `cargo clippy --all-targets -- -D warnings` (the README gate) fails with 3 errors, all in the newly added file:
- `multisat.rs:108` — `run()` returns `Result`, missing `# Errors` doc section
- `multisat.rs:225` — `u64 as i64` cast may wrap
- `multisat.rs:324` — manual `is_multiple_of`

No `rust-toolchain` pin exists; on the installed toolchain (rustc/clippy 1.97.1) the gate does not pass. The brief requires "whole-workspace gate green." Verify: `cargo clippy --all-targets -- -D warnings`. (Lib tests pass, 15/15; fmt passes. It is specifically clippy.)

**F2 — The headline is confounded by a maximally benign trajectory (HIGH / high confidence).** Truth is a perfectly constant-velocity straight line (`multisat.rs:414-418`) that the constant-velocity `FilterStub` models with *zero* dynamics error. There is no manoeuvre, no wave/slam, no coordinated turn. The D51/D52 single-sat study that produced tens-of-km used the real mission generator *with* a 90° coordinated turn + wave/slam + speed-scaled IMU bias (`highspeed.rs:220-241`). Here, position stays bounded chiefly because it starts at truth (GNSS-aided to ±0.5 m), velocity is Doppler-aided, and the dynamics model is exact — i.e. near-perfect dead-reckoning of a benign track, not strong position observability. The claim "makes DENIED position observable at the 100–200 m class, closing D51" overreaches for this scenario. Verify: the only denied-phase perturbation is the tiny IMU stressor `2.0e-5 m/s²` (`multisat.rs:235`); a manoeuvre — the thing that breaks single-sat — is absent.

**F3 — The D54-required single-vs-multi geometry isolation was NOT performed (HIGH / high confidence).** D54 mandates "single/multi-sat comparison to isolate the geometry effect." But *every* tier, including N=1, hands over among 45+ visible satellites, so N=1 carries temporal LOS diversity (46–295 nuisance SVs recorded). There is no fixed-single-LOS baseline. Consequently the N-sweep cannot attribute the result to simultaneous geometry vs temporal diversity vs benign dynamics — it "proves nothing about single-vs-multi," exactly the risk flagged. The report *admits* "N=1 is not a reproduction of D51's fixed single-ISS fixture" (`STUDY.md:37`) — which is an honest admission that the required experiment was not run, yet the headline still leads with "reached <=200 m with 1 requested satellite" (`multisat.rs:127-135` picks `min(requested)` = the confounded N=1 case).

**F4 — Undisclosed idealization: receiver clock and transmit bias are exactly zero (MEDIUM / high confidence).** `clock_drift_mps` is hard-set to `0.0` in both measurement generation (`multisat.rs:~290`) and filter prediction (`fusion-executive:284,303`), and the per-SV transmit bias is `0.0` in the generated truth (`predict(... , 0.0, ...)`, `multisat.rs:~291`). Unknown receiver-clock drift is the dominant real nuisance in Doppler-only positioning; zeroing it (and the SV bias) is a large observability gift. The per-SV nuisance augmentation is exercised structurally but has essentially nothing to estimate. STUDY.md's `[UNVERIFIED]` list (`:39`) covers orbits/noise/cadence/track but omits this. Combined with F2, the measurement environment is idealized well beyond the orbital caveat — reflected in the gate rejecting only ~1.5% (exactly 2 per satellite-stream), because the measurements are so clean.

**F5 — Reconciliation of the apparent D51 contradiction: it is a confounded comparison, not a refutation and not leakage (informational).** N=1 reaching 34 m does not contradict D51's tens-of-km, because ≥4 variables changed at once: benign vs manoeuvring truth (F2), 30 s vs 1800 s Doppler cadence (60× more updates), handover-among-45 vs fixed-ISS (F3), and zeroed clock/bias (F4). D51 stands; this study's "N=1" is simply not a controlled comparison to it.

**F6 — Headline rests on one deterministic draw (LOW).** The N-sweep is non-monotonic (leg: N=1 34 m, N=2 75 m, N=4 65 m, N=8 6 m). The report correctly says "one synthetic seed is not an accuracy distribution" (`STUDY.md:37`) — but then the machine-generated headline still asserts the class was reached "with 1 satellite," the weakest-isolated, luckiest point.

**F7 — Mislabel (LOW).** "D45 100 km … passage" imports the D45 passage association while dropping its manoeuvre/wave content; it is a bare constant-velocity leg. Partly mitigated by the "constant-heading" wording.

### Bottom line
The plumbing is honest — real EKF, real gate, no leakage/clamping/formula, fully reproducible, well-caveated on orbital synthesis. That is genuine and worth keeping. But the *scientific headline* the project needs — "multi-satellite LOS diversity makes denied position observable at 100–200 m, closing the D51 single-sat limitation" — is **not** honestly demonstrated here: the study runs the most benign possible trajectory with a zeroed receiver clock, never builds the single-fixed-LOS baseline D54 demanded, and its own N=1 result (which drives the headline) is confounded rather than a clean multi-sat proof. On top of that the clippy gate is red, an objective violation of an explicit deliverable.

Route back: (1) fix the 3 clippy errors; (2) add a true fixed-single-LOS baseline and a manoeuvring/coordinated-turn leg; (3) inject a realistic unknown receiver-clock drift (and per-SV bias) so the nuisance augmentation is actually stressed; (4) run multiple seeds before stating any class, and reframe the headline around N=8 with those controls rather than the confounded N=1.

**FAIL**