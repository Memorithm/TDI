# TDI-11.2 — Prospective Instrumentation Pre-Arm Contract

Status: **ACTIVE PRE-ARM — MODEL EXECUTION FORBIDDEN**

TDI-11.2 moves the TDI-11 programme from deterministic controlled-world reference semantics toward prospective instrumentation of local/open models. This document defines the preparation boundary only. It does not authorize a model run and is not a substitute for the later model/observation freeze.

## Scientific purpose

The first TDI-11.2 objective is H11-A: determine whether declared model/runtime-visible trajectory signals carry prospective information about later unsupported atomic assertions under the frozen TDI-11 controlled-world oracle.

The experiment must distinguish three statements:

1. a signal is correlated with unsupported output after the fact;
2. a signal is available strictly before the scored assertion and predicts later unsupported output;
3. acting on that signal improves a separately gated control objective.

TDI-11.2 concerns the second statement. It does not yet authorize the third.

## Entry evidence

The pre-arm assumes the following merged foundations on `main`:

- TDI-11.0 preregistration and exact controlled-world semantics;
- deterministic controlled-world oracle and Development/Validation generator;
- deterministic reference evaluator, rejection, accounting and provenance;
- immutable prospective assertion boundary and pre/post-hoc timing classification;
- leakage-safe observation-adapter contract with only explicitly model/runtime-visible source classes.

## Domain boundary

Only `Development` and `Validation` instrumentation may eventually be authorized by the TDI-11.2 non-final gate.

This pre-arm creates no `Final` population, no final seeds, no final dataset, no final runner and no final result payload. Development and Validation material must be derived without consulting or materializing any future final population.

## Blocking freeze fields

A concrete model must not execute until a later merged TDI-11.2 freeze supplies every field below exactly and the implementation gate passes on `main`:

1. **model artifact identity** — repository/source, immutable revision or artifact hash, weight/quantization variant where relevant;
2. **adapter identity** — adapter implementation identity and version;
3. **tokenizer/chat-template identity** — exact tokenizer and serialization/template version;
4. **decoding configuration** — maximum generated tokens and every active deterministic/sampling parameter, including seed semantics where applicable;
5. **prompt serializer** — exact controlled-world visible-evidence/query serialization version;
6. **observation registry** — exact channel names and their `ObservationSourceClass` values;
7. **observation timing** — exact event-index semantics and the point at which each channel is captured;
8. **H11-A primary eligibility** — primary evidence must exist strictly before the first scored atomic-assertion event; post-hoc observations remain diagnostics only;
9. **Development/Validation population derivation** — deterministic domain-separated task/seed derivation with no final material;
10. **resource accounting** — model, adapter, verifier, retrieval/tool, resampling and other declared work;
11. **typed rejection** — malformed output, unavailable channels, timing violations, source mismatch, resource-envelope violation and provenance failure must fail closed;
12. **provenance** — exact model/adapter/prompt/observation/population/configuration identities plus timing and accounting records.

Any unresolved field keeps `MODEL_EXECUTION_AUTHORIZED: NO`.

## Allowed source classes

The pre-arm inherits the non-final observation-adapter source classes introduced in TDI-11.1:

- decoder statistics;
- hidden-state summaries when the runtime explicitly exposes them;
- visible-evidence summaries;
- verifier results;
- retrieval/tool results;
- action history;
- runtime-resource summaries;
- bounded resample summaries.

The exact channel vector is intentionally **not selected here**. No class is presumed predictive and entropy/uncertainty is not presumed sufficient.

## Leakage prohibition

No adapter, detector, verifier or controller input may contain or be derived from:

- complete-world hidden truth;
- evaluator support/error labels;
- hidden difficulty annotations;
- future trajectory state;
- alternative-arm outcomes;
- final seed or final-population material.

Evaluator-owned truth may be joined only after the prospective model record is sealed for scoring.

## Assertion timing

The first scored atomic assertion boundary remains the TDI-11.1 prospective boundary. H11-A primary observations must have event indices strictly smaller than that boundary. Observations at the boundary are invalid. Later observations are retained as `PostHocOnly` diagnostics and cannot be reclassified by a caller.

## Population discipline

Development and Validation must use separate deterministic namespaces. Candidate signal engineering, detector fitting and threshold exploration may use Development. Validation may evaluate already-declared non-final candidates but must not be silently recycled into Development after results are seen.

No final population derivation rule is created by this pre-arm.

## Resource discipline

Instrumentation must account for the work needed to obtain a signal. A signal is not treated as free merely because it is exposed by a runtime. At minimum the later freeze must distinguish model forward/generation work, adapter extraction work, verifier work, retrieval/tool work, resampling work and any additional state materialization or transfer required by the observation.

## Pre-arm non-claims

This document does not claim that:

- any listed signal predicts hallucination;
- hidden-state features are transferable across architectures;
- uncertainty is a sufficient detector;
- a detector can prevent an unsupported assertion;
- a controller improves risk/coverage/utility;
- the selected future model represents commercial or closed models;
- TDI-11.2 is ready for final or confirmatory evaluation.

## Authorization rule

`MODEL_EXECUTION_AUTHORIZED: NO`

The only authorized work under this pre-arm is documentation, schema/gate implementation, integrity checks, adapter stubs that cannot execute a concrete model, and deterministic tests of the gate itself.

A later PR may change the model-execution state only by freezing all blocking fields above, pinning the resulting contract identity, passing the TDI-11.2 implementation gate on its exact PR head, merging to `main`, and updating the agent roadmap separately.