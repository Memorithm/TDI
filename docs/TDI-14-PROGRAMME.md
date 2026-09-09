# TDI-14.x — Common Predictive Geometry Across Internal States

Status: **active Stage 0 bootstrap; not frozen; no confirmatory execution authorised**.

## Conjecture

KV state, recurrent state and bounded associative memory may be different coordinate systems over a common predictive geometry. On task-relevant trajectory subsets, mappings between representations may preserve predictive neighbourhoods substantially better than raw-coordinate similarity would suggest.

## Primary null

Cross-representation predictive neighbourhood agreement is fully explained by shared labels, time index or task difficulty and does not survive held-out trajectories or intervention.

## Stage map

- **TDI-14.0** — freeze matched tasks, state extractors, predictive targets and geometry metrics.
- **TDI-14.1** — deterministic bounded-memory reference systems with exact state access.
- **TDI-14.2** — learn or construct cross-state maps using development data only.
- **TDI-14.3** — evaluate neighbourhood, ordering and intervention-response preservation on held-out trajectories.
- **TDI-14.4** — architecture transfer to KVLab/ASSR-compatible state adapters if justified.

## Required controls

Time-index matching, target-label-only embeddings, random orthogonal maps, equal-capacity projections, shuffled trajectories and task-family holdouts.

## Decision principle

Support requires held-out preservation of frozen predictive relations across at least two distinct state families beyond competent nuisance controls.

## Ecosystem boundary

TDI-8 provides bounded recurrent/associative systems; KVLab may expose KV-state adapters; SciRust may provide reusable geometry algorithms. No representation equivalence is assumed in advance.

## Non-claims

TDI-14 does not claim global bi-Lipschitz equivalence, architecture independence, or lossless conversion between attention and recurrent state.