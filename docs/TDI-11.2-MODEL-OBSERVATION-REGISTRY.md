# TDI-11.2 model-observation registry / timing / H11-A eligibility scaffolding

Status: **non-executing software scaffolding — NOT A PIN and not a freeze**.

This tranche lands exact inherited source-class taxonomy wiring, a fail-closed
caller-supplied channel registry candidate, a fail-closed per-channel timing
contract candidate, and an exact H11-A pre-assertion eligibility classifier
candidate. It does **not** authorize model execution, invent scientific channel
names, or move any of the twelve TDI-11.2 freeze fields off
`unresolved_blocking`.

Distinct from:

- #213 rejection + provenance scaffolding;
- #216 resource-accounting scaffolding.

## Scientific / stage boundary

- `model_execution_authorized` remains **false**.
- `final_execution_authorized` remains **false**.
- Freeze pins stay **0/12**.
- Holdouts TDI-7.2 / TDI-8.2 / TDI-9.2 remain forbidden contact.
- Inherited source *classes* from `docs/tdi11.2-prearm.yaml` are enumerated
  exactly; exact channel names remain caller-supplied and unpinned.

Orchestrator holdout #186 continues to forbid AUTO_MERGE of a TDI-11.2 freeze
that invents model/observation pins. This scaffolding PR must keep every field
`unresolved_blocking`.

## What landed

### Exact inherited source-class taxonomy (candidate only)

`INHERITED_OBSERVATION_SOURCE_CLASS_KEYS` enumerates the eight pre-arm classes:

| Key | Class |
| --- | --- |
| `decoder_statistics` | DecoderStatistics |
| `hidden_state_summary` | HiddenStateSummary |
| `visible_evidence_summary` | VisibleEvidenceSummary |
| `verifier_result` | VerifierResult |
| `retrieval_tool_result` | RetrievalToolResult |
| `action_history` | ActionHistory |
| `runtime_resource_summary` | RuntimeResourceSummary |
| `resample_summary` | ResampleSummary |

This is **not** an exact channel registry pin.

### Caller-supplied channel registry (candidate only)

`ModelObservationChannelRegistry` admits a non-empty, unique, caller-supplied
channel→class map. Empty / duplicate / undeclared lookups fail closed via
`0x07xx` rejection codes.

Non-authorizing candidate id: `ModelObservationChannelRegistry`
(`CANDIDATE_OBSERVATION_REGISTRY`). Does **not** pin
`exact_observation_registry`.

### Per-channel timing contract (candidate only)

`ModelObservationTimingContract` requires exact coverage of a registry's
channels with one of:

- `require_strictly_before_assertion`
- `allow_post_hoc_diagnostic`

Admission refuses unknown assertion boundaries, observation-at-boundary events,
and post-assertion events on strict-primary channels.

Non-authorizing candidate id: `ModelObservationTimingContract`
(`CANDIDATE_OBSERVATION_TIMING_CONTRACT`). Does **not** pin
`exact_observation_timing`.

### Exact H11-A eligibility classifier (candidate only)

`ModelObservationH11AEligibilityRule` implements:

- `PrimaryEligible` iff `event_index < first_assertion_event`
- `PostHocOnly` iff `event_index > first_assertion_event`
- fail-closed on unknown boundary or observation at the assertion boundary

Non-authorizing candidate id: `ModelObservationH11AEligibilityRule`
(`CANDIDATE_H11A_ELIGIBILITY_RULE`). Does **not** pin
`primary_pre_assertion_eligibility`.

## Qualification surfaces

- module: `tdi-ai/src/hallucination_model_observation_registry.rs`
- rejection codes: `0x07xx` in `tdi-ai/src/hallucination_model_observation_rejections.rs`
- tests: `tdi-ai/tests/tdi11_model_observation_registry.rs`
- gate: `scripts/check-tdi11.2-model-observation-registry.sh`
- workflow: `.github/workflows/tdi11.2-model-observation-registry.yml`

The registry module is intentionally **not** promoted into the stable
`tdi-ai` `lib.rs` API (same posture as rejection/provenance/accounting scaffolds).

## Explicitly not frozen / not authorized

- any scientific channel-name registry;
- model artifact, adapter, tokenizer/template, decoding, prompt serializer;
- numeric resource envelopes;
- Dev/Val population derivation;
- `exact_observation_registry`, `exact_observation_timing`,
  `primary_pre_assertion_eligibility`, `resource_accounting_contract`,
  `typed_rejection_contract`, `provenance_contract`;
- any concrete model runner or final surface.

A later reviewed freeze may pin registry / timing / eligibility fields only with
explicit reviewed values and without setting execution flags true while other
blockers remain.
