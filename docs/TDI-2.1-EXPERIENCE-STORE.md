# TDI-2.1 — Experience-store contract

Date frozen: 2026-09-16

## Purpose

The experience store is the only persistent information that distinguishes the main intuition arm from a matched no-experience arm. Its construction and use must therefore be auditable.

## Candidate representations

Before holdout opening, Development experiments may compare:

- normalized prototypes / centroids;
- fixed-size associative memories;
- kernelized prototype embeddings;
- hierarchical pattern banks;
- bounded sufficient statistics.

A candidate may not silently fall back to retaining the full protected dataset.

## Capacity controls

Every arm must report:

- number of stored patterns;
- dimension of each representation;
- numeric type;
- total logical storage bytes;
- whether raw Development examples are retained;
- insertion, replacement and eviction policy.

Comparisons that claim an advantage from a retrieval rule must either match capacity or report the capacity difference explicitly.

## Retrieval contract

A retrieval operation receives only current `x`, permitted context `g` and the frozen experience store. It returns a deterministic or seed-bound score vector plus retrieval diagnostics. It may not consult target labels, future states or holdout aggregate statistics.

## Mutation policy

The confirmatory no-consolidation experiment freezes the store before holdout scoring. Online mutation is permitted only in a separate consolidation experiment with a preregistered feedback stream and update rule.

## Failure modes to measure

- prototype collision;
- metastable/ambiguous retrieval;
- domination by high-frequency motifs;
- stale or contradictory experience;
- memory-capacity saturation;
- sensitivity to representation scaling;
- retrieval instability under small perturbations.

## Non-claims

The store is not called episodic, semantic or human-like memory unless an experiment specifically operationalizes such a property. Associative retrieval is treated as a computational mechanism, not a biological equivalence claim.
