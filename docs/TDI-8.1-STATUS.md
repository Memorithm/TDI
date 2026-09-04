# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: active — A0/A1/A2/A3 references, symbolic T1/T2/T3 generators, leakage-safe execution/encoding, exact target-blind readout, evaluable-invalid prediction accounting, concrete A0/A1 adapters and a transactional A2 adapter are merged; A3 routing is separated and qualified, but the concrete A3 task/VSA policy and final bounded experimental configuration are not yet frozen
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
- TDI-8.1 leakage-safe binary64 task-encoding qualification merge: `385e5f4153dd27389297a65aeacd95828c9d45ca` (PR #112)
- TDI-8.1 conservative percentile interval preflight merge: PR #113
- TDI-8.1 exact target-blind recurrent readout merge: PR #114
- TDI-8.1 evaluable-invalid prediction contract merge: PR #115
- TDI-8.1 concrete A0/A1 adapter merge: PR #117
- TDI-8.1 transactional A2 adapter merge: PR #125
- TDI-8.1 separated A3 VSA/A2 read-routing merge: PR #127
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

PR #127 subsequently separated the VSA read route from the embedded A2 associative read key. `A3VsaReadRoute::Skip` permits an event to execute the unchanged A2 path without forcing an unrelated VSA unbind, while keyed VSA reads remain explicit. This removes a routing ambiguity but deliberately does not select the task-level A3 store/read policy.

No concrete experimental workspace width, role seed, fusion gain, event policy or budget is frozen by these software-oracle fixtures.

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

T3 generator-side collision classes remain metadata only and are not allowed to stand in for measured physical A2/A3 slot collisions. A frozen concrete evaluator must verify actual occupancy/collision pressure under its concrete associative layout/projection.

## Symbolic execution and leakage boundary

PR #110 merged the symbolic execution contract between the generator-owned task and concrete binary64 arm adapters:

- the runner resets one adapter per generator-level instance and preserves source event order;
- exact query targets remain runner-owned and are never passed to the adapter API;
- association `source_index` remains runner-owned provenance metadata rather than an explicit adapter feature;
- T3 `collision_class` remains runner-owned stress metadata and is never supplied as an arm feature;
- association adapters receive only the stable key code and input value;
- T2 payload order is conveyed by event order rather than an extra source-position feature;
- T2 query position is exposed because it is the symbolic request, while its exact target remains hidden;
- adapter errors retain exact source event indices and arm identity cannot drift during an instance.

PR #115 strengthened this boundary by introducing `TaskPrediction::Invalid`. A technically completed query that produces a finite non-canonical output remains in the evaluation denominator as an explicit failure instead of being converted into an adapter error or silently rejected.

## Qualified binary64 encoding and exact readout

PR #112 qualified a bounded leakage-safe encoding candidate:

- every symbolic `u64` maps losslessly to two exact finite binary64 32-bit limbs;
- the canonical decoder rejects non-finite, out-of-range, off-grid and negative-zero alternate representations;
- recurrent arm-facing frames require only fields exposed by `SymbolicTaskAdapter`;
- query targets, source indices and generator collision classes cannot be encoded as arm features;
- logical A2/A3 association/payload keys are deterministic and physical projection diagnostics remain runner-side.

PR #114 qualified an exact target-blind recurrent readout candidate. The caller supplies the recurrent width and two distinct coordinates; only those recurrent-state coordinates are decoded through the same canonical two-limb symbol decoder. There is no tolerance, rounding, nearest-neighbour vocabulary or target-conditioned decoder.

Neither PR freezes final recurrent width, readout coordinates or learned/reference recurrent parameters.

## Merged concrete A0/A1/A2 adapters

PR #117 qualified concrete bounded A0 and A1 `SymbolicTaskAdapter` implementations. A0 performs exact namespaced full-history queries. A1 composes the leakage-safe encoder, recurrent reference and exact target-blind readout, with finite non-canonical output becoming `TaskPrediction::Invalid`.

PR #125 qualified an explicit transactional A2 adapter policy. Writes and distractors use an instance-scoped neutral logical read key that is never written; queries use only their logical association/payload keys. T2 payload routing advances only after the underlying A2 step succeeds. The bounded T1 physical fixture reconciles runner-side projection auditing with runtime lookup diagnostics and requires exact target recovery without replacement writes or query collision misses in that software-oracle fixture.

These adapter fixtures are qualification oracles only; they do not establish H8-A quality or freeze the final A2 capacity/projection/parameter configuration.

## Merged frozen primary decision rules

PR #106 transcribed the already-frozen TDI-8.0 evidence classifier into `tdi-bench::decision_v8` without changing scientific thresholds:

- exactly nine primary cells per hypothesis;
- `delta = 0.02`;
- exact non-zero-baseline relative effect and zero-baseline branch;
- Beneficial / Harmful / Equivalent / Inconclusive cell classification;
- fail-closed invalid or missing intervals;
- fixed nine-cell H8-A/H8-B aggregation with missing/rejected cells becoming Inconclusive.

The classifier consumes an interval but does not construct one.

## Paired-resampling and interval candidate

PR #109 added validated generator-level paired resampling, caller-supplied deterministic replicate count/seed, rejection-sampled bounded draws, exact zero-baseline accounting and frozen Bonferroni family/per-cell/tail alpha accessors.

PR #113 qualified a conservative non-interpolated percentile interval candidate. It requires complete replicate accounting, rejects undefined relative effects caused by zero-baseline bootstrap replicates, and uses the exact frozen two-sided tail allocation `1/360`. It does not freeze the final interval method, replicate count, seed or degenerate-replicate policy.

## Remaining TDI-8.1 work

Bounded TDI-8.1 still requires:

1. qualify one explicit concrete A3 task/VSA adapter policy: event-level VSA store payload, role/key routing, `Skip` versus keyed reads, cleanup/readout behavior and transactional ordering, without target/evaluator leakage;
2. add exact reference operation accounting and typed provenance/rejection records across A0/A1/A2/A3 so the bounded evaluator can compare quality under declared matched memory and report actual compute separately;
3. use bounded train/development/validation evidence to select and freeze concrete recurrent dimensions/parameters/readout coordinates, A2 capacity/projection settings, A3 VSA width/seed/fusion/store policy, matched dynamic-memory budget and Short/Medium/Long numeric horizons;
4. freeze the late-retrieval deficit, intervention sites/recovery observable and closed rejection taxonomy using only observations available before the target retrieval;
5. complete qualification and freeze one deterministic paired interval implementation, including replicate count, resampling seed and degenerate-replicate policy, then integrate it with the already-frozen nine-cell classifier;
6. freeze non-final/final population domains and sample counts without creating or accessing a TDI-8.2 result surface;
7. add a final TDI-8.1 readiness/integrity gate proving every experimental choice is frozen and no TDI-8.2 executable, seed/result payload or authorization surface exists.

TDI-8.2 remains future human-only and is not authorized by this status file.
