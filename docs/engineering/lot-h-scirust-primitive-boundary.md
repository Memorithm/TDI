# Lot H — SciRust reusable-primitive promotion boundary

Status: **candidate / non-executing / no scientific authority**

Tracking PR: **#283**. The branch was resynchronized after TDI #281 onto default-branch commit `5f365613d466559a46c2de6d940aef8ffdd55da4`; this documentation commit exists to re-run normal exact-head qualification on that integrated base. This status remains candidate until every applicable workflow succeeds on the exact final head and material review findings are resolved.

This slice defines the TDI-owned interchange boundary for proposing a genuinely reusable mathematical or representation primitive to `Memorithm/scirust`. It does not copy SciRust implementation code into TDI, execute SciRust, modify a SciRust branch, or authorize promotion.

## Audited SciRust source

The contract is pinned to SciRust default-branch source:

- repository: `Memorithm/scirust`;
- source SHA: `aa13f62ea829c5c42e370a32d51143f016a0dc44`;
- crate: `scirust-tensor-ir`;
- representation module: `scirust-tensor-ir/src/representation.rs`;
- representation module blob: `753bc57e1da862c9cd73a47ef9f72b981fbdd177`;
- audited public surfaces, in frozen order: `PrimitiveRepresentation`, `RepresentationPlan`, `StorageBits`.

At that source, `RepresentationPlan` is backend-neutral and separate from logical `TensorType`; `StorageBits` is an exact physical storage-bit count rather than an entropy estimate; and `PrimitiveRepresentation` owns the representation-family declarations. This TDI adapter does not widen those SciRust semantics.

## Adapter protocol identity

SciRust does not need to publish a network protocol for this slice. The protocol below is explicitly **TDI-owned interchange metadata** used only to bind the common `PartnerAdapter/v1` to the audited source/API surface:

- name: `tdi.scirust.representation-promotion`;
- version: integer `1`;
- schema identity: `5fa966102003d48811e14bf17043e43a4eaaa7ed649f1d5b024a54b24ec54c62`.

The schema identity is SHA-256 over this exact ASCII descriptor:

```text
tdi.scirust.representation-promotion/v1|source=aa13f62ea829c5c42e370a32d51143f016a0dc44|crate=scirust-tensor-ir|module=representation|apis=PrimitiveRepresentation,RepresentationPlan,StorageBits|blob=753bc57e1da862c9cd73a47ef9f72b981fbdd177
```

It is an adapter-schema pin, not a SciRust release digest or a claim of source-tree identity.

## Contract

`SciRustPromotionContract/v1` binds:

- one exact common `AdmittedPartnerStep/v1` for partner `scirust`;
- the exact audited SciRust repository/source/module/blob/public API tuple above;
- a Development or Validation TDI `ArtifactDescriptor/v1` containing the candidate description/evidence;
- one of the bounded candidate classes `representation-primitive` or `storage-accounting-primitive`;
- `promotion_scope = candidate-only`;
- SciRust as the independent review owner.

The candidate artifact uses media type `application/vnd.tdi.scirust-primitive-candidate.v1+json` and must carry an access class matching the declared Development/Validation domain.

The Hub binding uses the already qualified non-executing `tdi.prepare` capability boundary. The compiled request is review metadata only; it is not a command or an invocation surface.

## Ownership invariants

TDI retains ownership of its scientific question, split/holdout lineage, stage authorization and verdict semantics. SciRust retains ownership of reusable generic mathematical/IR primitives, API design, correctness qualification, implementation and merge decisions. Hub retains orchestration, registry admission, transport, leases/fences and publication.

The contract permanently keeps these authority bits false:

- SciRust execution qualified;
- primitive promotion authorized;
- protected holdout access authorized;
- scientific stage authorized;
- scientific verdict authorized;
- runtime actuation authorized.

Promotion requires independent SciRust-side review and validation. A TDI candidate must never be treated as a SciRust feature merely because this adapter accepts its metadata.

## Non-goals

This slice does not claim that any TDI primitive is novel, correct, faster, more compact or appropriate for SciRust. It does not serialize a `RepresentationPlan`, infer backend materialization, authorize a representation automatically, execute a benchmark, inspect protected/final material, or transfer TDI-specific policy/state machines into SciRust.

## Qualification

The dedicated `TDI SciRust partner contracts` gate must validate the common G3/partner binding plus source/protocol/API drift, exact artifact-domain binding, JSON integer type fidelity, candidate-only scope, independent-review ownership and authority-escalation rejection. Merge remains forbidden until every applicable exact-head workflow succeeds and material review findings are resolved.
