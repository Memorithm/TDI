# Lot H — TDI → Forge scientific-search boundary

Status: candidate until PR #277 is fully qualified on its exact final head.

This slice consumes the qualified common `PartnerAdapter/v1` / `AdmittedPartnerStep/v1` boundary and compiles a TDI-owned, leak-safe search request into the wire shape accepted by Forge's scientific external-domain contract. It does not execute Forge.

## Audited Forge contract

The adapter is pinned to `Memorithm/Forge` source `8946e702e697c144c85e0fb166a21fe759cdae46` and to:

- `ExternalDomainManifestV1`, schema version 1;
- `ScientificExternalDomainManifestV1`, schema version 1;
- protocol name `forge.scientific-external-domain`, version 1.

At that Forge source, the scientific wrapper validates the generic external-domain manifest, requires non-empty candidate-generation and independent-verification source sets, rejects development/validation overlap, and inherits the generic final-holdout leakage guard. Forge remains responsible for independently validating any compiled interchange payload.

## TDI-owned semantics

`ForgeSearchContract/v1` binds:

- the exact G3-admitted Forge partner step and audited Forge source;
- the upstream TDI repository, exact Git object id, and scientific/search contract SHA-256;
- explicit candidate dimensions Forge may search;
- distinct generation, verification, and final-holdout source identities;
- the independent verification adapter identity;
- named minimize/maximize objectives;
- environment fingerprint/isolation requirements.

The contract is content-addressed by TDI canonical encoding. It rejects JSON Boolean aliases on integer version fields, non-ASCII Forge-incompatible repository syntax, duplicate bounded lists, source/protocol drift, development/validation overlap, final-holdout leakage, and authority escalation.

## Authority boundary

A valid contract keeps all of the following false:

- Forge execution qualified;
- protected/final holdout access authorized;
- scientific stage authorized;
- scientific verdict authorized;
- runtime actuation authorized.

The embedded common partner binding also retains `partner_execution_qualified: false`. Compilation to a Forge manifest is an interchange operation only; it is not execution qualification and is not a TDI scientific result.

Forge continues to own candidate proposal/mutation, independent verification, measurement and selection. TDI continues to own scientific split identities, admissible evidence, stage authorization and verdict semantics. scirust-hub remains the owner of workflow orchestration, registry resolution, leases/fencing, transport, physical artifact storage and authoritative publication.

## Qualification

The dedicated `TDI Forge partner contracts` workflow compiles the G3, common-partner and Forge-specific contract modules and runs their regression suites together. Normal repository gates remain authoritative as well.

No candidate search, protected holdout, scientific result or performance measurement is produced by these tests.