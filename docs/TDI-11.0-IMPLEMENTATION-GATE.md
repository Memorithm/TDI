# TDI-11.0 → TDI-11.1 Implementation Gate

TDI-11.1 non-final evaluator implementation may begin only after all conditions below are true on `main`:

1. `docs/TDI-11.0-HALLUCINATION-DYNAMICS-PREREGISTRATION.md` is merged.
2. Its Git blob identity matches `docs/TDI-11.0-HALLUCINATION-DYNAMICS-PREREGISTRATION.gitblob`.
3. `scripts/check-tdi11-bootstrap.sh` passes.
4. Root `AGENTS.md` and `.github/copilot-instructions.md` encode the TDI-11 leakage, timing, and stage boundaries.
5. No TDI-11 final runner, final seed list, final dataset, final result payload, or equivalent confirmatory surface exists.
6. TDI-7.2 and TDI-8.2 protected boundaries remain intact.

## Frozen TDI-11.0 invariants

TDI-11.1 must preserve:

- primary operational phenomenon: unsupported generation under evaluator-owned evidence semantics;
- exact atomic labels `SUPPORTED_EXPLICIT`, `SUPPORTED_DERIVED`, `TRUE_HIDDEN_UNSUPPORTED`, `CONTRADICTED`, `NONEXISTENT`, `INDETERMINATE`, and `ABSTAINED`;
- primary mapping `UNSUPPORTED = TRUE_HIDDEN_UNSUPPORTED OR CONTRADICTED OR NONEXISTENT`;
- deterministic structured response grammar `ASSERT <subject_id> <relation_id> <object_id>` or `ABSTAIN`;
- no LLM judge in the primary controlled-world scorer;
- finite deterministic world/rule semantics with evaluator-owned complete truth and model-visible evidence separation;
- four primary families F1 explicit support, F2 derived support, F3 hidden-truth insufficiency, and F4 contradiction/override stress;
- three difficulty strata per family, therefore exactly 12 primary cells;
- B0 fixed inference, B1 static verification, B2 risk-gated decision, and B3 adaptive recovery control ladder;
- prospective timing requirement for H11-A;
- no hidden truth, evaluator label, hidden difficulty, final seed, future trajectory state, or alternative-arm outcome as controller input;
- ordered control verdict: contract validity → coverage non-inferiority → task-success non-inferiority → unsupported-emission reduction;
- explicit resource accounting for verification, resampling, recovery, policy, and model work;
- typed fail-closed rejection and machine-readable provenance;
- development/validation/final domain separation;
- preservation of negative, harmful, equivalent, and inconclusive outcomes.

## TDI-11.1 authorized work

After this gate is satisfied, agents may implement and validate on non-final domains:

- finite controlled-world types and canonical serialization;
- exact forward-chaining derivation/oracle semantics;
- visible/hidden evidence partition validation;
- structured response parser and exact atomic scorer;
- F1/F2/F3/F4 deterministic generators and Shallow/Intermediate/Deep parameter surfaces;
- typed rejection and provenance records;
- deterministic resource-accounting records;
- event/timing records required for future prospective precursor studies;
- development/validation domain-separated seed derivation;
- bounded reference B0/B1 scaffolding and interfaces for later B2/B3 work;
- software tests, independent small oracles, and bootstrap/integrity CI.

## Still forbidden after TDI-11.1 authorization

This gate does **not** authorize:

- any TDI-11 final/confirmatory run;
- materialization of final seeds or final datasets;
- selecting numerical decision margins from final evidence;
- presenting exploratory detector/controller performance as confirmatory evidence;
- using complete-world hidden truth as a verifier/controller feature;
- replacing the deterministic primary scorer with an LLM judge;
- claiming HAC novelty, universality, production safety, or superiority;
- claiming cross-model or cross-architecture transfer without separately frozen evidence.

## Next freeze before model confirmation

Before any confirmatory model evaluation can exist, TDI-11.1/11.2 must additionally freeze:

- concrete generator sizes and difficulty parameters;
- exact model/adaptor identities and decoding settings;
- exact observation vector and observation timing;
- exact verifier/resampling/recovery semantics;
- exact matched resource envelopes;
- exact prospective risk metric and calibration/risk-coverage procedures;
- numerical coverage/success/unsupported-risk decision margins;
- family-wise statistical procedure and population size;
- final future-entropy source/event and canonical seed derivation if autonomous confirmation is used;
- exact rejection and result/provenance schemas;
- a no-retry final-evaluation rule.

No later stage may weaken these requirements in response to observed final results.
