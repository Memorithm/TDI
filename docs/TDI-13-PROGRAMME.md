# TDI-13.x — Causally Conditioned Dimensional Collapse

Status: **active Stage 0 bootstrap; not frozen; no confirmatory execution authorised**.

## Conjecture

For structured memory corpora and model-state collections, conditioning on a causally coherent region reduces the intrinsic dimension required for useful retrieval or prediction:

`d_int(X | causal region) << d_int(X)`

in regimes where a global low-dimensional projection is inadequate.

## Primary null

Any apparent dimensional reduction is explained by smaller sample count, topical homogeneity, label leakage or trivial partition size rather than causal conditioning.

## Stage map

- **TDI-13.0** — freeze causal-region generators, non-causal matched partitions, intrinsic-dimension estimators and retrieval/prediction endpoints.
- **TDI-13.1** — deterministic synthetic graphs with known causal structure.
- **TDI-13.2** — compare causal partitioning against random, topical and size-matched partitions.
- **TDI-13.3** — representation sweeps from full dimension to aggressively compressed projections.
- **TDI-13.4** — external transfer using CCOS/OctaSoma-derived non-final corpora if justified.

## Required controls

Matched shard cardinality, shuffled edges, community partitions without causal semantics, PCA/JL/full-dimensional baselines, estimator sensitivity and held-out region transfer.

## Decision principle

Support requires a reproducible reduction in the dimension needed to retain a frozen retrieval or predictive criterion after controlling for shard size and non-causal clustering.

## Ecosystem boundary

CCOS and OctaSoma may provide adapters and realistic corpora, but TDI owns the scientific claim. Their existing retrieval measurements are motivation, not TDI-13 evidence.

## Non-claims

TDI-13 does not claim that three dimensions are universally sufficient, that causal partitioning always helps, or that low intrinsic dimension implies semantic completeness.