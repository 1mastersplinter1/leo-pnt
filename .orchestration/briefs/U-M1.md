# Brief U-M1 — MAVLink GPS_INPUT publisher + ArduPilot SITL harness

Contract version: v2. Workspace: dedicated git worktree on branch `unit/U-M1`. Commit here;
never merge to main. Read first: `.orchestration/CONTRACTS.md` (v2 rate contract),
`docs/design/DESIGN_BASELINE.md` (GPS_INPUT interface section: msg 232, GPS1_TYPE=14, 5 Hz,
yaw field, ODOMETRY prohibited, vd=0/MSL alt rules), `docs/design/ARCHITECTURE.md` module 12.

## Goal
1. `tools/mavlink_bridge/`: Python 3 package (pymavlink) that reads solution epochs as
   newline-delimited JSON on stdin (define the JSON schema in the package README — mirror
   the v2 SolutionEpoch fields plus horiz/speed/vert accuracies and yaw; accuracies are a
   REQUIRED input, sourced from the estimator in a later unit) and publishes MAVLink
   `GPS_INPUT` (msg 232) at 5 Hz with dead-reckoned fill per the baseline: publish
   continuously; when no fresh epoch arrives, repeat last epoch with inflating accuracy
   fields and set the appropriate `ignore_flags`; never exceed GPS_TIMEOUT_MS silence. Unit
   tests (pytest) for the mapping: lat/lon/alt from ECEF (document the geodesy library and
   datum), NED velocities, accuracy fields, yaw encoding (including the 0=north wrap rule
   per MAVLink spec — verify against the spec, cite it), fix_type degradation rules.
2. `tools/sitl/`: scripts + README that (a) clone/build ArduPilot Rover SITL at a PINNED
   commit — record commit hash and artifact sha256 in the README and report; (b) configure
   FRAME_CLASS=2, GPS1_TYPE=14; (c) run the bridge against SITL with a scripted synthetic
   solution stream (circle or straight-leg trajectory generator you write in the bridge
   package); (d) assert acceptance automatically: SITL's EKF reaches a 3D-fix-equivalent
   state, reported position tracks the injected trajectory within a stated tolerance, and
   injected accuracy fields are visible via GPS_RAW_INT/EKF status. Capture the evidence
   (MAVLink log excerpts) into the report.
3. If SITL cannot be built/run in this environment, deliver everything else, make the SITL
   step a documented, deterministic script with expected outputs marked [UNVERIFIED — not
   run here], and say exactly what failed. Do not fake evidence: your report must split
   VERIFIED (ran it) from ASSUMED.

## Method
pytest + ruff for the Python; TDD for the mapping functions. You may install Python deps
into a venv under tools/ and apt-level build deps only if non-interactive.

## Files owned
`tools/**`, `.orchestration/reports/U-M1.md`. Do not touch `crates/**` or `docs/design/**`.

## Report
`.orchestration/reports/U-M1.md`: VERIFIED vs ASSUMED evidence, pinned firmware hash +
checksum, JSON schema decisions, open uncertainties, contract version.
