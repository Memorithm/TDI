# TDI-8.1 — Integrated bounded A3 / ASSR-H reference

Status: **BOUNDED SOFTWARE-ORACLE IMPLEMENTATION — NON-FINAL ONLY**

Parent preregistration blob:
`fe80e7053d89824a77ef6790794f6930d1b424e2`

## Purpose

This reference integrates the deterministic A2 recurrent + bounded associative-memory mechanism with the bounded VSA workspace required by TDI-8.1.

It does not run T1/T2/T3 as scientific evidence, choose a final architecture configuration, access a holdout, or establish H8-B evidence.

## Integrated A3 operation order

The original compatibility surface remains:

`A3Reference::step(input, read_key, write_key)`

and preserves its historical operation order exactly:

1. read/unbind the current VSA workspace with `read_key`;
2. fuse the resulting readout coordinate-wise with the external input as `input[i] + vsa_fusion_gain * readout[i]`, in ascending coordinate order;
3. reject before A2 mutation if the runtime width is wrong or any input/fused coordinate is non-finite;
4. pass the complete fused input to the unchanged A2 `step` implementation using the same `read_key` and supplied `write_key`;
5. A2 performs its already-reviewed recurrent computation, associative lookup-before-write fusion and optional direct-mapped write.

The compatibility method delegates to the explicit routed form:

`step_routed(input, A3VsaReadRoute::Key(read_key), read_key, write_key)`.

A software-oracle regression test requires the two forms to remain bit-exact.

## Independent VSA and A2 read routing

A task-level A3 adapter may need different routing decisions for the VSA workspace and the tagged associative table. The reference therefore also exposes:

`A3Reference::step_routed(input, vsa_read, a2_read_key, a2_write_key)`.

`A3VsaReadRoute` has two explicit states:

- `Skip`: do not read or fuse the VSA workspace; pass the original external input directly to A2;
- `Key(k)`: unbind the VSA workspace with logical key `k`, fuse it into the external input, then execute A2 with its independently supplied read/write keys.

This separation is necessary because an associative-table neutral key and a VSA-neutral read are not equivalent concepts. A tagged direct-mapped associative read can return `Empty` or `CollisionMiss` for a logical key that was never written. A superposed VSA workspace has no tagged miss state: unbinding any key from a non-empty superposition yields a deterministic vector and may therefore inject cross-talk.

The routed API prevents a future adapter from having to create VSA cross-talk merely to preserve an independently qualified A2 read-key policy.

## VSA writes and cross-mechanism atomicity

`A3Reference::store_vsa(key, payload)` remains a standalone atomic VSA primitive. It delegates to candidate-before-commit bundling and does not mutate A2.

A concrete task adapter can also require one logical event to update A2 and the VSA workspace together. Sequentially calling `step_routed(..., Skip, ...)` and then `store_vsa(...)` is not a sufficient fail-closed transaction: the A2 transition could commit before a later VSA allocation or numeric validation failure.

The reference therefore exposes the narrower primitive:

`A3Reference::step_skip_vsa_and_store(input, a2_read_key, a2_write_key, vsa_store_key, vsa_payload)`.

Its ordering is:

1. validate the external A3 input before persistent mutation;
2. compute and validate the complete next VSA superposition into one owned width-sized temporary vector;
3. execute the unchanged atomic A2 step on the original external input, with no VSA read/fusion;
4. only if A2 succeeds, replace the persistent VSA components with the already-prepared vector using an infallible commit.

This creates a cross-mechanism fail-closed boundary for a `Skip` transition plus one VSA store. A VSA preparation failure cannot mutate A2; an A2 failure after VSA preparation cannot commit the prepared workspace.

The primitive deliberately does not decide which symbolic events invoke it or what payload they store. Those remain separate evaluator-policy choices.

## Width and numerical invariants

The VSA workspace width is required to equal the recurrent input width so that coordinate-wise integration uses no hidden projection or unaccounted adapter.

