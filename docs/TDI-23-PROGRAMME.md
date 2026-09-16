# TDI-23.x — Categorical / Dagger Attention Research Programme

Status: **active Stage 0 bootstrap; not frozen; no confirmatory execution authorised**.

## Research question

Can a typed categorical representation of finite-dimensional attention operators make semantic equivalence, legal rewrites, and lowering decisions explicit enough to improve verification and later execution planning without changing the declared attention semantics?

TDI-23 studies the scientific and formal semantics. FLAT-ATTENTION remains the owner of production attention execution, kernels, routing, device qualification, and performance claims.

## Stage-0 mathematical convention

TDI-23.0 begins with finite-dimensional **real** Hilbert spaces only. The Stage-0 carrier is therefore a bounded `FdHilb_R` analogue using finite `f64` vectors, the standard Euclidean inner product, and row-major dense linear maps.

For a linear map

`f : H -> K`,

the Stage-0 dagger is the real adjoint

`f^dagger : K -> H`,

implemented as matrix transpose under the declared orthonormal bases.

For query and key kets

`q : R -> H` and `k : R -> H`,

the scalar composition

`k^dagger o q : R -> R`

must equal the Euclidean pairing

`<k, q> = k^T q`.

This is the bounded formal identity that connects the Stage-0 categorical scaffold to the dot-product core used by conventional attention.

## Critical semantic boundary

TDI-23 MUST NOT claim that complete softmax attention is a morphism of `FdHilb`.

`softmax` is nonlinear. Boolean logic, `F2`, algebraic-normal-form/Zhegalkin operations, and max-plus operations also retain their own algebraic laws and MUST NOT be silently treated as `FdHilb` morphisms.

Later work may study explicit interfaces between such domains, but every interface must declare what structure is preserved and what structure is not preserved.

## Primary null

A categorical/dagger layer provides no reproducible verification, rewrite, or planning advantage over direct typed linear algebra after accounting for implementation complexity and runtime overhead.

A null, negative, equivalent, or harmful result is publishable under the original TDI-23 stage number.

## Stage map

| Stage | Purpose | Status |
| --- | --- | --- |
| **TDI-23.0** | Stage-0 bootstrap: formal boundary, real finite-dimensional dagger scaffold, exact structural laws, freeze template and fail-closed gates | active bootstrap; not frozen |
| **TDI-23.1** | Freeze a typed Categorical Attention IR grammar and deterministic equivalence fixtures | blocked until 23.0 gate |
| **TDI-23.2** | Rewrite calculus: composition, dagger, tensor/direct-sum distinctions, identity elimination and legality checks | planned |
| **TDI-23.3** | Attention-score mapping and batched `QK^T` equivalence fixtures | planned |
| **TDI-23.4** | Lower one abstract graph to multiple CPU reference execution plans | conditional |
| **TDI-23.5** | Explicit interfaces to Boolean / `F2` / ANF / max-plus domains | conditional |
| **TDI-23.6** | Dagger-constrained representation experiments such as `W^dagger W ~= I` | conditional; requires separate preregistration |
| **TDI-23.7** | Rewrite/planning search with matched semantic and resource accounting | conditional |
| **TDI-23.8** | FLAT-ATTENTION graduation decision and independent systems qualification | evidence-gated |

## Stage-0 exact / engineering objectives

The bootstrap is allowed to establish only bounded algebra/implementation facts:

1. typed domain/codomain dimensions are preserved by construction;
2. the dagger swaps domain and codomain and transposes the declared dense matrix;
3. dagger involution holds on finite Stage-0 maps: `(f^dagger)^dagger = f`;
4. composition rejects incompatible dimensions;
5. dagger reverses composition: `(g o f)^dagger = f^dagger o g^dagger`;
6. a query/key ket composition `k^dagger o q` equals the declared Euclidean dot product on deterministic finite fixtures;
7. non-finite inputs and non-finite derived values fail closed;
8. no Stage-0 surface authorises confirmatory attention-quality or hardware-performance claims.

These are algebra/implementation checks, not evidence that categorical attention is better than another attention mechanism.

## Required controls for later evaluator stages

Before any comparative evaluator is frozen, TDI-23 must include at least:

- direct dense linear-algebra execution as the semantic reference;
- identity and zero maps;
- rectangular maps;
- non-commuting compositions;
- dimension-mismatch rejection;
- non-finite-input rejection;
- rewrite-disabled control;
- matched operation/memory accounting when execution-plan comparisons begin.

## Tensor versus direct-sum discipline

Multi-head concatenation MUST NOT be represented as a tensor product merely because the programme is monoidal. Ordinary head concatenation is naturally modeled as a direct sum/biproduct-like construction. Tensor products are reserved for declared joint/composite systems where the multiplicative dimension is intended.

TDI-23.0 does not yet implement either construction in the public scaffold; the distinction is recorded now to prevent an invalid later IR design.

## FLAT-ATTENTION boundary

TDI-23.0 does not modify FLAT-ATTENTION.

A later FLAT integration is permitted only if a TDI-23 candidate produces reproducible evidence and a precisely versioned semantic contract. FLAT-ATTENTION must then independently qualify:

- equivalence to its own reference semantics;
- numerical tolerances;
- memory/storage consequences;
- WGPU or other device lowering;
- latency, bandwidth and throughput on real hardware;
- routing and fallback behaviour.

TDI evidence alone cannot establish those systems claims.

## Ecosystem boundary

SciRust may later host reusable, generally useful algebraic primitives only after their semantics stabilize. Forge may later search legal rewrite/planning spaces only after TDI-23 defines a bounded grammar and validity oracle. Neither promotion is authorised by TDI-23.0.

## Scientific interpretation rule

A positive bounded TDI-23 result would establish only that a declared categorical representation or rewrite procedure satisfied the frozen semantic and evaluation criteria on the declared tasks and budgets. It would not by itself establish novelty, universal optimality, replacement of softmax attention, quantum computation, lower asymptotic complexity, or hardware speedup.
