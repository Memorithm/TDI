# TDI industrialization qualification reconciliation — 2026-09-16 R2 (refreshed 2026-09-17)

This document is the current-state overlay for `industrialization-qualification-audit-2026-09-16.md`. It does not rewrite historical evidence. The rule is unchanged: **merge presence, local tests, an older SHA, or a subset of green workflows is not exact-head qualification**.

Nothing in this overlay authorizes a protected/final scientific stage, hardware claim, runtime actuation, scientific verdict, or performance claim.

## Default-branch checkpoint

- TDI `main`: `84ab1e81693fa7ddbe9ead67c44c740495d1b430`, merge of PR #465 after qualified PR #385.
- Protected `main` requires the nine baseline contexts Formatting, Tests, Clippy, Preregistration integrity, their Public counterparts, and Rust MSRV. Conditional/domain/hardware gates remain additional application-level requirements when applicable.
- PR #385 final head `b20cbaf053fa446dba9391e521edfdc167b881e5` was non-draft and mergeable, had no unresolved material review thread, and all 42 returned exact-head workflows completed successfully before merge. Its merge presence is therefore accompanied by recorded exact-head software evidence; it still creates no scientific-stage or partner-execution authority beyond the contracts it implements.
- The branch still contains the older dated audit. Where that audit conflicts with this overlay on current repository state, use this overlay while retaining the older file as historical evidence.

## Reconciled merged work

### PR #296 — shared research analysis

- PR #296 is merged on TDI as `cc07d5c55bd2b5aca78b6b8074da21e508a1c735`.
- The merged #296 tree still contains the earlier SciRust candidate pin `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0`; merge presence therefore does not by itself close provenance.
- SciRust #1452 is now qualified and merged: final head `b92414e88744315d19cf1810a0482040621a654b` had every returned applicable pull-request workflow complete successfully, including repository CI and `Research statistics reference qualification`, and merged as `be7fcca3b31cedf722d71a2a56db8f6d088037cf`.
- The unrelated Dream/Thor-only changes and retained physical negative evidence remain isolated in draft SciRust #1455 rather than being reinterpreted as statistics qualification. The retained observation is still `CUBLAS_STATUS_NOT_INITIALIZED` in the first Dream attention `q_proj`; no benchmark JSON exists and the root cause is not established.
- Corrective TDI PR #466 now pins the shared analysis workflow and provenance environment to SciRust merge `be7fcca3...` and updates the paired-analysis documentation. Current #466 exact head is `3f8b0f6819ed0538af0433413761b5a0d6d6e273`; local YAML/diff/rustfmt checks and the 4/4 protocol tests passed, while its GitHub exact-head matrix remains incomplete and therefore non-green as a whole.
- State: **merged #296 with corrective provenance candidate #466**. The stale source-pin debt closes only after #466 itself completes exact-head qualification, clears material review, and merges.

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
- #389 has been retargeted to `main` and non-destructively refreshed through merged #465. Current exact head: `4cc7c022f74c780b3d01af8cac2a188d84bc3f23`.
- The refresh preserved the implementation tree while adding current `main` ancestry; the branch also pins the qualified Forge #39 merge `28067ab0aa1d52a2260d9bb2bf35a346547a292a`.
- The preceding head `2a74502d5b84d17658db5fd7467b3108646a6d56` completed every returned exact-head workflow successfully, including Rust/Public Rust/MSRV, operational engine, real-library adapters, Forge search, shared research analysis and the dedicated shared-admission-evidence gate. That evidence does not qualify `4cc7c022...`.
- The PR remains **draft**. On the refreshed head the dedicated shared-admission-evidence gate, real-library adapters, Elastic admission and Forge search are already green while many baseline/research workflows are still queued; queued/pending/in-progress checks are not green.
- The retained 16-trial/32-output fixture records 1,328,144 reconstructed JSON bytes versus 150,890 stored result-plus-proof JSON bytes. This is a scoped logical/storage-fixture observation only: it is not SQLite-page savings, latency, total memory, DRAM, general throughput, or disk power-loss durability evidence (`fsync=volatile`).
- State: **candidate; blocked on its own complete exact-head qualification and final review**.

### PR #387 — bounded sensitivity and ablation

- The branch is non-destructively refreshed through TDI `main` `84ab1e81693fa7ddbe9ead67c44c740495d1b430`; current exact head is `9c5866e35cc5f484ae53d31ec503c6dc25988949`.
- The former upstream provenance blocker is resolved at source: workflow checkout and `TDI_SCIRUST_SOURCE_COMMIT` now pin qualified SciRust #1452 merge `be7fcca3b31cedf722d71a2a56db8f6d088037cf`.
- The dedicated `TDI sensitivity and ablation` workflow has completed successfully on this exact head, but many baseline/repository workflows remain queued; the PR intentionally remains draft.
- State: **draft candidate; upstream pin corrected, still blocked on its own complete exact-head qualification and final material review**.

### PR #388 — measured engine baselines

- Current published head is `b16b0701f13aeabe68d886723c471827d461a195` and the PR remains draft.
- Its retained development baseline remains scoped evidence only; cancelled, partial, or superseded executions remain non-green evidence and are not converted into performance claims.
- State: **candidate, not qualified**; a complete exact-head benchmark/software matrix is required before promotion.

### PR #465 — local industrial contract routing

- Final exact head: `884c458d0c5d903414526f1e10a973a14519c129`; merged as `84ab1e81693fa7ddbe9ead67c44c740495d1b430`.
- Scope is limited to eight `runs-on` changes for artifact/provenance, Hub edge, common partner, Forge, ElasticXxx, SciRust, FLAT-ATTENTION and NNIS contract workflows: same-repository work uses the repository-scoped `tdi` runner while fork PRs remain on GitHub-hosted `ubuntu-24.04`.
- Immediately before merge, every workflow returned for the exact final head had completed with `success`, including Rust/Public Rust/MSRV, artifact/provenance, Hub edge and all five partner-contract gates. Codex review had completed with no material finding and GitHub returned no unresolved review thread.
- State: **qualified and merged for the declared CI-routing scope**. This changes runner placement only; it grants no scientific stage, partner execution, actuation, scheduler, lease or artifact-store authority.

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

1. Complete corrective #466 exact-head qualification and merge it only if every applicable gate and material review is green; this is the closure point for #296's stale SciRust source pin.
2. Complete #387 exact-head qualification on its refreshed, final-SciRust-pinned head before considering it ready.
3. Complete #389 exact-head qualification on its refreshed `main` ancestry and resolve any material P1/P2 findings before considering it ready.
4. Requalify #388 on its final head while preserving all cancelled/negative executions; continue FLAT/NNIS hardware qualification separately without converting software contracts into hardware, model-quality, or performance conclusions.
