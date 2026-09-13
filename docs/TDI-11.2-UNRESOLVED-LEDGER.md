# TDI-11.2 unresolved ledger

Status: **content-addressed snapshot of the unresolved contract — NOT A PIN and not a freeze**.

This ledger documents the current 12-field TDI-11.2 model/observation contract
without resolving any scientific value. It does **not** authorize model
execution, invent an adapter/tokenizer/decoding pin, or move
`model_execution_authorized` off `false`.

## Content address

Canonical SHA-256 of `docs/tdi11.2-model-observation-freeze.json` (raw UTF-8
file bytes, sidecar `docs/tdi11.2-model-observation-freeze.sha256`):

`dec8bd504a903b0125fdb88f66821940d286358eec6d59ee1a194fc6c63e9421`

Machine-readable sidecar:

```text
dec8bd504a903b0125fdb88f66821940d286358eec6d59ee1a194fc6c63e9421  docs/tdi11.2-model-observation-freeze.json
```

Recompute with `sha256sum docs/tdi11.2-model-observation-freeze.json`.
`scripts/check-tdi11.2-readiness.py` fail-closes if this digest drifts.

## Unresolved field registry (12/12)

All fields remain `unresolved_blocking` with `value: null` and
`pin_provenance: null`. No identifier closed by TDI-10.13/#198, TDI-10.14/#200, TDI-10.15,
TDI-10.16, TDI-10.17, TDI-10.18, or any earlier merged tranche supplies a reviewable exact value.

| Field | Status | Why still blocked |
| --- | --- | --- |
| `model_artifact_identity` | unresolved_blocking | No immutable local/open model artifact has been selected or hashed. |
| `adapter_identity_and_version` | unresolved_blocking | No reviewed adapter implementation identity/version exists. |
| `tokenizer_and_template_identity` | unresolved_blocking | No tokenizer/chat-template identity has been frozen. |
| `decoding_configuration` | unresolved_blocking | No complete decoding/generation settings have been chosen. |
| `prompt_serializer_version` | unresolved_blocking | No serializer revision is pinned. |
| `exact_observation_registry` | unresolved_blocking | Inherited source *classes* are listed in the pre-arm; the exact channel registry is not selected. |
| `exact_observation_timing` | unresolved_blocking | Event timing per channel is unspecified. |
| `primary_pre_assertion_eligibility` | unresolved_blocking | H11-A pre-assertion eligibility rule is not frozen as an executable contract. |
| `development_validation_population_derivation` | unresolved_blocking | Dev/Val derivation is not a closed deterministic map. |
| `resource_accounting_contract` | unresolved_blocking | Numeric envelopes / accounting rules are caller- or later-freeze items. |
| `typed_rejection_contract` | unresolved_blocking | TDI-11.2 has no closed model-observation rejection vocabulary of its own. TDI-8.1 / TDI-9.1 rejection pins do not transfer. |
| `provenance_contract` | unresolved_blocking | Machine-readable provenance schema for model traces is not frozen. |

## Execution flags (hard)

- `model_execution_authorized`: **false**
- `final_execution_authorized`: **false**
- `scientific_status`: `unresolved_blocking`

Readiness fail-closes if either execution flag is true while any field above
remains `unresolved_blocking`, or if a forbidden surface
(`concrete_model_runner`, `final_seed_list`, `final_dataset`, `final_runner`,
`final_result_payload`) appears.

## Holdout boundary

No TDI-7.2 / TDI-8.2 / TDI-9.2 contact. This ledger is documentation and
integrity only.
