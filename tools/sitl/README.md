# ArduPilot Rover SITL acceptance harness

The harness pins the official ArduPilot `Rover-4.6.3` tag at commit
`3fc7011a7d3dc047cbb17d8bd98ee94577d144c6`. `build.sh` performs a detached checkout,
initialises the pinned tree's submodules, builds Rover SITL, and writes the resulting
binary's SHA-256 to `ardurover.sha256`. The artifact checksum is machine/build dependent,
so the checked-in report records the checksum observed by this unit rather than claiming a
universal upstream binary checksum.

This unit's GCC 15.2.0 build produced
`abd0088642cb85d4fd2e7511acd225d5c4626f6ccd9298d38140b7bb2cb3f499`.

Run from the repository root:

```sh
tools/sitl/build.sh
tools/sitl/run.sh --duration 45 --speed-mps 0.1 --tolerance-m 10
```

The runner wipes SITL state, loads and confirms `FRAME_CLASS=2`, `GPS1_TYPE=14`, and
`GPS2_TYPE=0` at startup, and injects a 5 Hz, 0.1 m/s straight-leg stream through
the same mapping/send code used by the stdin bridge. It fails unless:

- EKF_STATUS_REPORT has the absolute-horizontal-position bit and not constant-position mode;
- every sampled GLOBAL_POSITION_INT after aiding, including the endpoint, is within 10 m of
  the contemporaneous injected position; and
- GPS_RAW_INT exposes injected fix type 3 and the injected 0.8 m horizontal accuracy as 800 mm.

Evidence is written to ignored `tools/sitl/evidence/sitl.log` and `mavlink.jsonl`. The
expected final stdout is one JSON `ACCEPTANCE` record. Until a run succeeds, these outputs
must be labelled `[UNVERIFIED — not run here]`; the runner never synthesises a passing log.

On 2026-07-22 this environment built and ran SITL successfully. The acceptance record had
EKF flags 831, 15 post-aiding continuous samples, maximum tracking error 0.0784 m, and final
tracking error 0.0784 m. This is driver/telemetry-path evidence, not a claim about estimator
fusion accuracy under dynamically representative motion.
