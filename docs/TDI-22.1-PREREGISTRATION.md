# TDI-22.1 — Matched Torsor-Attention Preregistration

Status: **preregistration candidate; non-final; no evaluator or result execution authorised by this file**.

## 1. Scientific question

TDI-22.1 freezes a bounded reference comparison for asking whether the declared torsor/twist structure contributes measurable behavior beyond (a) ordinary vector scoring and (b) a matched six-component non-torsor control. The Stage-0 algebra is already frozen separately and is not itself evidence of useful attention behavior.

The primary torsor-specific contrast is **T3 versus T4**. T3 versus T0 is contextual only and cannot establish a torsor-specific effect.

## 2. Reference arm ladder

All arms operate on the same evaluator-owned episode and receive the same admissible source fields. They are deterministic reference mechanisms; no training occurs in the initial TDI-22.2 development evaluator.

### T0 — vector reference

A conventional six-component query/key score. Query and key are represented as two three-component blocks but no Varignon transport or cross product is used. This arm establishes a competent same-width vector baseline.

### T1 — torsor value only

Uses the T0 score unchanged. Values are represented as `(R, C)` where `C = M(P) + P x R`; weighted aggregation is performed component-wise. T1 isolates the value representation from score semantics.

### T2 — hybrid score

Uses one preregistered convex score mixture

`score = alpha * score_T0 + (1 - alpha) * score_T3`

with `alpha` fixed before any TDI-22.2 result execution. TDI-22.1 does not select a numerical alpha from results. Until a later freeze resolves it, T2 remains implementation-blocking rather than silently defaulting to a convenient value.

### T3 — full torsor score

Uses the frozen TDI-22.0 dual pairing. For query twist `(v, omega)` at query position `Q`, and key torsor represented by `(R, C)`:

`score_T3 = (v + Q x omega) . R + omega . C`.

No conventional vector-score term is added in T3.

### T4 — matched six-component non-torsor control

Uses the same six scalar query channels and six scalar key channels as T3 and the same scalar dot-product reduction width, but excludes the geometric transport term `Q x omega`. Its score is

`score_T4 = v . R + omega . C`.

T4 therefore controls six-component width and the `(R, C)` key storage while removing the declared Varignon query transport. It is not claimed to match T3's exact arithmetic operation count; operation counts are recorded separately and never hidden inside the quality verdict.

## 3. Task families

The initial bounded evaluator must contain all three families. Their roles are intentionally different so the study is not tautologically built only from torsor-consistent targets.

### F1 — transport-consistent geometric retrieval

Evaluator-generated finite `(v, omega, Q)` queries and finite torsor keys define the target by the frozen direct dual pairing. Distractors include keys whose local moments differ but whose resultants or origin moments can be similar. Reduction points are varied independently. This is the positive structural-control family for the declared torsor law.

### F2 — generic six-dimensional bilinear retrieval

Targets are generated from an evaluator-owned generic six-component dot-product rule that does not apply Varignon transport. The same six scalar payload channels are available to all arms. This family checks whether T3 is merely privileged by F1 task construction; T4/T0 competence must remain visible.

### F3 — position-nuisance associative retrieval

Target identity is determined by a content key independent of geometric position. Positions are varied as nuisance variables while the content/target mapping is unchanged. This family checks whether torsor transport introduces avoidable degradation when geometry is irrelevant.

A later materially different task family requires a new preregistered TDI-22 amendment; it cannot be added after observing TDI-22.2 results and then substituted into the primary set.

## 4. Geometry registry

Initial deterministic geometry families are:

- `G0_ORIGIN`: every query/key reference position is `(0,0,0)`; this removes transport and acts as a degeneracy control.
- `G1_LINEAR`: item index `i` maps to `(i,0,0)` after exact conversion to binary64 within the bounded index domain.
- `G2_HELIX`: item index maps to a fixed deterministic helix whose numerical constants must be frozen before TDI-22.2 execution.
- `G3_SUPPLIED`: F1 may use evaluator-supplied finite three-dimensional coordinates generated independently of candidate outputs.

Learned latent geometry is **not** part of the first TDI-22.2 evaluator. It remains a later TDI-22.5 question because adding training would confound the initial score-semantic comparison.

## 5. Normalisation and readout

