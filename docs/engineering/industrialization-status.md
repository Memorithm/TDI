# TDI engine industrialization status

This is the durable engineering handoff for the industrialization programme. It records software evidence only and does not authorize or reinterpret a scientific stage.

## Baseline

- Historical audit: `2b9cf772d3c0711349f9a70ba7cd7f1bfef90aca`.
- Programme baseline revalidated 2026-09-15: `c973b47355f8bd3eb630a2d267355f733dfde1b2`.
- Foundation A/B: PR #250, head `07b01760f23200665dbc713e73c0cdc35bb49e76`, merged `3b8686047a95c4665f6c1b70b87ab9a9ac3591be` after applicable exact-head workflows succeeded.
- Linux containment C: PR #252, head `1b4735b8aae65ad8f951803d18330706224fa33c`, merged `32e51b9b51a254680d1c5bca0d8e96c2ac39f53f`; exact-head real-kernel and repository workflows succeeded.
- Experiment contract D: PR #253, final head `98a894437cbd6b1b72b23aeb2bdd84548e32124a`, merged `4dd4e0db2837fcffd2e6177542ab14cd289c8540`. All returned pull-request workflows on that exact head completed successfully, including Rust, public Rust, MSRV, TDI AI contracts and real-kernel engine containment.
- Lot-E base: `4dd4e0db2837fcffd2e6177542ab14cd289c8540`.
- Current branch: `industrialize/checkpoint-dag-e`.
- Scientific constraints remain those in `AGENTS.md` and the ecosystem roadmap/overlays.

## Delivered / candidate capabilities

| Requirement | State | Evidence |
| --- | --- | --- |
| A strict worker JSON | qualified | PR #250 rejects duplicates, non-finite/overflow numbers including `1e999`, invalid UTF-8 and configured structural limits. |
| A CLI/error separation | qualified | PR #250: technical failure `20`, contract `21`, storage `22`, cancellation `130`; scientific `Rejected` can still be technical success `0`. |
| B incremental journal | qualified | PR #250 validates history once, then hashes one new event per append. |
| B transaction/migration/audit | qualified | PR #250 atomic event/index transition, non-destructive legacy migration, full `--audit-only`, optional external anchor. |
| C schema-2 resource contract | qualified | PR #252 binds verified memory/swap/CPU/PID limits while leaving schema 1 compatible. |
| C process-tree cleanup | qualified for declared cgroup profile | PR #252 dedicated attempt cgroup + `cgroup.kill`, pidfd recovery owner and exact-head real-kernel tests. |
| C resume semantics | qualified | PR #252 reconciles stale unfinished containment before durable `Interrupted`; no silent scientific retry. |
| C hostile-code sandbox | blocked | cgroup v2 is insufficient; `trust=untrusted` fails closed. |
| C hard GPU-memory quota | blocked | visibility is not a VRAM quota; requested hard limit fails closed. |
| D ExperimentSpec / identities | qualified | PR #253 binds question, ExperimentSpec-plan, execution-plan, trial, attempt and scientific-result identities while preserving schemas 1/2. |
| D worker response v2 | qualified | PR #253 caller-owned identity/seed binding, authoritative step/observation budget counters and content-addressed result artifacts; exact-head software and real-kernel gates passed. |
| D cross-language canonical numbers | qualified for v1/v2 scope | JSON-safe identity integers, decimal u64 seeds, timeout normalization, and explicit float exclusion from canonical scientific result values. |
| E CheckpointManifest/v1 | candidate | content-addressed state plus exact experiment/plan/trial/step/adapter/backend/input/RNG binding; frozen step/observation budgets are embedded in checkpoint identity and enforced against resume inputs. |
| E declarative execution graph | candidate | topological Graph/v1 with per-step identity binding to exact Hub ComponentId, component version/manifest digest, capability-contract version, inputs, parameters and checkpoint ports. |
| E pinned Hub structural limits | candidate | Graph/v1 matches audited Hub v1/default limits for max concurrency, step count, input count, serialized parameters, timeout and input-name grammar before producing a preview. |
| E Hub bridge | structural preview only | `compile_hub_workflow_preview()` emits `execution_authorized: false`; current WorkflowSpec/v1 cannot atomically enforce component/capability pins, registry state or portable-artifact identity. |
| E generic scheduling | intentionally delegated | scirust-hub owns ready-set scheduling, retries, cancellation, workflow lifecycle, artifacts, leases and remote transport; TDI does not duplicate them. |

## Qualification matrix progress

