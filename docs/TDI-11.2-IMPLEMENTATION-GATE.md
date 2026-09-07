# TDI-11.2 Prospective Instrumentation Implementation Gate

Status: **PRE-ARM — CONCRETE MODEL EXECUTION BLOCKED**

This gate separates preparation of TDI-11.2 instrumentation from actual Development/Validation model execution.

## Pre-arm gate

The repository may merge this pre-arm only if all of the following are true:

1. TDI-11.0 remains blob-pinned and `scripts/check-tdi11-bootstrap.sh` passes.
2. TDI-11.1 controlled-world, evaluator, prospective timing and observation-adapter foundations are present on `main`.
3. `docs/TDI-11.2-PROSPECTIVE-INSTRUMENTATION-PREARM.md` declares `MODEL_EXECUTION_AUTHORIZED: NO`.
4. `docs/tdi11.2-prearm.yaml` has both `model_execution_authorized: false` and `final_execution_authorized: false`.
5. every field under `required_before_model_execution` remains explicitly `unresolved_blocking` at this pre-arm stage;
6. only Development and Validation domains are named;
7. no TDI-11.2 concrete-model runner, final seed list, final dataset, final runner or final-result payload exists;
8. no protected TDI-7.2 or TDI-8.2 material is introduced.

Passing the pre-arm gate authorizes only contract/schema/gate work. It does not authorize a model invocation.

## Model-execution transition gate

A later TDI-11.2 freeze may authorize **non-final Development/Validation instrumentation** only after all blocking fields are replaced by exact frozen values and reviewed as one coherent contract:

- immutable concrete model artifact identity;
- adapter identity and version;
- tokenizer/chat-template identity;
- complete decoding/generation settings;
- prompt serializer version;
- exact observation channel registry and source classes;
- exact event timing for every observation channel;
- strict pre-assertion H11-A primary eligibility rule;
- deterministic Development/Validation population derivation;
- exact resource accounting;
- typed rejection semantics;
- machine-readable provenance schema.

The resulting freeze must be content-addressed, CI-verified on the exact PR head and merged to `main` before the first concrete model execution.

## Mandatory fail-closed conditions

Instrumentation must reject or abort when:

- a required model/config identity is absent or mutable;
- a signal channel is undeclared or arrives from the wrong source class;
- a primary H11-A observation is not strictly before the first scored assertion event;
- a signal is non-finite or otherwise invalid under its frozen channel contract;
- hidden complete-world truth or evaluator labels reach model-facing observation state;
- Development/Validation provenance is incomplete or domain separation is violated;
- declared resource accounting is exceeded or inconsistent;
- any future final-evaluation material is visible to candidate construction or tuning.

## Explicitly not authorized

Neither this pre-arm nor the later non-final instrumentation freeze may by itself authorize:

- final/confirmatory TDI-11 evaluation;
- a final seed list, final dataset, final runner or final result payload;
- result-conditioned modification of the frozen observation vector;
- a claim that any signal is causal, sufficient, universal or production-safe;
- adaptive control claims belonging to later TDI-11 stages.

A future final confirmation requires a separately frozen final derivation/statistical/provenance contract and explicit stage authorization.