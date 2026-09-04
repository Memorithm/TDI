# TDI-9.1 — Bounded C0/C1/C2/C3 reference policies

Status: non-final TDI-9.1 software-reference qualification only. **This document does not freeze primary-cell schedules, adaptive thresholds, resource envelopes, final seeds, final populations or any TDI-9.2 surface.**

## Purpose

This tranche defines deterministic reference policy logic above the already-qualified TDI-9.1 task generators and reference execution engine. It instantiates the four policy identities frozen by TDI-9.0 without changing their information boundaries:

- C0: fixed compute schedule;
- C1: static pre-inference allocation using task-family identity only;
- C2: observation-conditioned `CONTINUE` / `STOP`;
- C3: observation-conditioned `CONTINUE` / `VERIFY` / `BACKTRACK` / `STOP`.

The implementation is a bounded software oracle. Its configurable schedules and thresholds are development parameters supplied by the caller and are not primary experimental choices until a later TDI-9.1 freeze explicitly says so.

## Structural information boundaries

### C0

`C0FixedPolicy::decide(step_index)` receives only the current transition index. It does not accept `PolicyObservation`, a task object, task-family identity, evaluator metadata, hidden difficulty, seed material or verifier output.

### C1

`C1StaticPolicy::plan(family)` receives only `AdaptiveTaskFamily`. The resulting `C1Plan::decide(step_index)` receives only the transition index.

The API deliberately has no task length, difficulty stratum, generator seed, evaluator target or live trajectory observation parameter. Two instances of the same family therefore select the same schedule under one C1 configuration even when their hidden strata, seeds or lengths differ.

### C2

`C2AdaptivePolicy::decide(observation)` accepts only the leakage-safe `PolicyObservation` already qualified by the TDI-9.1 foundation and execution layer. It calls `validate_for_arm(C2AdaptiveStopping)`, so verifier signals and checkpoint metadata fail closed.

The non-final reference stopping predicate uses only:

- current step index;
- residual;
- state delta;
- current score margin.

No evaluator-owned target or future task event is available to the policy type.

### C3

`C3RecoveryPolicy::decide(observation)` accepts the same observation carrier and validates it for C3. It may additionally consume only the already-authorized current/past verifier signal and checkpoint availability embedded in that carrier.

Its reference state machine is:

- verifier `Violated` + checkpoint available → `BACKTRACK`;
- verifier `Violated` + remaining work but no checkpoint → `CONTINUE`;
- terminal verifier `Violated` with no recovery path → typed fail-closed rejection;
- verifier `Satisfied` → `STOP`;
- verifier `Indeterminate` → `CONTINUE`;
- no verifier signal + configured verification-before-stop condition → `VERIFY`;
- no verifier signal + adaptive stop condition → `STOP`;
- no verifier signal + configured verification cadence → `VERIFY`;
- otherwise → `CONTINUE`.

The verifier output remains a constraint-satisfaction signal, not an evaluator success label.

## Non-final policy parameters

The module exposes caller-supplied development parameters so TDI-9.1 can later calibrate and falsify policies on non-final surfaces:

- C0 fixed stop step;
- C1 family-specific static stop steps;
- C2/C3 minimum step;
- C2/C3 maximum residual for adaptive stopping;
- C2/C3 maximum state delta for adaptive stopping;
- C2/C3 minimum absolute score margin;
- C3 minimum verification step;
- C3 verification cadence;
- C3 verify-before-stop toggle.

Floating-point thresholds must be finite and non-negative. Fixed/static schedules and verification cadence must be strictly positive where required. Invalid configurations are typed failures.

None of these concrete values is frozen here for the nine TDI-9 primary cells.

## Policy operation accounting

Every policy decision returns a `PolicyDecision` containing an explicit `PolicyCharge`. The execution layer must charge this amount through `charge_policy_decision` before applying the selected action.

The logical reference model is:

- C0 runtime decision: 2 logical operations, 64 policy bits;
- C1 family planning: 2 logical operations, 192 policy bits;
- C1 runtime decision: 2 logical operations, 64 policy bits;
- C2 runtime decision: 10 logical operations, 256 policy bits;
- C3 runtime decision: 17 logical operations, 385 policy bits.

These are deterministic reference logical operations and exact logical policy-state representations, not CPU instruction counts, ABI sizes, latency estimates or hardware claims.

C2 and C3 evaluate their scalar predicates and boolean composition without short-circuiting so the declared decision charge is path-invariant. C3 similarly computes cadence/checkpoint/recovery predicates before action dispatch.

## Integration qualification

The software fixture requires all of the following:

1. C0 changes action only at its fixed schedule and has no observation input.
2. C1 gives the same plan to same-family instances despite differing non-policy metadata.
3. C2 rejects verifier/checkpoint information.
4. C2 `CONTINUE` and `STOP` paths carry the same policy charge.
5. C3 `CONTINUE`, `VERIFY`, `BACKTRACK` and `STOP` paths carry the same policy charge.
6. Invalid configurations fail closed.
7. A real generated P3 development instance preserves the mechanistic contrast: a C2 full-forward path remains committed to the decoy while the C3 policy can use paid `VERIFY`, `BACKTRACK` and replay to recover.
8. Evaluator target access remains outside the policy module and is performed only through the qualified post-`STOP` execution boundary.

The P3 fixture is a software-oracle qualification, not H9-B evidence. Its one configuration and seed are non-final test fixtures.

## Deliberately not included

This tranche does not:

- select or freeze concrete primary-cell schedules or adaptive thresholds;
- freeze Shallow / Intermediate / Deep generator values;
- freeze matched maximum-compute or maximum-memory envelopes;
- define development/validation populations or final seeds;
- define uncertainty intervals or aggregate scientific verdicts;
- perform policy search or Forge integration;
- define final-entropy derivation;
- create a TDI-9.2 runner, final dataset, final seed list or result payload;
- claim language-model transfer, hardware speedup, optimal adaptive compute or reconstruction of any proprietary model.

Those remain separate evidence-gated TDI-9.1 or later tasks.