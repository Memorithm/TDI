# TDI-21.0 — Causal sequence-to-distribution materializer

Status: **development-only bridge; not frozen; not confirmatory; no final/holdout material**.

This increment removes a manual step between the exact B4 identifiability audit and the conflict-preserving distributional objective. Explicit causal sequence cases are executed through the same evaluator-only counterfactual audit used by the v1 identifiability result, and the resulting predicate assignment plus `ADMIT`/`INHIBIT` success bits are materialized automatically.

## Why this bridge exists

A future Development/Validation family must not hand-author Boolean labels for observable states. The v1 counterexample already establishes that one observable assignment can legitimately occur with opposite hindsight-optimal actions in different episodes. Manual transcription would make it too easy to overwrite, majority-collapse or inconsistently encode those conflicts.

`materialize_admission_cases` therefore accepts explicit `AdmissionAuditCase` values and produces `AdmissionOutcomeSample` records directly from execution.

## Causal/evaluator boundary

For each case:

1. the causal B3 prefix is replayed;
2. the production predicate adapter observes only the current write key and current B3 state;
3. evaluator-only branches clone that same causal state and execute both `ADMIT` and `INHIBIT`;
4. the declared future recall probe scores both branches;
5. only the candidate-visible predicate assignment and the two evaluator success bits are placed in the materialized sample.

The future probe and counterfactual outcomes remain evaluator-side research material. They are never runtime candidate features.

## Batch invariants

The materializer:

- rejects empty batches;
- admits at most the same 4,096 samples as the distributional objective;
- preserves input order and one output sample per input case;
- reports the exact failing case index if the underlying identifiability audit rejects a case;
- counts the four counterfactual outcome classes separately: both, admit-only, inhibit-only, neither;
- uses checked summary counters and fail-closed allocation.

It does not choose a policy, assign Development/Validation membership, tune weights or perform ANF search.

## Qualification targets

Development tests derive by execution:

- the exact `0b011001` v1 conflict pair from the existing keys `1`, `5`, `9`;
- all four counterfactual classes, including a deliberately unsatisfied evaluator target for the `neither` class;
- a direct end-to-end bridge from materialized conflict samples into `DevelopmentAdmissionSet` and bounded distributional ANF search, retaining the exact one-failure feature-conflict floor;
- indexed failure on an invalid second case;
- empty and over-budget batch rejection before materialization.

These fixtures validate software semantics only. They do not define the scientific sequence populations.

## Next step

After this bridge is qualified, Development and Validation **family generators** can be defined separately. Those generators must freeze their provenance, disjointness, strata and weighting before comparative interpretation. They must generate explicit causal cases first and use this materializer to derive outcomes; they must not synthesize policy labels directly.

Admission/replacement remains a qualification surface for Boolean routing. Once this pipeline is trustworthy, TDI-21 must return to relational binding/composition tasks and state evolution that address broader attention functionality.
