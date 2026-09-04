# TDI-8.1 A3 atomic skip-and-store qualification

Status: bounded software-oracle qualification only; **not an A3 task-adapter policy, not H8-B evidence, and not a final TDI-8.1 configuration freeze**.

## Problem

The merged A3 reference exposes separately atomic operations for:

- an A3/A2 transition through `step_routed`;
- a VSA superposition update through `store_vsa`.

A future task adapter may require one symbolic write event to update both mechanisms. Calling those operations sequentially is not itself a cross-mechanism transaction: A2 could commit before a later VSA allocation or numeric validation failure.

The reference therefore needs a fail-closed composition primitive before any event-level A3 storage policy is selected.

## Qualified primitive

`A3Reference::step_skip_vsa_and_store(...)` represents exactly one composition:

1. the transition does not read/fuse VSA (`Skip` semantics);
2. the external input is validated;
3. the complete next VSA superposition is prepared into one owned width-sized vector;
4. the unchanged A2 step executes with independently supplied associative read/write keys;
5. the prepared VSA vector commits only after A2 success.

The VSA preparation path is shared with ordinary `BoundedVsaWorkspace::bundle`, so standalone bundling and prepared bundling remain bit-exact.

## Failure atomicity

The qualification requires both asymmetric failures to be safe:

- if VSA preparation fails, A2 and VSA persistent state are unchanged;
- if A2 fails after VSA preparation succeeds, the prepared VSA state is discarded and both persistent mechanisms remain unchanged because A2 itself is already atomic.

The post-A2 VSA commit performs no allocation, validation or floating-point arithmetic.

## Temporary-memory accounting

The atomic path deliberately uses VSA `Skip` semantics. Its peak working storage is therefore:

- one VSA-width prepared next-state vector;
- the existing A2 recurrent candidate temporary.

This is the same component-wise peak already declared for a keyed VSA-read A3 step: one VSA-width temporary plus the A2 temporary. The new primitive does not require a second simultaneous VSA-width working vector and therefore does not increase declared A3 dynamic-memory bits.

## Deliberately not selected

This tranche does **not** select:

- which task events should call the atomic primitive;
- whether association or payload events should write VSA;
- whether distractors ever write VSA;
- the VSA payload representation;
- query read/cleanup semantics beyond the already-qualified routed API;
- VSA width, role seed, fusion gain or associative parameters;
- matched experimental budgets, horizons, populations or split ranges;
- deficit, operation-count, interval or intervention semantics;
- an H8-B verdict;
- any TDI-8.2 runner, token, seed range or result surface.

A concrete `SymbolicTaskAdapter` policy remains a separate qualification after this transaction substrate is merged.

## Qualification target

The tranche passes only if prepared and direct bundling are bit-exact, successful atomic composition matches the successful sequential oracle bit-for-bit, both rejection directions preserve the full A3 persistent snapshot, existing routed/legacy A3 semantics remain green, exact memory accounting is unchanged, and all TDI-8 integrity gates remain intact.
