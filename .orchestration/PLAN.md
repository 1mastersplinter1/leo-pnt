# PLAN.md — GPS-denied maritime PNT from LEO SoOP (bladeRF)

Coordinator seat: **Fable 5** (claude-fable-5) — verified per header self-check.
Source brief: `docs/HANDOFF_PROMPT_BLADERF.md` (normative input; design baseline will supersede it as the working reference).

Sizing: **Substantial** — full protocol.

User constraints for this effort:
- All work in this git repo (`/home/od/work/leo-pnt`).
- Minimize Fable token spend: Fable coordinates/reviews; drafting and implementation are delegated (see DECISIONS.md D2).

## Units

| Unit | Title | Author model | Mode | Depends on | Status |
|------|-------|--------------|------|-----------|--------|
| U-D1 | Design baseline + architecture draft | Sol (codex) | SERIAL (gates wave 2) | — | **done — approved (D11)** |
| U-R1 | bladeRF/LNB/host market + tooling research (live web) | Grok | PARALLEL | — | done (unreviewed, D5 gate) |
| U-R2 | Iridium STL, terrestrial SoOP, Danish regs research (live web) | Grok | PARALLEL | — | done (unreviewed, D5 gate; stdout reset D9) |
| U-C1 | CONTRACTS v2 (bus msgs, frames, time owner, gnss_authority) + Rust workspace scaffold + fusion executive skeleton | Sol (codex) | SERIAL | U-D1 reviewed | **done — merged (D15)** |
| U-E1 | Ephemeris propagator (SupGP/SGP4) + Doppler predictor | Sol (codex) | PARALLEL after U-C1 | U-C1 | in flight (worktree UE1) |
| U-F1 | EKF core: clock bias/drift states, numeric-vs-analytic Jacobian check | Sol (codex) | PARALLEL after U-C1 | U-C1 | in flight (worktree UF1) |
| U-M1 | MAVLink GPS_INPUT publisher + ArduPilot SITL harness (pinned firmware) | Grok | PARALLEL after U-C1 | U-C1 | in flight (worktree UM1) |
| U-B1 | Bill of materials, live-verified EU pricing | Sonnet | after U-R1/U-R2 | U-R1, U-R2 | **done — verified (D5 met)** |
| U-S1 | Safety case (authority grant/revoke, watchdog, backstop) | Opus | after U-D1 | U-D1 | fix round 1 (dual FAIL, D14) |
| U-I1 | Integration unit (merges, full build + smoke per merge) | Fable | SERIAL, one merge at a time | all code units | pending |

Review pipeline per header: dual non-author review incl. Opus for substantial units; Grok never sole reviewer; final gate = Fable + least-author frontier model.

Ordering rationale (from handoff failure modes): executive before modules (#4), predictor before reject gate (#6), estimator states only with measurement paths (#8).

## Availability notes
- codex-cli 0.144.5 and grok 0.2.106 verified working this session (round-trip test).
- Ruflo MCP tools not connected this session — worker spawn/routing via CLIs (header: ruflo is routing-only anyway).
