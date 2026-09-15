# TDI engine industrialization status

This is the durable engineering handoff for the industrialization programme. It records software evidence only and does not authorize or reinterpret a scientific stage.

## Baseline

- Historical audit: `2b9cf772d3c0711349f9a70ba7cd7f1bfef90aca`.
- Programme baseline revalidated 2026-09-15: `c973b47355f8bd3eb630a2d267355f733dfde1b2`.
- Foundation A/B: PR #250, head `07b01760f23200665dbc713e73c0cdc35bb49e76`, merged `3b8686047a95c4665f6c1b70b87ab9a9ac3591be` after applicable exact-head workflows succeeded.
- Linux containment C: PR #252, head `1b4735b8aae65ad8f951803d18330706224fa33c`, merged `32e51b9b51a254680d1c5bca0d8e96c2ac39f53f`; the exact-head `TDI engine Linux containment` workflow and returned repository workflows succeeded.
- Lot-D base after concurrent scientific merges: `0262fbe99c495dccb405eb54b5a61433cc806555`.
- Current branch: `industrialize/experiment-contract-d`.
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
| D ExperimentSpec / identities | candidate | schema 3 adds question, ExperimentSpec-plan, execution-plan, trial, attempt and scientific-result identities without changing schema 2. |
| D worker response v2 | candidate | caller-owned experiment/plan/trial/attempt/backend/domain/seed bindings, decimal u64 seed transport, bounded progress/artifact/error surfaces. |
| D cross-language canonical numbers | candidate | identity integers limited to JSON-safe range; canonical result v2 rejects floats until a representation is frozen. |
| E and later lots | planned | no completion claim yet. |

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
| Q10 | candidate | schema-3 plan/trial/attempt binding; exact-head CI and checkpoint binding remain before Q10 as a whole is qualified. |
| Q11-Q12 | planned | scheduler and common adapter SDK. |
| Q13-Q14 | partial | legacy journal migration qualified; portable registry/artifact export remains. |
| Q15-Q17 | planned | controlled cache and distributed fencing/deduplication. |
| Q18-Q30 | planned | cross-repository integrations, statistics, CLI/viewer/exports, hardware adapters, docs and final integrated qualification. |

## Lot-D candidate validation

Non-privileged contract tests cover:

```bash
PYTHONPATH=scripts python3 -m py_compile \
  scripts/tdi_experiment_contract.py \
  scripts/tdi_linux_contract.py \
  scripts/tdi_linux_runner.py \
  scripts/prepare_tdi_experiment_plan.py
PYTHONPATH=scripts python3 -m unittest \
  scripts/test_tdi_experiment_contract.py \
  scripts/test_tdi_linux_contract_v3.py \
  scripts/test_prepare_tdi_experiment_plan.py -v
```

The exact-head workflow must additionally pass a real-kernel schema-3 run/resume fixture under the deliberately delegated cgroup-v2 parent. A skipped privileged test does not count as qualification.

## Cross-repository decisions

- TDI declares Rust 1.85; current scirust-hub and ElasticXxx manifests previously audited declared 1.89. No mandatory direct Rust dependency is introduced merely for convenience; revalidate versions at the integration lot.
- scirust-hub remains owner of generic remote workers, leases, heartbeats and transport; TDI remains owner of scientific meaning and accepted publication.
- ElasticXxx remains owner of generic observe/forecast/plan/validate/act/verify/commit-or-rollback resource policy.
- Forge remains owner of candidate proposal/search; TDI controls allowed evaluation observations and verdicts.
- `ExperimentSpec/v1` is additive infrastructure and must not be used to synthesize or authorize a protected/final series surface.

## Next

1. Qualify Lot D on its exact PR head, including the real-kernel schema-3 run/resume test and Rust worker-response/v2 fixture build.
2. Add checkpoint binding and DAG execution semantics (Lot E), reusing Hub mechanisms rather than duplicating generic orchestration.
3. Add registry, provenance verification, artifact CAS/export and controlled cache.
4. Add a real Hub edge with fencing/authoritative publication.
5. Add ElasticXxx, Forge, SciRust, FLAT-ATTENTION and NNIS integrations only through qualified versioned contracts.
6. Continue statistics/sensitivity, CLI/API/viewer, external exports and measured engine benchmarks.
