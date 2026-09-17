# TDI industrialization qualification reconciliation — 2026-09-16 R2 (refreshed 2026-09-17)

This document is the current-state overlay for `industrialization-qualification-audit-2026-09-16.md`. It does not rewrite historical evidence. The rule is unchanged: **merge presence, local tests, an older SHA, or a subset of green workflows is not exact-head qualification**.

Nothing in this overlay authorizes a protected/final scientific stage, hardware claim, runtime actuation, scientific verdict, or performance claim.

## Default-branch checkpoint

- TDI `main`: `eca2f3d6892b9a8758e067140297993208ffb4c3`, merge of PR #385.
- Protected `main` requires the nine baseline contexts Formatting, Tests, Clippy, Preregistration integrity, their Public counterparts, and Rust MSRV. Conditional/domain/hardware gates remain additional application-level requirements when applicable.
- PR #385 final head `b20cbaf053fa446dba9391e521edfdc167b881e5` was non-draft and mergeable, had no unresolved material review thread, and all 42 returned exact-head workflows completed successfully before merge. Its merge presence is therefore accompanied by recorded exact-head software evidence; it still creates no scientific-stage or partner-execution authority beyond the contracts it implements.
- The branch still contains the older dated audit. Where that audit conflicts with this overlay on current repository state, use this overlay while retaining the older file as historical evidence.

## Reconciled merged work

### PR #296 — shared research analysis

- PR #296 is merged on TDI as `cc07d5c55bd2b5aca78b6b8074da21e508a1c735`.
- It still pins the earlier SciRust candidate `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0`, despite its declared dependency on the reviewed final SciRust revision.
- SciRust #1452 has been narrowed back to its statistics-only scope. Its current exact head is `b92414e88744315d19cf1810a0482040621a654b`; the Dream/Thor-only changes and their retained physical negative evidence were extracted to draft SciRust #1455 rather than silently discarded.
- On #1452 head `b92414e...`, Research statistics reference qualification, Workspace Rustdoc, Native ARM64, M53/M54, WGPU, API lexicon and the other returned completed workflows are green; repository `CI` is still in progress at this refresh. Therefore #1452 is **not yet qualified or merged**, but it is no longer blocked by treating the unrelated Dream gate as applicable to the statistics diff.
- The retained Dream negative result remains owned by draft #1455: on physical Thor the checkpoint loads, then the first attention `q_proj` fails with `CUBLAS_STATUS_NOT_INITIALIZED` from `cublasLtMatmulAlgoGetHeuristic`; no benchmark JSON exists and the root cause is not established.
- State: **merged but not finally qualified across the TDI↔SciRust pin**. Closure requires a fully qualified SciRust #1452 merge, then a corrective TDI pin PR to the resulting merge SHA and its own exact-head qualification.

### PR #331 — real library replay adapters

- PR #331 is merged as `c5f6dcad6094f945af3d4df211490a76a24f8416`.
- This overlay does not infer retroactive exact-head qualification solely from its presence on `main`.

### PR #385 — durable Forge scientific search

- Final TDI head: `b20cbaf053fa446dba9391e521edfdc167b881e5`.
- Forge #39 is merged as `28067ab0aa1d52a2260d9bb2bf35a346547a292a`, and the final #385 workflow pins that qualified merge.
- PR #385 is merged into TDI `main` as `eca2f3d6892b9a8758e067140297993208ffb4c3` after its complete returned exact-head workflow set succeeded and its material review thread was resolved.
- The implementation keeps ownership boundaries intact: Forge supplies proposals/stage permits/paid retries/Pareto selection; Hub owns workflows/orchestration/publication; TDI persists scientific search state, independently evaluates candidate evidence, retains measured attempt costs, and reconciles cancellation/recovery.
- State: **qualified software integration for the declared contract**. No protected/final holdout, model result, generated hostile code, GPU result, scientific promotion, second scheduler, lease manager, or generic artifact-store ownership is implied.

### ElasticXxx boundary

- PR #279 final head `29d9f5555483b59c591ccb7d4871e84ac02d677c` merged as `6e2a15a5711f8dc2eb7e77886bbed247aa425e6b`; the original non-actuating boundary has the exact-head software evidence recorded by the earlier audit.
- PR #288 later hardened admitted root-evidence binding and is present on `main`; this overlay does not manufacture retroactive qualification from merge presence alone.
- Execution, actuation, holdout, stage and verdict authority remain absent unless separately qualified.

## Active industrialization frontier

### PR #389 — shared admission-evidence storage

