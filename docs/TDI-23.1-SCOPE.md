# TDI-23.1 Scope — Typed Categorical Attention IR

Status: **ACTIVE DEVELOPMENT / NOT FROZEN / NO CONFIRMATORY EXECUTION AUTHORIZED**

## Purpose

TDI-23.1 introduces the first bounded typed graph grammar for categorical/dagger attention research. The objective is to make semantic legality explicit before any rewrite search, execution-plan search, or FLAT-ATTENTION integration is attempted.

The Stage-0 real finite-dimensional carrier remains authoritative for bounded linear semantics. TDI-23.1 adds an IR layer above that carrier; it does not redefine the Stage-0 mathematics and does not silently resolve the TDI-23.0 scientific freeze template.

## Versioned contract

The implementation contract is:

`tdi23.1-categorical-attention-ir-v1`

implemented in `tdi-ai/src/tdi23_ir.rs` behind the existing `experimental` feature.

## In-scope grammar

The bounded IR must represent:

1. named finite-dimensional real Hilbert objects with stable object identities;
2. atomic objects;
3. explicit direct-sum objects with additive dimension;
4. explicit tensor-product objects with multiplicative dimension;
5. concrete Stage-0 real linear maps with exact domain/codomain object handles;
6. identity morphisms;
7. typed composition `outer o inner`;
8. dagger of boundary-free linear subgraphs;
9. explicit opaque nonlinear/algebraic boundaries for softmax, Boolean, `F2`, ANF/Zhegalkin, and max-plus;
10. object-level coordinate-reduction annotations;
11. boundary-free lowering back to the Stage-0 `RealLinearMap` carrier;
12. reduced lowering using only explicit source/target object annotations;
13. structural composition-defect auditing using the Stage-0 omitted-middle-path diagnostic.

## Legality rules

The IR is fail-closed.

- Composition requires **exact identity of the middle object**, not merely equal dimensions.
- A concrete linear map must match the dimensions of its declared domain and codomain objects.
- A dagger may not be constructed across any nonlinear/algebraic boundary.
- Linear evaluation must reject any subgraph containing a nonlinear/algebraic boundary.
- A coordinate-reduction annotation must have the same ambient dimension as its object.
- Reduced lowering must fail if either endpoint object lacks an explicit reduction annotation.
- Composition-defect auditing must fail if the requested node is not a composition or if required object reductions are missing.
- Direct sum and tensor product remain distinct constructions. Ordinary head concatenation must not be represented as a tensor product merely because the programme is monoidal.
- Graph construction may reference only already-declared objects and nodes; this keeps the current builder acyclic by construction.

## Nonlinear boundary discipline

The following operations/domains are represented as opaque boundaries rather than `FdHilb` morphisms:

- softmax;
- Boolean logic;
- `F2` operations;
- ANF/Zhegalkin operations;
- max-plus/tropical operations.

TDI-23.1 may type their placement in a pipeline, but it does not assign them linear semantics. Any later inter-domain adapter must separately declare what structure it preserves.

## Reduction discipline

Coordinate reduction remains the Stage-0 development model:

`R(f) = E_K^dagger f E_H`.

TDI-23.1 stores coordinate reductions on objects and uses them only when a caller explicitly requests reduced lowering or composition-defect auditing.

The IR does **not** claim that this reduction is lossless, generally functorial, a quotient category, or a categorical equivalence. The composition diagnostic remains the direct omitted-path structural term from TDI-23.0 and is not a measured floating-point residual between two independently accumulated matrix products.

## Deterministic controls required in this slice

TDI-23.1 development tests must include at least:

- versioned IR contract;
- direct-sum versus tensor-product distinction;
- rejection of composition across distinct same-dimension middle objects;
- dagger/composition equivalence to the Stage-0 linear carrier;
- semantic neutrality of left/right identity composition;
- explicit softmax boundary rejecting linear evaluation and dagger;
- reduced lowering through explicit object annotations;
- rejection when reduction annotations are missing;
- zero composition defect when the middle object is fully retained;
- non-zero composition defect when a contributing middle path is omitted.

## Explicit non-goals

TDI-23.1 does not authorize or implement:

- rewrite enumeration or rewrite search;
- canonicalization claims;
- tensor-network contraction planning;
- batched `QK^T` lowering;
- attention-quality experiments;
- learned subspace reductions;
- numerical tolerance claims beyond existing bounded tests;
- Boolean / `F2` / ANF / max-plus execution semantics;
- GPU/WGPU kernels;
- FLAT-ATTENTION integration;
- hardware benchmarking;
- confirmatory or final evaluation.

## Exit condition

This engineering slice is complete only when:

1. the versioned IR module is exposed solely through the experimental facade;
2. the legality rules above have deterministic tests;
3. the dedicated TDI-23.1 integrity script passes formatting, Clippy, and targeted tests;
4. the full repository Rust validation remains green;
5. no unresolved review finding remains.

Passing this slice does not freeze the TDI-23.1 scientific design. A later explicit freeze decision is still required before comparative or confirmatory use.
