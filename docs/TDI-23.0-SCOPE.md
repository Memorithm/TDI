# TDI-23.0 — Stage-0 Scope and Stage Gate

Status: **ACTIVE STAGE-0 BOOTSTRAP / NON-FINAL**

## Purpose

TDI-23.0 bootstraps the categorical/dagger-attention programme without freezing comparative scientific choices or authorising confirmatory runs.

It lands:

- the TDI-23 programme map and Stage-0 status/scope surfaces;
- a machine-readable freeze **template** whose scientific fields remain `unresolved_blocking`;
- a fail-closed validator that rejects invented pins and execution authorisation;
- an opt-in `tdi-ai` `experimental` module implementing the bounded real finite-dimensional linear-map/adjoint scaffold;
- deterministic tests for dagger involution, reversed composition, `k^dagger o q = <k,q>`, dimension rejection, and non-finite rejection;
- a bootstrap integrity script that validates documentation, the freeze template, formatting, clippy, and Stage-0 Rust tests.

## Stage-0 semantic carrier

The only implemented categorical carrier in TDI-23.0 is a real finite-dimensional Hilbert-space analogue:

- scalar type: finite `f64`;
- inner product: standard Euclidean dot product;
- linear maps: finite row-major dense matrices;
- dagger: matrix transpose under the declared orthonormal bases;
- composition: ordinary matrix composition with typed dimensions;
- query/key kets: maps from a one-dimensional scalar space into a finite-dimensional carrier.

This is deliberately narrower than general `FdHilb` over complex scalars.

## Exact Stage-0 claims

| Claim | Status |
| --- | --- |
| Constructor rejects zero dimensions, storage mismatch and non-finite entries | **EXACT engineering contract** |
| Dagger swaps domain/codomain and transposes matrix entries | **EXACT** |
| `(f^dagger)^dagger = f` on accepted Stage-0 maps | **EXACT** |
| Composition rejects incompatible domain/codomain dimensions | **EXACT engineering contract** |
| `(g o f)^dagger = f^dagger o g^dagger` on deterministic finite fixtures | **EXACT** |
| Ket construction represents `R -> H` as a column map | **EXACT engineering convention** |
| `k^dagger o q` equals the declared Euclidean dot product on deterministic finite fixtures | **EXACT** |
| Non-finite vectors/derived arithmetic are rejected rather than silently propagated | **EXACT engineering contract** |
| `softmax` is outside the implemented linear `FdHilb_R` scaffold | **EXACT scope boundary** |

## Explicit non-claims / forbidden upgrades

Stage 0 does **not**:

- claim that softmax is linear or a morphism of `FdHilb`;
- claim that Boolean, `F2`, Zhegalkin/ANF, or max-plus operators are `FdHilb` morphisms;
- claim a categorical representation improves model quality, memory, latency, throughput, or asymptotic complexity;
- implement or claim a quantum attention algorithm;
- freeze a Categorical Attention IR grammar;
- authorize TDI-23.6 dagger-constrained training experiments;
- modify FLAT-ATTENTION;
- authorize confirmatory or final TDI-23 execution.

## Stage gate toward TDI-23.1

TDI-23.1 IR work may begin only when:

1. `docs/TDI-23-PROGRAMME.md`, `docs/TDI-23.0-SCOPE.md`, and `docs/TDI-23.0-STATUS.md` are merged;
2. `docs/tdi23/tdi23.0-stage0-freeze.template.json` remains present with both execution flags `false`;
3. `scripts/check-tdi23.0-freeze-template.py --self-test` passes;
4. `scripts/check-tdi23-stage0-bootstrap.sh` passes;
5. the experimental Rust scaffold remains explicitly real-valued and does not silently generalize numerical assumptions;
6. any field needed by TDI-23.1 is resolved only by an explicit later PR.

## Required Stage-0 controls

The Stage-0 Rust tests must include:

- identity-compatible finite maps;
- at least one rectangular map;
- at least one non-commuting composition chain;
- dimension-mismatch rejection;
- non-finite-entry rejection;
- query/key score equivalence through explicit ket/bra composition.

These controls do not freeze later evaluator populations or performance metrics.

## Promotion boundary

Promotion to FLAT-ATTENTION, SciRust, Forge, NNIS, or another Memorithm repository requires a separate contract. TDI-23.0 produces only bounded research scaffolding and integrity checks.
