# TDI industrialization qualification reconciliation — 2026-09-16 R2 (refreshed 2026-09-17)

This document is the current-state overlay for `industrialization-qualification-audit-2026-09-16.md`. It does not rewrite historical evidence. The rule is unchanged: **merge presence, local tests, an older SHA, or a subset of green workflows is not exact-head qualification**.

Nothing in this overlay authorizes a protected/final scientific stage, hardware claim, runtime actuation, scientific verdict, or performance claim.

## Default-branch checkpoint

- TDI `main`: `9d24ec9ce73370f4948144b1c0ae55cd6c82ce91`, merge of qualified PR #467 after merged TDI-24.0 #406 and qualified PR #466.
- Protected `main` requires the nine baseline contexts Formatting, Tests, Clippy, Preregistration integrity, their Public counterparts, and Rust MSRV. Conditional/domain/hardware gates remain additional application-level requirements when applicable.
- PR #385 final head `b20cbaf053fa446dba9391e521edfdc167b881e5` was non-draft and mergeable, had no unresolved material review thread, and all 42 returned exact-head workflows completed successfully before merge. Its merge presence is therefore accompanied by recorded exact-head software evidence; it still creates no scientific-stage or partner-execution authority beyond the contracts it implements.
- The branch still contains the older dated audit. Where that audit conflicts with this overlay on current repository state, use this overlay while retaining the older file as historical evidence.

## Reconciled merged work

### PR #296 — shared research analysis

- PR #296 is merged on TDI as `cc07d5c55bd2b5aca78b6b8074da21e508a1c735`.
- Its original workflow pinned SciRust candidate `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0`, creating a provenance debt against its declared final-source requirement.
- SciRust #1452 subsequently qualified on exact head `b92414e88744315d19cf1810a0482040621a654b` and merged as `be7fcca3b31cedf722d71a2a56db8f6d088037cf`; the unrelated Dream/Thor negative result remains isolated in draft SciRust #1455.
- Corrective TDI #466 pins that qualified SciRust merge in checkout and `TDI_SCIRUST_SOURCE_COMMIT`, qualified on exact head `729f35693e21ab93c4522987cca7ca70cb62feaa` with all 50 returned pull-request workflows successful and no unresolved review thread, then merged as `c43baf742ad3c85674b83956a01807bf7664771b`.
- State: **qualified provenance correction on `main` for the shared research-analysis consumer**. This closes the stale #296 SciRust source pin only; it does not retroactively create hardware, performance, model-quality or scientific-stage evidence.
- The unrelated Dream/Thor negative evidence remains owned by draft SciRust #1455: the checkpoint loads, then the first attention `q_proj` fails with `CUBLAS_STATUS_NOT_INITIALIZED` from `cublasLtMatmulAlgoGetHeuristic`; no benchmark JSON exists and the root cause remains unestablished.


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

- Final head `4cc7c022f74c780b3d01af8cac2a188d84bc3f23` was refreshed through qualified #465 and pins the qualified Forge #39 merge `28067ab0aa1d52a2260d9bb2bf35a346547a292a`.
- PR #389 completed its applicable exact-head qualification and merged as `ce2a620b39ec9209f668852594428c135966f60c`.
- The retained 16-trial/32-output fixture records 1,328,144 reconstructed JSON bytes versus 150,890 stored result-plus-proof JSON bytes. This remains a scoped logical/storage-fixture observation only: it is not SQLite-page savings, latency, total memory, DRAM, general throughput, or disk power-loss durability evidence (`fsync=volatile`).
- State: **qualified and merged for the declared shared-evidence/indexing software scope**.

### PR #387 — bounded sensitivity and ablation

- After #467 merged, the branch was non-destructively refreshed through current `main`; current exact head is `4fbd8f8a383a2bb5ddb156eb4fb4cdf8851fa9c2`.
- The branch pins qualified SciRust merge `be7fcca3b31cedf722d71a2a56db8f6d088037cf` and no longer has the former SciRust-candidate provenance blocker.
- The dedicated exact-head `TDI sensitivity and ablation` and `TDI shared admission evidence` workflows have succeeded on this head. Several required/applicable workflows, including Rust/Public Rust/MSRV and additional research gates, remain queued or in progress.
- State: **draft candidate; blocked on completion of its own refreshed exact-head workflow matrix and review reconciliation**. Queued/pending/in-progress checks are not green.


