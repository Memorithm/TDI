# TDI portable provenance and export contract

Status: **engine software contract; storage-agnostic; no scientific-stage or export authorization**.

Lot F adds portable evidence identities on top of the qualified ExperimentSpec, worker-response, checkpoint and graph contracts. It deliberately does not create another content-addressed store, registry, scheduler or remote transport. Those generic services remain owned by the selected backend, including `Memorithm/scirust-hub`.

## PortableArtifact/v1

A portable artifact descriptor binds:

- logical name and role;
- media type;
- exact byte length;
- **raw SHA-256 of the actual artifact bytes**;
- access class: `public`, `development`, `validation`, or `restricted-reference`;
- optional exact producer lineage.

The producer record binds experiment, plan, trial, attempt, step and optional scientific-result identity. A claimed producer is not decorative metadata: inside a portable export it must match an included provenance record that lists the artifact among its outputs. If the producer exists outside the export closure, the descriptor must use `producer: null` rather than inventing or partially reproducing lineage.

`artifact_identity()` is a separate domain-separated identity over the complete descriptor. Renaming an artifact or changing its access/producer metadata therefore changes the descriptor identity even when its byte SHA-256 remains unchanged.

`verify_artifact_bytes()` checks both byte length and raw SHA-256 against actual bytes.

## StorageBinding/v1

Storage provider metadata is separated from portable artifact identity. A binding records:

- portable artifact identity;
- portable raw SHA-256;
- provider name;
- provider artifact identifier;
- provider digest namespace;
- provider digest.

For the audited `scirust-hub` contract, the provider artifact identifier must be a canonical UUID and the digest namespace is exactly:

```text
scirust-hub:artifact-blob:v1
```

Hub's `ContentDigest` is **not** the portable raw SHA-256 even though both use SHA-256 internally. Hub hashes a framed, domain-separated preimage beginning with `scirust-hub-digest:v1\0`; TDI portable SHA-256 hashes the raw artifact bytes. The two values therefore live in different identity namespaces and must never be substituted for one another. `StorageBinding/v1` preserves both explicitly.

A storage binding is provider metadata, not proof that the provider currently contains the bytes. A real Hub edge must fetch/verify authoritative provider metadata or bytes before publication or execution.

## ProvenanceRecord/v1

A provenance record binds one concrete attempt/step execution to:

- experiment, plan, trial and attempt identities;
- step identity;
- optional checkpoint identity;
- optional scientific-result identity;
- adapter, backend and environment identities;
- immutable input/output artifact identities and raw SHA-256 values;
- named dependency identities.

Input/output reference order and dependency order are canonicalized. Attempt and environment identity remain semantic: changing either changes provenance identity. No wall-clock timestamp participates in the v1 identity.

## ExportManifest/v1

A portable export is a closed manifest rooted in one experiment/plan pair. It contains:

- complete portable artifact descriptors;
- complete provenance records;
- one or more provenance roots.

Every provenance artifact reference must resolve to an included artifact with the same raw SHA-256. Every artifact that claims a producer must be closed by an included matching provenance output. Export roots must name included provenance records.

### Access authority is external

`allowed_access_classes` is a mandatory caller argument to export validation. It is not stored inside the manifest and the manifest cannot authorize itself.

For example, validating with:

```python
allowed_access_classes={"public", "development"}
```

fails closed if the manifest contains a `validation` or `restricted-reference` artifact. Exporting those classes requires a separate caller authority that explicitly includes them. This software contract does not define or grant that authority and does not weaken any TDI series-specific holdout rule.

## Payload verification

`verify_export_payloads()` receives a dictionary keyed by portable artifact identity. The payload key set must match the manifest artifact set exactly; missing and extra payloads fail closed. Every payload is then checked against its declared byte length and raw SHA-256 before the export identity is accepted.

## Checked-in fixture

The software-only fixture consists of:

- `docs/examples/portable-export-v1.json`
- `docs/examples/portable-export-input-v1.txt`
- `docs/examples/portable-export-result-v1.txt`

Verify the raw payload hashes:

```bash
sha256sum \
  docs/examples/portable-export-input-v1.txt \
  docs/examples/portable-export-result-v1.txt
```

Expected:

```text
cec961afc611ca290796c2cc9500015899b0f29df0e3c00de54e797cec228c92  docs/examples/portable-export-input-v1.txt
7f7c33a035184c88261c03af30f6c984a28e51f2bb7c6d7d32bf3ba745e643ef  docs/examples/portable-export-result-v1.txt
```

The fixture's portable artifact identities are:

```text
input:  1af547d8713a6fa201a025b891cf881ded63fb0f9145f49d166a4ede25b00e73
result: c8a2a1a80fa7b24df2bf954b3528ca779968e0066046e93e25899e48a2da989f
```

Its root provenance identity is:

```text
220702f8d6e5841a75c5d1339bf168016ad733568f403739c736a5643c2ba2a9
```

## Qualification

Targeted non-privileged qualification:

```bash
PYTHONPATH=scripts python3 -m py_compile scripts/tdi_provenance_contract.py
PYTHONPATH=scripts python3 -m unittest scripts/test_tdi_provenance_contract.py -v
```

The test suite covers actual-byte verification, storage namespace separation, provenance canonicalization, access-class authorization, reference closure, producer closure, root requirements and exact payload sets. The existing engine workflow also executes these tests alongside the earlier A-E contracts.

## Explicit non-goals

Lot F does not:

- implement a physical CAS or registry;
- equate Hub `ContentDigest` with raw SHA-256;
- authorize Validation or restricted-reference export;
- create a scientific verdict;
- implement cache reuse;
- implement Hub component pin enforcement, distributed leases or fencing.

Controlled cache reuse is a later contract because reuse policy must incorporate access class, scientific identity and anti-leakage rules rather than merely matching bytes.
