# Brief U-RT0b — Fetch a LARGER real constellation TLE snapshot (grok, stdout)
The 40-SV sample (U-RT0) was too sparse for an N=8 visibility check. Fetch a MUCH larger real
snapshot from CelesTrak so >=8 satellites are simultaneously visible above 5 deg from a mid-
latitude coastal site (Danish waters ~56N): aim ~250 Starlink, ~60 OneWeb, ~40 Iridium NEXT
(use CelesTrak GROUP queries: gp.php?GROUP=starlink / oneweb / iridium-NEXT, FORMAT=tle — take
a representative slice if the full set is too large to print). Every line exact; state the
CelesTrak URLs + access date (2026-07-24) + element epochs. DO NOT fabricate elements — if the
fetch is partial, deliver what you actually got and say the real count. VERIFIED vs ASSUMED.
## Output (stdout, NO files): `===RT0B-TLE===` then TLE blocks by constellation, then
`===RT0B-REPORT===` then URLs/epochs/actual-counts/notes.
