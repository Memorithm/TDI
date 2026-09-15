# TDI-21.0 — Local event-to-predicate boundary for B4

Status: **development-only interface; not frozen; not a learned routing result; no confirmatory execution**.

This increment defines the first versioned observation boundary that can later connect bounded ANF search to a real B4 routing decision. It deliberately exposes less information than the underlying memory implementation.

## Permitted input

For a current `Write` event, the public adapter receives the B3 stream and the event. It derives the write key internally and observes exactly the bucket addressed by that key before encoding predicates. Callers do not provide a separate `RouteObservation` to the public encoder.

The encoded state contains:

1. the current marker's two declared bits;
2. a bounded observation of the B3 bucket addressed by the current write key.

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

The public adapter rejects before observation:

- non-Write events;
- markers outside the existing two-bit domain;
- streams that are not the two-way B3 substrate.

The internal observation encoder also rejects:

- impossible occupancy/fullness combinations;
- an exact-presence claim for an empty bucket;
- replacement-victim indices outside the addressed bucket;
- a second-way replacement cursor on a non-full bucket, which the current B3 state machine does not produce.

Synthetic observation shapes are exercised only inside module-local contract tests; the production-facing encoder does not accept caller-constructed observations.

## Accounting and causality

`BooleanStream::observe_key` is candidate-side observation work, not an accepted sequence event. `StreamCounters::route_observations` records these calls separately. Each call also incurs the existing counted route transform, bounded slot probes and tag equality checks.

`encode_b4_write_predicates` performs its own observation from the current `Write` key. An unrelated probe performed earlier cannot be substituted into that encoding. Invalid event, marker or memory-mode calls return before observation.

Because the encoder derives its observation only from the current key and current memory state, the interface is causal. This does not by itself prove that a future task generator or search process is leakage-free; Development/Validation generation and the labels used for search still require their own contract.

## Qualification tests

The software contract checks:

- an empty B3 bucket and exact counter deltas;
- exact-present, one-occupied, full-bucket and post-eviction observations using the existing colliding keys `1`, `5` and `9`;
- repeated observation without changing replacement state or memory footprint;
- exact predicate bits derived from the current write's own bucket state;
- independence from current key/payload values under otherwise identical local state;
- an unrelated manual probe cannot be supplied to the encoder and the encoder performs a fresh current-key observation;
- rejection of non-Write, invalid-marker and direct-memory calls before observation;
- rejection in module-local tests of unreachable non-full/second-victim observations;
- zero recorded pairwise token comparisons.

These are interface tests, not evidence that any searched policy improves B3.

## Next experiment

After this boundary is qualified, a later development slice may generate Development-labelled local-routing cases from explicit sequence tasks and fit a sparse ANF policy through the bounded search API. Validation rows must remain unavailable to `fit_sparse_anf` and may be evaluated only after selection.

A useful first falsifiable question is whether an ANF policy over this six-bit local state can reduce a declared class of harmful B3 replacements without worsening a separately declared class of successful recalls under the same memory substrate. The tradeoff, search cost and negative cases must be retained; no policy is presumed superior.
