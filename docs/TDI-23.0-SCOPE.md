# TDI-23.0 — Stage-0 Scope and Stage Gate

Status: **BOOTSTRAP MERGED / SCIENTIFIC FREEZE UNRESOLVED / NON-FINAL**

## Purpose

TDI-23.0 bootstraps the categorical/dagger-attention programme without freezing comparative scientific choices or authorising confirmatory runs.

It lands:

- the TDI-23 programme map and Stage-0 status/scope surfaces;
- a machine-readable freeze **template** whose scientific fields remain `unresolved_blocking`;
- a fail-closed validator that rejects invented pins and execution authorisation;
- an opt-in `tdi-ai` `experimental` module implementing the bounded real finite-dimensional linear-map/adjoint scaffold;
- a second opt-in development module implementing coordinate-subspace global reduction, lift, reconstruction residual, and composition-defect auditing;
- deterministic tests for dagger involution, reversed composition, `k^dagger o q = <k,q>`, reduction/dagger commutation, explicit lossy composition defects, dimension rejection, and non-finite rejection;
- a bootstrap integrity script that validates documentation, the freeze template, formatting, clippy, and Stage-0 Rust tests.

The engineering bootstrap is merged, but the Stage-0 scientific freeze remains unresolved. The Stage-0 gate therefore remains normative and fail-closed while later TDI-23.x engineering stages proceed.

## Stage-0 semantic carrier

The only implemented categorical carrier in TDI-23.0 is a real finite-dimensional Hilbert-space analogue:

- scalar type: finite `f64`;
- inner product: standard Euclidean dot product;
- linear maps: finite row-major dense matrices;
- dagger: matrix transpose under the declared orthonormal bases;
- composition: ordinary matrix composition with typed dimensions;
- query/key kets: maps from a one-dimensional scalar space into a finite-dimensional carrier.

This is deliberately narrower than general `FdHilb` over complex scalars.

## Stage-0 global-reduction carrier

The first reduction model is deliberately narrower than arbitrary subspace compression. For each object `H = R^n`, Stage 0 may retain an ordered, duplicate-free coordinate subset represented by a standard-basis isometry `E_H : R^r -> R^n`.

For `f : H -> K`, reduction is

`R(f) = E_K^dagger f E_H`,

and lift is

`L(R(f)) = E_K R(f) E_H^dagger`.

This coordinate model is development-only. It exists to establish and falsify structural preservation claims before any learned, approximate, low-rank, or data-dependent reduction is considered. `docs/TDI-23.0-GLOBAL-REDUCTION.md` is the normative Stage-0 derivation.

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
| Coordinate reductions reject empty, duplicate and out-of-range coordinate sets | **EXACT engineering contract** |
| `R(f^dagger) = R(f)^dagger` for the Stage-0 coordinate reduction with swapped source/target reductions | **EXACT on accepted finite maps** |
| `reduce(lift(reduce(f))) = reduce(f)` for the selected coordinate block | **EXACT on accepted finite maps** |
| Composition preservation is not assumed; a max-absolute defect is computed explicitly | **EXACT scope boundary / diagnostic contract** |
| `softmax` is outside the implemented linear `FdHilb_R` scaffold | **EXACT scope boundary** |

## Explicit non-claims / forbidden upgrades

Stage 0 does **not**:

- claim that softmax is linear or a morphism of `FdHilb`;
- claim that Boolean, `F2`, Zhegalkin/ANF, or max-plus operators are `FdHilb` morphisms;
- claim the coordinate reduction is a quotient category, categorical equivalence, or generally functorial compression;
- claim a categorical representation or reduction improves model quality, memory, latency, throughput, or asymptotic complexity;
- implement or claim a quantum attention algorithm;
- freeze a Categorical Attention IR grammar;
- freeze an approximate-error tolerance or reduction-selection policy;
- authorize TDI-23.6 dagger-constrained or learned-subspace training experiments;
- modify FLAT-ATTENTION;
- authorize confirmatory or final TDI-23 execution.

## Stage gate toward TDI-23.1

The prerequisites that allowed TDI-23.1 engineering work to begin remain preserved as historical and ongoing integrity conditions:

1. `docs/TDI-23-PROGRAMME.md`, `docs/TDI-23.0-SCOPE.md`, `docs/TDI-23.0-STATUS.md`, and `docs/TDI-23.0-GLOBAL-REDUCTION.md` are merged;
2. `docs/tdi23/tdi23.0-stage0-freeze.template.json` remains present with both execution flags `false`;
3. `scripts/check-tdi23.0-freeze-template.py --self-test` passes;
4. `scripts/check-tdi23-stage0-bootstrap.sh` passes;
5. the experimental Rust scaffold remains explicitly real-valued and does not silently generalize numerical assumptions;
6. the reduction scaffold continues to expose composition loss rather than silently asserting functoriality;
7. any Stage-0 scientific field needed by a later TDI-23.x stage is resolved only by an explicit later freeze decision.

## Required Stage-0 controls

The Stage-0 Rust tests must include:

- identity-compatible finite maps;
- at least one rectangular map;
- at least one non-commuting composition chain;
- dimension-mismatch rejection;
- non-finite-entry rejection;
- query/key score equivalence through explicit ket/bra composition;
- invalid coordinate-reduction rejection;
- coordinate reduction commuting with dagger;
- reduce/lift/reduce idempotence on the selected block;
- zero composition defect with a fully retained ordered intermediate basis on a deterministic fixture;
- a deliberately omitted intermediate path producing its predicted non-zero composition defect;
- a non-zero reduce/lift residual when discarded entries exist.

These controls do not freeze later evaluator populations, reduction policies, tolerances, or performance metrics.

## Promotion boundary

Promotion to FLAT-ATTENTION, SciRust, Forge, NNIS, or another Memorithm repository requires a separate contract. TDI-23.0 produces only bounded research scaffolding and integrity checks.
