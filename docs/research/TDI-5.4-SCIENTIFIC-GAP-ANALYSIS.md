# TDI-5.4 Scientific Gap Analysis

Status: audit complete; scientific question selected; no TDI-5.4 evaluator implemented and no TDI-5.4 scientific run performed.

Date: 2026-07-19

## Executive conclusion

TDI-5.3 is internally trustworthy for its frozen estimands: within the preregistered random branching-system generator, seed blocks, exclusions, horizons, ridge-model family, and loss definitions, early overlap predicts later dynamic deficit; O2 adds substantial held-out information beyond the baseline and O1; and O1 is practically equivalent to redundant once O2 is included in the primary standardized-U6 ridge comparison. All TDI-5.1, TDI-5.2, and TDI-5.3 hash chains verify, and the released TDI-5.3 result log is complete and hash-identical to the release asset.

The evidence does **not** establish nonlinear conditional sufficiency, calibrated prediction at width 6, causality, generator robustness, or a universal property of dynamic systems. The most immediate unresolved threat to the TDI-5.3 interpretation is model-class misspecification: every confirmatory sufficiency comparison used an additive ridge model. TDI-5.4 will therefore test whether O1 remains conditionally redundant given O2 under a small, scientifically motivated set of deterministic nonlinear model families.

No defect was found that invalidates a frozen TDI-5.3 verdict. Several limitations materially narrow those verdicts and must be carried forward: width-6 transfer is a strong *relative* improvement with poor absolute fit; O1 slightly improves standardized U6 loss but worsens reconstructed-O6 loss; the bootstrap conditions on fitted models and therefore omits training uncertainty; the generator represents a narrow, dense branching regime; and the current reproduction script's final-output completeness check is not schema-exact.

## Audit scope and integrity disposition

The audit covered all preregistrations, evaluators, reproduction scripts, incident/correction reports, result formats, CI workflows, `tdi-core` mathematical definitions, and `tdi-bench` statistical procedures present at the frozen commit. It also read the complete 3,404-line TDI-5.3 result log.

### Repository and release identity

