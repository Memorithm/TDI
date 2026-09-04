# TDI-9.1 recorded rejection taxonomy

Status: **bounded non-final evaluator provenance — no TDI-9.2 material**

Tracks #139 and the active TDI-9 programme #92. This tranche builds on the integrated evaluator merged by #135.

## Purpose

A trajectory that cannot complete normally must remain distinguishable from a completed trajectory that produced an incorrect candidate. Resource exhaustion, policy-contract failure, decision-guard exhaustion, accounting failure and similar technical conditions are therefore recorded as typed rejections rather than coerced into `success = false`.

The existing evaluator, execution, policy and adaptive-inference layers already expose typed failures. This tranche preserves them losslessly and adds only evaluator-side recording.

## Exact rejection structure

`ReferenceRejectionCode` preserves the origin layer:

- evaluator integration failures remain evaluator-level variants;
- execution failures become `ReferenceExecutionRejectionCode`;
- policy failures become `ReferencePolicyRejectionCode`;
- nested inference-contract/accounting failures become `AdaptiveInferenceRejectionCode` without discarding their arm/action/field/envelope payloads.

All conversions use exhaustive Rust `match` expressions. A future source-error variant therefore makes this layer fail compilation until the rejection mapping is deliberately extended.

No code is mapped to Beneficial, Equivalent, Harmful, Inconclusive, correct or incorrect. Those are scientific interpretations of completed evidence, not technical rejection labels.

## Recorded outcome API

The already-qualified

`evaluate_generated_task(...) -> Result<ReferenceEvaluationRecord, ReferenceEvaluatorError>`

remains unchanged.

A separate `evaluate_generated_task_recorded(...)` entry point returns exactly one of:

- `ReferenceRecordedOutcome::Completed(ReferenceEvaluationRecord)`;
- `ReferenceRecordedOutcome::Rejected(ReferenceRejectionRecord)`.

The rejection record carries arm, task family, hidden difficulty stratum, generator seed and the exact rejection code. Stratum and seed are copied from the evaluator-owned record only by this recording layer. They are not inserted into `PolicyTask`, `PolicyObservation`, `ReferenceExecution` policy-visible state or C0/C1/C2/C3 decision APIs.

## Required distinctions

The qualification explicitly proves that:

- zero runtime-decision limit is a rejection;
- exhausting a caller-supplied runtime decision guard is a rejection, not an implicit STOP;
- resource-envelope exhaustion is a nested execution/inference rejection;
- a completed recorded outcome is identical to the normal evaluator result;
- policy-origin and nested inference payloads are preserved exactly.

## Deliberately not selected

This tranche does **not** choose or freeze:

- primary C0/C1 schedules;
- C2/C3 thresholds or cadence;
- task populations or sample counts;
- development/validation seed domains;
- resource-envelope values for primary cells;
- rejection-rate acceptance thresholds;
- uncertainty/statistical procedures;
- a future entropy source, target round/event or entropy-to-seed mapping;
- any TDI-9.2 executable, seed list, dataset or result payload.

Those remain separate TDI-9.1 freeze work. No rejection recorded here is scientific evidence for H9-A or H9-B by itself.