The VSA fusion gain is an explicit finite constructor parameter. This reference does not select or freeze its experimental value.

For every routed transition, input shape and finiteness are checked before persistent mutation. For `Key(k)`, VSA unbind/allocation and all finite fusion checks complete before the mutating A2 step. For `Skip`, the exact original input is handed directly to A2.

For atomic skip-and-store, the next VSA state is fully allocated and numerically validated before A2 mutation, while the VSA commit after A2 success performs no allocation or arithmetic.

## Exact memory accounting

A3 reports:

- the actual A2 recurrent-state bits;
- the actual A2 associative payload bits;
- the actual A2 associative metadata bits;
- the VSA persistent workspace bits;
- integrated temporary working storage;
- all static parameters/constants.

The declared temporary working storage remains the maximum across admitted A3 transitions. A keyed VSA read keeps one VSA-width readout/fused-input vector live while A2 computes its recurrent candidate vector. An atomic skip-and-store instead keeps one prepared VSA-width next-state vector live while A2 computes the same recurrent candidate. Neither path requires two simultaneous VSA-width temporaries, so A3 temporary storage remains the sum of the existing A2 temporary vector and one VSA width-sized temporary vector. The explicit VSA fusion gain is included as 64 static bits.

A plain `Skip` transition may use less temporary memory at runtime, but the architecture-level declaration remains the peak admissible A3 path rather than a result-dependent lower value.

No runtime allocator overhead is converted into an architecture-level claim; accounting follows the same representation-level policy already used for A1/A2 and the standalone VSA oracle.

A software-oracle test provides one synthetic partition in which A1, A2 and A3 have exactly equal declared dynamic-memory totals. Those fixture dimensions only prove that the accounting contract can represent a matched budget; they are not TDI-8.1 experimental choices.

## Software-oracle coverage

The reference tests that:

- constructor width and finite-gain guards fail closed;
- an empty VSA workspace preserves A2 step semantics bit-for-bit;
- legacy `step` and explicit same-key `step_routed` remain bit-exact;
- `Skip` ignores a non-empty VSA workspace and preserves A2 transition semantics bit-for-bit;
- VSA and A2 read keys are independently routable and observable;
- prepared VSA bundling is bit-exact with direct standalone bundling;
- atomic skip-and-store is bit-exact with the successful sequential transition + store path;
- rejected VSA preparation cannot mutate either A2 or the VSA workspace;
- an A2 numeric failure after successful VSA preparation cannot commit the VSA candidate;
- a keyed stored VSA record changes the integrated recurrent input as declared;
- rejected integrated steps cannot mutate A2 or VSA persistent state;
- rejected standalone VSA stores remain atomic;
- A3 accounting includes VSA persistent, temporary and static storage;
- a synthetic A1/A2/A3 partition passes exact `MatchedDynamicBudget` validation;
- A3 snapshots contain both A2 and VSA persistent state;
- reset clears both mechanisms without changing layouts/seeds;
- a downstream integration test consumes the public A3 API.

## Deliberately not frozen here

This reference and transaction qualification do not freeze:

- which symbolic events use `Skip`, `Key`, or atomic skip-and-store;
- which payload is stored in the VSA workspace;
- recurrent input/state dimensions;
- recurrent matrices or bias;
- associative slot count or payload width;
- associative projection seed or fusion gain;
- VSA workspace width or role seed;
- VSA fusion-gain value;
- task encoding or VSA storage policy;
- A1/A2/A3 matched experimental budget;
- T1/T2/T3 horizon strata;
- train/development/validation/final seed ranges;
- sample counts;
- paired interval implementation, seed or replicate count;
- rejection taxonomy for the evaluator;
- any TDI-8.2 runner, token, seed range or result surface.

## Scientific boundary

This is software-reference infrastructure only. It does not claim that A3 is better than A2, that ASSR-H is novel, that the mechanism replaces Transformers or Mamba, or that it has favorable runtime, memory, GPU, energy or language-model performance.
