# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: active — foundation and associative-memory reference merged; A1/A2 recurrent reference under review
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- TDI-8.1 foundation merge: `7cfe1b66e11f1eb5d67a5890b07e6f41fa175670` (PR #89)
- TDI-8 post-foundation integrity merge: `9ee32002603942b9f4152cad47f7fb59331f8c7a` (PR #90)
- TDI-8.1 associative-memory merge: `96deedc454f2bdff03b7ce39565e713f1992dde1` (PR #91)
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

## Current A1/A2 recurrent-reference tranche

The current bounded tranche defines:

- deterministic fixed-order recurrent accumulation with hard-tanh clipping;
- A1 recurrent-only state transitions with fail-closed atomic rejection;
- A2 lookup-before-write semantics over the PR #91 associative table;
- coordinate-wise deterministic retrieval/state fusion;
- complete A1/A2 persistent snapshots;
- exact A1/A2 recurrent, associative, temporary and static accounting.

No concrete experimental state width, table size, projection seed, recurrent
matrix or fusion gain is frozen by this implementation tranche.

## Remaining TDI-8.1 work

After the A1/A2 reference is merged, bounded TDI-8.1 still requires:

1. A3 VSA/holographic workspace software oracles and A3 integration;
2. competent deterministic A0 full-history reference semantics;
3. T1/T2/T3 task generators and short/medium/long horizon strata;
4. exact operation accounting and typed rejection/provenance records;
5. the frozen four-way cell classifier and nine-cell hypothesis aggregator;
6. a deterministic paired interval implementation satisfying the frozen
   Bonferroni family-wise coverage rule;
7. bounded train/development/validation work to freeze concrete dimensions,
   budgets, horizons, non-final/final seed ranges, sample counts, interval
   replicate count/seed and closed rejection taxonomy;
8. a final TDI-8.1 readiness gate proving no TDI-8.2 execution surface exists.

TDI-8.2 remains future human-only and is not authorized by this status file.
