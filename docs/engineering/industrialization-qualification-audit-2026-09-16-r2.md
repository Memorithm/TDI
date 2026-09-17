# TDI industrialization qualification reconciliation — 2026-09-16 R2

This document is a current-state overlay for `industrialization-qualification-audit-2026-09-16.md`. It does not rewrite historical evidence. It records only GitHub state re-verified after PR #332 and applies the same rule throughout: **merge presence, a local pass, or a subset of green workflows is not exact-head qualification**.

No statement below authorizes a protected/final scientific stage, a hardware claim, runtime actuation, a scientific verdict, or a performance claim.

## Default-branch checkpoint

- TDI `main`: `71f49fa6335aae51845e92b36a71ca5e7beec6fa` (current default-branch checkpoint through merged PR #462).
- The branch contains the older dated audit. Where that audit still calls #296 or #331 open candidates, this R2 overlay supersedes only that stale repository-state fact; its qualification cautions remain in force.

## Reconciled merged work

### PR #296 — shared research analysis

- PR #296 is merged on TDI; merge SHA `cc07d5c55bd2b5aca78b6b8074da21e508a1c735`.
- Its consumer was pinned to SciRust candidate revision `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0` with an explicit requirement to repin after the reviewed, fully exact-head-green SciRust #1452 merge.
- SciRust #1452 is still open. Its current candidate head is `cbfb6b43ea7f9e0fd303008ef5e701620fe905a2`. The preceding exact head retained a physical Thor/cuBLASLt failure; the new head pins model/tokenizer/dataset/source provenance and preserves failure artifacts, so its newly started exact-head workflow set must complete before any downstream promotion.
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

- TDI head: `c270ac658384aa2ea732418c5f2c9927b2480754`.
- Forge #39 is now merged as `28067ab0aa1d52a2260d9bb2bf35a346547a292a`, and #385 is repinned to that merge revision.
- The refreshed TDI branch is mergeable and has no unresolved material review thread, but its current exact-head workflow set remains incomplete/queued.
- State: **candidate, upstream Forge pin resolved; blocked on TDI exact-head qualification**. No partner execution, holdout or scientific verdict authority is inferred from the merged Forge dependency.

### PR #389 — shared evidence storage

- Head: `13d46681f8a9f629680eb9c9b77180bf5db8484b`.
- Base is `industrialize/scientific-search`, not `main`; it is intentionally stacked on #385.
- State: **stacked candidate**. Do not retarget or promote it before #385 and its upstream Forge dependency are qualified and integrated.

### PR #387 — bounded sensitivity and ablation

- Head: `54dff997ede3b948b420ab0c021af2933334ad63`.
- It still identifies SciRust #1452 as the numerical dependency that must be reviewed, fully exact-head green and merged before final repin/promotion.
- State: **blocked on final SciRust #1452 provenance and its own exact-head gates**.

### PR #388 — measured engine benchmarks

- Current head: `9e8ba6bb5e2cad4d6fa1eca6b33b3fb0a371f6da`; the PR remains draft and currently non-mergeable against the advanced default branch.
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
