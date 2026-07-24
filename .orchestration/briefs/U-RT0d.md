# Brief U-RT0d — Fetch CelesTrak SupGP (Supplemental GP) for Starlink (grok, stdout, tight)
The baseline specifies SupGP over plain TLEs (operator-supplied, far more accurate than SGP4-on-
TLE ~1km error). Fetch ~120 Starlink SupGP records from CelesTrak's SUPPLEMENTAL endpoint
(supplemental data: celestrak.org/NORAD/elements/supplemental/sup-gp.php?FILE=starlink&FORMAT=tle
or the SupGP JSON/OMM equivalent). Give EXACT lines. State the exact URL + access date (2026-07-24)
+ epoch + the format (TLE or OMM). Confirm it is SUPPLEMENTAL (SupGP), not the general GP/GROUP
product — say which you actually got. DO NOT fabricate; deliver the real count. Keep output focused
(records + <=4 provenance lines) to avoid stdout truncation.
## Output (stdout, NO files): `===RT0D===` then the records, then `===RT0D-REPORT===` then
URL/date/epoch/format/count/is-supplemental (<=5 lines).
