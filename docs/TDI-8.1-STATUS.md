# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: active — A0/A1/A2/A3, symbolic T1/T2/T3 generators, frozen primary decision rules and paired-resampling foundation merged; task-adapter foundation under review
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- TDI-8.1 foundation merge: `7cfe1b66e11f1eb5d67a5890b07e6f41fa175670` (PR #89)
- TDI-8 post-foundation integrity merge: `9ee32002603942b9f4152cad47f7fb59331f8c7a` (PR #90)
- TDI-8.1 associative-memory merge: `96deedc454f2bdff03b7ce39565e713f1992dde1` (PR #91)
- TDI-8.1 A1/A2 recurrent-reference merge: `ee734d16ddfc10509d827b3e2ed90769990970bd` (PR #94)
- TDI-8.1 VSA workspace merge: `a2b23eb6bc52cb5a6c8a5f41ef339fe4cc94b3cf` (PR #97)
- TDI-8.1 integrated A3 merge: `c69d94ded34998353452cbd21474ce5530dbde3b` (PR #96)
- TDI-8.1 A0 full-history-reference merge: `39112f90724bc3bec59f35685da2dd1fb83860fc` (PR #103)
- TDI-8.1 symbolic T1/T2/T3 generator merge: `bbc8333ef02739d9ac89070344d2a3fdde4a1ae4` (PR #104)
- TDI-8.1 frozen primary-cell/nine-cell decision merge: `9d89be38430128dc8469fc545a36f8e6bc8ece8c` (PR #106)
- TDI-8.1 arm-accounting contract hardening merge: `f423b4f8a307f9639b83e8471f399918189fb117` (PR #108)
- TDI-8.1 paired-resampling foundation merge: `85912fde2b3869e91e01058156ef2b1c895d03d4`
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

## Merged symbolic T1/T2/T3 generators

PR #104 merged architecture-neutral symbolic instances for the three frozen task families before any arm-specific binary64 encoding is selected. T3 generator-side collision classes remain metadata only and cannot stand in for measured physical A2/A3 slot collisions.

## Merged frozen primary decision rules

PR #106 transcribed the already-frozen TDI-8.0 evidence classifier into `tdi-bench::decision_v8`: exactly nine primary cells per hypothesis, `delta = 0.02`, the exact zero-baseline branch, Beneficial/Harmful/Equivalent/Inconclusive classification, and fixed nine-cell H8-A/H8-B aggregation. The classifier consumes an interval but does not construct one.

## Merged paired-resampling foundation

The merge `85912fde2b3869e91e01058156ef2b1c895d03d4` adds validated generator-level paired deficits, exact relative mean-deficit point statistics, caller-supplied replicate count and deterministic seed with no defaults, paired bootstrap draws, unbiased bounded-index sampling, explicit zero-baseline resample accounting and the frozen Bonferroni alpha constants.

It intentionally does not freeze percentile, BCa, studentized, normal-approximation or another interval construction. Concrete interval method, replicate count, seed and degenerate-replicate policy remain later non-final TDI-8.1 decisions.

## Task-adapter foundation under review

The current bounded tranche maps one already-generated symbolic task instance into evaluator-side call schedules without selecting experimental architecture parameters:

- every symbolic `u64` is encoded losslessly into two finite exact binary64 limbs;
- A1/A2/A3 receive one common nine-coordinate minimum event frame, with caller-selected larger widths using deterministic zero padding rather than a hidden default;
- A0 receives namespaced exact keys and exact symbolic values so distractors cannot alias task targets;
- A2/A3 association and payload events receive deterministic logical read/write keys, while distractor events use a read-only key proven outside the instance write set;
- A3 receives explicit VSA store-role keys but the adapter itself performs no arm mutation;
- physical direct-mapped A2/A3 projection pressure is measured using the concrete `address_for` rule;
- generator-side T3 collision-class reuse and physical replacement collisions are recorded separately.

This module belongs to `tdi-bench`, not `tdi-ai`, because task scheduling and encoding are evaluator policy rather than architecture semantics.

The tranche does not define recurrent output/readout decoding, declare task success, emit deficits, construct paired intervals, run H8-A/H8-B or create any TDI-8.2 surface.

## Remaining TDI-8.1 work

After the task-adapter foundation is merged, bounded TDI-8.1 still requires:

1. frozen recurrent-arm output/readout semantics and actual per-generator A0/A1/A2/A3 execution over the shared schedule;
2. exact operation accounting plus typed observation/rejection/provenance records;
3. non-final qualification and freeze of one deterministic paired interval implementation satisfying the frozen Bonferroni family-wise rule, including replicate count, resampling seed and degenerate-replicate policy;
4. bounded train/development/validation work to freeze concrete dimensions, matched budgets, horizons, population ranges, sample counts and closed rejection taxonomy;
5. integration of typed paired observations and qualified intervals with the already-merged primary-cell classifier and nine-cell evidence records;
6. a final TDI-8.1 readiness gate proving no TDI-8.2 execution surface exists.

TDI-8.2 remains future human-only and is not authorized by this status file.
