# TDI-21.0 — Relational binding and bounded composition prototype

Status: **development-only task harness; not frozen; not confirmatory; no final/holdout material**.

This prototype is the first TDI-21 slice that deliberately leaves admission/replacement policy qualification and returns to a broader function normally supplied by attention: binding a subject to an object under a relation, retrieving that object later, and composing several relations.

## Architectural rule

The prototype does not use Q/K/V projections, token-token similarity, cosine distance, softmax, all-pairs Hamming scores, dense `N x N` score matrices, attention fallback or a scan over historical events.

A relation/subject pair is encoded as one exact bounded symbolic identifier. The existing causal `BooleanStream` provides bounded direct/2-way associative storage and full-tag collision checks. A relation path performs one exact-address lookup per hop.

This is intentionally not yet learned semantic addressing. Passing these tasks only establishes the mechanics and the evaluator/task contract required for later B4/B5 learning experiments.

## Binding

For bounded entity and relation identifiers:

`Bind(relation, subject, object)` stores an exact symbolic fact.

`Recall(relation, subject)` returns either `Hit(object)` or explicit `Miss`.

A stored object equal to zero remains distinct from `Miss`.

## Composition

`compose_path([r1, r2, ...], subject)` repeatedly feeds the object returned at one hop into the subject position of the next relation.

The path length is explicitly bounded to four hops. Empty or oversized paths, out-of-range entities/relations, or insufficient event budget fail before the first path lookup. Missing intermediate facts terminate as `Miss`.

Therefore work for a valid path is proportional to the declared hop count, not the total amount of historical context. This is a software-semantic property, not a wall-clock/energy claim.

## Development checks

The private prototype tests:

- exact one-hop binding and retrieval;
- two-hop and three-hop composition;
- one memory read per successful hop;
- explicit missing-intermediate behavior;
- equivariance under a consistent renaming of entity and relation identifiers;
- irrelevant relation distractors when capacity is sufficient;
- visible bounded-memory eviction rather than false hits;
- zero recorded token-pair comparisons;
- fail-closed relation/entity/path bounds and atomic rejection when event budget is insufficient.

## What this does not establish

The exact identifier packing makes this task easier than natural-language semantic binding. It is a reference harness, not evidence that B4 has learned a relation, generalized a Boolean encoder, or matched attention quality.

The next scientific increment must place learned/synthesized Boolean routing in front of this harness and separate Development from renamed-identifier Validation. A successful result would require generalization to new entity/relation identifiers and later to relation structures not explicitly enumerated during selection.

Only after those controls should TDI-21 compare this mechanism with B0/B1 attention references on the same relational task and matched resource envelopes.
