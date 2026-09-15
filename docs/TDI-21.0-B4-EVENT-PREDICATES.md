# TDI-21.0 — Local event-to-predicate boundary for B4

Status: **development-only interface; not frozen; not a learned routing result; no confirmatory execution**.

This increment defines the first versioned observation boundary that can later connect bounded ANF search to a real B4 routing decision. It deliberately exposes less information than the underlying memory implementation.

## Permitted input

For a current `Write` event, the adapter receives:

1. the current marker's two declared bits;
2. a bounded observation of the B3 bucket addressed by the current key.

The bucket observation contains only:

- whether the full route tag is already present;
- whether the bucket has any occupied way;
- whether both ways are occupied;
- the number of occupied ways;
- the fixed way count;
- the deterministic next-victim way.

It does **not** contain the stored payload, the evaluator's expected answer, future events, Validation labels, a history list, attention weights, similarity scores or a scan of all stored facts.

The local observation probes at most the two ways of the addressed B3 bucket. Address derivation, slot probes and occupied-slot tag equality checks are counted. The observation does not increment payload-memory reads or writes and does not alter replacement state.

## Predicate vector v1

`tdi21-write-local-predicates-v1` is a six-bit vector:

| Bit | Predicate |
| ---: | --- |
| 0 | current marker bit 0 |
| 1 | current marker bit 1 |
| 2 | exact route tag already present |
| 3 | addressed bucket contains at least one entry |
| 4 | addressed bucket is full |
| 5 | next deterministic replacement victim is the second way |

No current key bit or payload bit is encoded. Therefore two writes with different keys and payloads but the same marker and identical local bucket metadata produce the same v1 predicate assignment.

This restriction is intentional. The initial B4 search should first test whether simple local memory-pressure/state relations are useful before exposing progressively richer state. Any later predicate-width expansion must be versioned and separately justified.

## Fail-closed validation

The adapter rejects:

- non-Write events;
- markers outside the existing two-bit domain;
- observations that are not from the two-way B3 substrate;
- impossible occupancy/fullness combinations;
- an exact-presence claim for an empty bucket;
- replacement-victim indices outside the addressed bucket.

Synthetic or caller-constructed observations therefore cannot bypass the declared structural invariants.

## Accounting and causality

`BooleanStream::observe_key` is candidate-side observation work, not an accepted sequence event. `StreamCounters::route_observations` records these calls separately. Each call also incurs the existing counted route transform, bounded slot probes and tag equality checks.

Because it receives only the current key and current memory state, the interface is causal. This does not by itself prove that a future task generator or search process is leakage-free; Development/Validation generation and the labels used for search still require their own contract.

## Qualification tests

The software contract checks:

- an empty B3 bucket and exact counter deltas;
- exact-present, one-occupied, full-bucket and post-eviction observations using the existing colliding keys `1`, `5` and `9`;
- repeated observation without changing replacement state or memory footprint;
- exact bit assignments for empty, exact-present and full/next-victim states;
- independence from current key/payload values under otherwise identical inputs;
- rejection of non-Write, invalid-marker and direct-memory observations;
- rejection of impossible synthetic observation shapes;
- zero recorded pairwise token comparisons.

These are interface tests, not evidence that any searched policy improves B3.

## Next experiment

After this boundary is qualified, a later development slice may generate Development-labelled local-routing cases from explicit sequence tasks and fit a sparse ANF policy through the bounded search API. Validation rows must remain unavailable to `fit_sparse_anf` and may be evaluated only after selection.

A useful first falsifiable question is whether an ANF policy over this six-bit local state can reduce a declared class of harmful B3 replacements without worsening a separately declared class of successful recalls under the same memory substrate. The tradeoff, search cost and negative cases must be retained; no policy is presumed superior.
