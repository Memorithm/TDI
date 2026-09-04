# TDI-9.1 reference evaluator integration

Status: **bounded non-final software qualification — no TDI-9.2 material**

Tracks #134 and #92. This tranche composes the already-merged TDI-9.1 task, execution and reference-policy layers. It does not change the frozen TDI-9.0 scientific questions or select primary experimental settings.

## Integration boundary

`adaptive_evaluator.rs` receives exactly four caller-controlled inputs:

1. one already-generated `GeneratedTask`;
2. one already-constructed `ReferencePolicy` (C0/C1/C2/C3);
3. one caller-supplied `ResourceEnvelope`;
4. one caller-supplied technical runtime-decision limit.

The generated instance is split once into policy-visible `PolicyTask` and evaluator-only `EvaluatorRecord`. `ReferenceExecution` is constructed from `PolicyTask` only. Seed, hidden difficulty stratum, exact target and construction oracle do not enter a policy decision.

The runtime-decision limit is a fail-closed technical guard against a non-terminating policy/configuration. It is not a task horizon and cannot produce a quality verdict. Exhausting it returns `DecisionLimitExceeded`.

## Policy execution

Every runtime decision follows one fixed integration sequence:

1. obtain the policy decision from only the information allowed to that arm;
2. verify that the returned arm identity has not drifted;
3. charge the complete `PolicyCharge` through `ReferenceExecution::charge_policy_decision`;
4. execute exactly one frozen action: CONTINUE, VERIFY, BACKTRACK or STOP.

C1 additionally pays its family-only pre-inference planning charge before any runtime decision. The planned family is taken from `PolicyTask::family()` and no evaluator metadata is supplied to planning.

## Evaluator-only access

`evaluate_stopped` is invoked only after `ReferenceExecution::stop()` returns `StoppedCandidate`. The immutable output record can then contain evaluator-owned provenance because the live trajectory is already closed.

The record contains:

- arm identity;
- task family;
- evaluator difficulty stratum and generator seed for provenance;
- exact success/failure against the evaluator target;
- emitted solver candidate;
- stop step;
- runtime policy-decision count;
- exact `ExecutionAccounting`, including component-wise compute/memory usage and checkpoint traffic.

A technical, policy or resource failure returns a typed `ReferenceEvaluatorError`. It is not converted into successful/failed task evidence by this layer.

## Software-oracle qualification

The integration fixture uses small non-final configurations only. It checks:

- C0 and C1 terminate through the common evaluator path on P1/P2/P3 fixtures without verifier/checkpoint use;
- C1 family planning is explicitly charged and creates the expected larger policy-memory high-water mark versus its runtime state;
- a full-forward C2 P1 fixture succeeds through post-STOP evaluation;
- the already-qualified P3 contrast survives composition: C2 remains on the decoy while C3 succeeds only with paid verifier/checkpoint/backtrack/replay work;
- an insufficient caller decision limit rejects technically without an evaluator verdict.

These fixtures are software oracles, not H9-A/H9-B evidence.

## Deliberately not frozen

This tranche does not select or freeze:

- C0/C1 schedules;
- C2/C3 thresholds or verification cadence;
- P1/P2/P3 primary difficulty values;
- maximum compute or memory envelopes;
- permitted primary observation-vector settings beyond the already-merged type contract;
- development/validation/final population domains or sample counts;
- paired uncertainty, primary-cell aggregation or rejection taxonomy;
- future public-entropy source or final seed derivation.

`adaptive_evaluator.rs` remains outside the stable `tdi-ai` public module surface during qualification.

## Holdout boundary

- TDI-9.2 runner: absent;
- TDI-9.2 seed list/dataset/result: absent;
- TDI-7.2 interaction: forbidden;
- TDI-8.2 interaction: forbidden.
