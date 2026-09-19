# TDI-21.0 — Relational binding and bounded composition prototype

Status: **development-only task/substitution harness; not frozen; not confirmatory; no final/holdout material**.

This prototype is the first TDI-21 slice that deliberately leaves admission/replacement policy qualification and returns to a broader function normally supplied by attention: binding a subject to an object under a relation, retrieving that object later, and composing several relations.

## Architectural rule

The prototype does not use Q/K/V projections, token-token similarity, cosine distance, softmax, all-pairs Hamming scores, dense `N x N` score matrices, attention fallback or a scan over historical events.

A relation/subject pair becomes one bounded symbolic identifier. The existing causal `BooleanStream` provides bounded two-way associative storage and full-tag collision checks. A relation path performs one addressed lookup per hop.

This is intentionally not yet learned semantic addressing. Passing these tasks establishes mechanics, accounting and evaluator/task contracts required for later B4/B5 learning experiments.

## Two addressing controls

`ExactPacked` uses the declared bit layout directly:

`address = (relation << entity_bits) | subject`.

`AnfIdentity` synthesizes one Zhegalkin/ANF program for every output address bit from the complete truth table over the declared `(relation, subject)` input. For the v1 relational family, four relation bits plus four entity bits give eight ANF input variables, within the existing exact-synthesis ceiling.

`AnfIdentity` is required to reproduce `ExactPacked` exactly. It is a substitution control, not learning. Its ANF monomial work is accounted separately and is committed only when the corresponding stream operation succeeds.

The private differential fixture checks all 256 possible 4+4-bit input assignments and then repeats the comparison on both relational Development and renamed-identifier Validation episodes. Exact and ANF modes must agree on stream state and answers while only the ANF mode reports nonzero ANF work.

## Binding

For bounded entity and relation identifiers:

`Bind(relation, subject, object)` stores one exact symbolic fact.

`Recall(relation, subject)` returns either `Hit(object)` or explicit `Miss`.

A stored object equal to zero remains distinct from `Miss`.

One semantic address derivation is counted for each bind or recall in addition to the existing BooleanStream routing/memory counters. This is not a claim about CPU instructions or cycles.

## Composition

`compose_path([r1, r2, ...], subject)` repeatedly feeds the object returned at one hop into the subject position of the next relation.

The path length is explicitly bounded to four hops. Empty or oversized paths, out-of-range entities/relations, or insufficient event budget fail before the first path lookup. Missing intermediate facts terminate as `Miss`.

Therefore work for a valid path is proportional to the declared hop count, not the total historical context. This is a software-semantic property, not a wall-clock/energy claim.

## Development / renamed Validation family

The v1 task family contains four topologies in each split: one-hop recall, two-hop composition, three-hop composition, and two-hop composition with irrelevant relational distractors.

Development and Validation use disjoint four-bit entity namespaces and disjoint four-bit relation namespaces while preserving topology. The evaluator derives expected answers independently with a dictionary keyed by `(relation, subject)`; expected answers are not hand-authored labels and are not candidate inputs.

Split-overlap checking compares candidate-visible facts and queries only. Split markers, case IDs and evaluator answers are deliberately excluded from the overlap definition.

The chosen nominal identifiers avoid capacity collisions under the versioned v1 reference configuration so that the nominal composition test is not accidentally a memory-eviction test. Separate low-capacity fixtures retain explicit eviction failures.

## Development checks

The private prototype covers:

- exact one-hop binding and retrieval;
- two-hop and three-hop composition;
- one memory read per successful hop;
- explicit missing-intermediate behavior;
- equivariance under consistent identifier renaming;
- disjoint Development/Validation identifier namespaces;
- irrelevant relation distractors when capacity is sufficient;
- visible bounded-memory eviction rather than false hits;
- exact relational-address derivation accounting;
- zero recorded token-pair comparisons;
- fail-closed relation/entity/path bounds and atomic rejection when event budget is insufficient;
- exact-vs-ANF differential behavior over all 256 v1 address inputs and both task splits.

## What this does not establish

The symbolic input identifiers make this task easier than natural-language semantic binding. `AnfIdentity` is deliberately equivalent to manual packing. Neither is evidence that B4 has learned a relation, learned an encoder, generalized semantics, or matched attention quality.

The next scientific increment must replace the fixed identity address function with a bounded Development-only learned/synthesized Boolean program, then evaluate it unchanged on renamed Validation identifiers. Any richer feature representation must remain causal and must not expose evaluator answers or future recalls.

A stronger result will also require relation structures not explicitly enumerated during selection and matched B0/B1 attention references on the same relational task under explicit memory/search/inference budgets.
