# TDI portable artifact, provenance, export, and cache contract

Status: Lot F contract layer qualified by PR #261 on final exact head `dceb67d4a9c49ce293c08347cab61db0f3347871`, merged as `494240ebc5cd8d9418ce8c174dd65a4bc4e17651`. This contract is infrastructure only. It does not authorize a scientific stage, a retry, publication, or a protected/final decision.

## Ownership boundary

TDI owns portable scientific identities, provenance bindings, access semantics, export verification, and the conditions under which a caller-declared cache reuse can be considered an exact semantic reuse.

scirust-hub owns physical content-addressed storage, registry persistence, artifact transport, workflow scheduling, leases, retries, cancellation, and distributed execution. TDI must not become a second CAS, registry, scheduler, or lease manager.

The Hub source audited for this boundary is `Memorithm/scirust-hub@4bf6186841e1ea70ed15cd84faf33de9b48429cd`.

## ArtifactDescriptor/v1

A descriptor binds a stable name, ordinary raw SHA-256 of payload bytes, exact byte length, media type, and access class. `verify_artifact_bytes()` checks length and raw digest without retaining the payload.

TDI raw SHA-256 is intentionally **not** the same digest construction as Hub `ContentDigest`. The pinned Hub implementation prefixes and length-frames a domain such as `scirust-hub:artifact-blob:v1` before the payload. A future Hub edge must therefore verify an explicit translation from TDI raw bytes to Hub domain-separated CAS identity; copying the 64-hex wire value is invalid.

## ProvenanceRecord/v1

A provenance record binds one TDI artifact identity to exact experiment/plan/trial/attempt/step identities, implementation identity, Development or Validation domain, named input artifact identities, and named dependency identities. Named sets are canonicalized by name and duplicates fail closed.

The record does not infer scientific authorization. It records the execution/scientific lineage supplied by the owning protocol and engine.

## ExportManifest/v1

An export manifest references artifact-descriptor identities and provenance identities. `verify_export()` requires a complete, exact member set and verifies payload bytes independently. Provenance must point back to the same artifact identity.

Access control is monotone: an export may become more restrictive than a member artifact but may never weaken that artifact's declared access class. No call in this module publishes, uploads, or stores an artifact.

## CacheRequest/v1

Controlled cache semantics are deliberately narrow:

- `disabled` fails closed;
- `exact-domain` binds Development/Validation domain, experiment plan, step identity, implementation identity, backend identity, named input identities, and parameter identity into a deterministic cache key;
- domain drift, backend drift, implementation drift, input drift, or parameter drift changes the key;
- `validate_cache_reuse()` additionally requires an explicit `cache_authorized=True` supplied by the owning scientific protocol/stage policy.

The boolean authorization is not derived here. This module therefore cannot open a HOLDOUT, authorize protected/final reuse, or convert a cache hit into a scientific verdict. It also performs no cache lookup or storage.

## Qualification

PR #261 qualified the contract checks on final exact head `dceb67d4a9c49ce293c08347cab61db0f3347871` before merge `494240ebc5cd8d9418ce8c174dd65a4bc4e17651`:

```bash
PYTHONPATH=scripts python3 -m py_compile \
  scripts/tdi_artifact_contract.py \
  scripts/test_tdi_artifact_contract.py
PYTHONPATH=scripts python3 -m unittest \
  scripts/test_tdi_artifact_contract.py -v
```

The dedicated `TDI artifact and provenance contracts` workflow executed the same checks successfully on that exact PR head.

## Remaining integration work

The qualified Lot F contract layer defines portable descriptors, provenance, export verification, and exact-cache semantics. It still does **not** provide an authoritative Hub publication edge, physical CAS/registry integration, distributed fencing, or a durable cache index. Those are later industrialization lots and must preserve the Hub/TDI ownership split. In particular, TDI must remain fail-closed for authoritative remote publication until Hub-owned attempt fencing/generation semantics are qualified.