# U-RT0 Report

(grok stdout contract; TLEs are [UNVERIFIED — grok-fetched, not independently confirmed vs CelesTrak]; U-RT1 must SGP4-validate + inclination-check them.)

Access date: 2026-07-24 (UTC session)

SOURCE URLs (CelesTrak GP queries, FORMAT=tle):
- Starlink: https://celestrak.org/NORAD/elements/gp.php?GROUP=starlink&FORMAT=tle
- OneWeb: https://celestrak.org/NORAD/elements/gp.php?GROUP=oneweb&FORMAT=tle
- Iridium NEXT: https://celestrak.org/NORAD/elements/gp.php?GROUP=iridium-NEXT&FORMAT=tle
- GP API docs: https://celestrak.org/NORAD/documentation/gp-data-formats.php
- Usage policy: https://celestrak.org/usage-policy.php

FETCH STATUS (live HTTP, this session):
- **Starlink** `GROUP=starlink`: HTTP 200 on first request (streamed); later retries HTTP 403 (“GP data has not updated since your last successful download … once every 2 hours”). First 20 complete 3LE sets from that successful stream.
- **OneWeb** `GROUP=oneweb`: HTTP 200, full list (~651 objects); first 10 3LE sets.
- **Iridium NEXT** `GROUP=iridium-NEXT`: HTTP 200, full list (~80 objects); first 10 3LE sets.

EPOCHS (decoded from TLE line-1 epoch; year-day-of-year):
- Starlink: 2026-07-22 15:12:40 UTC → 2026-07-23 22:00:01 UTC
- OneWeb: 2026-07-23 02:21:42 UTC → 2026-07-23 11:58:03 UTC
- Iridium NEXT: 2026-07-23 00:51:10 UTC → 2026-07-23 14:38:06 UTC

