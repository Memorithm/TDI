# TDI industrialization qualification reconciliation — 2026-09-16 R2

This document is a current-state overlay for `industrialization-qualification-audit-2026-09-16.md`. It does not rewrite historical evidence. It records only GitHub state re-verified after PR #332 and applies the same rule throughout: **merge presence, a local pass, or a subset of green workflows is not exact-head qualification**.

No statement below authorizes a protected/final scientific stage, a hardware claim, runtime actuation, a scientific verdict, or a performance claim.

## Default-branch checkpoint

- TDI `main`: `d8785a04d112e732bf92bd115d23640e7bfb60e7` (merge of PR #332).
- The branch contains the older dated audit. Where that audit still calls #296 or #331 open candidates, this R2 overlay supersedes only that stale repository-state fact; its qualification cautions remain in force.

## Reconciled merged work

### PR #296 — shared research analysis

- PR #296 is merged on TDI; merge SHA `cc07d5c55bd2b5aca78b6b8074da21e508a1c735`.
- Its consumer was pinned to SciRust candidate revision `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0` with an explicit requirement to repin after the reviewed, fully exact-head-green SciRust #1452 merge.
- SciRust #1452 is still open. Its current candidate head is `7981120d9b703a5d82796608e7565e3763d8c611`; at this reconciliation the Dream Real Cache Policy run on that exact head is failed and multiple other workflows are queued.
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

- TDI head: `135fc20520fed390d23e78f6374a6a0207a26a4d`.
- It pins Forge #39 candidate `0d48a91a5eed6a9ae09a5eacd7c4f18df8bdd333`.
- Forge #39 is open; both `Scientific ask tell process qualification` and `CI` are queued on that candidate revision at this reconciliation.
- State: **blocked on upstream final Forge qualification**. TDI #385 must also satisfy its own full exact-head gate set afterward.

### PR #389 — shared evidence storage

- Head: `13d46681f8a9f629680eb9c9b77180bf5db8484b`.
- Base is `industrialize/scientific-search`, not `main`; it is intentionally stacked on #385.
- State: **stacked candidate**. Do not retarget or promote it before #385 and its upstream Forge dependency are qualified and integrated.

### PR #387 — bounded sensitivity and ablation

- Head: `54dff997ede3b948b420ab0c021af2933334ad63`.
- It still identifies SciRust #1452 as the numerical dependency that must be reviewed, fully exact-head green and merged before final repin/promotion.
- State: **blocked on final SciRust #1452 provenance and its own exact-head gates**.

### PR #388 — measured engine benchmarks

- Head: `fdd0f57dc3f5fe885601229b9c2d570e54b35635`.
- The dedicated benchmark workflow checks out the exact PR head and pins scirust-hub `ccdcb99a4573dbefb944af0df713101b100b5f78`.
- At this reconciliation the returned workflow set is not green: many runs are queued and several runs, including `TDI measured engine benchmarks`, are currently cancelled.
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

1. Qualify Forge #39, then TDI #385, then the stacked #389 path without bypassing Hub/TDI ownership boundaries.
2. Qualify and merge SciRust #1452; open a corrective TDI pin PR for merged #296 and repin #387 before either path is considered final.
3. Re-run/complete #388 measured-engine gates on its exact final head; preserve cancelled or negative executions as evidence rather than silently replacing them with claims.
4. Keep FLAT/NNIS hardware qualification separate and blocked until the required physical evidence exists.
