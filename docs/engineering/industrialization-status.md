# TDI engine industrialization status

This is the durable engineering handoff for the industrialization programme. It records software evidence only and does not authorize or reinterpret a scientific stage.

## Baseline

- Historical audit: `2b9cf772d3c0711349f9a70ba7cd7f1bfef90aca`.
- Programme baseline revalidated 2026-09-15: `c973b47355f8bd3eb630a2d267355f733dfde1b2`.
- Foundation A/B: PR #250, head `07b01760f23200665dbc713e73c0cdc35bb49e76`, merged `3b8686047a95c4665f6c1b70b87ab9a9ac3591be` after all applicable exact-head workflows succeeded.
- Current branch: `industrialize/linux-containment-c`.
- Scientific constraints remain those in `AGENTS.md` and the ecosystem roadmap/overlays.

## Delivered / candidate capabilities

| Requirement | State | Evidence |
| --- | --- | --- |
| A strict worker JSON | qualified | PR #250 rejects duplicates, non-finite/overflow numbers including `1e999`, invalid UTF-8 and configured structural limits. |
| A CLI/error separation | qualified | PR #250: technical failure `20`, contract `21`, storage `22`, cancellation `130`; scientific `Rejected` can still be technical success `0`. |
| B incremental journal | qualified | PR #250 validates history once, then hashes one new event per append. |
| B transaction/migration/audit | qualified | PR #250 atomic event/index transition, non-destructive legacy migration, full `--audit-only`, optional external anchor. |
| C schema-2 resource contract | candidate | Additive Linux runner binds memory/swap/CPU/PID limits while leaving the qualified schema-1 supervisor unchanged. |
| C process-tree cleanup | candidate | dedicated attempt cgroup + `cgroup.kill`; independent pidfd recovery owner. |
| C resume semantics | candidate | stale unfinished attempt reconciled before durable `Interrupted`; no silent scientific retry. |
| C hostile-code sandbox | blocked | cgroup v2 is insufficient; `trust=untrusted` fails closed. |
| C hard GPU-memory quota | blocked | visibility is not a VRAM quota; requested hard limit fails closed. |
| D and later lots | planned | no completion claim yet. |

## Qualification matrix progress

| ID | State | Evidence / remaining work |
| --- | --- | --- |
| Q01-Q03 | qualified | PR #250 subprocess failure/JSON/corruption/transition regressions. |
| Q04 | qualified for hash-growth property | one new hash per append; Lot M still owes size-scaling benchmark and environment capture. |
| Q05 | qualified at trial boundary | continuous/restart equivalence and interrupted trial retention. |
| Q06 | partial | distinct storage failure exists; explicit disk-full/fsync injection remains. |
| Q07 | candidate | pidfd reaper + tree kill; exact-head real-kernel CI required. |
| Q08 | candidate | real-kernel memory/PID/CPU tests exist; exact-head CI required. |
| Q09 | partial | unsupported hard VRAM quota fails closed; measured GPU qualification remains. |
| Q10-Q12 | planned | ExperimentSpec/checkpoint/scheduler/adapter SDK. |
| Q13-Q14 | partial | legacy journal migration qualified; portable registry/artifact export remains. |
| Q15-Q30 | planned | must not be inferred from current infrastructure. |

## Local Lot-C validation

```bash
PYTHONPATH=scripts python3 -m py_compile \
  scripts/tdi_linux_containment.py scripts/tdi_linux_contract.py \
  scripts/tdi_linux_runner.py scripts/tdi_linux_campaign.py \
  scripts/tdi_cgroup_exec.py scripts/prepare_tdi_experiment_plan.py
PYTHONPATH=scripts python3 -m unittest \
  scripts/test_tdi_linux_containment.py \
  scripts/test_tdi_linux_contract.py \
  scripts/test_tdi_linux_campaign.py \
  scripts/test_prepare_tdi_experiment_plan.py -v
```

Non-privileged Lot-C tests pass locally. Real-kernel tests are separate and require a deliberately delegated cgroup parent; a skip does not count as success.

## Cross-repository decisions

- TDI declares Rust 1.85; current scirust-hub and ElasticXxx manifests declare 1.89. No mandatory direct Rust dependency is introduced merely for convenience.
- scirust-hub remains owner of generic remote workers, leases, heartbeats and transport; TDI remains owner of scientific meaning and accepted publication.
- ElasticXxx remains owner of generic observe/forecast/plan/validate/act/verify/commit-or-rollback resource policy.
- Forge remains owner of candidate proposal/search; TDI controls allowed evaluation observations and verdicts.

## Next

1. Qualify and merge Lot C on the exact PR head, including real-kernel cgroup checks.
2. Implement versioned ExperimentSpec / attempt / worker-response contracts while preserving scientific gates.
3. Add registry, provenance verification, artifact CAS/export and controlled cache.
4. Add DAG execution and a real Hub edge with fencing/authoritative publication.
5. Add ElasticXxx, Forge, SciRust, FLAT-ATTENTION and NNIS integrations only through qualified versioned contracts.
6. Continue statistics/sensitivity, CLI/API/viewer, external exports and measured engine benchmarks.
