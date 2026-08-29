# TDI-7.0 — Attention recovery preregistration

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-1:

> On deterministic associative-recall and copy tasks, do early intervention-conditioned recovery descriptors predict later retrieval deficit beyond competent static attention diagnostics?

The confirmatory object is incremental predictive value. Correlation between a TDI recovery feature and failure is not sufficient.

## 2. Scope

TDI-7.0 freezes the scientific protocol only. It contains no confirmatory evaluator result, no final holdout output, no language-model training, and no FLAT-ATTENTION performance claim.

The first implementation remains a cheap deterministic mechanistic gate. FLAT-ATTENTION is an execution/oracle target once semantics are frozen, not the discovery environment.

## 3. Confirmatory task families

Exactly two task families are confirmatory:

1. **Associative recall** — key/value associations, a query key, distractors, and a deterministic retrieval target.
2. **Copy** — a source subsequence reproduced at a declared downstream position under deterministic sequence-mixing semantics.

Overwrite, induction, language modelling, or additional tasks remain exploratory unless separately preregistered.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds.

The evaluator must freeze four disjoint seed ranges before generation:

- training;
- development;
- validation;
- final holdout.

No seed may occur in more than one split. Exact ranges and population sizes must be constants emitted in provenance.

The final holdout must not be used to choose interventions, observation depths, baseline features, recovery metric, model class, regularization, or decision margins. Any such change after holdout access requires a new preregistration and a fresh disjoint holdout.

## 5. Reference semantics

The evaluator must use one deterministic attention or sequence-mixing semantic with independently testable scalar/reference behavior.

If StandardSoftmax is used, its mathematical output is

`softmax(Q K^T / sqrt(d)) V`

under an explicitly declared numerical policy.

Semantic identity, dimensions, mask/causality rule, accumulation type, and tolerance policy must be emitted in provenance. GPU execution may later be compared but must not define the scientific semantics.

## 6. Intervention taxonomy

The confirmatory intervention family is restricted to single-site state perturbations that leave the task target unchanged.

At least two intervention locations must be represented when exposed by the semantic, selected from token representation, key/value state, memory slot, or structured operator mode/coefficient.

The evaluator freezes the exact intervention operator and amplitude using training/development data only.

The intervention is applied once. Reference and perturbed trajectories then evolve under identical downstream dynamics. Interventions that alter the correct task label are invalid.

## 7. Early TDI recovery descriptors

TDI features are computed from downstream reference/perturbed observable pairs strictly before the target evaluation depth.

The evaluator must freeze the observable, recovery metric, early observation depths, and any transforms.

At minimum the TDI block contains the raw recovery values at each frozen early depth. Derived features are allowed only if declared before final holdout access and computed solely from these frozen early values.

No target label or late retrieval deficit may enter a TDI feature.

## 8. Static baseline descriptors

The competent static baseline contains, when defined:

- mean row Shannon entropy;
- mean normalized row entropy;
- mean row maximum weight;
- mean row L2 concentration;
- mean entropy-derived effective support;
- Frobenius norm;
- operator dimensions and task-local size controls.

Entropy-derived support must not be called effective rank.

If effective-rank or spectral features are added, their exact numerical definition and tolerance policy must be frozen before holdout access. Historical exact finite-state spectral moments must not be transferred by name to floating-point attention.

Task-local controls include sequence length, distractor count, and retrieval distance whenever they vary.

## 9. Prediction target

For each example/intervention pair the late target is a scalar retrieval deficit:

- `0` means no observed late degradation relative to the reference;
- larger positive values mean greater degradation.

The exact formula must be deterministic, finite, and bounded or accompanied by an explicit validity range. It is evaluated strictly after all early observation depths.

Reference-task failures unrelated to the intervention are reported separately.

## 10. Primary model ladder

Exactly two nested feature blocks define the primary comparison:

- **B0 — static/task baseline:** task-local controls + static attention/operator diagnostics;
- **B1 — TDI augmented:** B0 + frozen early TDI recovery descriptors.

The primary model class is identical for B0 and B1. Hyperparameters are fitted or selected using training/development data only.

ITD structural descriptors are evaluated separately and jointly in the cross-project harness; they do not replace B0 in the TDI primary comparison.

## 11. Primary loss

The primary predictive loss is mean squared error on retrieval deficit.

B0 and B1 are evaluated on the same holdout example/intervention pairs.

`relative_MSE_reduction = (MSE_B0 - MSE_B1) / MSE_B0`

