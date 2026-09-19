# TDI-21.0 — Relational B0/B1 attention controls

Status: **development-only reference controls; stacked after learned relational addressing; not frozen; not confirmatory**.

This slice runs the existing isolated B0/B1 attention references on the same explicit relational facts and bounded relation-path queries used by the Boolean relational harness.

It does **not** claim a matched total budget, trained Transformer quality, or performance superiority.

## Shared task contract

For each relational fact

```text
(relation, subject) -> object
```

the reference key is the same exact symbolic packed identifier used by the direct relational control. A query path performs one attention lookup per hop.

Development and Validation therefore share:
- the same relational episode topology,
- the same independent evaluator-owned expected answers,
- the same entity/relation namespaces used by the current task family,
- the same maximum composition depth.

Only the retrieval substrate differs.

## B0 and B1

- **B0** uses the existing dense f64 Q/K encoding, dot products, softmax and weighted value readout.
- **B1** uses the existing packed-binary Q/K XOR/POPCOUNT score while retaining softmax and f64 values.

Both references retain all admitted writes up to their declared history capacity. They do not evict silently.

## Work accounting

Every successful relation hop scores **all retained writes**.

For an episode with `F` stored facts and a query path of `H` hops:

```text
pairwise_scores = F * H
lookups         = H
```

B0 additionally counts 64 floating dot terms per pairwise score. B1 counts one XOR/POPCOUNT word per pairwise score.

The weighted value, exponential, normalization, presence and readout counters remain those of the existing attention reference.

This is deliberately contrasted with the bounded Boolean relational substrate, which performs addressed bucket probes and records zero token-pair comparisons. It is a semantic software-work comparison, not a wall-clock, bandwidth, energy or hardware-throughput result.

## Distractor control

The v1 relational family includes a two-hop episode with irrelevant stored facts. B0/B1 must retain the same correct answer but pay more pairwise-score work than the corresponding clean two-hop episode.

This makes the scan cost visible rather than hiding it behind correctness.

## Fail-closed behavior

Episode execution validates:
- entity/relation ranges,
- path length,
- event budget,
- retained-history capacity

before committing the episode. An episode that cannot fit the declared attention history is rejected before any write or lookup.

## Non-claims

The current B0/B1 relation controls use exact symbolic relation/subject identifiers. They are not semantic language encoders and are not trained competitors.

Their storage representation is substantially larger than the current Boolean memory substrate and is reported separately. No matched-total-budget conclusion is permitted until a later slice explicitly matches memory, search/tuning cost and inference work.