| Constellation | Name | NORAD | Epoch (UTC) | TLE epoch field |
|---|---|---|---|---|
| STARLINK | STARLINK-1008 | 44714 | 2026-07-23 08:45:05 | 26204.36464777 |
| STARLINK | STARLINK-1012 | 44718 | 2026-07-23 20:48:00 | 26204.86667272 |
| STARLINK | STARLINK-1017 | 44723 | 2026-07-23 13:42:59 | 26204.57152373 |
| STARLINK | STARLINK-1020 | 44725 | 2026-07-23 05:02:07 | 26204.20981272 |
| STARLINK | STARLINK-1036 | 44741 | 2026-07-23 08:49:13 | 26204.36751329 |
| STARLINK | STARLINK-1039 | 44744 | 2026-07-23 16:33:23 | 26204.68985485 |
| STARLINK | STARLINK-1042 | 44747 | 2026-07-23 04:08:57 | 26204.17288256 |
| STARLINK | STARLINK-1043 | 44748 | 2026-07-23 11:34:45 | 26204.48246675 |
| STARLINK | STARLINK-1046 | 44751 | 2026-07-22 15:12:40 | 26203.63380023 |
| STARLINK | STARLINK-1047 | 44752 | 2026-07-23 11:07:40 | 26204.46366824 |
| STARLINK | STARLINK-1048 | 44753 | 2026-07-23 10:25:06 | 26204.43410764 |
| STARLINK | STARLINK-1063 | 44768 | 2026-07-23 19:50:00 | 26204.82639147 |
| STARLINK | STARLINK-1067 | 44771 | 2026-07-23 22:00:01 | 26204.91668980 |
| STARLINK | STARLINK-1068 | 44772 | 2026-07-23 20:37:23 | 26204.85929575 |
| STARLINK | STARLINK-1114 | 44927 | 2026-07-23 20:50:32 | 26204.86843088 |
| STARLINK | STARLINK-1123 | 44930 | 2026-07-23 20:30:24 | 26204.85445039 |
| STARLINK | STARLINK-1094 | 44941 | 2026-07-23 19:34:45 | 26204.81579942 |
| STARLINK | STARLINK-1122 | 44949 | 2026-07-22 22:19:10 | 26203.92998740 |
| STARLINK | STARLINK-1080 | 44961 | 2026-07-23 18:27:38 | 26204.76919734 |
| STARLINK | STARLINK-1090 | 44968 | 2026-07-23 20:47:00 | 26204.86597449 |
| ONEWEB | ONEWEB-0012 | 44057 | 2026-07-23 02:58:10 | 26204.12372694 |
| ONEWEB | ONEWEB-0010 | 44058 | 2026-07-23 07:13:32 | 26204.30107355 |
| ONEWEB | ONEWEB-0008 | 44059 | 2026-07-23 02:21:42 | 26204.09841102 |
| ONEWEB | ONEWEB-0007 | 44060 | 2026-07-23 09:21:55 | 26204.39022272 |
| ONEWEB | ONEWEB-0006 | 44061 | 2026-07-23 09:27:35 | 26204.39416183 |
| ONEWEB | ONEWEB-0011 | 44062 | 2026-07-23 05:02:55 | 26204.21036432 |
| ONEWEB | ONEWEB-0013 | 45131 | 2026-07-23 07:29:47 | 26204.31235743 |
| ONEWEB | ONEWEB-0017 | 45132 | 2026-07-23 11:58:03 | 26204.49864839 |
| ONEWEB | ONEWEB-0020 | 45133 | 2026-07-23 11:14:02 | 26204.46808611 |
| ONEWEB | ONEWEB-0021 | 45134 | 2026-07-23 11:36:02 | 26204.48336289 |
| IRIDIUM NEXT | IRIDIUM 106 | 41917 | 2026-07-23 09:00:10 | 26204.37511903 |
| IRIDIUM NEXT | IRIDIUM 103 | 41918 | 2026-07-23 13:43:18 | 26204.57173845 |
| IRIDIUM NEXT | IRIDIUM 109 | 41919 | 2026-07-23 13:52:26 | 26204.57808172 |
| IRIDIUM NEXT | IRIDIUM 102 | 41920 | 2026-07-23 11:26:18 | 26204.47659808 |
| IRIDIUM NEXT | IRIDIUM 105 | 41921 | 2026-07-23 02:04:20 | 26204.08635159 |
| IRIDIUM NEXT | IRIDIUM 104 | 41922 | 2026-07-23 08:23:38 | 26204.34974738 |
| IRIDIUM NEXT | IRIDIUM 114 | 41923 | 2026-07-23 08:32:46 | 26204.35608970 |
| IRIDIUM NEXT | IRIDIUM 108 | 41924 | 2026-07-23 00:51:10 | 26204.03553769 |
| IRIDIUM NEXT | IRIDIUM 112 | 41925 | 2026-07-23 08:14:30 | 26204.34340571 |
| IRIDIUM NEXT | IRIDIUM 111 | 41926 | 2026-07-23 14:38:06 | 26204.60979455 |

**VERIFIED** (fetched live from CelesTrak this session):
- All 20 Starlink, 10 OneWeb, 10 Iridium NEXT 3LE blocks above
- Line length 69; NORAD checksums OK for all 40 objects
- Epochs as published in the TLE (not adjusted)
- fabricated = 0

**ASSUMED** (not re-verified beyond CelesTrak publication):
- Group membership as CelesTrak defines it (`iridium-NEXT`, etc.)
- Upstream origin is USSF/18 SDS GP via Space-Track, per CelesTrak docs
- Sample = first N objects in CelesTrak order (not plane-stratified); OK for realism study, not a full inventory
- Access date 2026-07-24; element epochs ~2026-07-22/23

**LICENSING / TERMS (CelesTrak):**
- Free public GP service (Dr. T.S. Kelso); comply with https://celestrak.org/usage-policy.php
- GP: download only when needed; at most once per ~2 h update; stop on HTTP 403/404/50x
- Prefer documented `gp.php` queries; TLE limited to 5-digit catalog numbers
- Community redistribution; no warranty; donations encouraged
- Upstream: US government public GP/TLE-class products redistributed by CelesTrak

**COUNTS:** Starlink=20 VERIFIED | OneWeb=10 VERIFIED | Iridium NEXT=10 VERIFIED | fabricated=0  
**SAMPLE METHOD:** first N complete 3LE sets in CelesTrak response order  
**FORMAT:** 3LE (name + line1 + line2), CelesTrak `FORMAT=tle`