### PR #388 — measured engine baselines

- Current published head is `24981bb99a1662502c901f62d1b1da035fe8fa96` and the PR remains draft. GitHub currently reports it non-mergeable against the newer default branch, so it must be refreshed before a new exact-head matrix can qualify it.
- Its retained development baseline remains scoped evidence only; cancelled, partial, or superseded executions remain non-green evidence and are not converted into performance claims.
- State: **candidate, not qualified**; a complete exact-head benchmark/software matrix is required before promotion.

### PR #465 — local industrial contract routing

- Final exact head: `884c458d0c5d903414526f1e10a973a14519c129`; merged as `84ab1e81693fa7ddbe9ead67c44c740495d1b430`.
- Scope is limited to eight `runs-on` changes for artifact/provenance, Hub edge, common partner, Forge, ElasticXxx, SciRust, FLAT-ATTENTION and NNIS contract workflows: same-repository work uses the repository-scoped `tdi` runner while fork PRs remain on GitHub-hosted `ubuntu-24.04`.
- Immediately before merge, every workflow returned for the exact final head had completed with `success`, including Rust/Public Rust/MSRV, artifact/provenance, Hub edge and all five partner-contract gates. Codex review had completed with no material finding and GitHub returned no unresolved review thread.
- State: **qualified and merged for the declared CI-routing scope**. This changes runner placement only; it grants no scientific stage, partner execution, actuation, scheduler, lease or artifact-store authority.

### PR #467 — deterministic TDI-8/9 freeze qualification

- Final head `4cf21995d105dc64d978b2a1f3744bec62dbac83` merged as `9d24ec9ce73370f4948144b1c0ae55cd6c82ce91`.
- The repair was triggered by exact-head failures where self-hosted runners selected Rust 1.85 without `rustfmt`, despite another workflow step having installed Rust 1.97.1. The affected TDI-8.1/TDI-9.1 freeze workflows now pin immutable checkout/toolchain actions, bind checkout to the exact PR head, disable persisted credentials, install Rust 1.97.1 with `rustfmt`/`clippy`, and export `RUSTUP_TOOLCHAIN=1.97.1` for nested bootstrap scripts.
- Every workflow returned by GitHub for the final head completed with `success`, including Rust/Public Rust/MSRV and both repaired configuration-freeze workflows.
- State: **qualified and merged for deterministic freeze-gate execution**. This is CI/reproducibility evidence only; it creates no scientific, novelty, performance, hardware, or final-stage result.

### PR #466 — qualified SciRust source correction

- Final head `729f35693e21ab93c4522987cca7ca70cb62feaa` was based on merged #389.
- All 50 returned pull-request workflows on that exact head completed with `success`, including Rust/Public Rust/MSRV, operational engine, shared research analysis, Hub/partner contracts and the SciRust contract gate; GitHub returned no unresolved review thread.
- PR #466 merged as `c43baf742ad3c85674b83956a01807bf7664771b`.
- State: **qualified and merged for the declared provenance correction**; no scientific result or hardware claim is created by the source repin.

### PR #464 — runner-load scoping

- Current published head: `115581115cfeaaeca77d09555dce25fe87e5311d`. GitHub currently reports it non-mergeable against the newer default branch, so it must be refreshed before promotion.
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

1. Complete #387 exact-head qualification on refreshed head `4fbd8f8a383a2bb5ddb156eb4fb4cdf8851fa9c2`; if every applicable workflow is green and review remains clear, promote it from draft and merge only on that exact head.
2. Refresh #388 from current `main`, resolve its non-mergeable state without dropping retained negative/cancelled evidence, then re-run the complete exact-head benchmark/software matrix.
3. Refresh #464 from current `main`, preserve the frozen-workflow exclusions and nine protected baseline contexts, then qualify its new exact head before promotion.
4. Continue FLAT/NNIS hardware qualification separately; do not convert software/canonical-evidence contracts into hardware, model-quality, or performance conclusions.
