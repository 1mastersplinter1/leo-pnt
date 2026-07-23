# Brief U-V2 — Estimator consistency & the velocity-degradation investigation

Contract version: v5.1. Workspace: dedicated worktree, branch `unit/U-V2`. Commit here; never
merge to main. Read first: `crates/pnt-estimator/**` (the EKF), `crates/pnt-mission/**`
(mission/replay study machinery), `.orchestration/DECISIONS.md` D39 (the OPEN velocity
question this unit must answer), `docs/design/DESIGN_BASELINE.md` (10–20 min position
observability claim; velocity-instrument framing), `docs/design/PARAMS_PROPOSAL.md`
(NEES/coverage validation plans this unit begins executing).

## Goal
Extend `crates/pnt-studies` — coordinate with the concurrent U-V1 unit by owning ONLY these
files: `crates/pnt-studies/src/estimator/**` (your module), `src/bin/estimator-study.rs`,
`docs/studies/estimator/**`, your report. If pnt-studies' crate skeleton doesn't exist yet
in your branch, create the minimal crate + your module (merge is by-file; do not touch
tracker study files).

Studies (seeded, deterministic, JSON + STUDY.md):
1. **Filter consistency (NEES/NIS)**: on synthetic missions (multiple seeds/geometries),
   compute per-epoch NEES against truth and innovation NIS per measurement type; report
   chi-square coverage vs nominal (is the covariance honest, optimistic, or pessimistic —
   quantified). This is the wrong-Jacobian/wrong-covariance detector the handoff worries
   about, run as a statistical campaign.
2. **THE D39 QUESTION — why does Doppler degrade velocity?** Controlled experiments on the
   prior-initialized denied replay: sweep (a) Doppler measurement variance fed to the
   update, (b) velocity process noise, (c) pass geometry (multiple TLE epochs/offsets →
   different LOS evolution), (d) observation rate. Identify the mechanism (e.g. LOS-collinear
   velocity component absorbing range-rate residuals; stub-tuning mismatch; lever of the
   nuisance bias state) with EVIDENCE — show the velocity-error decomposition along/across
   LOS. Deliverable: either a tuning that removes the degradation (show the four-way table
   with it) or a demonstrated structural explanation with the fix routed to a named future
   unit. This closes or precisely scopes D39's open item.
3. **Position observability vs leg duration**: the handoff claims position emerges over
   10–20 min constant-heading legs. Sweep mission leg duration (2–30 min, several seeds):
   denied-mode (prior-only vs prior+Doppler) position error vs duration curve. Does the
   synthetic stack reproduce the claimed convergence shape? Manoeuvre-reset check: insert a
   turn mid-leg and show the convergence reset the handoff predicts (or report honestly
   that the stub filter cannot show it and why).
4. **Stale-ephemeris integrity**: generate Doppler from ephemeris propagated at epoch+Δ for
   Δ ∈ {0, 1h, 6h, 24h} while the replay pipeline uses the fresh ephemeris: measure induced
   innovation bias vs Δ and whether the chi-square gate (threshold 9) actually rejects at
   which staleness — evidence for/against the 6 h age-gate choice and the
   PARAMS_PROPOSAL's t_eph reasoning.

## Method
Same harness rules as U-V1 (deterministic, --quick flag, run the full study, commit JSON +
STUDY.md under docs/studies/estimator/). Gate: cargo test && cargo clippy --all-targets --
-D warnings && cargo fmt --all -- --check.

## Files owned
`crates/pnt-studies/src/estimator/**`, `crates/pnt-studies/src/bin/estimator-study.rs`,
minimal shared crate scaffolding ONLY if absent, `docs/studies/estimator/**`,
`.orchestration/reports/U-V2.md`. If you need pnt-mission/pnt-replay API additions, document
them in the report instead of editing those crates.

## Report
Consistency verdict (with numbers), the D39 answer (mechanism + evidence + fix-or-route),
observability-curve verdict vs the handoff claim, age-gate evidence, [UNVERIFIED] list.