If `MSE_B0 <= 0` or is non-finite, the primary comparison is Inconclusive.

Uncertainty uses paired resampling at the generator-example level. The evaluator reports the point estimate and a two-sided 95% paired bootstrap interval.

## 12. Frozen decision rule

Let `r` be relative MSE reduction and `[L, U]` its 95% paired bootstrap interval.

- **Beneficial** if `r >= 0.02` and `L > 0`;
- **Harmful** if `r <= -0.02` and `U < 0`;
- **Equivalent** if the complete interval lies inside `[-0.02, +0.02]`;
- **Inconclusive** otherwise.

The ±2% band is a preregistered relevance margin for TDI-7.0. It is not inherited from historical TDI effect sizes and is not claimed universal.

## 13. Multi-task gate

Associative recall and copy are classified separately.

Overall TDI-7 H-AI-1 gate:

- **Beneficial** if neither task is Harmful and at least one is Beneficial;
- **Equivalent** if both are Equivalent;
- **Harmful** if either task is Harmful;
- **Inconclusive** otherwise.

Negative, equivalent, and inconclusive outcomes remain valid scientific results.

## 14. Intervention-location reporting

All preregistered intervention locations remain in the aggregate result. No location may be removed because it weakens the result.

Location-specific effects are secondary in TDI-7.0. A dedicated intervention-heterogeneity hypothesis belongs to a later TDI-7.x preregistration.

## 15. Determinism and provenance

The evaluator must emit at least:

- TDI commit SHA;
- external harness commit SHA when used;
- semantic identifier;
- task generator version;
- split seed ranges;
- intervention definition and amplitude;
- observation depths;
- feature schema;
- model configuration;
- bootstrap seed and replicate count;
- numerical policy;
- classifier margins.

Repeated execution with the same inputs must reproduce generated populations and deterministic model inputs. Bit-exact cross-platform floating-point identity is not assumed unless independently demonstrated.

## 16. Final-holdout lock

The evaluator must include an explicit final-run confirmation guard.

CI and ordinary tests must validate generator invariants, split disjointness, feature construction, one-shot intervention semantics, target orientation, paired-resampling mechanics, and classifier boundaries without producing final holdout results.

The final-run confirmation token must not be supplied by CI, tests, or examples.

## 17. Required reporting

For both tasks the final report must contain:

- B0 MSE;
- B1 MSE;
- relative MSE reduction;
- 95% paired interval;
- Beneficial / Equivalent / Harmful / Inconclusive;
- invalid/rejected counts and reasons;
- intervention-location summaries;
- provenance.

No negative or failed result may be omitted.

## 18. Relationship to ITD Simulator

ITD Simulator remains the comparative experimental harness. The intended ablation ladder is:

`static/task baseline -> ITD structural descriptors -> TDI recovery descriptors -> ITD+TDI`.

TDI and ITD are distinct information sources and must remain separately measurable.

## 19. Relationship to FLAT-ATTENTION

A positive TDI-7 result does not itself select a FLAT semantic.

It only establishes that early intervention-conditioned recovery adds predictive information beyond the frozen static baseline on at least one preregistered mechanistic task.

Promotion of any non-softmax FLAT semantic still requires a frozen mathematical specification, scalar Rust oracle, invariant tests, matched controls, failure-mode analysis, and reproducible cost/quality evidence.

## 20. TDI-7.x programme map

The initial numbering is reserved as follows:

- **TDI-7.0** — H-AI-1 preregistration and holdout protocol;
- **TDI-7.1** — bounded deterministic evaluator and software-oracle validation, without final holdout execution;
- **TDI-7.2** — confirmatory H-AI-1 holdout execution and frozen result;
- **TDI-7.3** — intervention-location heterogeneity / coupling-stability study (H-AI-2), if justified;
- **TDI-7.4** — long-horizon joint-information/synergy study (H-AI-3), if justified;
- **TDI-7.5+** — static spectral insufficiency and candidate FLAT semantic comparisons (H-AI-4 and successors), each behind its own preregistration.

Later numbering may be extended, but a confirmatory stage must never be retroactively renumbered to hide a failed or negative result.

## 21. Non-claims

Freezing TDI-7.0 does not establish that TDI improves attention, transfers to Transformers, identifies causality, selects a FLAT semantic, or provides any GPU performance gain.

The next implementation step after this document is merged and hashed is TDI-7.1: implement the bounded evaluator against the frozen protocol without changing these scientific choices.
