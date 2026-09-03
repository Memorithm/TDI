# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: active — A0/A1/A2/A3, symbolic T1/T2/T3 generators and frozen primary decision logic merged; task-adapter foundation under construction
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- TDI-8.1 foundation merge: `7cfe1b66e11f1eb5d67a5890b07e6f41fa175670` (PR #89)
- TDI-8 post-foundation integrity merge: `9ee32002603942b9f4152cad47f7fb59331f8c7a` (PR #90)
- TDI-8.1 associative-memory merge: `96deedc454f2bdff03b7ce39565e713f1992dde1` (PR #91)
- TDI-8.1 A1/A2 recurrent-reference merge: `ee734d16ddfc10509d827b3e2ed90769990970bd` (PR #94)
- TDI-8.1 VSA workspace merge: `a2b23eb6bc52cb5a6c8a5f41ef339fe4cc94b3cf` (PR #97)
- TDI-8.1 integrated A3 merge: `c69d94ded34998353452cbd21474ce5530dbde3b` (PR #96)
- TDI-8.1 A0 full-history-reference merge: `39112f90724bc3bec59f35685da2dd1fb83860fc` (PR #103)
- TDI-8.1 symbolic task-generator merge: `bbc8333ef02739d9ac89070344d2a3fdde4a1ae4` (PR #104)
- TDI-8.1 primary decision-rule merge: `9d89be38430128dc8469fc545a36f8e6bc8ece8c` (PR #106)
- TDI-8.1 declared-MSRV CI merge: PR #107
- TDI-8.1 memory-accounting contract tightening: `f423b4f8a307f9639b83e8471f399918189fb117` (PR #108)
- Frozen TDI-8.0 preregistration blob: `fe80e7053d89824a77ef6790794f6930d1b424e2`
- Final holdout: does not exist
- Confirmatory runner: does not exist
- Human confirmation token: does not exist
- TDI-8.2 seed range: does not exist
- TDI-7.2 interaction: forbidden

## Merged reference-arm foundation

The merged TDI-8.1 reference stack contains all four preregistered software-oracle arms:

- A0 competent deterministic full-history contextual reference;
- A1 bounded recurrent-state-only reference;
- A2 bounded recurrent state plus deterministic associative memory;
- A3 A2 plus a bounded deterministic VSA/holographic workspace.

The common accounting contracts enforce non-zero defining components, exact component-wise storage accounting, inclusion of temporary working storage, and exact matched total dynamic-memory validation for A1/A2/A3. A0 cumulative history is reported separately and is not treated as a matched-budget baseline. PR #108 additionally made A0 reject recurrent-state leakage and require cumulative-history storage, while A2/A3 now fail closed if associative metadata/addressing storage is omitted.

These contracts and implementations are infrastructure only. They are not H8-A/H8-B evidence.

## Merged bounded associative-memory reference

PR #91 added the deterministic bounded direct-mapped associative-memory oracle with explicit address projection, empty/hit/collision reads, deterministic collision replacement, pre-mutation finite/width checks, fallible host allocation and exact payload/metadata/static accounting.

## Merged A1/A2 recurrent reference

PR #94 added deterministic fixed-order recurrent accumulation, fail-closed A1 transitions, A2 lookup-before-write semantics, coordinate-wise retrieval/state fusion, complete snapshots and exact recurrent/associative/temporary/static accounting.

## Merged bounded VSA workspace and A3 integration

PR #97 added deterministic bipolar binding, bundling/superposition, unbinding/retrieval and fixed-order similarity with exact accounting. PR #96 then composed that primitive with A2 under one explicit deterministic A3 rule, complete snapshots and integrated temporary-memory accounting.

No concrete experimental workspace width, seed, fusion gain or budget was frozen by those software-oracle fixtures.

## Merged deterministic A0 full-history reference

PR #103 added the competent contextual A0 control required by TDI-8.0:

- every accessible fixed-width key/value item is retained in insertion order;
- no eviction, truncation, compression, hashing or hidden projection is used;
- content read scans complete history with fixed-order squared-L2 distance;
- the smallest finite distance wins and exact ties select the most recent item;
- one-hot read coefficients are exposed explicitly;
- invalid append/read inputs fail closed without partial logical mutation;
- cumulative history, explicit count metadata, peak read temporaries and layout constants are accounted separately from the A1/A2/A3 matched budget.

The hard-content rule is deterministic reference semantics, not a claim that A0 reproduces Transformer softmax attention.

## Merged symbolic T1/T2/T3 generators

PR #104 implements architecture-neutral symbolic instances for the three frozen task families before any arm-specific binary64 encoding is selected:

- T1 associative recall with unqueried distractor associations, positive delay and delayed keyed targets generated before execution;
- T2 ordered delayed copy with exact payload targets;
- T3 controlled shared-prefix interference keys, reused generator-side interference classes and queries that always include both oldest and most-recent associations;
- explicit `Short` / `Medium` / `Long` horizon labels plus a caller-supplied strictly increasing `HorizonPlan` with no numeric defaults;
- deterministic domain-separated generation and fail-closed allocation/count guards.

T3 generator-side collision classes are metadata only and are not allowed to stand in for measured physical A2/A3 slot collisions. A frozen adapter/evaluator must verify actual occupancy/collision pressure under its concrete associative layout/projection.

This tranche froze no dimensions, horizon values, budgets, population ranges or sample counts and created no TDI-8.2 surface.

## Merged primary decision logic

PR #106 transcribed the already-frozen TDI-8.0 primary-cell and nine-cell hypothesis decision rules into `tdi-bench::decision_v8`, including the exact `delta = 0.02` threshold, zero-baseline branch, fail-closed invalid-deficit handling and nine-cell H8-A/H8-B aggregation. It does not construct uncertainty intervals and has emitted no H8 result.

## Current task-adapter foundation

The current bounded tranche connects symbolic task identity to evaluator call surfaces without selecting experimental model dimensions:

- exact lossless `u64` to finite binary64 encoding using two 32-bit limbs;
- one shared recurrent event frame for A1/A2/A3, with a minimum lossless width and deterministic zero padding rather than an experimental default width;
- namespaced exact A0 key/value mapping so distractors cannot become exact target-key aliases;
- deterministic A2/A3 logical read/write schedules, with a read-only distractor key proven outside the instance write-key set;
- an explicit A3 VSA store-role schedule without mutating A3 inside the adapter;
- direct measurement of actual A2/A3 address projection pressure through the concrete `DirectMappedAssociativeMemory::address_for` rule;
- separate reporting of T3 generator-class reuse and physical replacement collisions.

This layer is evaluator infrastructure. It does not define recurrent output/readout decoding, declare task success, produce deficits, run H8-A/H8-B, or create a TDI-8.2 surface.

## Remaining TDI-8.1 work

After the task-adapter foundation is reviewed and merged, bounded TDI-8.1 still requires:

1. frozen recurrent-arm output/readout semantics plus actual A0/A1/A2/A3 per-generator execution over the common task schedule;
2. exact operation accounting and typed rejection/provenance records;
3. a deterministic paired interval implementation satisfying the frozen Bonferroni family-wise coverage rule;
4. bounded train/development/validation work to freeze concrete dimensions, budgets, horizons, non-final/final seed ranges, sample counts, interval replicate count/seed and closed rejection taxonomy;
5. a final TDI-8.1 readiness gate proving no TDI-8.2 execution surface exists.

The frozen primary classifier and nine-cell aggregator are already merged and are no longer listed as unfinished work.

TDI-8.2 remains future human-only and is not authorized by this status file.
