# TDI-23.1 — Rooted IR Provenance and Independent Validation

Status: **ACTIVE DEVELOPMENT / NON-FINAL / NO REWRITE SEARCH AUTHORIZED**

## Purpose

This slice adds a deterministic provenance representation and an independent structural validator for the merged TDI-23.1 Categorical Attention IR.

The target is deliberately narrower than graph canonicalization. The implementation serializes one **rooted reachable subgraph** in local construction-index order. Two independently created IR instances with the same construction order and payloads must produce the same manifest even though their runtime owner tokens differ.

This is a reproducibility/provenance contract, not a claim that isomorphic graphs with different construction orders have a unique common normal form.

## Versioned manifest

The manifest contract is:

`tdi23.1-rooted-ir-manifest-v1`

and is implemented in `tdi-ai/src/tdi23_ir_provenance.rs` behind the existing `experimental` feature.

## Runtime identity versus stable provenance

TDI-23.1 now maintains two deliberately separate notions:

1. **Runtime ownership** — `ObjectId` and `NodeId` carry a process-local owner token so handles cannot be transplanted between mutable IR instances.
2. **Provenance serialization** — the rooted manifest omits the owner token entirely and serializes only deterministic local structure and payloads.

The runtime owner token MUST NOT be interpreted as a persistent experiment identifier, hash, serialized graph identity, or cross-process provenance key.

## Independent rooted validation

Before emitting a manifest, the provenance layer independently traverses the rooted subgraph through the public IR surface and checks:

- all reachable handles belong to the supplied IR instance;
- composite object references point to prior objects;
- direct-sum dimensions recompute additively;
- tensor-product dimensions recompute multiplicatively;
- reachable object names are unique;
- attached coordinate reductions match object ambient dimensions;
- node references point to prior nodes;
- concrete linear-map dimensions match endpoint objects;
- identity nodes use one exact object on both ends;
- composition middle/endpoints are structurally consistent;
- dagger endpoints reverse the source endpoints;
- dagger source subgraphs contain no nonlinear/algebraic boundary.

This validator is intentionally separate from the builder path. It re-checks stored semantics rather than treating successful construction as sufficient evidence of integrity.

## Manifest encoding

The manifest is line-oriented and deterministic for a fixed local construction order.

It includes:

- manifest and IR contract versions;
- root local node index;
- reachable objects sorted by local object index;
- object names encoded as UTF-8 bytes in lowercase hexadecimal, avoiding delimiter/newline ambiguity;
- object dimensions and explicit `atomic`, `direct_sum`, or `tensor_product` construction;
- coordinate-reduction ambient dimension and ordered retained coordinates;
- reachable nodes sorted by local node index;
- exact local domain/codomain indices;
- operation-specific references;
- nonlinear-boundary kind;
- every linear-map `f64` entry encoded as its exact `to_bits()` value using 16 lowercase hexadecimal digits.

The bit encoding intentionally distinguishes values such as `+0.0` and `-0.0`; the manifest records the accepted Stage-0 carrier payload exactly rather than normalizing it numerically.

## Determinism boundary

The current manifest is **construction-order-sensitive**. This is intentional.

It does not claim:

- graph-isomorphism canonicalization;
- semantic equivalence of differently constructed graphs;
- rewrite normal forms;
- cryptographic collision resistance;
- a stable content hash;
- a globally unique graph identifier.

A later slice may place a cryptographic digest over the canonical manifest after the serialization contract itself is qualified. This slice deliberately avoids `DefaultHasher` or any runtime-dependent hashing primitive.

## Required deterministic controls

The development tests must show at least:

- manifest contract version is frozen by string;
- two separate IR instances with identical construction/payloads produce identical manifests despite different runtime owner tokens;
- `+0.0` and `-0.0` produce different manifests through exact IEEE-754 bit encoding;
- adding a coordinate-reduction annotation changes the manifest and records its ordered coordinates;
- direct sum and tensor product produce distinct manifests;
- rooted validation counts explicit nonlinear boundaries and reachable reduction annotations;
- a root handle from another IR instance fails closed.

## Non-goals

This slice does not authorize:

- rewrite enumeration or search;
- graph canonicalization claims;
- content-addressed storage claims;
- cryptographic attestation claims;
- FLAT-ATTENTION integration;
- comparative attention-quality experiments;
- device lowering or hardware benchmarking;
- confirmatory/final TDI-23 execution.

## Exit condition

This slice may merge only when the dedicated provenance gate, TDI-23.1 IR gate, historical Stage-0 gate, full experimental-contract validation, and full Rust validation are green, with no unresolved review finding.
