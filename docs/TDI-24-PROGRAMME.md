# TDI-24.x — Vector vs Chiral Attention Research Programme

Status: **Stage-0 bootstrap; experimental; non-confirmatory**.

Tracker: #391.

## Research question

At matched representation width, parameter count, data, seeds, optimization/training budget, memory budget and evaluation protocol, does an explicitly mirror-coupled chiral representation provide a reproducible advantage, disadvantage, or equivalence relative to a conventional vector representation?

TDI-24 is intentionally narrow: **vector vs chiral**. Torsor semantics are excluded from the primary comparison and belong to TDI-22/TDI-25.

## Stage-0 chiral contract

The initial reference carrier has six components and a parity decomposition

```text
H = H+ ⊕ H-
x = (x+, x-),  x+,x- ∈ R^3.
```

Define

```text
M(x+,x-) = (x+,-x-)
J(x+,x-) = (x-,-x+)
```

with identities

```text
M² = I
Jᵀ = -J
J² = -I
M J M = -J.
```

For query `q` and key `k`, Stage 0 exposes three observables separately:

```text
s(q,k)   = qᵀ k
m(q,k)   = qᵀ M k
chi(q,k) = qᵀ J k.
```

Under a simultaneous mirror transformation,

```text
s(Mq,Mk)   =  s(q,k)
m(Mq,Mk)   =  m(q,k)
chi(Mq,Mk) = -chi(q,k).
```

`chi` is therefore the declared parity-odd channel. This algebraic property is a testable identity, not evidence that the channel improves attention.

A scalar candidate family may later use

```text
score_C = alpha*s + beta*m + gamma*chi,
```

while the matched vector control uses the same six-component carrier and

```text
score_V = s.
```

Score normalization, softmax/normalizer choice, training, task populations and any learned parameterization are deliberately outside Stage 0 and must be frozen by later slices before evaluation.

## Primary arms

- **V6 — matched vector control.** Six-component conventional dot-product representation.
- **C6 — chiral candidate.** Same carrier width with the declared M/J structure and parity-odd channel.

Ablations are attribution controls, not additional primary hypotheses.

## Core attribution ablations

Later preregistered slices may test:

- `gamma = 0` — removes the parity-odd channel;
- `beta = 0` — removes the mirror-even coupling;
- direct-only — collapses C6 back to the matched vector score;
- parity shuffle — preserves six values but destroys the declared H+/H- organization;
- fixed versus learned structure — only after an exact structure-preserving parameterization is qualified.

## Required task families

The campaign must contain both positive and negative controls.

1. **Reflection-discriminative tasks** where paired inputs differ only by a declared parity transformation and the target depends on handedness.
2. **Reflection-nuisance tasks** where paired inputs differ by reflection but the target must remain unchanged.
3. **Directional/reversal tasks** that test ordered relations without silently making reflection the answer.
4. **Non-chiral controls** where no parity information is relevant and extra structure should not create a benefit by construction.

No task family may read protected labels inside the inference path. Development, Validation and any later protected/final population must be provenance-bound and disjoint.

## Matching requirements

Before any comparative claim, V6 and C6 must be matched or explicitly normalized for:

- carrier width;
- trainable parameter count;
- optimizer and update count;
- training examples and ordering;
- initialization policy;
- sequence length and masking;
- precision;
- compute/operation accounting;
- resident/peak memory accounting;
- evaluation seeds and task instances.

Any unavoidable mismatch is a reported experimental factor, not hidden capacity.

## Metrics

The primary quality metric is task-specific and must be frozen before the corresponding evaluation slice. Cross-cutting diagnostics include:

- paired task outcome difference;
- mirror-swap identity error;
- parity-equivariance/invariance error where applicable;
- calibration or confidence error when predictions expose scores;
- gradient/stability diagnostics for trained arms;
- operation count, memory, latency and throughput only under an explicitly qualified execution environment.

## Scientific interpretation

A positive TDI-24 result establishes only that the declared C6 candidate outperformed the matched V6 control on the frozen populations and budgets. It does not establish universal superiority, novelty, lower asymptotic complexity, language-model quality, FLAT-ATTENTION readiness or hardware speedup.

A negative, null, harmful, unstable or inconclusive result is retained and reported under its original campaign slice.

## Boundary with TDI-25

TDI-25 reuses this exact chiral contract and compares it with the TDI-22 torsor representation. TDI-25 must not redefine chirality. Conversely, TDI-24 must not import torsor semantics into its primary comparison.

## Boundary with FLAT-ATTENTION

TDI-24 owns scientific semantics and controlled evidence only. FLAT-ATTENTION remains responsible for any downstream production attention semantics, optimized kernels, hardware qualification or runtime integration. Promotion requires a separate evidence-gated decision.

## Campaign execution

The first bounded campaign consists of **50 substantive PR slices** described in `docs/TDI-24-CAMPAIGN-50.md`. PR count is not permission for empty churn: every slice must add a concrete protocol, implementation, test, analysis, reproducibility or evidence artifact and satisfy its predecessor gates.