| Item | Verified state |
|---|---|
| Local `main`, `origin/main`, and `HEAD` before branching | `989a4c772024aa9ee1219d771d477357c42484f6` |
| Frozen tag | Annotated tag `tdi-5.3-preregistered-results`, dereferencing to the frozen commit |
| Ancestry | `origin/main` equals the frozen merge commit; no additive successor was present at audit time |
| Release | [TDI-5.3 preregistered results](https://github.com/Memorithm/TDI/releases/tag/tdi-5.3-preregistered-results) points to the frozen state |
| Result-log SHA-256 | `fbe3feb2883d52de018e61e2ee5771a808fa334d756bd5de87c91d772be034ef` |
| Release archive SHA-256 | `ffc8d1f13f13e1b5ac643b0591cb374a4f07e7c7527b5615bbca39a8548275b6` |

All downloaded release assets corresponding to local scientific artifacts were byte-identical. The archive checksum asset embeds the originating absolute archive path; this is a portability/hygiene weakness, not a scientific-content mismatch.

### Frozen hash verification

Every entry in each of the nine TDI-5.1, TDI-5.2, and TDI-5.3 preregistration, evaluator, and scientific-code manifests passed `sha256sum --check`.

| Revision | Preregistration content SHA-256 | Evaluator content SHA-256 | Scientific-manifest file SHA-256 |
|---|---|---|---|
| TDI-5.1 | `25b65a07b7f248df3e043b9b7f63611c360f60f3d49a600a5612305440131852` | `d69d42fa31d973603eabd0ded8ffd8ca2f0a4b0b8fcec5f9de42ed8c7ce37444` | `906dc938a9cab96299119f4028fbf566b50c7f90eb32d861b7dd79bc265da9ab` |
| TDI-5.2 | `f57a054bc95eb2e041434d6e2049509b0dce1a5397f9666d274b1bbac332be35` | `2308607729659c7546a17530e69773f982d9a1cf41656ea7898e0123ca469ef7` | `7f8eeea2304ef14c1ab2fdb6835a6f5397be83209a80aaf08ccdabd54ccf3d61` |
| TDI-5.3 | `7223128dcfd751ebeb6488c01c3512d0a10b35937ec170504984295eb421682e` | `93181fb75d4882be2ca0b26c1babe9db747583d6e08e6992617ec12b7f65460f` | `2659e0afae239074262b1900ff2d5f6754df5247a31a7e0c729fc3fda759e7c6` |

The authoritative frozen sources are the [TDI-5.3 preregistration](../TDI-5.3-INDEPENDENT-OVERLAP-ACTIVATION-PREREGISTRATION.md), [TDI-5.3 evaluator](../../tdi-bench/src/bin/tdi-independent-overlap-ablation-v53.rs), [scientific manifest](../TDI-5.3-SCIENTIFIC-CODE.sha256), and [reproduction script](../../scripts/reproduce-tdi5.3.sh).

### Validation performed during this audit

| Check | Outcome |
|---|---|
| `cargo test --workspace --all-targets --all-features --offline` | 395 passed; 0 failed; 0 ignored |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` | Passed |
| Shell syntax for all eight reproduction scripts | Passed |
| TDI-5.3 preflight | Passed; no scientific generation |
| TDI-5.3 termination smoke | Passed; bounded miniature path only |

The post-merge TDI-5.3 workflow attached to the frozen merge commit is displayed by GitHub as failed because its termination-smoke step was cancelled after formatting, tests, Clippy, hashes, and shell syntax had passed; the accessible check annotation records only "The operation was canceled" and does not identify a cause. The activation PR head `f1c50e397c4eba80c25a42e5c83245dc7d68f34a` has the exact same Git tree (`d1bb1b9bcd039ffbc8986d20c34d4b6c6e7245b0`) as the merge commit and passed the complete TDI-5.3 validation plus all general PR checks. The local validation above independently passed on the frozen merge commit. This is a CI-history caveat, not evidence of a scientific failure; it must not be represented as a green post-merge run.

No full scientific experiment was launched during this audit. The released result directory already present as untracked local data was read and preserved without modification.

## Frozen mathematical and statistical estimand

For a generated transition system, let `P_h` and `Q_h` be the exact state distributions at horizon `h` from the reference and perturbed initial states. `tdi-core` computes these distributions using exact rational arithmetic and stable `BTreeMap` iteration. Their overlap is

\[
O_h = \sum_s \min(P_h(s), Q_h(s)) = 1 - \operatorname{TV}(P_h,Q_h).
\]

The continuous deficit target is

\[
U_h = -\log_2(1-O_h),
\]

defined only when `O_h < 1`. Exact ratios are converted to `f64` only for features, transforms, fitting, and reporting. Successor states are sorted and deduplicated; scientific loops and reductions have fixed order.

TDI-5.3 used:

- observation horizons 1 and 2, with `O1` and `O2` as early-overlap predictors;
- target horizons 3, 4, 5, 6, and 8, with U6 primary;
- a 13-variable baseline containing reference/perturbed entropy, reachable-state, and path-count summaries at depths 1 and 2, plus width;
- feature layouts `B0` (baseline), `B1` (baseline + O1), `B2` (baseline + O2), and `B12` (baseline + O1 + O2); an O2-O1 layout was exploratory only;
- additive ridge regression with fixed `lambda = 1`, per-block feature and target scaling fitted on the combined width-3/4 training population, and no model selection;
- three disjoint deterministic seed blocks A, B, and C;
- paired percentile bootstrap intervals with 4,000 replicates per block and a block-stratified aggregate bootstrap.

The aggregate bootstrap resamples held-out records within each frozen seed block but does not refit models. Its intervals therefore measure held-out record uncertainty conditional on each fitted model, scaler, generator, and seed namespace; they do not include training-set/model-fitting uncertainty.

### Population accounting

Each seed block requested 55,000 accepted records: 15,000 width-3 training, 15,000 width-4 training, 5,000 width-3 holdout, 5,000 width-4 holdout, 10,000 width-5 OOD, and 5,000 width-6 OOD. Across all blocks, 165,000 records were accepted from 165,257 candidates. All 257 rejections were preregistered fully-recovered-at-O2 exclusions: 256 at width 3 and one at width 4. The overall rejection rate was 0.1555%.

Consequently, every TDI-5.3 inference is conditional on `O2 < 1` and on nonzero target deficit at every target horizon. It does not describe the fully recovered subpopulation.

## Exact evidence for TDI-5.3A through TDI-5.3E

Positive relative reduction means the challenger has lower standardized-U MSE. Intervals are frozen 95% percentile-bootstrap intervals. Percentages below are derived directly from the frozen printed MSE values; no threshold or verdict has been recomputed or changed.

### Primary and OOD aggregate evidence

| Criterion and comparison | Aggregate standardized-U MSE, reference -> challenger | Point relative reduction | Frozen 95% relative interval | Spearman, reference -> challenger | Absolute-fit context | Frozen verdict |
|---|---:|---:|---:|---:|---|---|
| A: B0 -> B12, in-distribution U6 | 0.342167207532 -> 0.260330313711 | 23.9172% | [22.8642%, 24.9179%] | 0.811552889489 -> 0.860774681117 | U-space R2 0.662303 -> 0.743071 | Passed |
| B: B1 -> B12, in-distribution U6 | 0.323676243793 -> 0.260330313711 | 19.5708% | [18.5838%, 20.5273%] | 0.820961491912 -> 0.860774681117 | U-space R2 0.680553 -> 0.743071 | Passed |
| C: B2 -> B12, in-distribution U6 | 0.260462109838 -> 0.260330313711 | 0.0506% | [0.0038%, 0.0988%] | 0.860663654490 -> 0.860774681117 | Entire relative interval lies inside +/-2% | Equivalent |
| D5: B0 -> B12, width-5 U6 | 0.134542742798 -> 0.070290392638 | 47.7561% | [47.4196%, 48.0935%] | 0.470995128157 -> 0.623482398358 | U-space R2 -0.633869 -> 0.146403 | Passed as part of D |
| D6: B0 -> B12, width-6 U6 | 0.413905953371 -> 0.152146056062 | 63.2414% | [63.0798%, 63.4027%] | 0.414128376337 -> 0.555183444700 | U-space R2 -11.877932 -> -3.733748 | Passed as part of D |

The per-block point reductions and relative intervals demonstrate that the aggregate results were not driven by one block:

| Comparison | Block A | Block B | Block C |
|---|---:|---:|---:|
| A, B0 -> B12 | 23.2167%; CI [21.4897%, 24.9328%] | 25.3339%; CI [23.4893%, 27.1489%] | 23.1282%; CI [21.2980%, 24.9342%] |
| B, B1 -> B12 | 18.9487%; CI [17.3315%, 20.5714%] | 20.7319%; CI [18.9988%, 22.4155%] | 18.9798%; CI [17.1719%, 20.7137%] |
| C, B2 -> B12 | 0.0474%; CI [0.0009%, 0.0948%] | 0.0569%; CI [-0.0336%, 0.1469%] | 0.0474%; CI [-0.0570%, 0.1532%] |
| D, width 5 | 48.5443%; CI [47.9346%, 49.1441%] | 44.7145%; CI [44.1186%, 45.2913%] | 50.0352%; CI [49.4676%, 50.6168%] |
| D, width 6 | 65.6066%; CI [65.3348%, 65.8977%] | 58.5226%; CI [58.2602%, 58.7875%] | 65.7281%; CI [65.4546%, 66.0036%] |

Criterion C's practical-equivalence conclusion is specifically about standardized U6 MSE in the ridge family. On reconstructed O6, adding O1 to B2 changed aggregate MSE from 0.000721690411 to 0.000736560276 and MAE from 0.005530677102 to 0.005554103365. The frozen improvement intervals were negative for both MSE `[-0.000021110, -0.000009473]` and MAE `[-0.000032633, -0.000014915]`. This secondary result does not overturn the preregistered U-space equivalence verdict, but it prohibits the broader claim that O1 is harmless or redundant under every loss and target scale.

Criterion D demonstrates comparative OOD signal, not calibrated width-6 prediction. At width 6, the challenger still had U-space R2 `-3.733748299550`, bias `-0.360668493472`, calibration slope `1.236654115911`, and reconstructed-O R2 `-5.581769383600`. Its predictions were substantially better than B0 while remaining worse in squared error than a constant-mean predictor under the printed R2 definition.

### Multi-horizon evidence

For criterion E, B12 improved over B0 at every secondary horizon in every seed block. The effect declined monotonically with distance from the observation window.

| Target | Aggregate MSE, B0 -> B12 | Aggregate point reduction | Frozen 95% relative interval | Block A / B / C point reductions |
|---|---:|---:|---:|---:|
| U3 | 0.379084493522 -> 0.196928061567 | 48.0517% | [47.1283%, 48.9271%] | 48.0106% / 48.8932% / 47.2076% |
| U4 | 0.352605491391 -> 0.223266974273 | 36.6808% | [35.6367%, 37.6677%] | 36.4278% / 37.4839% / 36.0919% |
| U5 | 0.345165381778 -> 0.245073844308 | 28.9981% | [27.9304%, 30.0039%] | 28.8008% / 29.8497% / 28.2992% |
| U8 | 0.341051968481 -> 0.281617673094 | 17.4268% | [16.4333%, 18.4337%] | 16.9638% / 18.3288% / 16.9281% |

The exact frozen terminal verdicts were A passed, B passed, C equivalent, D passed, and E passed.

## Established findings

The following statements are established only within the frozen TDI-5.3 scope:

1. Early overlaps jointly improve held-out U6 ridge prediction beyond the 13-variable baseline in all three seed blocks.
2. O2 improves held-out U6 ridge prediction beyond baseline + O1 in all three seed blocks, with an aggregate 19.57% MSE reduction.
3. Adding O1 to baseline + O2 changes primary standardized-U6 ridge MSE by much less than the preregistered +/-2% practical margin; all three block intervals and the aggregate interval lie inside that margin.
4. B12 improves relative prediction over B0 on width-5 and width-6 populations generated by the same mechanism, under the frozen D criteria.
5. B12 improves relative prediction over B0 at U3, U4, U5, and U8 in every block, with a diminishing effect at longer horizons.
6. The implementation has a deterministic, auditable execution contract under the frozen code, seeds, and ordered reductions, and the release artifacts preserve one hash-verified completed run. A repeated full-run or cross-architecture identity claim has not been established.

## Supported interpretations

The evidence supports the following qualified interpretations:

- O2 contains predictive information about later deficit not represented by the frozen baseline and O1 within an additive ridge model.
- O1's incremental contribution after O2 is practically negligible for the primary ridge/U6/MSE estimand.
- Early overlap carries a signal that persists beyond the immediately adjacent U3 target and appears in wider systems generated from the same transition-mask distribution.
- The strength of relative transfer can increase while absolute calibration degrades; relative signal and calibrated prediction are distinct properties.
- The sequence from TDI-3 through TDI-5.3 supports continuous deficit as a more informative target geometry than exact-recovery classification for these increasingly mixing systems.

## Unresolved questions and threats to validity

### Model-class limitations

- All confirmatory TDI-5.3 comparisons use linear additive ridge regression with one fixed penalty. O1-by-O2 interactions, thresholds, saturation, and conditional effects that vary with baseline geometry are absent.
- The equivalence bootstrap does not refit the model, so it does not propagate training-set variability or instability in coefficients and scalers.
- There was no validation split because TDI-5.3 froze a single model and hyperparameter. TDI-5.4 cannot select nonlinear complexity on the final test set.
- Three fitted seed-block models are evidence of replication, but not enough to establish model-algorithm universality.

### Possible confounding variables

- O2 is temporally closer to every target than O1. Its incremental value may reflect recency rather than a uniquely sufficient structural statistic.
- The baseline is a compact summary, not a complete adjustment set. Graph density, mixing time, local transition motifs, and other structural properties can jointly affect O2 and future deficit.
- Reference state zero, a flip of node `width - 1`, and future `Noop` actions are fixed. Generator symmetry makes this less alarming but does not prove invariance to state, node, or action choice.
- Conditioning the accepted population on `O2 < 1` and all target deficits being nonzero changes the estimand and can induce selection effects. The observed exclusion rate is small, but the fully recovered regime remains untested.
- Targets at different horizons are nested measurements of the same generated systems, so multi-horizon agreement is not independent replication.

### Finite-sample and inferential limitations

- Record counts are large, but there are only three deterministic seed blocks and one generator family.
- Percentile-bootstrap intervals resample individual held-out records. They do not resample model fits, generator hyperparameters, perturbation choices, or alternative seed-block definitions.
- Consecutive deterministic seeds are treated as draws from the frozen pseudorandom generator; the interval interpretation is conditional on that sampling mechanism rather than a physical population.
- Criterion E uses a preregistered compound sign/threshold rule rather than simultaneous confidence intervals at every horizon. Its pass should not be read as four independent hypothesis-test discoveries.

### Distributional limitations

- Each source state receives a mask intended to be uniform over all nonempty successor subsets. This makes transitions dense: roughly half of all states are successors, producing rapid mixing and near-ceiling overlaps at widths 5 and 6.
- The modulo mapping has a minute exact bias: because `2^64 mod (2^(2^width)-1) = 1` for widths 3 through 6, mask value 1 has one extra `u64` preimage. The effect is at most one part in `2^64` per mask draw and is not a plausible explanation of TDI-5.3's effects, but future generators should either remove it with bounded rejection sampling or explicitly retain and document it.
- Only widths 3 and 4 train the models; only widths 5 and 6 test width transfer; horizons stop at 8.
- Sparse branching, heterogeneous out-degree, alternate graph generators, non-Noop actions, rare-event regimes, adversarial regimes, and substantially wider systems remain untested.

### Information-leakage assessment

No direct leakage was found in TDI-5.3:

- all worst-case seed reservations are pairwise disjoint;
- models and scalers use only each block's width-3/4 training records;
- held-out width-3/4 and OOD width-5/6 labels are not used for fitting or threshold adaptation;
- O1 and O2 are computed only through observation horizon 2, while targets begin at horizon 3;
- TDI-5.3 mechanically activates the preregistered, frozen TDI-5.2 scientific design.

Residual leakage risk for TDI-5.4 is substantial if old test seeds are reused, because their outcomes are now public. TDI-5.4 must use a new, disjoint seed namespace and a new train/validation/test split. Any complexity choice must be made using training and validation data only, with the final test labels inaccessible until the protocol, evaluator, and manifests are frozen.

### Calibration and numerical weaknesses

- In-distribution standardized-U6 calibration is close to identity, but the nonlinear back-transform produces a B12 reconstructed-O6 calibration slope of 0.6695.
- At width 6, both U-space and O-space R2 remain strongly negative after the large relative improvement. Near-ceiling O values compress raw O-space errors and make tiny absolute changes look deceptively precise.
- The `calculate_metrics` implementation reports R2 and calibration slope as zero when the relevant variance is near zero rather than reporting an undefined/degenerate status.
- Relative reduction silently returns zero for nonfinite inputs or a near-zero baseline denominator. No such boundary occurred in the frozen result, but a new evaluator should fail explicitly or emit a typed degeneracy state.
- Ridge fitting uses normal equations and deterministic partial-pivot elimination without a reported condition number. Ridge regularization helps, but nonlinear feature expansions will require stronger numerical diagnostics.
- `ln`, `log2`, and `powf` plus ordinary `f64` arithmetic are not guaranteed bit-identical across all architectures. TDI-5.4 must specify tolerances and decision-boundary stability instead of claiming universal bit identity.

### Operational reproducibility weaknesses

- TDI-5.3 has no checkpoint/resume protocol.
- The reproduction script verifies that generic criterion names and `VERDICTS FINAUX` occur in the log, but the criterion names also occur in section headings. A truncated log ending after the final heading could satisfy this presence-only check. The released log is complete; the weakness concerns failure detection in future runs.
- Frozen metadata and the release archive checksum expose local absolute paths or host information. Future provenance should be scientifically sufficient and sanitized.
- CI is tied to a specific self-hosted ARM64 runner layout. Cross-architecture numerical reproducibility is not exercised.

### Causal limitations

O1 and O2 are post-perturbation observational summaries of a fixed generated transition system. They are not independently assigned interventions, and the baseline is not a proven causal adjustment set. TDI-5.3 therefore cannot identify the causal effect of changing O2, a mediation effect through O2, or an intervention that would improve future recovery. A future controlled-perturbation revision must define manipulable mechanisms and potential outcomes without calling observational association causality.

## Prohibited overclaims

The following claims are not justified and must not appear in TDI-5.4 motivation, results, or release language unless a new protocol directly establishes them:

- "O2 causes future recovery" or "manipulating O2 will change future deficit."
- "O1 is universally redundant given O2."
- "O2 is a sufficient statistic" without naming the generator, features, target, model families, loss, and tested populations.
- "TDI predicts width-6 outcomes accurately." The frozen evidence shows comparative improvement with poor absolute fit.
- "The result transfers to arbitrary widths, graph generators, action distributions, perturbations, or real systems."
- "Equivalence was shown under every metric." It was shown for primary standardized-U6 MSE with a +/-2% margin in the ridge family.
- "The bootstrap includes all sources of uncertainty" or "the confidence intervals are unconditional."
- "Multi-horizon success is independent replication across four outcomes."
- "Exact rational dynamics imply bit-identical end-to-end floating-point results on every architecture."

## Candidate next experiments

| Candidate | Validity question | Scientific value | Main cost or risk | Priority disposition |
|---|---|---|---|---|
| Nonlinear conditional sufficiency | Does O1 add held-out information after O2 when nonlinear interactions and thresholds are available? | Directly tests the weakest link in the current redundancy interpretation while holding the generator and estimand stable | Multiple model families can inflate false positives or overfit unless selection and multiplicity are frozen | **Selected for TDI-5.4** |
| Target-scale and calibration robustness | Do U-space conclusions agree with direct-O and calibration-aware losses? | Resolves the observed C metric disagreement and poor width-6 calibration | Could diffuse the primary question if made an unstructured metric search | Include as preregistered secondary/guardrail analyses in TDI-5.4 |
| Controlled perturbation | Does changing an identified mechanism associated with O2 change later deficit under structural controls? | First route toward causal evidence | A manipulable intervention and causal estimand are not yet defined | Retain as provisional TDI-5.5 only after reassessment |
| Generator/distribution robustness | Does the signal survive sparse, heterogeneous, rare, or adversarial regimes? | Addresses the strongest external-validity limitation | Large design space; generator choice can become adaptive | Retain as provisional TDI-5.6; consider promoting it after TDI-5.4 |
| Independent implementation | Does a separately structured evaluator reproduce essential outputs? | Detects shared implementation errors | Expensive and still shares mathematical definitions unless carefully isolated | Retain as TDI-5.7 |
| Minimal statistic/compression | Can a simpler interpretable statistic preserve O2's signal? | Improves interpretability and potential theory | Premature if nonlinear sufficiency fails | Retain as TDI-5.8 conditionally |

## Selected TDI-5.4 question

### Priority rationale

No more urgent defect was found that threatens the validity of the frozen TDI-5.3 estimand. The ridge-only nature of criterion C is nevertheless the closest threat to its most consequential interpretation. Testing model class now isolates one factor: the generator, perturbation, early-overlap definitions, target geometry, and conceptual B0/B1/B2/B12 comparisons can remain aligned while the function class becomes meaningfully nonlinear. This question is answerable with deterministic pure Rust, is falsifiable, and yields useful outcomes whether O1 is beneficial, equivalent, harmful, mixed across families, or inconclusive.

### Phase-A working formulation

This formulation is to be made exact in a TDI-5.4 preregistration before any activation evaluator is implemented.

For each preregistered model family `f`, define the held-out relative benefit of O1 given O2 as

\[
\Delta_f = \frac{L_f(B2)-L_f(B12)}{L_f(B2)},
\]

where `L` is the primary held-out standardized-U6 MSE and positive values favor B12.

- Primary question: does any preregistered nonlinear family reveal a replicable, practically meaningful conditional contribution from O1 that the frozen ridge model could not express?
- Null/absence-of-benefit hypothesis: O1 does not provide a preregistered practically meaningful improvement after O2 under the tested nonlinear families.
- Benefit alternative: at least one nonlinear family clears the frozen practical threshold with multiplicity-controlled aggregate evidence and preregistered block-replication requirements.
- Equivalence hypothesis: the uncertainty interval for O1's relative contribution is contained within the practical-equivalence region for every eligible confirmatory family under the frozen global interpretation rule.
- Proposed practical margin: retain TDI-5.3's +/-2% relative-MSE margin for continuity unless the preregistration supplies a pre-data scientific justification for a different value. It may not be changed after any new final-test outcome is examined.
- Model-class interaction question: is `Delta_f` materially different between ridge and a nonlinear family, rather than merely nonzero within one family?

The global result must distinguish at least: nonlinear benefit, cross-family practical equivalence, harmful contribution, model-class interaction/mixed evidence, and inconclusive evidence. A nonsignificant difference is not equivalence.

### Minimal scientifically motivated model set

The preregistration should prefer a small set with nonredundant purposes:

1. Frozen-style ridge as a continuity control, implemented in new additive TDI-5.4 code without modifying the old evaluator.
2. A deterministic degree-2 polynomial/interaction model with preregistered regularization, designed to detect smooth O1-by-O2 and O1-by-baseline effects.
3. A bounded deterministic piecewise model, such as a shallow regression tree or explicitly frozen hinge/spline basis, designed to detect thresholds and saturation that a polynomial misses.

A random-feature model or multilayer perceptron should be included only if the preregistration identifies a distinct remaining function class that the first two nonlinear challengers cannot test and supplies deterministic optimization, convergence, and runtime contracts. Model count is not evidence strength.

### Required design protections

The TDI-5.4 preregistration must, at minimum:

- allocate entirely new, pairwise-disjoint train, validation, test, OOD, bootstrap, model-initialization, and checkpoint seed domains;
- prohibit any reuse of the now-observed TDI-5.3 test seeds for confirmatory inference;
- freeze model grids and a nested validation rule before final-test access;
- compare B0, B1, B2, and B12 within every family using identical training and selection budgets;
- apply a preregistered multiplicity/global-decision rule across model families;
- carry direct-O error, calibration, rank correlation, clipping, and width-5/6 transfer as separately labeled secondary or guardrail outcomes;
- report training/validation/test loss gaps and model complexity to distinguish predictive benefit from overfitting;
- include training-aware uncertainty, or explicitly define the conditional uncertainty estimand if refitting bootstrap is computationally infeasible;
- define numerical degeneracy as a typed failure rather than silently substituting zero metrics;
- specify a cross-architecture numerical tolerance and demonstrate verdict stability at every decision boundary;
- implement schema-exact output validation and checkpoint/resume equivalence to an uninterrupted miniature oracle run.

### Scope and prohibited interpretation for TDI-5.4

Even a cross-family equivalence result would establish robustness only across the preregistered families and distributions. It would not prove mathematical sufficiency of O2 over all measurable functions, causal sufficiency, or generator universality. Conversely, a nonlinear benefit from O1 would falsify the broad ridge-derived redundancy interpretation without invalidating TDI-5.3's narrower frozen equivalence verdict.

## Roadmap reassessment

The provisional TDI-5.5 through TDI-5.8 roadmap remains scientifically plausible but is not activated by this audit. After TDI-5.4:

- if nonlinear O1 benefit is found, causal/interventional work should not presume O2 alone is the relevant mediator;
- if cross-family equivalence is found but calibration remains weak, calibration/target-scale work should precede strong transfer claims;
- if cross-family equivalence and calibration are both satisfactory, generator robustness may deserve priority before causal language;
- any mixed or negative result must be frozen and used to revise, postpone, or cancel later revisions rather than preserving the roadmap by default.

## Audit disposition

TDI-5.3 is **shareable with explicit caveats** for its preregistered predictive claims. The selected TDI-5.4 question is nonlinear conditional sufficiency of O2. The next lifecycle artifact is a complete TDI-5.4 preregistration; no TDI-5.4 evaluator or scientific activation code should be created before that preregistration is committed and pushed.
