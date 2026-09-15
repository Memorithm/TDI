# TDI engine industrialization status

This file is the durable engineering handoff for the TDI engine industrialization programme. It records software-engineering evidence only; it does not authorize or reinterpret any scientific stage.

## Baseline

- Historical audit baseline: `2b9cf772d3c0711349f9a70ba7cd7f1bfef90aca`.
- Revalidated default-branch baseline for this programme: `c973b47355f8bd3eb630a2d267355f733dfde1b2` on 2026-09-15.
- Working branch: `industrialize/engine-foundation-ab`.
- Scientific boundaries: `AGENTS.md`, `agent/ecosystem-roadmap:.agent/TDI_ECOSYSTEM_ROADMAP.yaml`, and the Boolean/Elasticity overlay where applicable.
- Existing research PRs #246, #247 and #248 are intentionally outside this engine-foundation branch.

## Status vocabulary

- **qualified**: implementation and directly relevant executable tests exist on the recorded revision.
- **partial**: a requested capability is materially implemented but the full lot remains open.
- **planned**: no completion claim is made.
- **blocked**: a named external capability or authorization is required.

## Current implementation slice

| Requirement | Status | Implementation / evidence |
| --- | --- | --- |
| A-JSON finite numbers | qualified locally; CI pending | `strict_json` rejects NaN/Infinity and conversion overflow such as `1e999`; duplicate keys, invalid UTF-8, excessive size/depth/item/string bounds are rejected. |
| A-CLI exit status | qualified locally; CI pending | Exit `0` is reserved for technically successful campaigns, including scientific `Rejected`; technical trial failure is `20`, contract failure `21`, durable-storage failure `22`, cancellation `130`. Subprocess tests exercise worker exit 3 and numeric-overflow plans. |
| A-output separation | qualified locally; CI pending | Versioned machine JSON is written to stdout; human diagnostics are written to stderr. Invalid UTF-8 worker output is preserved as base64 rather than replacement-decoded. |
| B-incremental journal head | qualified locally; CI pending | `Journal.append` hashes only the new event after one startup validation. The regression test counts one event hash per append. |
| B-transition/index atomicity | qualified locally; CI pending | `events` and `trial_state` are updated in one SQLite transaction; impossible Start/Finish transitions are rejected before commit. |
| B-v1 migration | qualified locally; CI pending | Existing event chains are preserved; v2 metadata/index tables are added and rebuilt from a validated legacy chain. |
| B-independent audit | qualified locally; CI pending | `--audit-only` rescans the full chain and checks the durable index and optional external anchor. |
| B-suffix anchoring | partial | Optional `--anchor` binds journal identity, head sequence and hash. It detects a clean suffix deletion only if the anchor is held on a separately trusted boundary. A colocated/rewriteable anchor is explicitly not tamper-proof. |
| C-containment | planned | Existing process-group cleanup remains; cgroup v2 / namespace / recovery-owner work is not claimed by this slice. |
| D+ experiment/spec/registry/scheduler/integrations | planned | No completion claim in this foundation slice. |

## Local qualification performed for this slice

Candidate contents matching the branch files were exercised with:

```bash
python3 -m py_compile scripts/tdi_experiment_supervisor.py
python3 -m unittest discover -s scripts -p test_tdi_experiment_supervisor.py -v
```

Observed in the local qualification environment: 19 tests discovered, 18 passed, one Rust-worker integration test skipped because `TDI_DURABLE_WORKER` was not supplied. The repository CI job supplies that variable after building `tdi-ai/examples/durable_worker`; therefore exact-head GitHub CI remains required before merge.

## Qualification matrix progress

| ID | Status | Current evidence / remaining work |
| --- | --- | --- |
| Q01 | implemented; CI pending | CLI subprocess test: worker exit 3 produces durable `WorkerFailed` and exit 20. |
| Q02 | implemented; CI pending | Duplicate keys, NaN/Infinity, `1e999`, invalid UTF-8, excessive nesting/item/string bounds. |
| Q03 | implemented; CI pending | Hash corruption, index mismatch and impossible transitions fail closed. |
| Q04 | implemented; CI pending | Event-hash counter demonstrates one new hash per append; larger benchmark/environment capture remains for Lot M. |
| Q05 | existing + retained | Continuous/restart equivalence and interrupted active trial tests retained. |
| Q06 | partial | Storage failure has a distinct exit class; explicit disk-full/fsync fault injection remains. |
| Q13/Q14 | partial | Legacy journal migration is tested; portable registry/artifact export is later work. |
| Remaining Q07-Q30 | planned | Must not be inferred from this foundation PR. |

## Next slices

1. Qualify and merge this A/B foundation on the exact PR head.
2. Implement Linux containment capability discovery and cgroup-v2 execution/recovery profiles without claiming cgroups are a complete sandbox.
3. Introduce the versioned experiment/attempt/worker-response contract while preserving v1 compatibility.
4. Build the experiment specification, registry/artifact layer and scheduler adapters only after the durable execution contract is stable.
5. Audit Hub, ElasticXxx, Forge, SciRust, FLAT-ATTENTION and NNIS contracts before adding cross-repository adapters.

Every later slice must update this file with the integrated SHA, exact validation commands, applicable CI status and any material qualification that was not executed.
