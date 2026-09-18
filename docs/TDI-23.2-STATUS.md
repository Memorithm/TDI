# TDI-23.2 Status

Status: **BOUNDED PREPARATORY CALCULUS MERGED / BLOCKED ON TDI-23.1 FREEZE / NOT PROMOTED OR FROZEN / NO REWRITE SEARCH / NO CONFIRMATORY EXECUTION**

## Current slice

The first bounded TDI-23.2 preparatory slice is the local rewrite calculus in `tdi-ai/src/tdi23_rewrite.rs`. It qualified on exact PR head `70b43d026235b2cec022599babcb7a75726e5571` and merged via PR #384 as `0c55d21681096f045a4d970ee961cf91377bfe29` on 2026-09-18.

It introduces only:

- `LeftIdentity`;
- `RightIdentity`;
- `DaggerIdentity`;
- `DoubleDagger`;
- explicit exactness classes;
- independent rooted validation before rule matching;
- exact semantic verification through the TDI-23.1 equivalence oracle;
- fail-closed behavior at nonlinear/algebraic boundaries.

This code is preparatory only. The canonical TDI-23 programme still keeps scientific promotion of TDI-23.2 blocked on a stable TDI-23.1 grammar. Qualification and merge of this bounded slice do not freeze TDI-23.1 or authorize promotion of TDI-23.2.

## Qualification state

PR #384 is merged after exact-head qualification. That software merge does **not** satisfy the separate scientific prerequisite: the TDI-23.1 grammar/foundation still requires an explicit qualified freeze compatible with this rule surface before TDI-23.2 can be promoted as an active scientific stage.

For any later TDI-23.2 increment, the following states remain explicitly non-green and therefore non-mergeable:

- no check run created;
- `queued`;
- `pending`;
- `in_progress`;
- `cancelled`;
- `skipped` when the gate is applicable;
- `failure`;
- `timed_out`;
- `action_required`;
- any CI result attached only to an older head SHA.

A local or post-merge validation does not substitute for required pull-request CI on a future increment, and the successful merge of PR #384 does not substitute for the missing TDI-23.1 scientific freeze.

## Scientific authorization state

- TDI-23.1 remains the active stage until separately frozen.
- TDI-23.2 is not promoted or frozen by this branch.
- No recursive rewrite application is authorized.
- No rewrite search or ranking is authorized.
- No approximate-equivalence policy is authorized.
- No tensor/direct-sum algebraic rewrites are authorized.
- No cross-boundary rewrite semantics are authorized.
- No FLAT-ATTENTION integration is authorized.
- No confirmatory or final execution is authorized.

## Next gate

First qualify and explicitly freeze the required TDI-23.1 grammar/equivalence foundation. Only then may the merged preparatory calculus be considered for promotion into active TDI-23.2 scientific work. After promotion, any additional rule family must carry its own exactness contract and negative fixtures. Reassociation remains excluded until a numerical-order contract exists.
