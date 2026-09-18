# Shared admission evidence storage

The measured baseline in TDI #388 found 1,328,144 serialized proof bytes for
32 outputs from 16 public counter trials, versus 5,756 payload bytes. Every
result repeated its campaign's complete G3 admission proof. Catalogue pagination
also scanned and sorted all rows instead of using an appropriate ordered index.

Catalogue schema 4 stores one content-addressed admission proof per distinct
proof value, plus each result's remaining evidence and its original complete
evidence identity. The result and newly referenced proof commit in the same
SQLite transaction. Reads verify both identities and reconstruct the original
full object. Exported archives, provenance, cache identities, scientific values,
Hub fencing/admission semantics and publication checks retain their formats.

Use `EngineStore.result(campaign, step, output)` or `EngineStore.results(...)`
to read evidence. Direct SQL reads of the private `results.evidence` field are
not the public API: new rows may contain a storage envelope and shared reference.
The paired-analysis consumer is updated; the sensitivity consumer's companion
change uses the same compatible API. No proof is shared across an identity
boundary by inference. Missing or modified proofs, modified result bodies and
changed replayed publications fail explicitly.

Two indexes support unfiltered and exact-phase pagination. The phase-filtered
query now has an explicit equality predicate, so SQLite can use the phase/time
index without a full scan and temporary sort. Ordering remains `(created_ns,id)`;
pagination remains offset-based and large offsets still have traversal costs.

## Migration and operation

Back up the catalogue before upgrading. Stop old writers and keep their pinned
deployment files available for any old campaign that still needs reconciliation.
Writers recognize exact schema 1/2/3 table inventories, add missing structures
and indexes transactionally, then set version 4. Existing result, event, export
and search bytes are not rewritten. Read-only code accepts all four versions;
older writers reject schema 4. Do not downgrade by editing `PRAGMA user_version`.
Restore an independently verified backup with the original software if rollback
is required. The new writer can operate on mixed legacy and shared-proof rows.

```python
from pathlib import Path
from tdi_engine_store import EngineStore

# Run with PYTHONPATH=scripts. The existing destination is refused.
with EngineStore(Path("tdi-campaigns.sqlite"), readonly=True) as catalogue:
    catalogue.backup(Path("before-schema-4.sqlite"))
with EngineStore(Path("tdi-campaigns.sqlite")) as catalogue:
    print(catalogue.list(limit=50, phase="completed"))
```

`backup` remains a coherent SQLite backup, including the shared-proof table.
Portable export materializes complete proofs; imported verified bundles are
stored using the same normalization and retain their original provenance.
There is no garbage collection or deletion API in this change. Manually editing
the shared table can invalidate every referring result.

## Validation and limits

Four focused tests execute actual Hub campaigns and verify proof sharing,
exact export/restore/backup reconstruction, duplicate publication, corruption,
missing proof refusal, atomic rollback on an injected result-commit failure,
successful subsequent reconciliation, schema-3 migration rollback/byte retention
and indexed pagination. A combined local run of 27 tests additionally covers
the search state machine and actual Forge execution, existing storage/transport,
observability migration and real SciRust analysis.

This optimizes persisted duplication and query access. It does not assert
linear total G3 computation or constant memory. Public evidence reconstruction,
integrity hashing and portable serialization still process complete proofs;
fully materializing all outputs can still repeat the graph in memory/on the wire.
Measurements retain that distinction. The filesystem's `fsync=volatile` baseline
is not a power-loss durability qualification, and shared-host timings are not
portable SLOs.

Measured report identity: `584fe8783ace698afb66d11f831a5fa356fdc1743a77ce986d61c3700c2519e0`. [Raw plan and 72 repetitions](benchmarks/2026-09-16-shared-evidence.json).


| Trials / outputs | Complete reconstructed JSON | Stored result + shared proof JSON | Reduction |
| --- | --- | --- | --- |
| 1 / 2 | 13,574 bytes | 10,496 bytes | 22.7% |
| 4 / 8 | 109,688 bytes | 38,546 bytes | 64.9% |
| 16 / 32 | 1,328,144 bytes | 150,890 bytes | 88.6% |

Byte counts are actual stored JSON payload lengths and reconstructed public
evidence lengths on the same completed campaigns. They exclude SQLite page/index
overhead and are not filesystem compression or total throughput claims.