| ID | State | Evidence / remaining work |
| --- | --- | --- |
| Q01-Q03 | qualified | PR #250 subprocess failure/JSON/corruption/transition regressions. |
| Q04 | qualified for hash-growth property | one new hash per append; Lot M still owes size-scaling benchmark and environment capture. |
| Q05 | qualified at trial boundary | continuous/restart equivalence and interrupted trial retention. |
| Q06 | partial | distinct storage failure exists; explicit disk-full/fsync injection remains. |
| Q07 | qualified for declared cgroup-v2 profile | PR #252 exact-head real-kernel process-tree cleanup/recovery qualification. |
| Q08 | qualified for declared cgroup-v2 profile | PR #252 real-kernel memory/PID/CPU limits and sibling isolation. |
| Q09 | partial | unsupported hard VRAM quota fails closed; measured GPU qualification remains. |
| Q10 | candidate pending Lot-E exact-head qualification | PR #253 qualified plan/trial/attempt identity; Lot E adds exact checkpoint lineage plus embedded frozen budget binding for resume. |
| Q11 | candidate for declarative graph semantics only | Lot E validates graph/component/capability pins and pinned Hub structural bounds; authoritative Hub execution is blocked until a versioned Hub edge enforces registry/artifact pins. |
| Q12 | planned | common adapter SDK remains separate from graph/checkpoint semantics. |
| Q13-Q14 | partial | legacy journal migration qualified; portable provenance/artifact export remains. |
| Q15-Q17 | planned | controlled cache and distributed fencing/deduplication. |
| Q18-Q30 | planned | cross-repository integrations, statistics, CLI/viewer/exports, hardware adapters, docs and final integrated qualification. |

## Lot-E candidate validation

Local non-privileged qualification currently covers 17 checkpoint/graph contract tests (6 checkpoint + 11 graph):

```bash
PYTHONPATH=scripts python3 -m py_compile \
  scripts/tdi_checkpoint_contract.py \
  scripts/tdi_execution_graph.py
PYTHONPATH=scripts python3 -m unittest \
  scripts/test_tdi_checkpoint_contract.py \
  scripts/test_tdi_execution_graph.py -v
```

Checkpoint regressions cover exact lineage, input/RNG drift, content identity, embedded progress limits and rejection of caller-side budget widening. Graph regressions include component-id/version/manifest/capability identity sensitivity, single-alias pin consistency, Hub input grammar, max 32 inputs, 16 KiB serialized parameters and the audited 3,600,000 ms timeout bound. The existing `TDI engine Linux containment` workflow executes these contracts alongside all previously qualified engine-contract tests on the exact PR head. The cgroup kernel job remains enabled so Lot E cannot accidentally regress the contained execution foundation.

## Cross-repository decisions

- TDI declares Rust 1.85; audited scirust-hub declares Rust 1.89. No mandatory direct Rust dependency is introduced for the graph bridge.
- The Lot-E graph contract pins `Memorithm/scirust-hub` source `4bf6186841e1ea70ed15cd84faf33de9b48429cd`, WorkflowSpec schema `1`, model `1.2.0`, capability/version grammar, canonical UUID identifiers and the relevant default RunSpec admission limits.
- Graph steps pin exact Hub ComponentId, component version, component-manifest digest and capability-contract version. Current Hub WorkflowSpec/v1 cannot enforce all those pins atomically during normal workflow submission, so Lot E exposes a non-executable preview only.
- scirust-hub remains owner of generic DAG scheduling, remote workers, leases, heartbeats, retries, cancellation and artifact transport; TDI remains owner of scientific meaning, checkpoint admissibility and accepted publication.
- Hub's current remote lease v1 has attempt/lease identity, heartbeat and expiry but no qualified fencing/generation token preventing stale publication after reassignment; distributed fencing remains a later Lot-H requirement.
- Hub already owns a content-addressed artifact store. Hub `ContentDigest` is domain-separated and is not interchangeable with TDI portable raw SHA-256; Lot F must define portable TDI provenance/export/cache rules and verification without duplicating physical CAS storage.
- ElasticXxx remains owner of generic observe/forecast/plan/validate/act/verify/commit-or-rollback resource policy.
- Forge remains owner of candidate proposal/search; TDI controls allowed evaluation observations and scientific verdicts.
- `ExperimentSpec/v1`, CheckpointManifest/v1 and Graph/v1 are infrastructure contracts and must not be used to synthesize or authorize a protected/final series surface.

## Next

1. Qualify and merge Lot E on its exact PR head; address review findings before merge.
2. Add portable provenance/artifact descriptors, export verification and controlled cache semantics without duplicating Hub's generic CAS/registry.
3. Add a real TDI↔Hub capability edge that atomically enforces component/capability pins, then qualify fencing/authoritative publication before distributed execution is promoted.
4. Add ElasticXxx, Forge, SciRust, FLAT-ATTENTION and NNIS integrations only through qualified versioned contracts.
5. Continue statistics/sensitivity, CLI/API/viewer, external exports and measured engine benchmarks.
