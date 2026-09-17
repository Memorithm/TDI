# TDI industrialization qualification reconciliation — 2026-09-16 R2 (refreshed 2026-09-17)

This document is a current-state overlay for `industrialization-qualification-audit-2026-09-16.md`. It does not rewrite historical evidence. It records only GitHub state re-verified after PR #332 and applies the same rule throughout: **merge presence, a local pass, or a subset of green workflows is not exact-head qualification**.

No statement below authorizes a protected/final scientific stage, a hardware claim, runtime actuation, a scientific verdict, or a performance claim.

## Default-branch checkpoint

- TDI `main`: `13c46500b6d079a515772286bfad3358fc116e6d` (current default-branch checkpoint through merged PR #463).
- PR #463 repaired exact-head checkout in the protected core Rust/Public/MSRV gates plus the Rust 1.89 real-library Clippy gate. Its final head `63b4e7546a2100ad5cf66f4643ac4264e9de3970` had 57 returned check runs complete with no queued/in-progress/failure/cancelled/skipped result and its material review thread resolved before merge. This records CI provenance only; it changes no scientific stage or result.
- The branch contains the older dated audit. Where that audit still calls #296 or #331 open candidates, this R2 overlay supersedes only that stale repository-state fact; its qualification cautions remain in force.

## Reconciled merged work

### PR #296 — shared research analysis

- PR #296 is merged on TDI; merge SHA `cc07d5c55bd2b5aca78b6b8074da21e508a1c735`.
- Its consumer was pinned to SciRust candidate revision `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0` with an explicit requirement to repin after the reviewed, fully exact-head-green SciRust #1452 merge.
- SciRust #1452 is still open. Its current candidate head is `96bad0998db5a3361a013fb4e1b29470f8ddaf35`. Its retained physical Dream/Thor gate failed in the first attention `q_proj` with `CUBLAS_STATUS_NOT_INITIALIZED`; no benchmark JSON was produced. Reduced same-host diagnostics did not reproduce the failure, so root cause remains unestablished. Every applicable exact-head workflow must complete green before any downstream promotion.
- Therefore the TDI↔SciRust analysis path is **merged but not finally qualified**. Required closure: qualify and merge SciRust #1452, repin TDI to its resulting merge SHA in a new PR, then run all applicable TDI gates on that new exact head.

### PR #331 — real library replay adapters

- PR #331 is merged on TDI; merge SHA `c5f6dcad6094f945af3d4df211490a76a24f8416`.
- This reconciliation does not possess a new complete exact-head workflow set proving the final #331 head after the fact.
- State: **merged; no retroactive qualification inferred from merge presence**.

### ElasticXxx boundary

- PR #279 final head `29d9f5555483b59c591ccb7d4871e84ac02d677c` merged as `6e2a15a5711f8dc2eb7e77886bbed247aa425e6b`; the original non-actuating boundary had complete exact-head software evidence recorded by the earlier audit.
- PR #288 later hardened binding of `OperatorConfig` to admitted root evidence and is merged as `6d0dadd5f6c481123d75211c43a5da0ae2f1d7fc`.
- This R2 overlay does not promote the later hardening to a new qualification merely because it is present on `main`. Execution, actuation, holdout, stage and verdict authority remain absent.

## Active industrialization frontier

### PR #385 — durable Forge search

- TDI head: `b20cbaf053fa446dba9391e521edfdc167b881e5`.
- Forge #39 is now merged as `28067ab0aa1d52a2260d9bb2bf35a346547a292a`, and #385 is repinned to that merge revision.
- After #463 merged, the branch was non-destructively refreshed onto `13c46500b6d079a515772286bfad3358fc116e6d`. The only merge conflicts were the already-equivalent warning fixes owned by #463; the new main versions were retained. The refreshed PR is mergeable and its new exact-head workflow set is authoritative.
- State: **candidate, upstream Forge pin resolved; blocked on TDI exact-head qualification**. No partner execution, holdout or scientific verdict authority is inferred from the merged Forge dependency.
- A `TDI real library adapters` exact-head run on a preceding #385 head failed because the pinned Rust 1.89 toolchain lacked `cargo-clippy`; that repair is now also present on `main` via #463. Local refresh checks on `b20cbaf...` passed formatting, YAML parsing, `git diff --check` and 15 engine/observability unit tests, but the new GitHub exact-head run remains authoritative; the local pass is not promoted to qualification.

### PR #389 — shared evidence storage

- Head: `7f9ac53573f123b9a1b72aa00baed4d4158d26e0` at this reconciliation checkpoint; parent #385 has since advanced to `b20cbaf053fa446dba9391e521edfdc167b881e5`, so #389 is now explicitly stale relative to its parent and requires another non-destructive refresh before qualification.
- Base is `industrialize/scientific-search`, not `main`; it is intentionally stacked on #385.
- State: **stacked candidate**. Do not retarget or promote it before #385 and its upstream Forge dependency are qualified and integrated.

### PR #387 — bounded sensitivity and ablation

- Head: `d5e5115add1689a59b0514fe999a636b5c2e404a`.
- It still identifies SciRust #1452 as the numerical dependency that must be reviewed, fully exact-head green and merged before final repin/promotion.
- State: **blocked on final SciRust #1452 provenance and its own exact-head gates**.

### PR #388 — measured engine benchmarks

- Current head: `3d31ae65546ada0ae8265ab624046eadd8c789a5`; the PR remains draft and currently non-mergeable against the advanced default branch.
- The dedicated benchmark workflow checks out the exact PR head and pins scirust-hub `ccdcb99a4573dbefb944af0df713101b100b5f78`.
- Earlier cancelled/partial runs are retained as negative infrastructure evidence. The current head must receive a fresh complete exact-head qualification after synchronization before promotion.
- State: **candidate, not qualified**. No cancelled, queued or partial result is converted into a benchmark or performance claim.

## Evidence discipline retained

- Hub owns generic orchestration, registry resolution, scheduling, leases, transport, artifact storage and authoritative publication. TDI must not duplicate those services.
- TDI owns scientific identities, admissibility, provenance, stage/verdict semantics and accepted evidence.
- Merge presence is not a qualification result.
- `queued`, `pending`, `in_progress`, `cancelled`, `failure` and unavailable hardware are not green.
- A software contract or interchange envelope is not partner execution evidence.
- A benchmark workflow that does not complete successfully on the exact candidate head yields no promotable performance result.
- Protected/final holdouts remain governed by `AGENTS.md` and the ecosystem roadmap/series overlays; this reconciliation opens none.

## Next closure order

1. Complete TDI #385 exact-head qualification on the final Forge #39 merge pin, then resynchronize/qualify the stacked #389 path without bypassing Hub/TDI ownership boundaries.
2. Qualify and merge SciRust #1452; open a corrective TDI pin PR for merged #296 and repin #387 before either path is considered final.
3. Resynchronize #388 with the current default branch, then re-run/complete its measured-engine gates on the exact final head; preserve cancelled or negative executions as evidence rather than silently replacing them with claims.
4. Keep FLAT/NNIS hardware qualification separate and blocked until the required physical evidence exists.
