# TDI-9.0 → TDI-9.1 Implementation Gate

TDI-9.1 evaluator implementation may begin only after all conditions below are true on `main`:

1. `docs/TDI-9.0-ADAPTIVE-INFERENCE-PREREGISTRATION.md` is merged.
2. Its Git blob identity matches `docs/TDI-9.0-ADAPTIVE-INFERENCE-PREREGISTRATION.gitblob`.
3. `scripts/check-tdi9-bootstrap.sh` passes.
4. The repository agent contracts explicitly preserve TDI-7.2 and TDI-8.2 protections while allowing the separately preregistered autonomous TDI-9 confirmation model.
5. No TDI-9.2 runner, final seed list, final dataset or result surface exists.
6. No final entropy source value has been obtained or embedded.

## Frozen TDI-9.0 invariants

TDI-9.1 must preserve:

- C0 fixed compute;
- C1 static preallocation;
- C2 observation-conditioned `CONTINUE` / `STOP`;
- C3 `CONTINUE` / `VERIFY` / `BACKTRACK` / `STOP`;
- primary H9-A contrast C2 vs C0;
- primary H9-B contrast C3 vs C2;
- exactly nine primary cells per hypothesis: three task families × three difficulty strata;
- frozen quality margin `delta_q = 0.02`;
- frozen compute margin `delta_k = 0.05`;
- quality non-inferiority gate before compute classification;
- at least 95% family-wise coverage across the 18 primary intervals per hypothesis;
- no arbitrary weighted quality/compute utility as a substitute for the two-stage primary rule;
- matched declared maximum-compute and maximum-memory envelopes;
- actual compute, verification, backtracking and checkpoint costs counted explicitly;
- no target, future event, hidden difficulty annotation or evaluator label supplied to adaptive policies;
- negative, harmful, equivalent and inconclusive outcomes preserved.

## Autonomous-research rule

During TDI-9.1, agents may iterate freely on non-final train/development/validation surfaces within the frozen TDI-9.0 scientific boundary. Candidate policies may be proposed, mutated, falsified and discarded automatically.

Before any TDI-9.2 implementation can exist, TDI-9.1 must additionally freeze:

- concrete task and difficulty parameters;
- exact observation vector;
- exact solver/verifier/checkpoint semantics;
- concrete resource envelopes;
- uncertainty implementation and all randomness used by it;
- non-final/final domain derivation rules;
- final public future-entropy source identity and target event;
- canonical entropy encoding and seed derivation;
- final population size;
- typed rejection taxonomy;
- exact evaluator/manifests and result schema.

## No human token

TDI-9 intentionally has no human confirmation token. This is not permission for agents to choose final seeds or final entropy.

The future entropy event is binding once frozen. Result-conditioned selection, retry, replacement or skipping of that event is protocol drift and must fail closed.

## Historical isolation

This gate does not alter TDI-7.2 or TDI-8.2 policy. TDI-7.2 remains frozen and non-rerunnable. TDI-8.2 remains absent and unauthorized while TDI-8.1 is active.
