(Sol verify pass on U-N1 fix round. Verdict: FAIL — residuals.)

1. **PASS — Freshness deadlines.** §2.4 proposes IMU 0.10 s, magnetometer 0.50 s, speed-log 1.00 s, and ephemeris governed by `t_eph`, derived from the baseline rate contract ([PARAMS_PROPOSAL.md:237](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:237)). A separate TOML block records them without pretending they are `AuthorityParams` fields ([PARAMS_PROPOSAL.md:627](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:627)). §5.4 correctly registers that `is_complete()` cannot enforce them and routes a v6 contracts action ([PARAMS_PROPOSAL.md:559](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:559)).

2. **PASS — 4,000-block provenance.** §4 cites `U-T1-review-opus-measurements.md` and carries the full material caveat: review probes, removed afterward, not shipped-test evidence, committed test only 24 blocks, and no reproducible seed/command ([PARAMS_PROPOSAL.md:370](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:370)). The same limitation appears in assumption A5 ([PARAMS_PROPOSAL.md:490](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:490)) and the report source list. This matches the cited artifact.

3. **FAIL — Correlation correction is not internally complete.** §4.2 and A2 correctly say positive dependence reduces the effective trial count and makes the independent-cell calculation conservative ([PARAMS_PROPOSAL.md:433](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:433), [PARAMS_PROPOSAL.md:476](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:476)). However, A1 then claims non-zero reference autocorrelation sidelobes imply both fewer effective cells **and a heavier-than-exponential tail** ([PARAMS_PROPOSAL.md:470](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:470)). Under the stated white complex-Gaussian-noise model, a non-ideal fixed reference correlates output bins but does not by itself make each normalized bin’s marginal non-exponential or heavier-tailed. Thus A1 is not a legitimate inflation basis as written. Real signals, colored/non-Gaussian noise, interference, or normalization mismatch could produce heavier marginals, but that requires separate evidence/modeling; A3 is legitimate.

4. **PASS — Extreme PFA language withdrawn.** `5.30e-9` is explicitly labeled analytic-model-only; the text says 4,000 blocks cannot validate it or justify the former `1e-8` bracket ([PARAMS_PROPOSAL.md:448](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:448)). “Very safe” is absent. The later `1e-7`–`1e-8` reference is clearly a proposed production false-observation budget, not an empirical bracket.

5. **PASS — TOML matches Rust field-for-field.** The appendix contains exactly the nine top-level scalar fields from `AuthorityParams`: `t_lease_s`, `t_dr_s`, `t_eph_s`, `dwell_clear_s`, `dwell_rearm_s`, `caution_enter`, `caution_clear`, `revoke_threshold`, and `t_ack_s`; plus `[aided]` and `[denied]`, each with the three exact `ProtectionLimits` fields ([PARAMS_PROPOSAL.md:574](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:574), [lib.rs:11](/home/od/work/leo-pnt/crates/pnt-integrity/src/lib.rs:11)). No field is missing, added, or misnamed.

6. **PASS — Revoke backstop.** It now says the scalar backstops a finite-but-dangerously-loose profile limit, while an absent limit independently fails `is_complete()` ([PARAMS_PROPOSAL.md:287](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:287)).

7. **PASS — Velocity anisotropy.** The derivation explicitly identifies the isotropic-Gaussian assumption, explains why scalar DRMS cannot guarantee a per-axis bound under anisotropy, and calls for a per-axis gate or covariance-shape rule ([PARAMS_PROPOSAL.md:103](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:103)). §5.2 registers the coupled covariance-shape freeze ([PARAMS_PROPOSAL.md:542](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:542)).

8. **PASS — Fisher-series scope.** The greater-than-ten-orders claim is now scoped to `Q = 32`; lower validation thresholds explicitly use the full sum ([PARAMS_PROPOSAL.md:415](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:415)).

9. **PASS — Boundary semantics.** The document correctly states that the profile accepts `≤100`, the scalar requires `<100`, and therefore only the scalar rejects exactly 100.0 m ([PARAMS_PROPOSAL.md:79](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:79)).

10. **PASS — Lease-margin basis.** The derivation now distinguishes the approximately 10 ms DR-fill renewal opportunity, making the lease approximately 100× that cadence, from the 200 ms publication cadence, where it is 5× ([PARAMS_PROPOSAL.md:166](/home/od/work/leo-pnt/docs/design/PARAMS_PROPOSAL.md:166)).

**NEW-1 — MEDIUM/high confidence:** A1’s assertion that fixed-reference autocorrelation sidelobes cause a heavier-than-exponential marginal tail is unsupported and conflicts with the otherwise-correct correlation-direction analysis. Rewrite A1 to say sidelobes create dependence/search-structure mismatch, which alone is conservative; reserve possible tail inflation for demonstrated non-Gaussian/non-stationary real-capture effects such as A3.

No other new inconsistency found. The subordinate, proposed-only, `[UNVERIFIED]`, no-authority framing remains explicit and intact.

FAIL
