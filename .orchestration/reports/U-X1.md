# U-X1 — Coherent DF-array feasibility (unknown terrestrial emitters as a nav aid)

Unit: U-X1 (Opus seat, deep engineering analysis) · 2026-07-23 · requirement `DECISIONS.md` **D48**
Deliverable: `docs/design/EMITTER_ARRAY_FEASIBILITY.md` (subordinate to `DESIGN_BASELINE.md`).
Files owned: that document and this report (these two only). No code, no commit.

## Sources read

- `.orchestration/DECISIONS.md` — D48 (this brief), D50 (H1 verdicts, heading breach), D51/D52 (the
  single-satellite unobservability finding + U-MS1 routing), D5 (Grok claim discipline), D46/D47.
- `docs/research/R6-unknown-emitter-array.md` — Grok research input; treated `[UNVERIFIED-grok]` per D5.
- `docs/design/HIGH_SPEED_ENVELOPE.md` + `.orchestration/reports/U-H1.md` — the heading-at-speed and
  manoeuvre-reset findings this proposal is meant to address.
- `docs/design/DESIGN_BASELINE.md`, `docs/design/ARCHITECTURE.md` — the EKF, bus, degradation contract,
  and the "interferometry = open research option, not baseline" statement.
- `crates/pnt-estimator/src/lib.rs` — read the actual augmentation machinery: 9-state core
  (`POS 0–2, VEL 3–5, HEADING 6, CLOCK_BIAS 7, CLOCK_DRIFT 8`, `CORE_DIM=9`); `augment_state`/
  `remove_state` runtime resize of `x`+covariance; `augment_satellite_bias`/`retire_satellite_bias`
  scalar per-SV-per-pass slots with index-shift on retire; scalar `update_heading` (direct measurement
  of `HEADING`). This is the precedent emitter states would reuse — and the lifecycle they'd invert.

## R6 claims CONFIRMED (re-derived or standard)

- **Instrumental DF ~1–2° ideal.** Re-derived the interferometer CRB `σ_θ ≈ 1/[√(2·SNR·N)·(2π·D/λ)·cosθ]`.
  At DAB (200 MHz, λ=1.5 m, ~1.5 m aperture → D/λ=1), 20 dB SNR, single snapshot → **~0.65°**. Confirms
  R6's ideal floor as a thermal-noise, calibrated-manifold number.
- **Field 3–15°+ dominated by multipath.** Confirmed directionally: multipath bias is not in the CRB and
  dominates; SFN (DAB/DVB) composite-direction lock is real. Adopted R6's field budgets as judgment.
- **BO-TMA manoeuvre-observability (Nardone–Aidala), correctly scoped.** The manoeuvre requirement is a
  standard result for a *moving* target with unknown own-scale. **Correction (verify-seat FAIL, adopted):
  it does NOT make the unknown-emitter case here "unobservable."** For a **fixed** emitter with
  **own-velocity known from the speed log**, a straight constant-velocity leg **is** observable in range
  (triangulation as the bearing sweeps past beam) — weak and slow, strengthened by a manoeuvre, not
  requiring one. My first draft over-stated this as impossibility; the corrected framing is
  "weak/slow, manoeuvre-or-scale-dependent," and the verdict is re-based on risk/evidence, not
  observability.
- **5-ch coherent is separate hardware; 2-ch bladeRF gives only an ambiguous interferometer.** Confirmed
  from array DoF theory; KrakenSDR (~USD 749 + 199) is the COTS 5-ch package.
- **KrakenSDR does not naturally share the FE-5680A reference** — own onboard clock to all 5 tuners;
  external 10 MHz is a hardware mod. Parallel RF chain, own discipline.

## R6 claims REJECTED / RECONTEXTUALISED / FLAGGED

- **REJECT "tens of metres" geolocation as a nav accuracy.** That is multi-point mobile triangulation
  over a long track on strong land signals, not a single-epoch marine fix. R6 tags it anecdotal; I make
  the rejection explicit. My bearing→position table: 3–5° at 10 km = **524–873 m** cross-range, before
  GDOP — the honest coastal number, not tens of metres.
