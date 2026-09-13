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
`pin_provenance: null`. Non-executing rejection/provenance *candidates* (#213),
resource-accounting *candidates* (#216), registry/timing/H11-A eligibility
*candidates* (#218), and Dev/Val population-derivation *candidates* (this slice)
do not supply a reviewed freeze pin.
No identifier closed by TDI-10.13/#198, TDI-10.14/#200, TDI-10.15–10.19/#214,
TDI-10.20/#217, TDI-12.0/#210–#215, TDI-11.2/#213/#216/#218, or earlier
merged tranches supplies a reviewable exact freeze value.
Required evidence *classes* (not values) are inventoried in
`docs/tdi11.2-blocker-evidence-classes.json` and rolled into
`docs/tdi-freeze-progress-summary.json`.

| Field | Status | Why still blocked |
| --- | --- | --- |
| `model_artifact_identity` | unresolved_blocking | No immutable local/open model artifact has been selected or hashed. |
| `adapter_identity_and_version` | unresolved_blocking | No reviewed adapter implementation identity/version exists. |
| `tokenizer_and_template_identity` | unresolved_blocking | No tokenizer/chat-template identity has been frozen. |
| `decoding_configuration` | unresolved_blocking | No complete decoding/generation settings have been chosen. |
| `prompt_serializer_version` | unresolved_blocking | No serializer revision is pinned. |
| `exact_observation_registry` | unresolved_blocking | Non-executing candidate `ModelObservationChannelRegistry` now exists (`docs/TDI-11.2-MODEL-OBSERVATION-REGISTRY.md`) but is **not** pinned. Channel names remain caller-supplied; inherited source *classes* alone are not a registry pin. |
| `exact_observation_timing` | unresolved_blocking | Non-executing candidate `ModelObservationTimingContract` now exists (`docs/TDI-11.2-MODEL-OBSERVATION-REGISTRY.md`) but is **not** pinned. |
| `primary_pre_assertion_eligibility` | unresolved_blocking | Non-executing candidate `ModelObservationH11AEligibilityRule` now exists (`docs/TDI-11.2-MODEL-OBSERVATION-REGISTRY.md`) but is **not** pinned. |
| `development_validation_population_derivation` | unresolved_blocking | Non-executing candidate `ModelObservationPopulationDerivationContract` now exists (`docs/TDI-11.2-MODEL-OBSERVATION-POPULATION.md`) but is **not** pinned. Scientific strata/sizes/seeds remain caller-supplied or later-freeze items; Final/holdout populations remain forbidden. |
| `resource_accounting_contract` | unresolved_blocking | Non-executing candidate taxonomy `ModelObservationResourceAccountingContract` now exists (`docs/TDI-11.2-MODEL-OBSERVATION-RESOURCE-ACCOUNTING.md`) but is **not** pinned. Numeric envelopes remain caller- or later-freeze items; TDI-11.1 `unlimited_for_development` does not transfer. |
| `typed_rejection_contract` | unresolved_blocking | Non-executing candidate vocabulary `ModelObservationRejectionCode` now exists (`docs/TDI-11.2-MODEL-OBSERVATION-REJECTIONS.md`) but is **not** pinned. TDI-8.1 / TDI-9.1 rejection pins do not transfer. |
| `provenance_contract` | unresolved_blocking | Non-executing candidate schema `ModelObservationTraceProvenance` now exists (`docs/TDI-11.2-MODEL-OBSERVATION-REJECTIONS.md`) but is **not** pinned. |

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
