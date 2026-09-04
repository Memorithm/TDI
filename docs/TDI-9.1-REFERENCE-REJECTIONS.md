# TDI-9.1 machine-readable reference rejection records

Status: **bounded non-final software qualification — no TDI-9.2 material**

Tracks #139 and #92. This tranche adds recorded rejection provenance above the already-qualified TDI-9.1 reference evaluator. It does not change the frozen TDI-9.0 hypotheses, policy ladder, task families, resource semantics or evaluator quality semantics.

## Scientific boundary

A trajectory that fails technically is not a correct or incorrect task result. Resource exhaustion, decision-limit exhaustion, policy failure, arithmetic failure, invalid observation/accounting state and other fail-closed errors remain **rejections**.

The rejection layer therefore never adds a `success` field to a rejection record and never maps a rejected trajectory to `success=true` or `success=false`.

## Compatibility

`evaluate_generated_task(...) -> Result<ReferenceEvaluationRecord, ReferenceEvaluatorError>` remains unchanged.

The additive entry point `evaluate_generated_task_recorded(...)` returns one of two structurally distinct outcomes:

- `ReferenceEvaluationOutcome::Completed(ReferenceEvaluationRecord)` after a normal explicit STOP and evaluator comparison;
- `ReferenceEvaluationOutcome::Rejected(ReferenceRejectionRecord)` after a technical/evaluator/policy/execution rejection.

The wrapper delegates execution to the already-qualified compatibility evaluator rather than duplicating trajectory semantics.

## Stable rejection codes

`ReferenceRejectionCode` uses an explicit `u16` representation. Existing numeric values are immutable within this TDI-9.1 contract; future reasons must use unused values rather than renumbering an existing code.

The current code ranges preserve the error path as well as the reason:

- `0x01xx`: evaluator-integration failures;
- `0x02xx`: direct reference-execution failures;
- `0x03xx`: adaptive-inference failures reached through reference execution;
- `0x04xx`: direct reference-policy failures;
- `0x05xx`: adaptive-inference failures reached through reference policy logic.

This distinction is intentional. For example, an `ActionForbidden` failure reached through an execution action and the same underlying contract failure reached while validating a policy observation receive different stable codes. The rejection record also retains the complete original `ReferenceEvaluatorError`, including typed payloads such as requested/maximum resource values.

## Evaluator-side provenance

A rejection record contains:

- selected policy arm;
- policy-visible task family;
- evaluator-only difficulty stratum;
- generator seed;
- stable rejection code;
- original typed evaluator error.

Stratum and seed are attached only on the evaluator/recording side. The wrapper does not add seed, stratum, target, construction oracle or future-event data to `PolicyTask`, `PolicyObservation`, `ReferenceExecution` or any C0/C1/C2/C3 decision API.

The rejection module is explicitly forbidden from reading evaluator target/oracle fields.

## Qualification

The dedicated qualification checks:

- a normal bounded trajectory remains a normal completed evaluation;
- caller decision-limit exhaustion becomes a rejection with exact arm/family/stratum/seed provenance;
- declared-memory exhaustion remains an execution/inference rejection rather than a quality failure;
- execution, policy and nested inference paths retain distinct codes;
- all 45 current numeric rejection codes match their exact frozen non-final values;
- the original typed `ReferenceEvaluatorError` remains available;
- the rejection layer is not promoted into the stable `tdi-ai` API;
- the existing TDI-9.1 evaluator/bootstrap gates still pass;
- no TDI-9.2/final executable or result surface is introduced.

These are software-contract tests, not H9-A or H9-B evidence.

## Deliberately not frozen

This tranche does not choose or freeze:

- C0/C1 schedules;
- C2/C3 thresholds or verification cadence;
- P1/P2/P3 primary difficulty values;
- primary maximum resource envelopes;
- development/validation populations or seed domains;
- permitted rejection-rate thresholds for a future scientific analysis;
- sample counts, paired uncertainty or primary-cell aggregation;
- future public entropy source, event or final seed derivation;
- any TDI-9.2 executable, dataset, seed list or result payload.

A later TDI-9.1 freeze may use these rejection records as machine-readable provenance, but it must define any scientific handling of rejection rates before final entropy is knowable.
