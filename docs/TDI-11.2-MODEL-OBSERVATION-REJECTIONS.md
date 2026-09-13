# TDI-11.2 model-observation rejection + provenance scaffolding

Status: **non-executing software scaffolding — NOT A PIN and not a freeze**.

This tranche lands a closed machine vocabulary over the already-qualified
TDI-11.1 prospective timing and leakage-safe observation-adapter contracts. It
does **not** authorize model execution, invent model/adapter/tokenizer pins, or
move any of the twelve TDI-11.2 freeze fields off `unresolved_blocking`.

## Scientific / stage boundary

- `model_execution_authorized` remains **false**.
- `final_execution_authorized` remains **false**.
- Freeze pins stay **0/12**.
- Holdouts TDI-7.2 / TDI-8.2 / TDI-9.2 remain forbidden contact.
- TDI-8.1 `SymbolicRejectionCode` and TDI-9.1 `ReferenceRejectionCode` do **not**
  transfer into TDI-11.2 (explicit non-authorizing surfaces).

Orchestrator holdout #186 continues to forbid AUTO_MERGE of a TDI-11.2 freeze
that invents model/observation pins. This scaffolding PR must keep every field
`unresolved_blocking`.

## What landed

### Typed rejection vocabulary (candidate only)

`ModelObservationRejectionCode` (`repr(u16)`) maps every currently represented
`ObservationAdapterError` / nested `ObservationContractError` path to a stable
numeric code:

| Range | Meaning |
| --- | --- |
| `0x01xx` | adapter identity / manifest construction |
| `0x02xx` | packet / channel declaration failures |
| `0x03xx` | source-class mismatch |
| `0x04xx` | prospective timing / observation contract |
| `0x05xx` | provenance scaffolding failures |

`ModelObservationRejectionRecord` retains the original typed adapter error plus
adapter name/version provenance. Technical rejection is never mapped to a
hallucination-quality outcome.

Non-authorizing candidate id: `ModelObservationRejectionCode`
(`CANDIDATE_TYPED_REJECTION_VOCABULARY`). This does **not** pin
`typed_rejection_contract`.

### Provenance schema scaffolding (candidate only)

`ModelObservationTraceProvenance` is a fail-closed, byte-length-framed record
for Development/Validation model-observation traces. It rejects empty identity
fields, forbidden Final-domain labels, and forbidden surface tokens such as
`final_seed_list` / `concrete_model_runner` / `complete_world_hidden_truth`.

Non-authorizing candidate id: `ModelObservationTraceProvenance`
(`CANDIDATE_PROVENANCE_SCHEMA`). This does **not** pin `provenance_contract`.

## Qualification surfaces

- module: `tdi-ai/src/hallucination_model_observation_rejections.rs`
- tests: `tdi-ai/tests/tdi11_model_observation_rejections.rs`
- gate: `scripts/check-tdi11.2-model-observation-rejections.sh`
- workflow: `.github/workflows/tdi11.2-model-observation-rejections.yml`

The rejection module is intentionally **not** promoted into the stable
`tdi-ai` `lib.rs` API (same posture as TDI-9.1 `adaptive_rejections`).

## Explicitly not frozen / not authorized

- model artifact, adapter, tokenizer/template, decoding, prompt serializer;
- exact observation registry / timing / H11-A eligibility;
- Dev/Val population derivation;
- numeric resource envelopes;
- `typed_rejection_contract` and `provenance_contract` freeze fields;
- any concrete model runner or final surface.

A later reviewed freeze may pin `typed_rejection_contract` /
`provenance_contract` only with explicit evidence and without setting execution
flags true while other blockers remain.
