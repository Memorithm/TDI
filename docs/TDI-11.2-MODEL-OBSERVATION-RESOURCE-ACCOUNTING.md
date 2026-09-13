# TDI-11.2 model-observation resource accounting scaffolding

Status: **non-executing software scaffolding — NOT A PIN and not a freeze**.

This tranche lands an exact component taxonomy and fail-closed overflow /
envelope-admission rules for prospective TDI-11.2 model-observation
instrumentation. It does **not** authorize model execution, invent numeric
freeze envelopes, or move any of the twelve TDI-11.2 freeze fields off
`unresolved_blocking`.

## Scientific / stage boundary

- `model_execution_authorized` remains **false**.
- `final_execution_authorized` remains **false**.
- Freeze pins stay **0/12**.
- Holdouts TDI-7.2 / TDI-8.2 / TDI-9.2 remain forbidden contact.
- TDI-11.1 `ReferenceResourceEnvelope::unlimited_for_development` does **not**
  transfer into TDI-11.2 as a freeze value (explicit non-authorizing surface).

Orchestrator holdout #186 continues to forbid AUTO_MERGE of a TDI-11.2 freeze
that invents model/observation pins. This scaffolding PR must keep every field
`unresolved_blocking`.

## What landed

### Exact component taxonomy (candidate only)

`RESOURCE_ACCOUNTING_COMPONENT_KEYS` enumerates eleven instrumentation meters:

| Key | Meaning |
| --- | --- |
| `model_input_tokens` | Model-facing input token charge |
| `model_output_tokens` | Model-facing output token charge |
| `model_decode_steps` | Decode / generation step charge |
| `adapter_ingest_events` | Guarded adapter ingest events |
| `adapter_declared_channel_emissions` | Declared channel emissions observed |
| `observation_packet_bytes` | Observation packet payload bytes |
| `timing_contract_checks` | Prospective timing-contract checks |
| `provenance_frame_bytes` | Provenance framing byte charge |
| `verifier_calls` | Verifier invocation charge |
| `retrieval_tool_calls` | Retrieval / tool call charge |
| `resamples` | Resample charge |

`ModelObservationResourceUsage` provides fail-closed checked charges and an
exact component-sum `total()`. Overflow maps to
`ModelObservationRejectionCode::AccountingOverflow` (`0x0601`).

### Caller-supplied envelope (candidate only)

`ModelObservationResourceEnvelope` admits usage under caller-supplied maxima.
`unbounded_caller_supplied()` is an engineering convenience for non-final
scaffolding tests only. Envelope exceed maps to
`AccountingEnvelopeExceeded` (`0x0602`).

Non-authorizing candidate id: `ModelObservationResourceAccountingContract`
(`CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT`). This does **not** pin
`resource_accounting_contract`.

## Qualification surfaces

- module: `tdi-ai/src/hallucination_model_observation_accounting.rs`
- rejection codes: `0x06xx` in `tdi-ai/src/hallucination_model_observation_rejections.rs`
- tests: `tdi-ai/tests/tdi11_model_observation_accounting.rs`
- gate: `scripts/check-tdi11.2-model-observation-accounting.sh`
- workflow: `.github/workflows/tdi11.2-model-observation-accounting.yml`

The accounting module is intentionally **not** promoted into the stable
`tdi-ai` `lib.rs` API (same posture as the rejection/provenance scaffold).

## Explicitly not frozen / not authorized

- any numeric `max_*` scientific envelope;
- model artifact, adapter, tokenizer/template, decoding, prompt serializer;
- exact observation registry / timing / H11-A eligibility;
- Dev/Val population derivation;
- `resource_accounting_contract`, `typed_rejection_contract`, `provenance_contract`;
- any concrete model runner or final surface.

A later reviewed freeze may pin `resource_accounting_contract` only with
explicit numeric rules and without setting execution flags true while other
blockers remain.
