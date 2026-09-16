# Lot H — FLAT-ATTENTION qualification boundary

Status: **candidate / non-executing / no performance or scientific authority**

This slice defines the TDI-owned interchange boundary for presenting an exact Development/Validation candidate to `Memorithm/FLAT-ATTENTION` for independent contract qualification. It does not invoke FLAT, select a backend, stage K/V, execute attention, open a protected/final holdout, or transfer FLAT attention semantics into TDI.

## Required FLAT governance read before this slice

The implementation selection followed current FLAT governance on its published main and off-main agent overlays: `AGENTS.md`, `ROADMAP.md`, `.agent/FLAT_ATTENTION_ECOSYSTEM_ROADMAP.yaml`, and `.agent/ML_MATURITY_5_OF_5.yaml`. Those sources keep mathematical/scalar correctness ahead of GPU optimization, forbid hidden CPU fallback, require real-device evidence for performance claims, and assign attention semantics/kernels to FLAT rather than TDI.

## Audited FLAT source

The contract is pinned to:

- repository `Memorithm/FLAT-ATTENTION`;
- default-branch source `1d5ac64cc87c5dd526e04527c3bb4b78ba0add33`;
- backend-neutral API module `src/api.rs`, Git blob `ffc5eb912afd7599e3e4b326877d0f8617bc45dd`, `api::v1::API_VERSION = 1`;
- Boolean mask module `src/boolean_attention_mask.rs`, Git blob `732e028251ad02e1f60633e9caa45023484fac24`, `BOOLEAN_ATTENTION_MASK_SCHEMA_VERSION = 1`;
- Boolean Q/K signature module `src/boolean_attention_signature.rs`, Git blob `6c03c9c414c7fb333bb2047a4187b78934709a74`, `BOOLEAN_ATTENTION_SIGNATURE_SCHEMA_VERSION = 1`.

The audited API explicitly keeps the backend-neutral v1 contract independent of WGPU. The Boolean mask is a canonical bit-packed `u64` block-admission representation with tail-bit validation. The Boolean signature surface provides exact packed XOR/Hamming distance, XNOR match count and a bounded Hamming admission rule. This TDI contract does not copy or reinterpret those algorithms.

## TDI-owned adapter identities

The following SHA-256 identities are **TDI adapter-schema pins derived from the audited FLAT source**, not digests published by FLAT:

- API projection: `e6f23e26b2ffb878de42b09873566f94204383a9b4e8af664d6c3c3c5f599f90`;
- Boolean mask projection: `9a5224386cb02cd45200288005c06628cb4967d8709d16e5d6c138ebf5823527`;
- Boolean signature projection: `1e669b85f5dea70fdc6883110482caa24b95180f04666fe1b432998b791abe45`;
- adapter protocol `tdi.flat-attention.qualification`, integer version `1`, schema identity `60d1ff3a1f648257dc15f95c2b877881504cf3be9d33838158ce5880d7b2d5eb`.

The adapter protocol identity is SHA-256 over the exact ASCII descriptor:

```text
tdi.flat-attention.qualification/v1|source=1d5ac64cc87c5dd526e04527c3bb4b78ba0add33|api_blob=ffc5eb912afd7599e3e4b326877d0f8617bc45dd|api_version=1|mask_blob=732e028251ad02e1f60633e9caa45023484fac24|mask_schema=1|signature_blob=6c03c9c414c7fb333bb2047a4187b78934709a74|signature_schema=1
```

The API/mask/signature projection identities likewise bind their exact source SHA, module blob and declared version. They are not source-tree hashes or performance evidence.

## Contract

`FlatQualificationContract/v1` binds:

- one exact common `AdmittedPartnerStep/v1` for partner `flat-attention`;
- the exact audited FLAT source/module/blob/version tuple;
- exact TDI adapter capabilities `flat.api.v1` and `flat.boolean-front-end.v1` plus their three pinned input contracts;
- one Development/Validation `ArtifactDescriptor/v1` containing candidate metadata/evidence;
- a bounded qualification scope: `api-contract` or `boolean-front-end-contract`;
- FLAT as independent qualification owner.

The candidate media type is `application/vnd.tdi.flat-qualification-candidate.v1+json`. The Hub binding uses only the qualified non-executing `tdi.prepare@1.0.0` boundary. Compilation produces review metadata; it is not an FLAT command.

## Required review gates

A valid contract requires all of these declarations to remain true:

- independent FLAT validation;
- dense/reference comparison;
- real-device evidence before any real-device performance claim.

It keeps all of these authorities false:

- FLAT execution qualified;
- real-device performance qualified;
- Boolean-front-end speedup qualified;
- protected holdout access authorized;
- scientific stage authorized;
- scientific verdict authorized;
- runtime actuation authorized.

In particular, the contract must never convert the research hypothesis

`T_boolean_front_end + T_flat_survivors < T_flat_dense`

into a result. That inequality requires reproducible measured evidence on the applicable exact source/head and device.

## Ownership

FLAT owns attention public/numerical semantics, scalar oracles, WGPU/open-codegen kernels, Boolean-mask consumption, attention-specific device qualification and any eventual end-to-end performance claim. TDI owns its scientific identity, admissible evidence, split/holdout and stage/verdict semantics. Hub owns orchestration, registry admission, transport, leases/fences and authoritative publication.

BooleanLab remains the research bench for candidate Boolean rules and KVLab remains the cache/page bench. This adapter does not promote either bench's candidate into FLAT automatically.

## Non-goals

This slice does not execute a dense or Boolean attention path, create a real FLAT request, bind numerical Q/K/V bytes, claim first-token readiness, qualify M13B.3/M13B.4 hardware behavior, access M48/M53 Thor, measure TTFT/TPOT/tokens/s, prove K/V bytes avoided, evaluate quality/recall/FNR/O-LSE, or authorize a 1-bit QK path.

## Qualification

The dedicated `TDI FLAT-ATTENTION partner contracts` workflow must run the common G3/partner boundary plus FLAT-specific source/protocol/surface/domain/type/authority regressions. Merge remains forbidden until every applicable exact-head workflow succeeds and all material review findings are resolved.
