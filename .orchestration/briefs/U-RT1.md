# Brief U-RT1 — Real-constellation geometry realism study

Contract v5.1. Worktree branch `unit/U-RT1`. Commit there; never merge; NO trailers. Read
first: DECISIONS.md D51/D55/D57/D62, crates/pnt-studies (multisat module — the pattern and the
result to realism-check: N=8 mean 116m/p95 554m on a SYNTHETIC Walker fixture), crates/pnt-ephemeris
(TLE/SGP4 loading), the new fixture crates/pnt-ephemeris/tests/fixtures/real/constellations-2026-204.tle
(40 real published Starlink/OneWeb/Iridium TLEs — [UNVERIFIED] grok-fetched), docs/research/R4-signal-structures.md
(published per-constellation orbital params to cross-check against).

## Goal (NEW pnt-studies module `realtle` — do NOT edit multisat/highspeed/endurance modules):
1. **VALIDATE the fixture first** (D5 discipline): load the real TLEs via the sgp4 crate; assert
   they parse and propagate; check each constellation's inclination matches published shells
   (Starlink ~53deg, OneWeb ~87.9deg, Iridium ~86.4deg from R4) within tolerance; report how many
   parsed/were usable. Mark provenance clearly: real published elements, grok-fetched, not
   independently confirmed vs CelesTrak — physically valid is what the study relies on, not
   exact currency.
2. **Realism study**: rerun the multisat geometry experiment (real Executive+EKF, production
   gate, single-vs-multi isolation, injected clock drift + per-SV bias, multi-seed) but with
   the REAL constellation TLEs instead of the synthetic Walker fixture — same honest method.
   Compare the denied position p50/p95 vs N against the synthetic multisat result (116/554):
   does real orbital geometry give similar, better, or worse observability? Report GDOP with
   real vs synthetic geometry. This is a realism CHECK, not a new headline.
3. **Honesty**: production gate on, real filter vs truth, no formula/clamp, no target-fitting;
   if real geometry is materially different from synthetic, say so and diagnose (GDOP, visible
   count, inclination diversity). [UNVERIFIED] on the TLE currency and the synthetic clock/noise.
4. Tests: TLE-validation test, determinism, gate on, workspace gate green.

## Files owned
crates/pnt-studies/src/realtle/** + bin + mod/member lines ONLY, docs/studies/realtle/**,
.orchestration/reports/U-RT1.md. (Note: U-ST1 concurrently adds an `endurance` module to
pnt-studies lib.rs/Cargo.toml — expect a union-merge conflict there at integration, resolved
by keeping both module declarations.)

## Report
TLE-validation result (how many usable, inclinations confirmed), the real-vs-synthetic geometry
comparison, the realism verdict on the 116/554 numbers, [UNVERIFIED] list.