- Parent #385 is now merged on `main`.
- #389 has been retargeted to `main` and non-destructively refreshed. Current exact head: `2a74502d5b84d17658db5fd7467b3108646a6d56`.
- The refresh preserved the implementation tree while adding current `main` ancestry; the branch also pins the qualified Forge #39 merge `28067ab0aa1d52a2260d9bb2bf35a346547a292a`.
- The PR remains **draft**. Queued/pending/in-progress checks are not green and prior-head evidence does not qualify the current head.
- The retained 16-trial/32-output fixture records 1,328,144 reconstructed JSON bytes versus 150,890 stored result-plus-proof JSON bytes. This is a scoped logical/storage-fixture observation only: it is not SQLite-page savings, latency, total memory, DRAM, general throughput, or disk power-loss durability evidence (`fsync=volatile`).
- State: **candidate; blocked on its own complete exact-head qualification and final review**.

### PR #387 — bounded sensitivity and ablation

- Current published head is `d5e5115add1689a59b0514fe999a636b5c2e404a`.
- It still depends on the eventual reviewed, fully exact-head-green SciRust #1452 merge and must then be repinned to that final merge SHA.
- State: **draft/blocked on final SciRust provenance plus its own exact-head gates**.

### PR #388 — measured engine baselines

- Current published head is `b16b0701f13aeabe68d886723c471827d461a195` and the PR remains draft.
- Its retained development baseline remains scoped evidence only; cancelled, partial, or superseded executions remain non-green evidence and are not converted into performance claims.
- State: **candidate, not qualified**; a complete exact-head benchmark/software matrix is required before promotion.

### PR #465 — local industrial contract routing

- Current exact head: `884c458d0c5d903414526f1e10a973a14519c129`.
- Scope is limited to eight `runs-on` changes for artifact/provenance, Hub edge, common partner, Forge, ElasticXxx, SciRust, FLAT-ATTENTION and NNIS contract workflows: same-repository work may use the repository-scoped `tdi` runner while fork PRs remain on GitHub-hosted `ubuntu-24.04`.
- At this refresh the PR is non-draft and mergeable, Codex review completed with no material finding, and multiple exact-head gates are green including Public Rust, MSRV, Forge/ElasticXxx/SciRust/FLAT partner contracts and Hub edge. Rust validation and several unrelated research workflows are still queued.
- State: **candidate; not mergeable under programme policy until every applicable exact-head workflow is complete and green**.

### PR #464 — runner-load scoping

- Current published head: `0d265b7754dbcd892ba0961bd8028b97a106cfa6`.
- It remains draft and preserves the nine server-required baseline contexts while scoping non-baseline engineering-document work and cancelling superseded runs only within workflow/ref scope.
- State: **draft candidate; exact-head qualification required before promotion**.

### PR #390 — this reconciliation

- The branch is based on current post-#385 `main` ancestry and is being updated only with current-state evidence.
- This documentation PR itself remains a candidate until every applicable exact-head workflow on its final head succeeds and no material review thread remains.

## Evidence and ownership discipline retained

- Hub owns generic orchestration, registry resolution, scheduling, leases, transport, artifact storage and authoritative publication. TDI does not duplicate those services.
- TDI owns scientific identities, admissibility, provenance, stage/verdict semantics and accepted evidence.
- Forge retains `PROPOSE/MUTATE → COMPILE → VERIFY → MEASURE → SELECT`; TDI integrations must not turn an unverified proposal into evidence.
- `queued`, `pending`, `in_progress`, `cancelled`, `failure`, missing checks and unavailable hardware are not green.
- A software contract or interchange envelope is not partner execution evidence.
- A benchmark workflow that does not complete successfully on the exact candidate head yields no promotable performance result.
- Protected/final holdouts remain governed by `AGENTS.md` and the series/programme preregistrations; this reconciliation opens none.

## Next closure order

1. Finish SciRust #1452 on its statistics-only exact head. If every applicable workflow is green and review remains clear, merge it; then open the corrective TDI #296 source-pin PR and repin #387 to the resulting SciRust merge SHA.
2. Complete #465 exact-head qualification before merging the local industrial-contract runner route; queued workflows are not evidence.
3. Complete #389 exact-head qualification on its refreshed `main` ancestry and resolve any material P1/P2 findings before considering it ready.
4. Requalify #388 on its final head while preserving all cancelled/negative executions.
5. Continue FLAT/NNIS hardware qualification separately; do not convert software/canonical-evidence contracts into hardware, model-quality, or performance conclusions.
