# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: active — A1/A2 and bounded VSA merged; integrated A3 reference under review
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- TDI-8.1 foundation merge: `7cfe1b66e11f1eb5d67a5890b07e6f41fa175670` (PR #89)
- TDI-8 post-foundation integrity merge: `9ee32002603942b9f4152cad47f7fb59331f8c7a` (PR #90)
- TDI-8.1 associative-memory merge: `96deedc454f2bdff03b7ce39565e713f1992dde1` (PR #91)
- TDI-8.1 A1/A2 recurrent-reference merge: `ee734d16ddfc10509d827b3e2ed90769990970bd` (PR #94)
- TDI-8.1 VSA workspace merge: `a2b23eb6bc52cb5a6c8a5f41ef339fe4cc94b3cf` (PR #97)
- Integrated A3 review: PR #96
- Frozen TDI-8.0 preregistration blob: `fe80e7053d89824a77ef6790794f6930d1b424e2`
- Final holdout: does not exist
- Confirmatory runner: does not exist
- Human confirmation token: does not exist
- TDI-8.2 seed range: does not exist
- TDI-7.2 interaction: forbidden

## Merged foundation

The merged TDI-8.1 foundation provides the typed A0/A1/A2/A3 reference-arm
vocabulary and exact component-wise memory accounting in `tdi-ai`.

It validates:

- non-zero recurrent state for A1/A2/A3;
- non-zero associative payload for A2/A3;
- non-zero VSA workspace for A3;
- exact matched total dynamic-memory accounting for A1/A2/A3;
- inclusion of temporary working storage in that matched budget;
- separate reporting of A0 cumulative history and static parameters;
- fail-closed overflow handling.

These contracts are infrastructure only. They are not evidence that A2 is
better than A1 or that A3 is better than A2.

## Merged bounded associative-memory reference

PR #91 added the deterministic bounded direct-mapped associative-memory oracle.
The primitive has explicit and tested address projection, empty/hit/collision
reads, deterministic collision replacement, pre-mutation finite/width checks,
fallible host allocation and exact declared payload/metadata/static accounting.

The concrete values in its tests remain synthetic oracle fixtures rather than
experimental choices.

## Merged A1/A2 recurrent reference

PR #94 defines:

- deterministic fixed-order recurrent accumulation with hard-tanh clipping;
- A1 recurrent-only state transitions with fail-closed atomic rejection;
- A2 lookup-before-write semantics over the PR #91 associative table;
- coordinate-wise deterministic retrieval/state fusion;
- complete A1/A2 persistent snapshots;
- exact A1/A2 recurrent, associative, temporary and static accounting.

No concrete experimental state width, table size, projection seed, recurrent
matrix or fusion gain was frozen by that implementation tranche.

## Merged bounded VSA workspace

PR #97 adds the standalone deterministic VSA software oracle required before A3
integration. It provides:

- deterministic bipolar binding from a seeded integer role projection;
- fixed-order additive bundling/superposition;
- deterministic unbinding/retrieval using the same bipolar role;
- fixed-order dot similarity for cleanup/readout experiments;
- atomic fail-closed rejection on invalid width/non-finite state;
- exact persistent-workspace, temporary-working and static-projection accounting;
- a downstream public-API integration test and additive TDI-8.1 integrity gate.

Its fixture width and role seed remain synthetic and do not freeze experimental
choices.

## Current integrated A3 tranche

PR #96 is being rebuilt on current `main` rather than using its obsolete
pre-#97 VSA implementation. The current tranche composes the merged A2 and VSA
oracles with one explicit operation order:

1. unbind/read the VSA workspace with the current read key;
2. fuse the readout coordinate-wise into the recurrent input using an explicit
   finite VSA fusion gain;
3. delegate the complete fused input to the unchanged A2 lookup-before-write
   step;
4. keep VSA storage as a separate candidate-before-commit operation so a task
   write policy is not silently embedded in the transition primitive.

The workspace width must equal recurrent input width for this reference rule;
there is no hidden adapter/projection. Integrated temporary accounting includes
the VSA readout/fused-input vector concurrently with the A2 recurrent candidate.
A synthetic accounting fixture proves that exact A1/A2/A3 matched dynamic
budgets are representable without treating those fixture dimensions as
experimental choices.

This tranche produces no H8-B evidence and freezes no concrete dimension, seed,
gain, task encoding, horizon or population.

## Remaining TDI-8.1 work

After the integrated A3 reference is merged, TDI-8.1 still requires:

1. competent deterministic A0 full-history reference semantics;
2. T1/T2/T3 task generators and short/medium/long horizon strata;
3. exact operation accounting and typed rejection/provenance records;
4. the frozen four-way cell classifier and nine-cell hypothesis aggregator;
5. a deterministic paired interval implementation satisfying the frozen
   Bonferroni family-wise coverage rule;
6. bounded train/development/validation work to freeze concrete dimensions,
   budgets, horizons, non-final/final seed ranges, sample counts, interval
   replicate count/seed and closed rejection taxonomy;
7. a final TDI-8.1 readiness gate proving no TDI-8.2 execution surface exists.

TDI-8.2 remains future human-only and is not authorized by this status file.
