# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: active — A0/A1/A2/A3, symbolic T1/T2/T3 generators, leakage-safe symbolic execution, frozen primary decision rules and paired-resampling foundation implemented; leakage-safe concrete task-adapter foundation under review and interval method not yet frozen
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- TDI-8.1 foundation merge: `7cfe1b66e11f1eb5d67a5890b07e6f41fa175670` (PR #89)
- TDI-8 post-foundation integrity merge: `9ee32002603942b9f4152cad47f7fb59331f8c7a` (PR #90)
- TDI-8.1 associative-memory merge: `96deedc454f2bdff03b7ce39565e713f1992dde1` (PR #91)
- TDI-8.1 A1/A2 recurrent-reference merge: `ee734d16ddfc10509d827b3e2ed90769990970bd` (PR #94)
- TDI-8.1 VSA workspace merge: `a2b23eb6bc52cb5a6c8a5f41ef339fe4cc94b3cf` (PR #97)
- TDI-8.1 integrated A3 merge: `c69d94ded34998353452cbd21474ce5530dbde3b` (PR #96)
- TDI-8.1 A0 full-history-reference merge: `39112f90724bc3bec59f35685da2dd1fb83860fc` (PR #103)
- TDI-8.1 symbolic T1/T2/T3 generator merge: PR #104
- TDI-8.1 frozen primary-cell/nine-cell decision merge: PR #106
- TDI-8.1 arm-accounting contract hardening merge: `f423b4f8a307f9639b83e8471f399918189fb117` (PR #108)
- TDI-8.1 paired-resampling foundation merge: `85912fde2b3869e91e01058156ef2b1c895d03d4` (PR #109)
- TDI-8.1 leakage-safe symbolic execution merge: `d11c3a6ccb34dd02cce70758a294295a6d597b31` (PR #110)
- Declared Rust MSRV is executable CI evidence since PR #107
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

The common accounting contracts enforce non-zero defining components, exact component-wise storage accounting, inclusion of temporary working storage, and exact matched total dynamic-memory validation for A1/A2/A3. A0 cumulative history is reported separately and is not treated as a matched-budget baseline. PR #108 additionally closed fail-open generic states by rejecting recurrent-state storage for A0, requiring non-zero A0 cumulative history, and requiring associative metadata as well as payload for A2/A3.

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

PR #104 merged architecture-neutral symbolic instances for the three frozen task families before any arm-specific binary64 encoding is selected:

- T1 associative recall with unqueried distractor associations, positive delay and delayed keyed targets generated before execution;
- T2 ordered delayed copy with exact payload targets;
- T3 controlled shared-prefix interference keys, reused generator-side interference classes and queries that always include both oldest and most-recent associations;
- explicit `Short` / `Medium` / `Long` horizon labels plus a caller-supplied strictly increasing `HorizonPlan` with no numeric defaults;
- deterministic domain-separated generation and fail-closed allocation/count guards.

T3 generator-side collision classes remain metadata only and are not allowed to stand in for measured physical A2/A3 slot collisions. A later frozen concrete adapter/evaluator must verify actual occupancy/collision pressure under its concrete associative layout/projection.

## Symbolic execution leakage boundary

PR #110 merged the symbolic execution contract between the generator-owned task and concrete binary64 arm adapters:

- the runner resets one adapter per generator-level instance and preserves source event order;
- exact query targets remain runner-owned and are never passed to the adapter API;
- association `source_index` remains runner-owned provenance metadata rather than an explicit adapter feature;
- T3 `collision_class` remains runner-owned stress metadata and is never supplied as an arm feature;
- association adapters receive only the stable key code and input value;
- T2 payload order is conveyed by event order rather than an extra source-position feature;
- T2 query position is exposed because it is the symbolic request, while its exact target remains hidden;
- adapter errors retain exact source event indices and arm identity cannot drift during an instance;
- the resulting record reports exact discrete success only and does not define the late-retrieval deficit.

A sequential arm can observe event order and maintain its own state. The leakage guarantee is specifically that evaluator annotations are not supplied as additional input features.

## Task-adapter foundation under review

The current bounded tranche maps each permitted symbolic arm stimulus into deterministic finite binary64 scheduling data without selecting experimental architecture parameters:

- every symbolic `u64` is encoded losslessly into two exact finite binary64 limbs;
- A1/A2/A3 receive one common five-coordinate minimum event frame with deterministic zero padding for any larger caller-selected width;
- the frame contains only the fields permitted by PR #110: event kind plus key/value, value, token, queried key, or queried payload position as appropriate;
- query targets, `source_index` and T3 `collision_class` are not encoded into recurrent arm input;
- A0 receives namespaced exact keys and exact values, while A0 read actions contain no target;
- A2/A3 association and payload events receive deterministic logical read/write keys, while distractor events use a read-only key proven outside the instance write set;
- A3 receives explicit VSA store-role keys but the adapter plan itself performs no arm mutation;
- physical direct-mapped A2/A3 projection pressure is measured using the concrete `address_for` rule;
- generator-side T3 collision-class reuse and physical replacement collisions are recorded separately by the evaluator-side audit.

The exact-u64 decoder also rejects negative zero so the accepted representation is canonical rather than merely numerically equivalent.

This module belongs to `tdi-bench`, not `tdi-ai`, because task scheduling and encoding are evaluator policy rather than architecture semantics.

The tranche does not define recurrent output/readout decoding, declare task success, emit deficits, construct paired intervals, run H8-A/H8-B or create any TDI-8.2 surface.

## Merged frozen primary decision rules

PR #106 transcribed the already-frozen TDI-8.0 evidence classifier into `tdi-bench::decision_v8` without changing scientific thresholds:

- exactly nine primary cells per hypothesis;
- `delta = 0.02`;
- exact non-zero-baseline relative effect and zero-baseline branch;
- Beneficial / Harmful / Equivalent / Inconclusive cell classification;
- fail-closed invalid or missing intervals;
- fixed nine-cell H8-A/H8-B aggregation with missing/rejected cells becoming Inconclusive.

The classifier consumes an interval but does not construct one.

## Merged paired-resampling foundation

PR #109 added the software substrate required before selecting the TDI-8.1 paired interval implementation:

- validated generator-level baseline/candidate deficit pairs;
- exact relative mean-deficit point statistic and zero-baseline branch;
- caller-supplied replicate count and deterministic seed with no defaults;
- deterministic paired bootstrap draws using one common sampled index for both arms;
- rejection-sampled bounded RNG draws rather than modulo-biased indexing;
- explicit zero/zero and zero/positive resample accounting;
- exact reconstruction of all requested replicate counts;
- frozen Bonferroni family/per-cell/tail alpha values exposed without selecting an interval estimator.

The foundation intentionally returns unsorted relative-effect replicates and does not freeze percentile, BCa, studentized, normal-approximation or another interval construction. Concrete method, replicate count, seed and degenerate-replicate policy remain later non-final TDI-8.1 decisions.

## Remaining TDI-8.1 work

After the task-adapter foundation is merged, bounded TDI-8.1 still requires:

1. explicit recurrent-arm output/readout semantics and actual per-generator A0/A1/A2/A3 execution over the leakage-safe schedule;
2. exact operation accounting and typed observation/rejection/provenance records;
3. non-final qualification and freeze of one deterministic paired interval implementation satisfying the frozen Bonferroni family-wise coverage rule, including replicate count, resampling seed and degenerate-replicate policy;
4. bounded train/development/validation work to freeze concrete dimensions, budgets, horizons, non-final/final seed ranges, sample counts and closed rejection taxonomy;
5. integration of paired intervals with the already-merged primary-cell classifier and exact nine-cell evidence records;
6. intervention-site/recovery integration using only early observations strictly before late retrieval;
7. a final TDI-8.1 readiness gate proving no TDI-8.2 execution surface exists.

TDI-8.2 remains future human-only and is not authorized by this status file.
