# TDI-22.x — Torsor Attention Research Programme

Status: **active Stage-0 bootstrap; not frozen; no confirmatory execution authorised**.

## Research question

Can a six-component torsor/twist representation provide useful attention semantics under controlled, matched comparisons, and can it eventually replace or complement conventional vector `QK^T` scoring without hiding extra capacity, geometry, or memory budget?

TDI-22 studies the scientific semantics only. FLAT-ATTENTION remains the owner of production attention execution, Kernel IR, WGPU kernels, routing, and performance qualification.

## Stage-0 mathematical convention

TDI-22.0 fixes one explicit three-dimensional sign convention for a wrench-like torsor reduced at point `P`:

```text
T(P) = (R, M(P))
M(Q) = M(P) + (P - Q) x R
```

Define the origin-reduced moment

```text
C = M(P) + P x R.
```

Then

```text
M(Q) = C - Q x R.
```

For a twist-like query

```text
xi = (v, omega),
```

the direct pairing at query position `Q` is

```text
s_direct = v . R + omega . M(Q).
```

By the scalar-triple-product identity, the same scalar can be factorized as

```text
s_factorized = (v + Q x omega) . R + omega . C.
```

TDI-22.0 treats this equality as a mathematical identity to be checked by deterministic Rust tests. It is **not** evidence that the scalar is a better attention score.

## Why the factorization matters

The source discussion proposed evaluating Varignon transport for each query/key pair. The factorization above separates the query-dependent and key-dependent terms:

```text
query:  (v + Q x omega, omega)
key:    (R, C)
```

This creates a six-component bilinear form without requiring one cross product per query/key pair. TDI-22 will test the semantics before any systems-level claim is made.

## Candidate/control ladder

The planned ladder deliberately separates representation effects from extra dimensionality:

- **T0 — Vector reference.** Conventional bounded vector attention reference used only as a scientific control.
- **T1 — Torsor value only.** Conventional score, torsor-valued aggregation.
- **T2 — Hybrid score.** Declared mixture of conventional vector score and torsor dual pairing.
- **T3 — Full torsor score.** Torsor dual pairing replaces conventional vector score.
- **T4 — Matched six-component non-torsor control.** Equal-dimensional generic bilinear control without Varignon/torsor structure.

The critical comparison for a torsor-specific effect is T3 versus T4, not only T3 versus T0.

## Stage map

| Stage | Purpose | Status |
| --- | --- | --- |
| **TDI-22.0** | Freeze mathematical convention, exact transport/factorization scaffolding, Stage-0 boundaries | active bootstrap; not frozen |
| **TDI-22.1** | Freeze deterministic tasks, matched budgets, metrics, geometry families, splits and rejection rules | blocked until 22.0 freeze |
| **TDI-22.2** | T1 torsor-value evaluation versus T0 | planned |
| **TDI-22.3** | T2 hybrid-score evaluation | planned |
| **TDI-22.4** | T3 full-torsor score versus T0 and T4 matched six-component control | planned |
| **TDI-22.5** | Position/geometry ablations | conditional |
| **TDI-22.6** | Paged/hierarchical torsor-memory experiments | conditional |
| **TDI-22.7** | Boolean-routing × torsor cooperation | conditional |
| **TDI-22.8** | FLAT-ATTENTION graduation decision | evidence-gated |

## Stage-0 exact properties

The Stage-0 Rust scaffold must test at least:

1. round-trip change of reduction point;
2. invariance of `||R||^2` under reduction-point change;
3. invariance of `R . M(P)` under reduction-point change;
4. independence of `(R, C)` from the selected reduction point;
5. equality of direct and factorized dual pairings up to declared floating-point tolerance;
6. invariance of the factorized pairing under a common translation of the coordinate origin;
7. fail-closed behavior for NaN and infinite inputs.

These are algebra/implementation checks only.

## Geometry is an experimental variable

For ordinary language tokens there is no automatically privileged physical `R^3` position. TDI-22 must therefore treat token geometry as an explicit arm, not as an assumed fact. Candidate families for later preregistration may include:

- linear embedding of token index, e.g. `(i, 0, 0)`;
- deterministic helical or periodic embeddings;
- learned three-dimensional latent positions;
- externally supplied physical coordinates for tasks that genuinely have spatial geometry.

No geometry family is promoted before TDI-22.1 freezes the relevant comparison.

## Separation from existing TDI programmes

TDI-22 is distinct from:

- TDI-7.x, which studies intervention-conditioned attention/memory recovery;
- TDI-8.x, which studies bounded recurrent/associative alternatives;
- TDI-21.x, whose Boolean candidate arms explicitly forbid attention scoring;
- FLAT-ATTENTION, which owns execution semantics and optimized kernels.

TDI-22 may use existing deterministic TDI task infrastructure only through an explicit later protocol that prevents final-evaluation leakage and matched-budget ambiguity.

## FLAT-ATTENTION boundary

TDI-22.0 does not modify FLAT-ATTENTION and makes no kernel or hardware claim.

A later FLAT integration is permitted only if the TDI series produces reproducible evidence for a precisely specified candidate. FLAT must then independently qualify:

- correctness against its own reference semantics;
- storage and KV consequences;
- WGPU execution;
- latency/bandwidth/throughput on real devices;
- any runtime routing or ElasticXxx integration.

TDI evidence alone cannot establish those systems claims.

## Scientific interpretation rule

A positive bounded TDI-22 result would establish only that a declared torsor candidate outperformed or matched declared controls on the frozen tasks and budgets. It would not by itself establish novelty, general language-model quality, universal replacement of vector attention, asymptotic superiority, lower KV memory, or real-hardware speedup.

Negative, null, harmful, equivalent, and inconclusive outcomes must be retained under their original stage numbers.
