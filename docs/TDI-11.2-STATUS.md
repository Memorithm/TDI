# TDI-11.2 Status

Status: **ACTIVE PRE-ARM — MODEL EXECUTION NOT AUTHORIZED**

## Current state

TDI-11.1 has integrated the deterministic non-final controlled-world reference foundation through PR #157. TDI-11.2 is now preparing the contract required to instrument prospective precursor signals on a concrete local/open model.

This stage is deliberately split so that repository preparation cannot silently become model experimentation.

## Current authorized work

At this status, authorized work is limited to:

- TDI-11.2 protocol and gate documents;
- machine-readable pre-arm state;
- integrity/CI checks;
- non-executing adapter/schema scaffolding;
- deterministic tests proving that unresolved model-observation fields keep execution blocked.

## Current blocking fields

Concrete model execution remains blocked until the repository freezes exact values for:

- model artifact;
- adapter version;
- tokenizer/template;
- decoding/generation configuration;
- prompt serializer;
- observation registry;
- observation event timing;
- prospective eligibility rule;
- Development/Validation population derivation;
- resource accounting;
- typed rejection;
- provenance.

The machine-readable source of this pre-arm state is `docs/tdi11.2-prearm.yaml`.

## Forbidden state transitions

The following are invalid at this stage:

- invoking a concrete model from a TDI-11.2 runner;
- adding a runnable TDI-11.2 model-experiment surface while any blocking field remains unresolved;
- producing a final seed list, final dataset or final result;
- selecting an observation vector after seeing validation/final outcomes and presenting it as preregistered;
- using hidden world truth or evaluator labels as model-facing signals.

## Next transition

The next transition is a separately reviewed TDI-11.2 model/observation freeze. That transition must pin exact non-final Development/Validation identities and settings, content-address the frozen contract, pass its integrity gate on the exact PR head, merge to `main`, and update the agent roadmap separately.

Only then may a concrete local/open model instrumentation runner be implemented or executed.