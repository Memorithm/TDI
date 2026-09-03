# TDI-8.1 — Deterministic A0 full-history reference

Status: **BOUNDED SOFTWARE-ORACLE IMPLEMENTATION — NON-FINAL ONLY**

Parent preregistration blob:
`fe80e7053d89824a77ef6790794f6930d1b424e2`

## Purpose

A0 is the competent attention-like full-history control frozen by TDI-8.0. It
exists to establish task solvability and contextualize bounded-memory failures.
It is not a hidden matched-budget baseline and is not part of the primary
A2-vs-A1 or A3-vs-A2 budget equality constraint.

This tranche implements A0 only as a deterministic software oracle. It does not
run T1/T2/T3, select experimental dimensions or create any TDI-8.2 surface.

## Complete-history representation

The reference stores every accessible item as one fixed-width binary64 key and
one fixed-width binary64 value. Items are retained in insertion order until an
explicit reset/clear. There is no eviction, truncation, compression, hashing or
projection.

The key and value widths are explicit constructor inputs. The implementation
tranche does not choose or freeze their experimental values.

## Content-based read

A0 uses deterministic hard content attention:

1. validate the query width and every query coordinate;
2. scan all accessible history items in insertion order;
3. compute squared L2 distance between query and key in ascending coordinate
   order using binary64;
4. reject if any subtraction, square or accumulated distance is non-finite;
5. select the smallest finite distance;
6. exact distance ties select the most recently appended item;
7. return the selected value and one read coefficient per history item, with a
   one-hot coefficient vector for the selected item.

This is an explicit competent content-addressed full-history oracle. It does not
claim to reproduce Transformer softmax attention and introduces no temperature,
learned projection or hidden normalization parameter.

The exposed coefficient vector preserves the frozen TDI-8.0 possibility of a
later A0 intervention at a history-item/read-coefficient site. The intervention
policy itself remains future evaluator work.

## Atomicity and fail-closed behavior

Appending validates key/value widths and finite coordinates before mutation.
Both history arrays reserve the required capacity before either payload is
extended, so an invalid input cannot leave a partially appended logical item.

Reads are non-mutating. Empty-history reads, invalid query widths, non-finite
query values and non-finite scoring intermediates fail without changing the
persistent history.

Host vector capacities are validated and reservations use fallible allocation.
Rust allocator/container headers are implementation overhead and are not turned
into architecture-level memory claims.

## Exact memory accounting

A0 reports separately:

- cumulative full-history payload bits for all stored keys and values;
- 64 bits of explicit logical history-count metadata;
- peak temporary working storage for the returned one-hot coefficient vector,
  selected value and selected-index/distance scalars;
- 128 static bits for the two declared `u64` layout widths.

It reports no recurrent state, associative table or VSA workspace.

Unlike A1/A2/A3, A0 cumulative history is allowed to grow with sequence length.
It is reported by `MemoryAccounting::cumulative_history` and is not folded into
`MatchedDynamicBudget`.

## Software-oracle coverage

The tranche tests that:

- zero key/value widths are rejected;
- every appended item remains accessible in insertion order;
- rejected appends cannot partially mutate logical history;
- full-history reads select the nearest content item;
- exact score ties use the declared most-recent rule;
- read coefficients are deterministic one-hot values;
- empty/non-finite reads are non-mutating failures;
- overflowing binary64 distance intermediates fail closed;
- cumulative-history and peak read temporary accounting grow exactly as
  declared;
- snapshots contain the complete persistent history;
- clear/reset preserves the declared layout;
- a downstream integration test consumes the public A0 API.

## Deliberately not frozen here

This tranche does not freeze:

- key or value width;
- task-specific encodings;
- T1/T2/T3 generators;
- short/medium/long horizon values;
- sample counts or seed ranges;
- bounded-arm matched memory budget;
- operation-count policy;
- intervention site selection;
- deficit/recovery adapters;
- paired interval implementation or replicate count;
- rejection taxonomy for the final evaluator;
- any TDI-8.2 runner, token, seed range or result surface.

## Scientific boundary

This implementation is infrastructure only. It does not establish H8-A or
H8-B, does not imply that A0 is memory efficient, and makes no Transformer,
Mamba, GPU, runtime, bandwidth, energy or language-model performance claim.