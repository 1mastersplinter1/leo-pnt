# Brief U-RT0c — Fetch ~150 real Starlink TLEs (grok, stdout, tight scope)
Prior fetches truncated. TIGHT scope to avoid truncation: fetch ~150 real Starlink TLEs from
CelesTrak (try gp.php?GROUP=starlink&FORMAT=tle; if rate-limited use the supplemental/NAME
endpoint grok found working before). Starlink ALONE at ~150 SVs gives ~7-8 simultaneously
visible above 5deg from a ~56N site — enough for the N=8 geometry check. Skip OneWeb/Iridium.
Every TLE line EXACT. DO NOT fabricate — deliver the real count you got and the CelesTrak URL +
epoch. Keep the output focused: just the TLE lines + 3 lines of provenance, nothing else, to
avoid stdout truncation.
## Output (stdout, NO files): `===RT0C-TLE===` then the ~150 Starlink TLE lines, then
`===RT0C-REPORT===` then URL + access date + actual count + epoch (3-5 lines max).
