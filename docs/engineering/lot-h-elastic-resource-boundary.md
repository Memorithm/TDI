# Lot H — TDI → ElasticXxx resource-control boundary

Status: **current-main requalification candidate / non-actuating / no scientific authority**. The original #279 contract and later root-evidence hardening are present on `main`; this refresh deliberately re-runs the dedicated exact-head partner-contract gate over the integrated implementation instead of treating merge presence as qualification. The separate executable capacity-admission path is independently pinned to qualified ElasticXxx #135 merge `f5af4f3129100f4bb52d47d1adfc202347d0b098`; its upstream exact-head `ci`, `packageability`, and `continuous-hardening` gates completed successfully before merge. That later path does not turn this interchange contract into execution authority.

The descriptor must advertise exactly `elastic.hub.run`, the `operator-config`
input and `runtime-evidence` output. Their identities are ordinary SHA-256 of
`crates/elastic-runtime/src/operator_config.rs` and
`crates/elastic-runtime/src/evidence.rs`, respectively, at the audited revision.
These are schema-owner implementation pins, not hashes of a particular
configuration or result. Empty, unrelated, or actuation-specific declarations
are rejected. Non-scalar domains and modes return the typed contract error.

This slice consumes the qualified common `PartnerAdapter/v1` / `AdmittedPartnerStep/v1` boundary and binds a TDI-owned non-actuating resource-control request to the process contract published by ElasticXxx. It does not execute ElasticXxx and does not reproduce ElasticXxx runtime semantics.

## Audited ElasticXxx contract

The adapter is pinned to `Memorithm/ElasticXxx` source `50bb85ea84191c01d95e5b4e5e3c81af10e95ebd` and to the repository documentation at that exact source:

- process protocol `elastic.hub.run@1.0.0`;
- OperatorConfig schema version `1`;
- runtime evidence schema `elastic-runtime-evidence-v1`;
- runtime evidence media type `application/vnd.elastic.runtime-evidence.v1+json`;
- runtime evidence source command `run`.

The TDI adapter protocol identity is `ae9712fbca20c666012260ef8537a8ced59a2bc7a17f8c4f5e504cb5984efc24`, derived from the ASCII descriptor `Memorithm/ElasticXxx@50bb85ea84191c01d95e5b4e5e3c81af10e95ebd:elastic.hub.run/1.0.0+OperatorConfig/v1+elastic-runtime-evidence-v1`. It is a TDI adapter identity anchored to the audited source and published contracts; it is not claimed to be an ElasticXxx-published digest.

The common admitted partner step is deliberately bound to the Hub `tdi.prepare` capability at capability-contract version `1.0.0`. This H2 slice is therefore a preparation/interchange boundary only. It does not claim that scirust-hub currently exposes an ElasticXxx-specific registered component or that the generic Hub fixture is an Elastic execution qualification.

## TDI-owned contract

`ElasticResourceContract/v1` binds:

- the exact admitted ElasticXxx partner descriptor;
- the audited ElasticXxx repository/source/protocol/schema identities;
- one portable TDI `ArtifactDescriptor/v1` for the OperatorConfig JSON bytes;
- the scientific domain (`Development` or `Validation`) and matching artifact access class;
- one requested non-actuating mode: `observe-only`, `plan-only`, or `dry-run`;
- an optional resource selector;
- the exact runtime evidence schema/media/source-command expected from a later independently qualified Elastic execution.

The contract rejects `apply`, source/protocol/schema drift, JSON Boolean aliases on integer schema fields, Development/Validation access mismatches, malformed resource identifiers at this transport layer, embedded common-adapter identity drift, and any authority-bit escalation.

## Independent Elastic validation remains mandatory

The contract intentionally does **not** parse or duplicate ElasticXxx `OperatorConfig` semantics. The compiled interchange envelope contains the content identity, raw SHA-256 and size of the referenced configuration and the requested non-actuating mode, but it is not executable authority.

Before any future invocation, the execution owner must independently:

1. retrieve the exact configuration bytes through an authenticated artifact path;
2. verify them against the bound `ArtifactDescriptor/v1`;
3. parse and validate them using the pinned ElasticXxx implementation;
4. verify that the actual OperatorConfig mode is compatible with the non-actuating intent recorded by TDI;
5. retain ElasticXxx as the authority for validation, actuation, verification and commit/rollback semantics.

No TDI code in this slice may infer that a declared `dry-run` string proves the contents of an opaque artifact are dry-run safe.

## Authority boundary

A valid contract keeps all of the following false:

- ElasticXxx execution qualified;
- physical actuation authorized;
- protected/final holdout access authorized;
- scientific stage authorized;
- scientific verdict authorized;
- runtime actuation authorized.

The common partner binding likewise retains `partner_execution_qualified: false`. Compilation produces an interchange envelope only; it is neither an invocation command nor a scientific result.

ElasticXxx remains owner of `OBSERVE → FORECAST → PLAN → VALIDATE → ACT → VERIFY → COMMIT / ROLLBACK`, generic resource objectives, trusted adapter validation, physical actuation, post-actuation verification and rollback. scirust-hub remains owner of workflow orchestration, registry resolution, leases/fencing, transport, physical artifacts and authoritative publication. TDI remains owner of scientific identities, admissible evidence, split/holdout boundaries, scientific-stage authorization and verdict semantics.

## Non-claims

This slice provides no production actuation, no resource-quality improvement, no runtime speedup, no hardware qualification, no model-quality result and no scientific result. It does not qualify `apply`, does not prove that a particular OperatorConfig is safe, and does not qualify an ElasticXxx deployment through Hub.
