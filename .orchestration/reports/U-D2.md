# U-D2 report — documentation-currency pass

Docs-only. No code, no crate READMEs, no `DESIGN_BASELINE.md`/`SAFETY_CASE.md`/
`CONTRACTS.md` touched. No git commit made (per brief).

## What was written

1. **`README.md`** (new, repo root): what the repo is (per
   `docs/HANDOFF_PROMPT_BLADERF.md`), the document hierarchy (DESIGN_BASELINE normative →
   ARCHITECTURE → SAFETY_CASE → PARAMS_PROPOSAL → BOM → research docs → `.orchestration/`),
   the crate map (one line each for all twelve `crates/*` plus `tools/mavlink_bridge` and
   `tools/sitl`), how to run the Rust gates (`cargo test`/`clippy`/`fmt`) and the Python
   gates via `tools/.venv` (pointer to the U-M1/U-M1.1 worktree note for provisioning
   detail, since that venv does not exist in this worktree), how to run `mission-study`, an
   honest status paragraph (research skeleton, synthetic end-to-end only, per D27), the
   specific open items before sea trial pulled from D27 and D39, and a pointer to
   `.orchestration/` for the review/orchestration record.
2. **`docs/design/ARCHITECTURE.md`**: appended (did not rewrite the reviewed body) an
   "Implementation status (2026-07-23)" section mapping all 13 architecture modules to their
   shipped crate/state, plus a `pnt-mission` capstone note and a "known deviations already
   ruled" subsection covering the gate-boundary (U-I2 F6) and linearisation-relocation (D26)
   points named in the task.
3. **`.orchestration/PLAN.md`**: appended a "Wave 5 / 6 — final units" table row set:
   U-T1 (D36), U-R3 (D35), U-R4 (unreviewed research unit, D5/D9 stdout-contract gate — no
   decision number closes it because research units are never dual-reviewed per D5/D9), U-E2
   (D38), U-N1 (D38), U-I3 (D39), and U-D2 (this unit).

## Sources checked (for the accuracy bar)

- `.orchestration/DECISIONS.md` D27-D39 in full (also skimmed D1-D26 for terms used, e.g.
  D10 Orbcomm receiver-clock exclusion referenced only indirectly).
- `.orchestration/PLAN.md` (existing unit table, wave 4 tail).
- `docs/design/ARCHITECTURE.md` (full reviewed body) and `docs/HANDOFF_PROMPT_BLADERF.md`
  (full).
- `ls crates/` and every crate's `src/lib.rs` top doc-comment; README.md for the four crates
  that have one (`pnt-journal`, `pnt-mission`, `pnt-replay`, `pnt-tracker`).
- `tools/mavlink_bridge/README.md`, `tools/sitl/README.md`, `tools/mavlink_bridge/pyproject.toml`.
- `Cargo.toml` (workspace member list, confirms all 12 crates including `pnt-tracker`/
  `pnt-mission` are wired into the build).
