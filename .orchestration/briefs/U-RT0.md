# Brief U-RT0 — Fetch real published TLE/OMM sets (grok, stdout contract)
Purpose: provide REAL orbital elements for U-RT1's realism study. Fetch from CelesTrak a
representative recent sample for each: Starlink (~20 sats), OneWeb (~10), Iridium NEXT (~10).
For each satellite give the 2-line TLE (or OMM). State the CelesTrak URL + access date
(2026-07-24) and the epoch of the elements. Note licensing (CelesTrak terms). If a live fetch
returns nothing, say so plainly rather than inventing elements — DO NOT fabricate TLEs
(fabricated orbital elements are worse than none). VERIFIED (fetched) vs ASSUMED split.
## Output (stdout, write NO files): `===RT0-TLE===` then the TLE blocks (grouped by
constellation, each line exact), then `===RT0-REPORT===` then source URLs/epochs/notes.
