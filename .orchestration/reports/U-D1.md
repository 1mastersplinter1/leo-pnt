# U-D1 Report

Contract version: **v1**

## Produced

- `docs/design/DESIGN_BASELINE.md`: the single normative baseline covering vessel and
  operating assumptions, sensor set, explicit rate contract, degradation behaviour,
  `gnss_authority` modes, and separate aided/denied acceptance profiles.
- `docs/design/ARCHITECTURE.md`: a subordinate architecture defining connected module
  boundaries, the measurement bus, time ownership, raw-IQ/measurement/truth storage and a
  failure-mode-driven build order.

No code, BOM/pricing work, web research or git commit was performed.

## Key decisions and rationale

- One executable path serves research and navigation. `gnss_authority` changes routing,
  while the acceptance profile changes by mode. This prevents GNSS from silently absorbing
  the denied project and ensures research exercises operational code.
- Denied acceptance uses 200 m horizontal position after a 20-minute constant-heading leg
  and 0.04 m/s per-axis velocity. These are estimates at the conservative edges of the
  handoff's approximately 100--200 m and 0.7--4 cm/s expectations. Aided targets are
  explicitly estimates: 25 m, 0.02 m/s per axis and 2 degrees heading.
- The fusion executive and an IMU-driven propagation slice come first. Ephemeris propagation
  and Doppler prediction precede the reject gate. A state cannot be declared without a
  measurement path. This directly addresses the handoff's disconnected-module, missing-
  propagation, decorative-gate and unobservable-state failures.
- The clock/time service alone owns runtime ordering, UTC mapping and monotonic deadlines;
  the estimator owns receiver clock states only. Authority timeouts never stop estimation.
- Truth is stored separately and has no online fusion edge in `recorded_only`, preventing
  circular validation.
- File container/codecs are left `[UNVERIFIED]` rather than inventing a factual dependency
  not present in the handoff. Required semantics and metadata are nevertheless explicit.

I found no reason to disagree with or deviate from the handoff.

## Assumptions and estimates

- Danish-strait operation and a displacement hull are working assumptions; the latter is
  load-bearing for reference tilt and dynamics.
- Rates not fixed by the handoff are estimates: IMU 100 Hz, each magnetometer 10 Hz, speed
  log 5 Hz, LEO observation target 1 Hz per tracked signal, GNSS at least 1 Hz, and solution
  generation 5 Hz. `GPS_INPUT` is nominally 5 Hz as required by the handoff.
- Aided position/velocity/heading and denied heading limits are engineering estimates, not
  verified guarantees. The denied position and velocity limits are estimates derived from
  the handoff's stated expectations.
- Nanosecond monotonic representation, RFC 3339 UTC and length-delimited binary journals
  are architecture estimates pending v2 schema selection.

## Open uncertainties / `[UNVERIFIED]` items

- Maximum acceptable SupGP ephemeris age.
- SDR sample rate corresponding to the mandated 2.5--5 MHz processing bandwidth.
- Statistical acceptance definition: percentile, test window and confidence.
- Replay equivalence tolerance.
- Concrete raw-IQ container, ADC sample packing, measurement-journal codec and truth-journal
  codec/access-control implementation.

## Evidence

Commands run:

```text
find .. -name AGENTS.md -print
sed -n '1,260p' .orchestration/CONTRACTS.md
sed -n '1,360p' docs/HANDOFF_PROMPT_BLADERF.md
ls -la docs docs/design .orchestration/reports
test -s <each of the three owned output files>
rg -n <required-term audit> docs/design/DESIGN_BASELINE.md docs/design/ARCHITECTURE.md
rg -n '\[UNVERIFIED\]' <the three owned output files>
git diff --check -- <the three owned output files>
git status --short -- <the three owned output files>
```

Observed: no `AGENTS.md` was present; contract v1 and the complete handoff were read; the
three owned output paths did not previously contain files. All three outputs are non-empty;
the required-term audit found the normative hierarchy, vessel assumptions, authority modes,
rates, profile limits, degradation model, time owner, three storage formats and mandated
build-order elements. Every inline `[UNVERIFIED]` topic is represented in the open-
uncertainties list. `git diff --check` returned no errors. `git status --short` showed only
the three owned files as new within the scoped status check.