- **FLAG R6 arithmetic slip (D5 pattern, same class as D50's grok slips).** R6 §4.1 gives FM received
  power "≈ −52 dBm @10 km". Recompute: FSPL(100 MHz,10 km)=32.4+40+20 = **92.4 dB**; 10 kW ERP = 70 dBm
  EIRP → Pr = **−22.4 dBm**, not −52 dBm. The −52 is most likely a **dBW/dBm unit slip** (−52 dBW =
  −22 dBm). Conclusion (FM strong nearshore) survives; the number is `[UNVERIFIED-grok]`.
- **FLAG R6 under-weights spatial-aliasing at high D/λ.** Its aperture table calls cellular "excellent
  aperture" without noting a fixed 1.5 m 5-whip UCA has ~0.88 m element spacing = **2.35 λ at 800 MHz**
  (>λ/2) → grating-lobe ambiguities. High D/λ buys ambiguity as well as accuracy.
- **FLAG the body-frame→geographic attitude coupling R6 does not surface — scoped to the POSITION mode
  only.** When a bearing constrains *own-position* (unknown-emitter SLAM / any geographic-bearing fix),
  the geographic bearing carries heading error **one-for-one** (`σ_geo²=σ_α²+σ_ψ²`), so that mode's
  quality is capped by the attitude solution — worst post-manoeuvre at planing (the U-H1 breach moment).
  **Correction (verify-seat FAIL, adopted): this does NOT apply to the known-beacon *heading*
  measurement.** There `ψ = β − α` *produces* absolute heading from computed `β` + measured `α`; the
  prior heading does not enter, so it is genuinely absolute and non-circular (a coarse prior is needed
  only for beacon association). The two cases are now cleanly separated in the doc so the known-beacon
  heading capability is not wrongly dismissed.

## Key independent numbers (shown inline in the deliverable)

- DF thermal floor by band (1.5 m array, 20 dB, 1 snapshot): FM ~1.3°, DAB ~0.65°, cellular ~0.16°.
- Bearing→cross-range position: 1°→175 m, 3°→524 m, 5°→873 m, 10°→1745 m (all at 10 km).
- Cellular element spacing 2.35 λ @800 MHz → spatial aliasing for a fixed 5-whip UCA.
- FM Pr recompute: −22.4 dBm @10 km (vs R6's −52 dBm).

## Assumptions

- 1.5 m mast aperture (R6's 0.5–1.5 m circular array, upper end).
- Coastal emitter ranges 5–30 km (R6 emitter-landscape section).
- Denied gates from baseline: 200 m position, 2°/5° heading — the yardsticks used throughout.
- KrakenSDR prices/specs taken as plausible `[UNVERIFIED-grok]`, not independently re-priced.

## Weakest-evidence list (most to least load-bearing on the verdict)

1. **Field DF accuracy on a planing hull.** Everything hinges on whether σ_θ is ~few° (helps heading) or
   ~10°+ (useless). No marine at-speed DF measurement exists; R6's marine field budgets are judgment.
   This gates the entire verdict — hence the U-EA1 kill criterion is built on it.
2. **Body-frame attitude quality at speed** for the AoA→geographic conversion in the **position** mode
   (not the known-beacon heading mode). Depends on the same unresolved U-H1 attitude environment.
3. **Unknown-emitter convergence time** at 7–30 kn — no universal constant; BO-TMA Fisher scaling only.
   Judgment that it is "minutes of diverse geometry"; sim (U-EA1) needed.
4. **BO-SLAM EKF initialisation** — the elongated range-unobservable initial covariance is a known EKF
   failure mode; assumed solvable with inverse-range/delayed-init but unproven in this codebase.
5. **Emitter density / usability of Danish-strait bands** (R6 §4) — `[UNVERIFIED-grok]`, and SFN bands
   (best coverage) are the worst for clean AoA.
6. **KrakenSDR cost/spec** figures — Grok-sourced, plausible, not re-verified.

## Verdict (one paragraph)

The array does **not** meaningfully help at high speed in the form D48 names — but on **risk/evidence/
prioritisation**, not observability impossibility (verify-seat correction adopted). Unknown-emitter
bearings-only SLAM **is** observable: a fixed emitter's range is recoverable from a metrically-known
track because the speed log supplies the scale anchor, and a manoeuvre strengthens it; the information is
just **weak, slow, and geometry-dependent**, so it gives no *fast* position fix and no *fast* heading
reset, and is weakest at the post-manoeuvre planing moment U-H1 flagged. The one *fast*, genuinely
valuable capability is a **known-beacon** absolute-heading (and ≥3-beacon position) resection — a real
absolute-heading measurement, **not** circularly capped by the existing heading solution — but it needs a
surveyed emitter database, needs few-degree field DF that a vibrating spray-blinded mast is unverified to
deliver (thermal floor ~1° at DAB, field 3–15°+, one-for-one attitude penalty only when a bearing
constrains *position*), and yields only hundreds of metres of position at coastal ranges. So it is
deprioritised for four concrete reasons — unverified field DF, hard multipath/association/initialisation,
coastal-range position above the 200 m PL, and a cheaper hardware-free alternative — against which the
already-required, hardware-free **multi-satellite LEO fixture (U-MS1)** wins for the position gap. The
array is a **research spur, not a near-term aid**: pursue only as a hardware-free synthetic PoC
(**U-EA1**, after U-MS1) whose kill criterion is whether even the known-beacon mode clears the 2° heading
/ 200 m gates under realistic DF + attitude error — and buy no hardware until it does.
