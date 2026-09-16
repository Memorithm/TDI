# TDI-23.2 Status

Status: **ACTIVE DEVELOPMENT / DRAFT PR / NOT FROZEN / NO REWRITE SEARCH / NO CONFIRMATORY EXECUTION**

## Current slice

The first TDI-23.2 slice is the bounded local rewrite calculus in `tdi-ai/src/tdi23_rewrite.rs`.

It introduces only:

- `LeftIdentity`;
- `RightIdentity`;
- `DaggerIdentity`;
- `DoubleDagger`;
- explicit exactness classes;
- independent rooted validation before rule matching;
- exact semantic verification through the TDI-23.1 equivalence oracle;
- fail-closed behavior at nonlinear/algebraic boundaries.

## Qualification state

This branch must remain unmerged until all applicable CI jobs on the exact PR head SHA have actually executed and concluded `success`.

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

- TDI-23.2 is not frozen.
- No recursive rewrite application is authorized.
- No rewrite search or ranking is authorized.
- No approximate-equivalence policy is authorized.
- No tensor/direct-sum algebraic rewrites are authorized.
- No cross-boundary rewrite semantics are authorized.
- No FLAT-ATTENTION integration is authorized.
- No confirmatory or final execution is authorized.

## Next gate

After this local calculus is qualified and merged, the next TDI-23.2 increment may add one explicitly bounded rule family with its own exactness contract and negative fixtures. Reassociation remains excluded until a numerical-order contract exists.