Primary score-semantic evaluation uses ranking before any softmax-like normalization:

1. target top-1 retrieval accuracy;
2. signed target margin = target score minus the maximum distractor score.

For T1 value-only episodes, the primary value metric is exact/declared-tolerance reconstruction of the evaluator-owned `(R, C)` target after aggregation under the same fixed weight rule used by its matched T0 episode.

Any normalized-weight/NLL analysis is secondary unless separately frozen. A later FLAT-ATTENTION kernel study owns production normalization semantics.

## 6. Matched resource ledger

Every arm records, without converting these counts into runtime claims:

- query payload bits;
- key payload bits;
- value payload bits;
- position bits retained by the candidate;
- dynamic state/cache bits;
- temporary scalar/vector slots required by the reference scoring step;
- scalar additions;
- scalar multiplications;
- cross-product evaluations;
- dot-product scalar lanes;
- comparisons used by ranking;
- static constants/parameter bits.

Reference arithmetic is deterministic IEEE-754 binary64. Integer indexing and accounting use checked integer arithmetic. Non-finite arithmetic fails closed.

No memory, complexity, bandwidth, latency, GPU or end-to-end FLAT claim follows from these reference counts.

## 7. Episode bounds

The first development evaluator must be finite and bounded. Before TDI-22.2 execution, the freeze must resolve concrete limits for:

- candidates per query;
- queries per episode;
- episode count per family/geometry cell;
- maximum absolute coordinate magnitude;
- maximum absolute component magnitude;
- fixed deterministic generation seed domains.

Unresolved numerical limits block execution; implementations must not invent defaults.

## 8. Split and seed discipline

TDI-22.2 development and validation populations use domain-separated deterministic derivation. The canonical labels are:

- `tdi22/dev/v1`;
- `tdi22/validation/v1`.

A concrete generator derives per-record seeds from `(domain_label, record_index)` using a separately implemented deterministic hash/PRNG contract that must be frozen before execution. Development and validation labels must never overlap.

No final/confirmatory TDI-22 seed list, dataset, result payload or runner is authorised here. Any future confirmation requires a separate preregistration that freezes population size, entropy/seed derivation, no-retry policy and decision thresholds before the final entropy/material is knowable.

## 9. Primary contrasts

The initial development evaluator reports paired observations for:

- `C1`: T1 versus T0 on value reconstruction;
- `C2`: T2 versus T0 and T3, descriptive until alpha is separately frozen;
- `C3`: T3 versus T4 on F1/F2/F3 ranking behavior — the critical torsor-specific contrast;
- `C0`: T3 versus T0 as contextual reference only.

No aggregate win/loss score across heterogeneous families may replace the family-wise records.

## 10. Rejection taxonomy

At minimum, the evaluator must fail closed with typed reasons for:

- malformed episode;
- non-finite input component;
- non-finite derived component;
- invalid geometry identifier;
- out-of-bound coordinate/component/index;
- missing target;
- duplicate target identity;
- candidate/target length mismatch;
- unresolved protocol parameter;
- arithmetic/accounting overflow;
- unexpected extra output.

Rejected records remain counted by reason. Result-dependent exclusion is forbidden.

## 11. Interpretation rules

A later positive F1 result for T3 is insufficient by itself. Torsor-specific support requires the preregistered T3-vs-T4 comparison, with F2 and F3 retained so specialization/harm is visible.

A bounded TDI-22 result does not establish novelty, general language-model quality, universal replacement of vector attention, better KV memory, asymptotic improvement, production safety, or device performance.

Negative, null, harmful, equivalent and inconclusive outcomes remain first-class evidence and retain their stage identity.

## 12. Stage gate

This preregistration does not itself authorize TDI-22.2 execution. Before TDI-22.2 evaluator implementation/execution:

1. this document and its status/gate checks must be merged;
2. TDI-22.1 must be separately content-addressed/frozen;
3. every unresolved numerical field required by the bounded evaluator must be resolved in that freeze;
4. the freeze must explicitly authorize non-final TDI-22.2 development/validation execution;
5. `bash scripts/check-tdi22-freeze.sh` and the TDI-22.1 preregistration integrity gate must pass on the exact merge candidate.

FLAT-ATTENTION remains downstream and untouched by TDI-22.1.