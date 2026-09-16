# TDI-23.1 — Boundary-aware exact equivalence oracle

Status: **DEVELOPMENT ONLY / NOT FROZEN / NO REWRITE SEARCH / NO CONFIRMATORY EXECUTION**

## Purpose

This slice adds a bounded oracle that can compare two already-built TDI-23.1 rooted fragments before TDI-23.2 introduces any rewrite calculus.

It answers only a narrow engineering question: given two roots in the same `CategoricalAttentionIr`, do they have the same exact source/target objects and, after independent rooted validation, do both lower through the existing Stage-0 real linear carrier to equal finite matrices?

This is not a graph-isomorphism procedure, a numerical-tolerance framework, a rewrite engine, or evidence that categorical representation improves attention.

## Versioned contracts

- exact comparison: `tdi23.1-boundary-aware-exact-equivalence-v1`;
- reduction summary: `tdi23.1-reduction-contract-summary-v1`.

The implementation lives in `tdi-ai/src/tdi23_ir_equivalence.rs` behind the existing `experimental` feature.

## Exact endpoint discipline

Equality of morphisms is only considered when both roots use the same exact domain object and the same exact codomain object inside one IR instance. Equal dimensions alone are insufficient.

This deliberately reuses the TDI-23.1 object-identity contract and prevents a future rewrite rule from silently identifying distinct same-dimensional spaces.

## Boundary discipline

Before comparison, both roots are passed through the independent rooted validator from `tdi23_ir_provenance`.

The comparison then uses the existing Stage-0 linear lowering. Any explicit softmax, Boolean, `F2`, ANF/Zhegalkin, or max-plus boundary therefore fails closed. This slice does not assign cross-domain equivalence semantics.

## Two exactness levels

The report exposes two distinct predicates:

1. ordinary finite-value equality (`f64` `==` for every entry);
2. IEEE-754 bit identity (`to_bits()` equality for every entry).

They are intentionally separate. For example, `+0.0` and `-0.0` are equal under ordinary finite-value equality but not bit-identical.

No epsilon, ULP budget, relative tolerance, approximate law, or reassociation claim is introduced here. Those require a separate frozen numerical contract.

## Reduction-contract summary

For one validated root, the module enumerates all reachable Hilbert objects, including dependencies of direct-sum and tensor-product objects, and records:

- object name and ambient dimension;
- whether an explicit coordinate reduction exists;
- ordered retained coordinates when present;
- number of annotated reachable objects;
- whether every reachable object is annotated;
- reduced dimensions of the root endpoints when annotated.

Completeness is bookkeeping only. It does not claim that the reduction is lossless, functorial, a quotient, an equivalence, or safe for a particular rewrite.

## Deterministic controls

The development tests require at least:

- versioned comparison and reduction-summary contracts;
- left/right identity compositions comparing bit-identically with the original map on a deterministic fixture;
- double dagger comparing bit-identically with its source;
- rejection of distinct same-dimensional endpoint objects;
- fail-closed comparison across a softmax boundary;
- explicit distinction between finite-value equality and IEEE-754 bit identity using `+0.0` versus `-0.0`;
- incomplete reduction summaries remaining incomplete when one reachable object lacks an annotation;
- complete summaries only after every reachable object is explicitly annotated.

## Non-goals and authorization boundary

This slice does **not** authorize:

- rewrite enumeration or rewrite search;
- associativity/reassociation claims under floating-point arithmetic;
- approximate equivalence or tolerance-based acceptance;
- nonlinear-boundary rewrites;
- learned or implicit reductions;
- FLAT-ATTENTION integration;
- GPU/WGPU lowering;
- attention-quality comparisons;
- performance or memory claims;
- confirmatory or final execution.

## Gate toward TDI-23.2

TDI-23.2 may use this oracle only as one bounded verifier for exact linear rewrite fixtures. Before rewrite enumeration is enabled, rewrite rules themselves must be explicitly versioned, local, terminating or bounded by construction, and classified by the exactness guarantee they require.
