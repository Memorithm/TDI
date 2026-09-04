# TDI-9.1 status

- Scientific series: TDI-9.x
- Stage: TDI-9.1 bounded autonomous adaptive-inference evaluator
- Status: active — policy/action/observation/resource-accounting foundation under qualification
- Parent TDI-9.0 merge: `bb13c59aa91e3e5e2e6a480f4ae12adfe168221b` (PR #122)
- Frozen TDI-9.0 preregistration blob: `babad0a4e309e67e57820281a0f31284ba1e5da0`
- TDI-9.2 runner: does not exist
- TDI-9.2 final seed list: does not exist
- TDI-9.2 final dataset: does not exist
- TDI-9.2 result payload: does not exist
- Human confirmation token: intentionally absent from TDI-9
- TDI-7.2 interaction: forbidden
- TDI-8.2 interaction: forbidden

## Current foundation

The first TDI-9.1 slice defines only infrastructure required by the frozen TDI-9.0 policy ladder:

- `C0FixedCompute`, `C1StaticPreallocation`, `C2AdaptiveStopping`, and `C3VerificationRecovery` identities;
- frozen `CONTINUE`, `VERIFY`, `BACKTRACK`, `STOP` action vocabulary;
- arm-level action legality, with verifier/backtracking restricted to C3;
- a deterministic policy-observation carrier containing only current/past trajectory summaries;
- C3-only verifier signal and checkpoint metadata;
- strictly positive common maximum compute/memory envelopes;
- explicit solver, verifier, policy-decision, checkpoint and replay operation accounting;
- simultaneous persistent, policy, checkpoint and temporary-peak memory accounting;
- atomic fail-closed rejection before resource-envelope overflow.

This slice intentionally does **not** define a solver, task generator, verifier algorithm, checkpoint transition engine, C0/C1/C2/C3 policy logic, policy search space, difficulty parameters, uncertainty method, non-final seed ranges, or final entropy source.

The module is initially qualified through an isolated compile/integration fixture before promotion into the stable `tdi-ai` public module surface. This avoids expanding the public research API before its core invariants pass CI and review.

## Remaining TDI-9.1 work

1. qualify and publicly expose the policy/accounting foundation;
2. implement deterministic trajectory/checkpoint state transitions and exact transition accounting;
3. implement P1/P2/P3 architecture-neutral mechanistic task generators without hidden difficulty leakage;
4. implement frozen solver and independent verifier semantics;
5. implement C0/C1 reference controls;
6. implement bounded C2/C3 policy interfaces and initial deterministic baselines;
7. define agent-search-safe policy mutation/evaluation contracts for development/validation only;
8. freeze concrete task/difficulty parameters, observation vector, resource envelopes and non-final split domains;
9. freeze one paired family-wise uncertainty implementation and rejection taxonomy;
10. freeze the future public-entropy source/event/encoding/derivation contract before its value is knowable;
11. prove no TDI-9.2 final material exists before the future-entropy gate.

No item in this status file authorizes TDI-9.2 execution.
