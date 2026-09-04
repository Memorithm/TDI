# TDI-8.1 A3 read-routing separation

Status: bounded software-oracle qualification only; **not H8-B evidence, not an A3 task-adapter policy, and not a final TDI-8.1 configuration freeze**.

## Problem

The merged A3 reference originally exposed one `read_key` to both mechanisms in an integrated step:

1. the VSA workspace unbound that key and fused the resulting vector into the recurrent input;
2. the embedded A2 associative memory looked up the same key.

That coupled API is valid as a deterministic software oracle, but it is too restrictive for a leakage-safe task adapter. The qualified A2 adapter uses an instance-scoped logical key that is never written for non-query associative reads. This is safe for a tagged direct-mapped associative table because the neutral key can only produce `Empty` or `CollisionMiss`.

A VSA superposition has no analogous tagged miss state. Unbinding any key from a non-empty workspace returns a deterministic vector, including a key that was never stored. Therefore an A2-neutral read key is **not** a neutral VSA read key. Reusing it would inject cross-talk into nominal writes or distractors merely because A2 requires a read key.

## Qualified API separation

`A3VsaReadRoute` makes the VSA routing decision explicit:

- `Skip`: do not unbind or fuse the VSA workspace for this transition;
- `Key(k)`: unbind the workspace with logical key `k` and fuse the result before A2 execution.

`A3Reference::step_routed(input, vsa_read, a2_read_key, a2_write_key)` independently supplies the associative A2 read/write routing.

The existing `A3Reference::step(input, read_key, write_key)` remains a compatibility wrapper and is required to be bit-exact with:

`step_routed(input, A3VsaReadRoute::Key(read_key), read_key, write_key)`.

This preserves all previously merged software-oracle semantics while exposing the additional degree of routing freedom required for a future explicit A3 task policy.

## Fail-closed and accounting invariants

The routed step preserves the existing ordering and validation rules:

- input width and finiteness are validated before any persistent mutation;
- `Skip` passes the original external input directly to the unchanged A2 reference;
- `Key(k)` completes VSA unbind, allocation and finite fusion checks before the mutating A2 step;
- VSA retrieval remains read-only;
- VSA storage remains a separate atomic `store_vsa` operation;
- the declared A3 temporary-memory accounting remains the maximum admissible routed path, including one VSA-width temporary vector plus A2 temporary storage.

The qualification tests require a non-empty workspace under `Skip` to leave A2 transition semantics bit-exact, and require the VSA key and A2 read key to be independently observable in one routed transition.

## Deliberately not selected

This tranche does **not** select:

- which symbolic events should use `Skip` or `Key`;
- which values, frames, recurrent states or other payloads should be stored in VSA;
- whether association, payload or distractor events should call `store_vsa`;
- a VSA cleanup or similarity policy;
- final VSA width, role seed or fusion gain;
- final A3 recurrent or associative dimensions;
- matched A1/A2/A3 dynamic-memory budgets;
- horizon strata, populations, non-final/final seed ranges or sample counts;
- late-retrieval deficit, operation-count or intervention semantics;
- any H8-B verdict, TDI-8.2 runner, token, seed range or result surface.

A concrete `SymbolicTaskAdapter` policy for A3 must be reviewed and qualified separately after this routing substrate is merged.

## Qualification target

The tranche passes only if the legacy A3 API remains bit-exact, `Skip` proves VSA-independent A2 execution even with a non-empty workspace, `Key` retains deterministic VSA fusion, VSA and A2 read keys can differ without ambiguity, existing A3 atomicity/accounting tests remain green, and the standard TDI-8 bootstrap/foundation integrity boundary remains intact.
