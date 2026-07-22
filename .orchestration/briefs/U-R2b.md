# Brief U-R2 — PNT services, terrestrial SoOP, regulations research (live web)

Contract version: v1 (read `.orchestration/CONTRACTS.md` first).

## Goal
Live web research, output as text (see Output format below). Operating-area working assumption: Danish waters / Danish straits (flag it as an assumption in the doc). Questions:

1. **Iridium STL (Satelles / Xona? — identify current owner/branding 2026)**: product status, receiver hardware options, how it is purchased (subscription? OEM only?), any public pricing, and the concrete path to getting a quote. Claimed performance (~20 m, sub-100 ns) — confirm from vendor or independent sources.
2. **Terrestrial SoOP in/around Danish waters**: DVB-T2 transmitter situation in Denmark/southern Sweden/northern Germany (SFN structure, whether GNSS-disciplined); R-Mode Baltic status in 2026 (operational? testbed? which stations, usable area); cellular (LTE/5G) coverage viability offshore; eLoran status in the region.
3. **Regulatory**: Danish Maritime Authority rules for a manned vessel running an experimental autopilot/semi-autonomous system in Danish waters — permits needed, any sandbox/test-area schemes, and COLREGs implications for a manned test platform. Note equivalent EU-level guidance (EMSA) if found.
4. **CelesTrak SupGP**: confirm current coverage (Starlink, OneWeb full-fleet?), update cadence, access method/terms.

## Method & rules
- Web search allowed and expected. Every claim: source URL + access date (2026-07-22).
- Split VERIFIED (read it) vs ASSUMED (inference). No unlabeled claims.
- Regulatory findings are NOT legal advice — say so in the doc; identify the authority to contact for a binding answer.

## Output format (REPLACES file writing)
Do NOT write or edit any files. Deliver everything as your final answer text, in this exact order:
line `===R2-DOC===`, then the complete research document, then line `===R2-REPORT===`, then the unit report content.

## Files owned
None — you write no files at all. No git. No code.

## Report
After `===R2-REPORT===`: summary, VERIFIED/ASSUMED split, dead ends, open uncertainties, contract version.
