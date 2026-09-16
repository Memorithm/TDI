# TDI-23.2 Status

Status: **CANDIDATE PREPARATION / DRAFT PR / BLOCKED ON TDI-23.1 FREEZE / NOT FROZEN / NO REWRITE SEARCH / NO CONFIRMATORY EXECUTION**

## Current slice

The first candidate TDI-23.2 slice is the bounded local rewrite calculus in `tdi-ai/src/tdi23_rewrite.rs`.

It introduces only:

- `LeftIdentity`;
- `RightIdentity`;
- `DaggerIdentity`;
- `DoubleDagger`;
- explicit exactness classes;
- independent rooted validation before rule matching;
- exact semantic verification through the TDI-23.1 equivalence oracle;
- fail-closed behavior at nonlinear/algebraic boundaries.

This code is preparatory only. The canonical TDI-23 programme still marks TDI-23.2 as blocked on a stable TDI-23.1 grammar. Qualification of this candidate does not itself freeze TDI-23.1 or authorize promotion of TDI-23.2.

## Qualification state

This branch must remain unmerged until all applicable CI jobs on the exact PR head SHA have actually executed and concluded `success`, and until the TDI-23.1 grammar/foundation has an explicit qualified freeze compatible with this rule surface.

The following states are explicitly non-green and therefore non-mergeable for this work:

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

A local or post-merge validation does not substitute for required pull-request CI.

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

First qualify and explicitly freeze the required TDI-23.1 grammar/equivalence foundation. Only then may this candidate be considered for promotion into active TDI-23.2 work. After promotion, any additional rule family must carry its own exactness contract and negative fixtures. Reassociation remains excluded until a numerical-order contract exists.
