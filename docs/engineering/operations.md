# Operating the research engine

Read the [local tutorial](operational-engine.md) for runnable commands and the
[profile limits](release-and-compatibility.md) before deploying. Store roots,
component files and backup destinations must be trusted. Keep credentials in
environment variables and scoped to a dedicated non-final deployment. The
viewer is local, read-only, credential-free and has no mutation endpoints.

## Prepare and observe

Create a new plan file, validate it, inspect its preview, then submit once.
Persist the returned campaign identity. Query `status`, `inspect` and `events`
before interpreting completion. A graph may contain verified partial results
and still fail overall. Resource admission is durable before dispatch and
cannot be retroactively attached to an already executing workflow.

Use `compare` to inspect protocols before comparing results. Use the paired
analysis protocol to define statistical comparability. The UI displays stored
evidence and snapshots; its search projection is not a replay qualification.
Optional reports are explicitly selected at viewer startup and bounded in size.
No HTTP parameter can select an arbitrary local file.

## Backup, restore and migration

1. Run `backup NEW_CATALOGUE.sqlite` using the operational CLI. It uses SQLite's
   backup API; copying a live SQLite file without its WAL is not a backup.
2. Retain Hub CAS and registry data through Hub's own backup policy. Also retain
   frozen input/toolchain/component manifests needed for executable recovery.
   A TDI result bundle contains evidence, not a complete executable environment.
3. Export each selected terminal campaign to a new result-bundle file. Record
   its receipt identity outside the bundle. Verify against that trusted identity
   before restoring. Internal hashes detect corruption but do not authenticate
   the author or prevent replacement of the entire bundle plus its hashes.
4. Before upgrading a writer, stop writers and keep a verified pre-upgrade
   catalogue backup. The current writer migrates supported versions atomically;
   interruption rolls back the schema transition. Old row bytes remain intact.
5. Open the restored catalogue with the current reader and inspect campaign,
   event and result inventories. Re-export selected evidence and check identities.
   Portable `restore` uploads verified blobs to the explicitly selected Hub and
   imports immutable historical evidence. It does not recreate runnable attempts.
6. For rollback, stop writers and restore the pre-upgrade backup plus the matching
   Hub/artifact snapshot. Do not lower `user_version`, drop new tables, or open a
   schema-4 catalogue with an old writer. Archive all post-upgrade evidence first;
   rollback of binaries alone cannot reverse a schema or recover new results.

Schema 4 stores common exact admission proofs once. Back up the entire database,
including `shared_proofs`; compact result rows are not self-contained backups.
Missing, malformed or hash-mismatched shared proofs stop reconstruction and
export. Do not regenerate an admission proof from a plausible current workflow.
There is no automatic proof garbage collector in this release candidate.

## Incident procedures

| Observation | Recovery action | Evidence preserved |
| --- | --- | --- |
| Submit response lost | Inspect Hub read-only; attach the existing workflow with matching graph, roots and admission pins. Resume reconciliation afterwards. | Durable intent and ambiguous state; no automatic second submission. |
| Worker/client stopped | Inspect current Hub workflow and attempt disposition; resume reconciles it. Use the qualified checkpoint path only when its bindings match. | Interrupted/failed attempt and progress; no new scientific retry implied. |
| Search cancellation remains pending | Inspect `cancel-requested` and pending campaign identities. Attach unknown submissions first; repeat cleanup until every mapped campaign is terminal. | Proposal costs, attempt lineage and non-dispatchable cancellation intent. |
| Disk full or fsync/SQLite failure | Stop further submissions, retain files, recover capacity, inspect/verify and restore from a trusted backup if needed. | Distinct storage error; no forced reset or discarded journal. |
| Proof, artifact or journal corrupt | Stop consumption and preserve the damaged material for diagnosis. Compare to a trusted receipt/anchor and restore verified original bytes. | Corruption remains explicit; hashes alone do not justify replacing evidence. |
| MLflow/OTLP endpoint unavailable | Inspect the durable export queue; reconcile unknown remote effects before an explicit retry. | Local results unchanged; duplicate remote metrics remain possible. |
| Worker component or interpreter changed | Refuse the existing pinned deployment; restore it or create a new explicit plan. | Original manifest, identity and refusal. |
| Capacity unavailable or GPU request unsupported | Keep the refusal and measured attempt costs. Provision a qualified profile or revise a new plan before execution. | Unknown capacity never becomes a fabricated permit or VRAM quota. |
| CI queued/absent/skipped | Inspect workflow/job state and runner administration where available. Retain the candidate; do not promote on the absence of failures. | Exact SHA, observed statuses and the unresolved cause. |

`cancel` records intent before remote cleanup. A successful query about that
intent is not proof that a process has stopped. Under the separately qualified
cgroup backend, confirm descendant cleanup with the kernel gate; under a remote
Hub, use Hub's authoritative attempt state and the deployed worker's guarantees.

## Access and retention

The operational profile rejects restricted-reference roots before reading or
uploading them. Catalogue persistence boundaries also fail closed when structured
metadata declares `access_class: restricted-reference`, covering campaign/search
events, result evidence, cache publication, export plans/receipts and restores.
Run `python3 scripts/tdi_engine.py --catalogue <path> access-audit` to traverse
all structured catalogue metadata and reject an existing tagged record. The audit
is label-based only: it does not classify unlabelled payload content. Hub
authorization remains deployment-wide, not a per-artifact scientific clearance
system. Keep protected/final stores separate. Do not point the viewer, search
process, export endpoints or ordinary agents at a mixed store.

Timing outputs use disabled cache policies. Exact deterministic-data cache reuse
requires explicit consent and complete dependency/domain agreement. It retains
the old provenance, with `new_execution: false`. Source result bytes remain the
authority when an external tracking service or display is unavailable.

Retention is an operational policy, not an inferred deletion permission. Retain
failed attempts, declared exclusions, source manifests and export receipts for
the campaign's declared retention period. Hub orphan-artifact cleanup is separate
from catalogue proof retention. This candidate does not implement a multi-tenant
ACL, automatic scientific data deletion or a hostile-code sandbox.
