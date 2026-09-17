# TDI engine industrialization status

This is the durable engineering handoff for the industrialization programme. It records software evidence only and does not authorize or reinterpret a scientific stage.

**SciRust statistics provenance update (2026-09-17):** Memorithm/scirust#1452 qualified on final head `b92414e88744315d19cf1810a0482040621a654b` and merged as `be7fcca3b31cedf722d71a2a56db8f6d088037cf`. Corrective TDI PR #466 qualified on exact head `729f35693e21ab93c4522987cca7ca70cb62feaa` and merged as `c43baf742ad3c85674b83956a01807bf7664771b`, so the stale SciRust candidate pin from merged #296 is now closed for the shared research-analysis consumer. Sensitivity PR #387 independently pins the qualified SciRust merge and must still complete its own exact-head qualification.

**Current exact-head qualification note (2026-09-17):** the current default-branch checkpoint for this refresh is `9d24ec9ce73370f4948144b1c0ae55cd6c82ce91` (merged PR #467, after TDI-24.0 #406 and qualified provenance correction #466). For current qualification state of earlier Lot-H/operational/export work and active candidates #387/#388/#390/#464, use [`industrialization-qualification-audit-2026-09-16-r2.md`](industrialization-qualification-audit-2026-09-16-r2.md), which is a reconciliation overlay on the historical [`industrialization-qualification-audit-2026-09-16.md`](industrialization-qualification-audit-2026-09-16.md). Where an older row below conflicts with that evidence, use the newer exact-head evidence and retain the older row only as historical programme context. Merge presence alone is not a qualification result.

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
- Hub authoritative-publication prerequisite: `Memorithm/scirust-hub` PR #48 merged `18b6a39d48c20f98c997644b9a5d835bbc437cfe` with `PublicationFence/v1`, PR #49 merged `547f71f9f57fcd850deb8fe5206b7c82d1908cae` with durable SQLite fence/publication persistence, PR #50 merged `dfc14fe832fd4c5e30f228dc7075b41e0bee9118` after exact-head CI qualified attempt fencing + authoritative publication + `FromStep` consumption, and PR #51 merged `a98f77dc52caa30d055d62acc84f77b176b9bc9b` after exact-head targeted and repository CI qualified the authenticated read-only publication endpoint. Hub issue #46 is closed; TDI may consume this boundary but may not mint fences or publish outputs itself.
- Hub exact execution-admission prerequisite: `Memorithm/scirust-hub` PR #54 final head `698895dc2985983c4c77b6bea20bc4291d3fb387` merged `7187d72e025e0a7a88b52d9fc4675bae26581f76` after exact-head CI succeeded and the review-required protocol fix exposed the full WorkflowSpec beside the exact step admission pins. Hub owns registry resolution and enforcement; TDI only consumes the resulting evidence.
- Lot G1 portable artifact binding: PR #264, final head `1d5e0d3df339ab18309b400cb7b2fbe867d48218`, merged `25292997f5ee30529e8fea08c09375d8158416bf` after all returned applicable exact-head workflows succeeded, including the dedicated `TDI Hub edge contracts` gate; no unresolved review thread remained.
- Lot G2 authoritative publication binding: PR #270, final head `6c4e4d6bda4693033dc48ed405880cd609b4e5dd`, merged `7b6401e91973d4b4895d717393942aceb0cf4e6b` after every returned applicable exact-head workflow succeeded, including `TDI Hub edge contracts`, full hosted tests/Clippy/formatting, Rust/Public Rust and MSRV. The review-required output-label correction aligned publication labels with the 1..=128-byte Hub/Graph grammar while retaining the strict step-key grammar. The contract pins Hub #51 merge `a98f77dc52caa30d055d62acc84f77b176b9bc9b`, binds exact workflow/step/attempt/generation/output lineage to the existing portable artifact binding, and keeps execution authorization false.
- Lot G3 exact execution-admission binding: PR #272, final head `a336f798ddbc230abc4a38e8233845d6c9b1e47d`, merged `39eb8a566855d017ea0dbb7094168b23c6655037` after all returned applicable exact-head workflows succeeded. The three review findings were resolved only after root-artifact proof became mandatory, restored evidence identities were recomputed from embedded canonical evidence, and WorkflowSpec equality preserved JSON type fidelity. Scientific-stage authorization remains hard-false.
- Lot H common partner-adapter foundation: PR #273, final head `53316ef9af6c402fbda6728707da03903dcbfa0a`, merged `5905cfeed38f4a12ec6f8c9d238a0ede16815d2a` after all returned exact-head workflows succeeded, including the dedicated partner-adapter gate. PR #275 then hardened the public G3 version fields against JSON Boolean aliases on exact head `2cadda0ae0fe6511b7a90ba9b3e6a1c86ce9b4eb`, merged `c3e2275d807a67aac7ac1be36412d1f8e1ed7f07` after every returned applicable workflow succeeded.
- Lot H Forge-specific boundary: PR #277, final head `942156e718c812ff0725f706ecc6adf51ea4a0a7`, merged `183fcd6af2ddfefce4a5708e72f683d14118dbdf` after every returned exact-head workflow completed successfully, including `TDI Forge partner contracts`, common partner/Hub/artifact gates, Rust/Public Rust and MSRV. It remains a non-executing interchange contract and grants no Forge, holdout, stage, verdict or actuation authority.
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
| E Hub bridge | qualified structural preview only | PR #254 `compile_hub_workflow_preview()` remains a non-authoritative structural compiler pinned to its audited source; G3 consumes the separately qualified Hub exact-admission surface rather than rewriting Lot E. |
| E generic scheduling | intentionally delegated | scirust-hub owns ready-set scheduling, retries, cancellation, workflow lifecycle, artifacts, leases and remote transport; TDI does not duplicate them. |
| F ArtifactDescriptor/v1 | qualified | PR #261: portable raw-SHA256/size/media/access descriptor plus byte verification; explicitly not interchangeable with Hub domain-separated `ContentDigest`. |
| F ProvenanceRecord/v1 | qualified | PR #261 binds artifact identity to experiment/plan/trial/attempt/step, implementation, domain, exact named inputs and dependencies. |
| F ExportManifest/v1 | qualified | PR #261 qualifies complete member-set verification, payload verification and monotone access restriction without storage or publication. |
| F exact-domain cache semantics | qualified | PR #261: cache key binds domain/plan/step/implementation/backend/inputs/parameters; disabled and unauthorized reuse fail closed; no cache store or scientific authorization is implemented. |
| F physical CAS/registry | intentionally delegated | Hub remains owner of physical content-addressed storage, registry persistence and transport. |
| G1 Hub portable artifact binding | qualified | PR #264 qualifies `HubArtifactBinding/v1` against the pinned Hub #47 source: Hub artifact id/domain digest plus exact TDI ArtifactDescriptor identity and raw-SHA256/size agreement. Execution and authoritative publication remain hard-false. |
| G2 authoritative publication binding | qualified | PR #270 final head `6c4e4d6bda4693033dc48ed405880cd609b4e5dd`, merge `7b6401e91973d4b4895d717393942aceb0cf4e6b`: exact-head Hub-edge, hosted full tests/Clippy/formatting, Rust/Public Rust, MSRV and all other returned gates succeeded. TDI validates and content-addresses exact authoritative publication lineage while keeping `execution_authorized: false`. |
| G3 exact workflow execution admission | qualified | PR #272 final head `a336f798ddbc230abc4a38e8233845d6c9b1e47d`, merge `39eb8a566855d017ea0dbb7094168b23c6655037`: full returned exact-head qualification passed after all P1/P2 review findings were corrected and resolved. A combined v3 artifact binding keeps authoritative publication evidence, sets Hub execution admission true and keeps scientific-stage authorization false. |
| H common partner adapter descriptor | qualified | PR #273 qualifies `PartnerAdapter/v1` + `AdmittedPartnerStep/v1` for ElasticXxx, Forge, SciRust, FLAT-ATTENTION and NNIS on exact admitted Hub steps while granting no holdout, verdict, stage, actuation or partner-execution authority; PR #275 hardens the embedded G3 version-type boundary. |
| H Forge scientific-search boundary | qualified | PR #277 final head `942156e718c812ff0725f706ecc6adf51ea4a0a7`, merge `183fcd6af2ddfefce4a5708e72f683d14118dbdf`: all returned exact-head workflows succeeded. The contract pins Forge external/scientific domain v1, leak-safe generation/verification/final-holdout identities and authority-free manifest compilation without qualifying execution. |
| H ElasticXxx resource-control boundary | qualified for original #279 contract; later hardening not requalified by this audit | PR #279 final head `29d9f5555483b59c591ccb7d4871e84ac02d677c` merged as `6e2a15a5711f8dc2eb7e77886bbed247aa425e6b`; it admits only observe/plan/dry-run intent and keeps execution/actuation/scientific authority false. PR #288 later hardened root-evidence binding and is present on `main`, but merge presence is not treated here as retroactive exact-head qualification evidence. |

## Qualification matrix progress

The original Q01–Q30 acceptance criteria are retained below. Earlier versions
incorrectly substituted portable contract checks for Q11/Q13/Q14/Q16/Q18.
Contract qualification alone does not establish an operational research product.
“Local pass” means implemented and tested, with final-head CI/merge still required.

| ID | Original requirement | State and evidence / remaining work |
| --- | --- | --- |
| Q01 | Failed worker has correct journal and CLI exit | Qualified #250; real Hub failure/restart scenario additionally passes locally. |
| Q02 | Malformed/duplicate/overflow/nonfinite/deep JSON rejected | Qualified #250; real HTTP parser failure boundaries pass locally. |
| Q03 | Impossible journal transitions and corruption detected | Qualified #250. |
| Q04 | No quadratic append reread; documented benchmark | Hash-growth property qualified #250; measured scaling benchmark remains. |
| Q05 | Crash before/after commit and consistent resume | Trial-boundary qualification #250/#252; real Hub restart without new attempts passes locally. |
| Q06 | Interrupted creation, disk-full and persistence failures | Partial; new atomic export/fsync/disk-full and pre-dispatch persistence fault tests pass locally. |
| Q07 | No abandoned descendant in qualified containment profile | Qualified cgroup-v2 profile #252; the Hub process path does not inherit that quota/sandbox qualification. |
| Q08 | Real Linux RAM/CPU/process limits verified | Qualified cgroup-v2 profile #252; not claimed for the new process-only Hub fixture. |
| Q09 | Explicit absent/insufficient GPU behavior | Unsupported hard VRAM quota fails closed; no hardware measurement qualification. |
| Q10 | Changed plan or wrong checkpoint refuses resume | Qualified lineage/budget contracts #254. |
| Q11 | Bounded concurrency, dependencies, aggregation order | Hub owns scheduling; real two-trial run→verify DAG passes locally. Broader remote failure/load matrix remains. |
| Q12 | Independent source, branches, caches and RNG | Partial; partner descriptors do not qualify backend execution. Multi-domain SDK conformance remains. |
| Q13 | Registry/artifact export/import preserves provenance | Local pass for result-evidence bundles, new Hub artifact receipts, re-export and SQLite backup; executable input/toolchain packaging is outside this bundle. |
| Q14 | Legacy journal migration preserves old evidence | Qualified non-destructive journal migration #250; unrelated to portable artifact contract #261. |
| Q15 | Cache dependency invalidation and domain separation | Local pass for durable exact deterministic-data index and verified lookup; source lineage preserved, no fresh trial/timing claim. |
| Q16 | Old attempt/expired lease cannot publish | Hub #48–#51 owns qualified fencing/publication persistence; portable digest translation alone does not establish this. |
| Q17 | Duplicate/disordered messages yield one authoritative commit | Hub publication contracts plus local no-redispatch/lost-response reconciliation; broader remote fault qualification remains. |
| Q18 | Real TDI plan, worker execution and verified result | Local pass: actual Rust durable_worker, real Hub HTTP/process/SQLite and independent counter oracle; pinned Hub #55 transfer is merged. |
| Q19 | Elastic resource control preserves protocol | Partial: #279 interchange and #282 validation repairs; actual resource-control integration remains. |
| Q20 | Forge excludes incorrect candidates before measurement | Partial: #277 nonexecuting contract; real search/evaluation integration remains. |
| Q21 | Shared SciRust primitive consumed against reference | Remaining; a promotion descriptor alone is insufficient. |
| Q22 | Correct statistical units, exclusions and uncertainty | Remaining general analysis/report integration. |
| Q23 | Search/sensitivity bounds, lineage and oracle qualification | Remaining. |
| Q24 | Prepare→run→inspect→cancel→resume→export→verify | Local pass for documented CLI, separate cancellation scenario and read-only JSON API. |
| Q25 | Real UI data, incomplete/error/permission states | Local pass for catalogue viewer, escaping, host policy and actual workflow results; statistical plots remain. |
| Q26 | MLflow/telemetry failure visible without durable loss | Remaining optional export/telemetry integration. |
| Q27 | Restricted data absent from logs/cache/exports/candidates | Partial: restricted roots rejected before reads and dedicated non-final store required; no per-artifact scientific ACL claim on a mixed Hub. |
| Q28 | FLAT/NNIS differential contracts and exact hardware status | Remaining; no GPU evidence claimed. |
| Q29 | Executed documentation on declared versions | Local pass for operational CLI examples with actual binaries; whole product documentation still in progress. |
| Q30 | Final-head CI, merge SHA and post-merge verification | Hub #55 head `af0d8195a88f96e3172961b5e6ff6d08e2aa9300` passed CI before merge `ccdcb99a4573dbefb944af0df713101b100b5f78`; TDI operational PR qualification remains. |

See [the executable operational guide](operational-engine.md) and the mandatory
`TDI operational engine` workflow for reproducible commands and exact limits.

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
- Graph steps pin exact Hub ComponentId, component version/manifest digest and capability-contract version. The original Lot-E compiler remains a non-authoritative preview because its pinned source predates workflow admission. Hub PR #54 now enforces those identities at execution admission; Lot G3 consumes that separately qualified surface instead of changing Hub ownership or rewriting Lot E.
- The pinned Hub `Version::parse` is semver-shaped rather than a full SemVer parser; TDI mirrors that exact accepted wire grammar instead of rejecting Hub-valid version labels.
- scirust-hub remains owner of generic DAG scheduling, registry resolution, remote workers, leases, heartbeats, retries, cancellation, artifact storage/transport and publication; TDI remains owner of scientific meaning, checkpoint admissibility, portable scientific provenance and accepted evidence semantics.
- Hub publication authority is qualified end-to-end for the current boundary: PR #48 defines `PublicationFence/v1`, PR #49 persists it durably, PR #50 issues a fresh fence per persisted attempt and makes downstream `FromStep` consume only authoritative publication, and PR #51 exposes a read-only authenticated publication DTO. TDI G2 consumes that evidence only; it cannot schedule, fence, publish, lease or store on Hub's behalf.
- Hub PR #54 qualifies exact workflow execution admission: every workflow step is covered by an immutable pin for component version, manifest digest and capability-contract version, while component id and capability name remain in WorkflowSpec. The protocol exposes WorkflowSpec beside the pin envelope, allowing TDI G3 to compare the complete submitted structure and identities without reimplementing registry resolution.
- Hub `ContentDigest` at pinned source is domain-separated (`scirust-hub-digest:v1` + framed domain + bytes) and is not interchangeable with TDI portable raw SHA-256. Lot F therefore records raw payload SHA-256 for portable verification and explicitly forbids treating that value as a Hub CAS key without verified translation.
- Hub PR #47 provides that verified translation surface for artifact identity only: TDI G1 checks the returned ordinary SHA-256 and byte count against ArtifactDescriptor/v1 while retaining Hub ContentDigest as opaque translation provenance. The bridge does not authenticate transport itself and does not create execution/publication authority.
- Lot F does not implement a physical CAS, registry, transport, or durable cache store. It defines TDI-owned identities/verification and exact reuse boundaries only.
- ElasticXxx remains owner of generic observe/forecast/plan/validate/act/verify/commit-or-rollback resource policy. PR #279 consumes only the published non-actuating process/config/evidence boundary and explicitly requires independent ElasticXxx validation of the actual config bytes before any future invocation.
- Forge remains owner of candidate proposal/search; TDI controls allowed evaluation observations and scientific verdicts. PR #277 qualifies only a non-executing interchange envelope.
- SciRust remains the promotion target for reusable mathematical primitives rather than a duplicate TDI scientific-state machine.
- FLAT-ATTENTION remains an execution/comparison target; TDI adapter evidence cannot authorize production routing or performance claims.
- NNIS owns NVIDIA-specific execution/qualification evidence; the common TDI adapter records identity only and does not create hardware evidence.
- `ExperimentSpec/v1`, CheckpointManifest/v1, Graph/v1 and all G1-G3/partner Hub evidence bindings are infrastructure contracts and must not be used to synthesize or authorize a protected/final series surface.

## Next

1. Continue from default branch `9d24ec9ce73370f4948144b1c0ae55cd6c82ce91`: #385/#389/#466/#467 are merged for their declared software scopes. #387 sensitivity is a draft candidate on current head `4fbd8f8a383a2bb5ddb156eb4fb4cdf8851fa9c2`; its dedicated sensitivity and shared-admission workflows have succeeded, but required and unrelated applicable workflows are still queued/in-progress, so it is not merge-qualified.
2. #388 measured engine benchmarks and #464 runner-load scoping have now both been refreshed non-destructively from current `main` and are structurally mergeable drafts at exact heads `9c1ea100aa2d6e1fcabe0f49dfb519c28eeda6d2` and `fbb5d3f1687c05d9e129cb642e7f26fbf2200858`. Neither is merge-qualified: their refreshed exact-head matrices remain incomplete. #464 additionally preserves the qualified TDI-8.1/TDI-9.1 freeze workflows byte-for-byte rather than applying runner-load scoping to them.
3. Treat the old #296 SciRust candidate pin as closed only through qualified corrective PR #466. SciRust #1452 qualified and merged as `be7fcca3b31cedf722d71a2a56db8f6d088037cf`; the unrelated Dream/Thor negative hardware/runtime evidence remains isolated in draft SciRust #1455 and must not be reinterpreted as statistics qualification.
4. Continue measured engine benchmarks, remaining statistics/sensitivity integration, and hardware-specific FLAT/NNIS qualification only when the required physical evidence is available. Do not convert queued/pending workflows, local passes, merge presence, or unavailable hardware into qualification claims.