- Code checks used to make specific claims verifiable rather than aspirational:
  - `crates/fusion-executive/src/lib.rs:52` — default elevation mask
    `elevation_mask_rad: Some(5.0_f64.to_radians())` (confirms the 5-degree default cited in
    the task and in D25's ruling).
  - `crates/fusion-executive/src/lib.rs:356-390` — two `Executive` constructors: one over
    `IntegrityStub` (fail-open placeholder), one over `AuthoritySupervisor::fail_closed`
    (the real, default production path per D33).
  - `crates/pnt-integrity/src/lib.rs` — `IntegrityStub` (line 76) and `AuthoritySupervisor`
    (line 223) both exist; confirms the stub was not deleted, it is a deliberately retained
    test/tooling placeholder, not the production default.
  - `crates/pnt-estimator/src/lib.rs` — `chi_square_threshold: Option<f64>` and the NIS
    accept/reject computation (`gate.is_none_or(...)`, lines ~340-355), confirming the
    integrity gate's chi-square logic lives in the estimator, not a standalone crate.
  - `.orchestration/reports/U-I2-review-opus.md` line 7 — exact source text for "F6 info
    gate-in-estimator boundary accepted", used verbatim as the citation for the known
    deviation.
  - `.orchestration/reports/U-I2.md` "U-I2.1 fix-round dispositions" item 1 — source for the
    linearisation-relocation-to-`pnt-predictor` claim (D26).
  - `.orchestration/reports/U-I3.md` (including its U-I3.1 fix-round table) — source for the
    exact four-way attribution numbers referenced in the mission-study README pointer and
    for confirming D39's Doppler-assimilation-in-replay claim.
  - `grep` for `.venv` across `.orchestration/reports/U-M1*.log` — confirms `tools/.venv` is
    the real, previously-used path for the Python gates (worktree `leo-pnt-wt-UM1`, not
    present in this worktree — README says so rather than claiming a venv that does not
    exist here).

## Accuracy notes / self-check

- Every module-status row in the ARCHITECTURE addendum is backed by either a DECISIONS.md
  entry number or a direct code/README read; no row states a capability not evidenced by one
  of those.
- Module 4 (SDR capture) and part of module 3 (physical sensor adapters) are stated as
  **not built** — this is honest-open, not aspirational; nothing in the crate tree or the
  decision log claims a live bladeRF or physical device adapter exists.
- The elevation-mask default is marked `[UNVERIFIED]` (tuning value), matching the source
  code comment and D25's ruling ("real mask value, `[UNVERIFIED]` tuning").
- `PARAMS_PROPOSAL.md` is referred to only as "PROPOSED — NOT FROZEN" (its own status line),
  never as settled.
- README's "what remains before sea trial" list is drawn directly from D27's ship-record
  sentence and D39's open tuning-study item; no items were invented beyond those two
  sources.

## Fix round (single-seat verification FAIL, six findings, all addressed)

1. **README.md false citation removed.** `.orchestration/reports/U-M1.1.md` does not exist
   (verified: the fix round's dispositions live inside `U-M1.md` itself, which states "Fix
   round: U-M1.1" in its header — confirmed by reading the file). Replaced with real
   provisioning commands derived directly from the tree: `tools/mavlink_bridge/pyproject.toml`
   (`pymap3d==3.2.0`, `pymavlink==2.4.43`, installed via `pip install -e tools/mavlink_bridge`,
   reusing the exact `DISABLE_MAVNATIVE=1` invocation already documented in
   `tools/mavlink_bridge/README.md`) and `tools/sitl/requirements-build.txt`
   (`empy==3.3.4`, `pexpect==4.9.0`, `dronecan==1.0.27`, `setuptools==80.10.2`, installed via
   `pip install -r tools/sitl/requirements-build.txt`). Every package name in the new README
   text was checked against these two files before being written. `pytest`/`ruff` are used by
   the gates but are pinned in neither file — stated as such rather than presented as
   repo-pinned.
2. **README.md pytest/ruff lines now runnable from a fresh checkout** following the new
   provisioning steps above (`python3 -m venv tools/.venv` first).
3. Same fix as (2); both gate lines sit under the new provisioning block.
4. **SITL note fixed**: states `tools/sitl/build.sh` is one-time and long-running (full
   ArduPilot waf build) and must run, using the same `tools/.venv`, before
   `tools/sitl/run.sh` will work.
5. **PLAN.md U-E2 row**: changed citation to `U-E2.1 applied (commit 9b8cf52, gate closed
   D38)` — verified `9b8cf52` is a real merge commit in this repo's history
   (`git log --oneline`: "Integrate tracker into synthetic mission pass", touching
   `crates/pnt-mission`, `crates/pnt-tracker`, `.orchestration/reports/U-E2.md`), matching
   D38's U-E2 gate-closure ruling. D37 is not cited for this line; D37 is the ruling that
   *ordered* the U-E2.1 follow-up, not evidence of its completion.
6. **PLAN.md U-D2 row**: changed to `fix round after single-seat verification`.

**LOW finding also fixed**: README.md's BOM hierarchy line now states the EU pricing was
verified live as of BOM.md's own stated access date, **2026-07-22** (confirmed by reading
`docs/design/BOM.md` line 7: "Access date for all live web claims in this document:
**2026-07-22**"), and flags currency beyond that date as `[UNVERIFIED]`.

### One deviation from the coordinator's literal wording, flagged rather than silently applied

Findings (2)+(3) asked me to "state clearly that `tools/.venv` is gitignored." I checked
this against the tree before writing it: `git check-ignore -v tools/.venv/bin/pip` returns
exit 1 (not ignored), and neither the root `.gitignore` (`target/` only) nor
`tools/mavlink_bridge/.gitignore` nor `tools/sitl/.gitignore` mentions `.venv`. Asserting
"gitignored" would itself have been an unverifiable/false claim under this unit's own
accuracy bar. I wrote the true, substantively equivalent fact instead — "It is not
committed to the repository and does not exist in a fresh checkout" — which supports the
same instruction (provision before use) without asserting something the tree contradicts.
Flagging this for the coordinator/reviewer rather than either silently complying or silently
diverging.
