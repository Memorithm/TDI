# TDI engine industrialization status

This is the durable engineering handoff for the industrialization programme. It records software evidence only and does not authorize or reinterpret a scientific stage.

## Baseline

- Historical audit: `2b9cf772d3c0711349f9a70ba7cd7f1bfef90aca`.
- Programme baseline revalidated 2026-09-15: `c973b47355f8bd3eb630a2d267355f733dfde1b2`.
- Foundation A/B: PR #250, head `07b01760f23200665dbc713e73c0cdc35bb49e76`, merged `3b8686047a95c4665f6c1b70b87ab9a9ac3591be` after applicable exact-head workflows succeeded.
- Linux containment C: PR #252, head `1b4735b8aae65ad8f951803d18330706224fa33c`, merged `32e51b9b51a254680d1c5bca0d8e96c2ac39f53f`; exact-head real-kernel and repository workflows succeeded.
- Experiment contract D: PR #253, final head `98a894437cbd6b1b72b23aeb2bdd84548e32124a`, merged `4dd4e0db2837fcffd2e6177542ab14cd289c8540`. All returned pull-request workflows on that exact head completed successfully, including Rust, public Rust, MSRV, TDI AI contracts and real-kernel engine containment.
- Checkpoint/DAG E: PR #254, final head `2afb9c1d391a79c2f6828fe5fcf2f30477c1d357`, merged `bad24136e081157886d9c94283f3d49d43be335a` after all returned applicable exact-head workflows succeeded and no unresolved review thread remained.
- Lot F: PR #261, final head `dceb67d4a9c49ce293c08347cab61db0f3347871`, merged `494240ebc5cd8d9418ce8c174dd65a4bc4e17651` after all returned applicable exact-head workflows succeeded, including the dedicated artifact/provenance contract gate.
- Post-Lot-F base: `494240ebc5cd8d9418ce8c174dd65a4bc4e17651`.
- Hub portable-artifact prerequisite: `Memorithm/scirust-hub` PR #47 merged as `9f666225b186fbca6160dd34068aea1c9af57040`; the qualified endpoint re-verifies stored bytes against Hub digest and size before returning ordinary SHA-256, with bounded portable-digest scan concurrency.
- Lot G1 portable artifact binding: PR #264, final head `1d5e0d3df339ab18309b400cb7b2fbe867d48218`, merged `25292997f5ee30529e8fea08c09375d8158416bf` after all returned applicable exact-head workflows succeeded, including the dedicated `TDI Hub edge contracts` gate; no unresolved review thread remained.
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
| E CheckpointManifest/v1 | qualified | PR #254: content-addressed state plus exact experiment/plan/trial/step/adapter/backend/input/RNG binding; frozen step/observation budgets are embedded in checkpoint identity and enforced against resume inputs. |
| E declarative execution graph | qualified | PR #254: topological Graph/v1 with per-step identity binding to exact Hub ComponentId, component version/manifest digest, capability-contract version, inputs, parameters and checkpoint ports. |
| E pinned Hub structural limits | qualified | PR #254 matches audited Hub v1/default limits for max concurrency, step count, input count, serialized parameters, timeout, input-name grammar and workflow-name admission before producing a preview. |
| E Hub text/version fidelity | qualified | PR #254 output labels reject Unicode `Cc` controls/whitespace like pinned Hub; version parsing intentionally matches Hub's semver-shaped (not full SemVer) grammar. |
| E Hub bridge | qualified structural preview only | PR #254 `compile_hub_workflow_preview()` emits `execution_authorized: false`; current WorkflowSpec/v1 cannot atomically enforce component/capability pins, registry state or portable-artifact identity. |
| E generic scheduling | intentionally delegated | scirust-hub owns ready-set scheduling, retries, cancellation, workflow lifecycle, artifacts, leases and remote transport; TDI does not duplicate them. |
| F ArtifactDescriptor/v1 | qualified | PR #261: portable raw-SHA256/size/media/access descriptor plus byte verification; explicitly not interchangeable with Hub domain-separated `ContentDigest`. |
| F ProvenanceRecord/v1 | qualified | PR #261 binds artifact identity to experiment/plan/trial/attempt/step, implementation, domain, exact named inputs and dependencies. |
| F ExportManifest/v1 | qualified | PR #261 qualifies complete member-set verification, payload verification and monotone access restriction without storage or publication. |
| F exact-domain cache semantics | qualified | PR #261: cache key binds domain/plan/step/implementation/backend/inputs/parameters; disabled and unauthorized reuse fail closed; no cache store or scientific authorization is implemented. |
| F physical CAS/registry | intentionally delegated | Hub remains owner of physical content-addressed storage, registry persistence and transport. |
| G1 Hub portable artifact binding | qualified | PR #264 qualifies `HubArtifactBinding/v1` against the pinned Hub #47 source: Hub artifact id/domain digest plus exact TDI ArtifactDescriptor identity and raw-SHA256/size agreement. Execution and authoritative publication remain hard-false. |
| G authoritative publication | blocked | Hub issue #46 still lacks a qualified durable generation/fencing contract atomically guarding authoritative publication across retries/restarts. TDI must not implement that generic lease/fence authority locally. |

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
| Q10 | qualified for exact checkpoint lineage | PR #254 binds exact checkpoint identity, frozen budgets, immutable inputs, RNG streams and caller-owned execution identity. |
| Q11 | qualified for declarative graph semantics only | PR #254 validates graph/component/capability pins, pinned Hub structural/text/version boundaries and a non-authoritative preview; authoritative Hub execution remains blocked until a versioned Hub edge enforces registry/artifact pins. |
| Q12 | planned | common adapter SDK remains separate from graph/checkpoint semantics. |
| Q13-Q14 | qualified for portable contract verification | PR #261 exact-head qualification passed for descriptor/provenance/export verification; external export integration remains later work. |
| Q15 | qualified for contract semantics | PR #261 exact-head qualification passed for exact-domain cache identity/refusal; durable cache index/storage remains outside this module and reuse still requires caller authorization. |
| Q16 | qualified for portable artifact identity translation | PR #264 exact-head qualification passed for the fail-closed G1 contract pinned to Hub #47; it verifies portable raw-SHA256/size translation and source/repository pins while granting no execution or publication authority. |
| Q17 | blocked | authoritative publication remains unauthorized until Hub-owned durable fencing/generation (issue #46) and atomic publication are qualified. |
| Q18-Q30 | planned | cross-repository integrations, statistics, CLI/viewer/exports, hardware adapters, docs and final integrated qualification. |

## Lot-E qualified validation

PR #254 qualified 20 checkpoint/graph contract tests (6 checkpoint + 14 graph) on exact head `2afb9c1d391a79c2f6828fe5fcf2f30477c1d357` before merge.

Checkpoint regressions cover exact lineage, input/RNG drift, content identity, embedded progress limits and rejection of caller-side budget widening. Graph regressions include component-id/version/manifest/capability identity sensitivity, single-alias pin consistency, Hub input grammar, max 32 inputs, 16 KiB serialized parameters, audited 3,600,000 ms timeout bound, workflow-name NUL rejection, pinned Hub prerelease-version behavior and Unicode control rejection for output labels. The existing `TDI engine Linux containment` workflow executes these contracts alongside all previously qualified engine-contract tests. The cgroup kernel job remains enabled so Lot E cannot accidentally regress the contained execution foundation.

## Lot-F qualified validation

PR #261 qualified the portable artifact/provenance/export/cache contract tests on exact head `dceb67d4a9c49ce293c08347cab61db0f3347871` before merge `494240ebc5cd8d9418ce8c174dd65a4bc4e17651`:

```bash
PYTHONPATH=scripts python3 -m py_compile \
  scripts/tdi_artifact_contract.py \
  scripts/test_tdi_artifact_contract.py
PYTHONPATH=scripts python3 -m unittest \
  scripts/test_tdi_artifact_contract.py -v
```

The dedicated `TDI artifact and provenance contracts` workflow succeeded on the final exact PR head. The qualified tests cover raw-payload digest/size verification, TDI-vs-Hub digest non-interchangeability, canonical provenance ordering, exact export member/provenance binding, monotone access restrictions, exact-domain cache identity, disabled/unauthorized cache refusal, and fail-closed structural bounds.

## Cross-repository decisions

- TDI declares Rust 1.85; audited scirust-hub declares Rust 1.89. No mandatory direct Rust dependency is introduced for the graph or artifact bridge.
- The Lot-E graph contract pins `Memorithm/scirust-hub` source `4bf6186841e1ea70ed15cd84faf33de9b48429cd`, WorkflowSpec schema `1`, model `1.2.0`, capability/version grammar, canonical UUID identifiers and the relevant default RunSpec admission limits.
- Graph steps pin exact Hub ComponentId, component version, component-manifest digest and capability-contract version. Current Hub WorkflowSpec/v1 cannot enforce all those pins atomically during normal workflow submission, so Lot E exposes a non-executable preview only.
- The pinned Hub `Version::parse` is semver-shaped rather than a full SemVer parser; TDI mirrors that exact accepted wire grammar instead of rejecting Hub-valid version labels.
- scirust-hub remains owner of generic DAG scheduling, remote workers, leases, heartbeats, retries, cancellation, artifact storage/transport and registry persistence; TDI remains owner of scientific meaning, checkpoint admissibility, portable scientific provenance and accepted publication semantics.
- Hub's current remote lease v1 has attempt/lease identity, heartbeat and expiry but no qualified fencing/generation token preventing stale publication after reassignment; distributed fencing remains a later Lot-H requirement.
- Hub `ContentDigest` at pinned source is domain-separated (`scirust-hub-digest:v1` + framed domain + bytes) and is not interchangeable with TDI portable raw SHA-256. Lot F therefore records raw payload SHA-256 for portable verification and explicitly forbids treating that value as a Hub CAS key without verified translation.
- Hub PR #47 provides that verified translation surface for artifact identity only: TDI G1 checks the returned ordinary SHA-256 and byte count against ArtifactDescriptor/v1 while retaining Hub ContentDigest as opaque translation provenance. The bridge does not authenticate transport itself and does not create execution/publication authority.
- Lot F does not implement a physical CAS, registry, transport, or durable cache store. It defines TDI-owned identities/verification and exact reuse boundaries only.
- ElasticXxx remains owner of generic observe/forecast/plan/validate/act/verify/commit-or-rollback resource policy.
- Forge remains owner of candidate proposal/search; TDI controls allowed evaluation observations and scientific verdicts.
- `ExperimentSpec/v1`, CheckpointManifest/v1, Graph/v1 and Lot-F artifact/provenance/cache contracts are infrastructure contracts and must not be used to synthesize or authorize a protected/final series surface.

## Next

1. Implement and qualify Hub issue #46 in Hub-owned orchestration: durable monotone fencing/generation plus atomic authoritative publication across retry, cancellation and restart. Only then may TDI advance G2 to an authoritative edge.
2. After that Hub boundary is qualified, bind exact component/capability pins and publication authority without duplicating Hub scheduling, leases or storage in TDI.
3. Add ElasticXxx, Forge, SciRust, FLAT-ATTENTION and NNIS integrations only through qualified versioned contracts.
4. Continue statistics/sensitivity, CLI/API/viewer, external exports and measured engine benchmarks.
