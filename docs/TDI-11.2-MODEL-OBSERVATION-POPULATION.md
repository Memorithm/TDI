# TDI-11.2 model-observation Development/Validation population-derivation scaffolding

Status: **non-executing software scaffolding — NOT A PIN and not a freeze**.

This tranche lands an exact authorized-domain taxonomy and fail-closed
caller-supplied population-derivation contract candidate for prospective
TDI-11.2 model-observation instrumentation. It does **not** authorize model
execution, invent scientific population sizes/strata/seeds, create Final /
confirmatory / holdout populations, or move any of the twelve TDI-11.2 freeze
fields off `unresolved_blocking`.

## Scientific / stage boundary

- `model_execution_authorized` remains **false**.
- `final_execution_authorized` remains **false**.
- Freeze pins stay **0/12**.
- Holdouts TDI-7.2 / TDI-8.2 / TDI-9.2 remain forbidden contact.
- Distinct from rejection/provenance (#213), resource-accounting (#216), and
  registry/timing/H11-A eligibility (#218) scaffolds.

Orchestrator holdout #186 continues to forbid AUTO_MERGE of a TDI-11.2 freeze
that invents model/observation pins. This scaffolding PR must keep every field
`unresolved_blocking`.

## What landed

### Exact authorized domain taxonomy (candidate only)

`AUTHORIZED_POPULATION_DOMAIN_KEYS` enumerates the exact pre-arm
`allowed_domains` list:

| Key | Meaning |
| --- | --- |
| `Development` | Non-final development instrumentation domain |
| `Validation` | Non-final validation instrumentation domain |

`Final` / holdout / confirmatory labels are rejected
(`PopulationForbiddenDomain`, `0x0807`). Forbidden surface tokens such as
`final_seed_list`, `final_population`, and `concrete_model_runner` fail closed
(`PopulationForbiddenSurfaceToken`, `0x0808` /
`PopulationFinalMaterialLeak`, `0x080B`).

### Caller-supplied derivation contract (candidate only)

`ModelObservationPopulationDerivationContract` admits caller-supplied strata
(`domain` + `stratum_id` + `seed_space_key`) under exact integrity rules:

- non-empty contract;
- exact coverage of both authorized domains;
- unique stratum ids;
- unique `(domain, seed_space_key)` pairs;
- deterministic domain-separated seed-commitment framing (engineering only;
  does not invent scientific seeds).

Non-authorizing candidate id: `ModelObservationPopulationDerivationContract`
(`CANDIDATE_POPULATION_DERIVATION_CONTRACT`). This does **not** pin
`development_validation_population_derivation`.

## Qualification surfaces

- module: `tdi-ai/src/hallucination_model_observation_population.rs`
- rejection codes: `0x08xx` in `tdi-ai/src/hallucination_model_observation_rejections.rs`
- tests: `tdi-ai/tests/tdi11_model_observation_population.rs`
- gate: `scripts/check-tdi11.2-model-observation-population.sh`
- workflow: `.github/workflows/tdi11.2-model-observation-population.yml`

The population module is intentionally **not** promoted into the stable
`tdi-ai` `lib.rs` API (same posture as rejection/provenance/accounting/registry
scaffolds).

## Explicitly not frozen / not authorized

- any scientific population size, stratum catalogue, or seed list;
- model artifact, adapter, tokenizer/template, decoding, prompt serializer;
- exact observation registry / timing / H11-A eligibility;
- numeric resource envelopes;
- `development_validation_population_derivation`,
  `resource_accounting_contract`, `typed_rejection_contract`,
  `provenance_contract`;
- any concrete model runner or Final surface.

A later reviewed freeze may pin `development_validation_population_derivation`
only with an explicit deterministic Dev/Val map and without setting execution
flags true while other blockers remain.
