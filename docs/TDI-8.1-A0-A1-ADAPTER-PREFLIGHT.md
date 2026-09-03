# TDI-8.1 A0/A1 symbolic-adapter preflight

Status: bounded software qualification only; **not H8-A/H8-B evidence and not a concrete TDI-8.1 experimental configuration freeze**.

## Purpose

PR #110 established the leakage-safe `SymbolicTaskAdapter` boundary. PR #112 qualified the exact binary64 task encoding candidate. PR #114 qualified an exact target-blind recurrent readout. PR #115 made finite but non-decodable query outputs explicit evaluated failures rather than technical adapter errors.

This tranche connects those already-qualified pieces to the A0 and A1 reference mechanisms before any A2/A3 event policy is selected.

## A0 adapter candidate

The A0 adapter uses the deterministic full-history reference without truncation or hidden projection:

- association events append the exact namespaced association key/value produced by the qualified encoding candidate;
- T2 payload positions are reconstructed only from chronological payload calls and are not supplied as generator provenance;
- distractors are appended under the disjoint distractor namespace;
- association and payload queries use only their exact namespaced query keys;
- the returned A0 value is decoded by the same canonical two-limb symbol decoder used elsewhere in the evaluator.

The bounded preflight requires exact T1 and T2 success for fixed software-oracle fixtures. This verifies adapter wiring and the competent full-history control semantics; it is not a comparative architecture result.

## A1 adapter candidate

The A1 adapter composes:

1. `LosslessTaskEncoder`;
2. `A1Reference`;
3. `ExactStateSymbolReadout`;
4. the `TaskPrediction` result contract.

Every event is encoded only from arguments exposed by `SymbolicTaskAdapter` and advances the recurrent reference exactly once. At query time:

- canonical readout -> `TaskPrediction::Symbol`;
- finite non-canonical readout -> `TaskPrediction::Invalid`;
- technical shape/numeric/reference failures remain typed adapter errors.

The preflight uses two deliberately artificial recurrent parameter fixtures:

- a key-echo fixture proving that a valid exact symbol can cross the encoder -> recurrence -> readout -> executor boundary;
- a finite non-canonical fixture proving every query remains in the dataset as an evaluated invalid failure.

Neither fixture is a trained model or a claim about A1 retrieval quality.

## Leakage and accounting boundaries

The adapters receive no exact target, source index or T3 collision-class annotation. A0 payload order and any later recurrent payload-key policy are reconstructed from chronological calls only.

This tranche introduces no hidden persistent learned state. Reference mechanism memory accounting remains owned by the existing A0/A1 implementations. The evaluator-local encoder/readout candidates are not promoted to a general public API by this preflight.

## Deliberately not selected

This tranche does **not** select or freeze:

- final recurrent state width or readout coordinates;
- learned/fitted recurrent parameters;
- matched A1/A2/A3 dynamic-memory budget;
- horizons, population sizes or train/development/validation seed ranges;
- late retrieval deficit or operation-count formulas;
- A2 associative event read/write policy;
- A3 VSA store/retrieve policy;
- final paired-bootstrap replicate count, seed or interval promotion;
- intervention/recovery sites;
- any TDI-8.2 runner, seed range, token or result surface.

## Why A2/A3 are deferred

`A2Reference::step` and `A3Reference::step` require an associative/VSA read key on every recurrent event. Choosing what is read and written for association, payload and distractor events is part of the architecture semantics, not mere plumbing. It therefore requires a separate bounded candidate and review rather than being silently embedded in this adapter tranche.

Physical A2/A3 replacement/collision behavior must later be measured from the concrete projection/table, independently of generator-side T3 collision classes.

## Qualification target

This preflight is successful only if:

- A0 executes bounded T1 and T2 fixtures with exact query outputs;
- A1 can emit a valid symbolic output through the complete adapter path;
- finite non-canonical A1 outputs are counted as failed queries, not rejected executions;
- A2/A3 adapter policy remains absent;
- the standard TDI-8 bootstrap and TDI-8.1 integrity gates remain green.
