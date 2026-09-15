# TDI-21.0 — B4 local-predicate identifiability audit

Status: **constructed Stage-0 counterexample; development-only; not confirmatory**.

Classification: **EXACT under the declared B3 memory and `tdi21-write-local-predicates-v1` semantics**.

## Question

Can a deterministic causal admission policy over the six v1 local predicates always choose the write action that is retrospectively best for the next recall?

For the declared v1 predicate map, the answer is **no**.

This is a limitation of the present observation map, not a refutation of Boolean routing, ANF search, B4 as a whole, or distribution-level policy improvement.

## Constructed pair

Use the existing two-way B3 memory with four entries, route salt zero and the already exercised colliding identifiers `1`, `5` and `9`.

The common causal prefix stores:

1. `key=1, payload=11`;
2. `key=5, payload=55`.

The current decision is identical in both episodes:

`Write { key=9, payload=99, marker=1 }`.

Immediately before that decision, the candidate sees the same local state in both episodes:

- marker bit 0 = true;
- marker bit 1 = false;
- key 9 is not already present;
- the addressed bucket has entries;
- the addressed bucket is full;
- the deterministic next victim is the first way.

Therefore both episodes produce exactly the same six-bit v1 predicate assignment:

`0b011001`.

The episodes differ only in an evaluator-owned future recall that is not available to the causal candidate.

### Episode A — preserve the old fact

The next recall asks for key `1` and expects payload `11`.

- **ADMIT** key 9: B3 evicts key 1, so the probe fails.
- **INHIBIT** key 9: key 1 remains, so the probe succeeds.

The unique hindsight-optimal label is therefore `INHIBIT`.

### Episode B — retain the new fact

The next recall asks for key `9` and expects payload `99`.

- **ADMIT** key 9: the new fact is stored, so the probe succeeds.
- **INHIBIT** key 9: key 9 is absent, so the probe fails.

The unique hindsight-optimal label is therefore `ADMIT`.

## Exact implication

Let `p = 0b011001` be the common v1 predicate vector and let a deterministic causal v1 policy be a function

`g : {0,1}^6 -> {ADMIT, INHIBIT}`.

The two constructed episodes require simultaneously

`g(p) = INHIBIT`

and

`g(p) = ADMIT`.

No deterministic function can satisfy both equations. Therefore **the v1 predicate map is non-identifying for per-episode next-recall-optimal admission on this constructed pair**.

This conclusion is independent of whether `g` is represented by ANF/Zhegalkin, a truth table, clauses or another deterministic Boolean representation: identical observed input cannot map to two different actions in the same deterministic policy.

## What this does not show

The counterexample does **not** show that:

- B4 cannot improve expected performance over a declared task distribution;
- ANF search is useless;
- more informative causal predicates cannot resolve some conflicts;
- randomized policies cannot express a distribution over actions;
- a recurrent Boolean state cannot carry additional useful information;
- future information should be leaked into the candidate;
- attention is necessary.

In particular, exposing the future recall itself would trivially remove this ambiguity but would violate the causal architecture objective. The appropriate response is to measure the tradeoff under a declared distribution and/or design richer **causal** state, not to leak evaluator futures.

## Evaluator boundary

`tdi21_predicate_identifiability` is evaluator-only. It clones a common causal B3 prefix, evaluates both `ADMIT` and `INHIBIT`, then applies one declared future recall. The future probe is used only to derive a hindsight label; it is never passed to `encode_b4_write_predicates` and is never a runtime observation.

The audit rejects direct-memory mode, non-admitted decision writes, missing expected probe values and prefixes longer than 32 events. It is a bounded diagnostic, not a general planner.

## Consequence for the search programme

The next ANF-search experiment must not use "perfect Development classification" as its scientific objective on a task family containing this kind of conflict unless the feature map is changed. A deterministic v1 policy must make a tradeoff.

A sound next experiment should therefore preregister a distributional objective, for example:

- protected old-fact recall success;
- new-fact recall success;
- overall expected exact recall;
- replacement count / memory work;
- explicitly stratified conflict cases.

Validation must remain post-selection. Negative tradeoffs must be retained. Predicate expansion, if attempted, must be versioned and justified using only causal present/past information.
