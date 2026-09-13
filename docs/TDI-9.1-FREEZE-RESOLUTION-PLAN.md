# TDI-9.1 configuration freeze — resolution plan

Status: **operational plan only — not H9-A/H9-B evidence and not a TDI-9.2 authorization**.

Tracks programme issue #92 and contract `docs/tdi9.1-configuration-freeze.json`.
Machine-readable companion: `docs/tdi9.1-blocker-evidence-classes.json`
(aggregated by `docs/tdi-freeze-progress-summary.json`).

## Rules

- Pin a field only from reviewable Development/Validation (non-holdout) evidence already in-repo or produced by existing qualification gates.
- Never invent schedules, thresholds, difficulty values, envelopes, or sample counts to clear `unresolved_blocking`.
- Keep all TDI-9.2 / final seed/dataset/runner/result flags hard-false until the future public-entropy gate is frozen and later satisfied.
- Human confirmation tokens remain intentionally absent from TDI-9.
- `scientific_status` stays `unresolved_blocking` until all registered fields are `pinned`.
- Each unresolved field below names a **required evidence class**. That class is an evidence *kind*, not a scientific value.
- **TDI-9.3 (#195) is non-authorizing** for every TDI-9.1 freeze pin (see inventory `non_authorizing_surfaces` / `non_authorizing_rule`).

## Field ledger

| Field | Pinability now | Required evidence class | Required evidence |
| --- | --- | --- | --- |
| `closed_rejection_taxonomy` | **pinned** (`ReferenceRejectionCode`) | `closed_rejection_vocabulary` | Docs + PR #140 |
| `p1_p2_p3_difficulty_parameters` | blocked | `nonfinal_dev_val_difficulty_selection` | Non-final Dev/Val selection |
| `c0_c1_schedules` | blocked | `nonfinal_dev_val_schedule_selection` | Non-final Dev/Val selection |
| `c2_c3_thresholds_and_verification_cadence` | blocked | `nonfinal_dev_val_threshold_cadence_selection` | Non-final Dev/Val selection |
| `permitted_observation_vector` | blocked | `experimental_observation_subset_selection` | `PolicyObservation` is a closed 8-field carrier (#123) but this freeze field requires an explicit *experimental subset*. C2 uses step/residual/delta/margin; C3 also uses verifier/checkpoints. No merged tranche selects the scientific subset. TDI-9.3 Boolean IR (#195) is **not** this class. |
| `common_max_compute_memory_envelopes` | blocked | `numeric_resource_envelope_constants` | `ResourceEnvelope` is a caller-supplied type, not frozen numeric constants |
| `paired_interval_method` / replicate / seed | blocked | `family_wise_coverage_under_frozen_alpha` / `explicit_domains_seed_declaration` | Family-wise coverage under frozen TDI-9.0 alpha rules |
| `development_validation_population_domains` | blocked | `disjoint_population_domains` | Disjoint domains; no final seeds |
| `future_public_entropy_source_event_encoding` | blocked | `pre_public_entropy_freeze` | Freeze before the public value is knowable |
| `final_seed_derivation_contract` | blocked | `deterministic_final_seed_map` | Deterministic map from future entropy |
| `agent_search_safe_policy_mutation_contract` | blocked | `closed_agent_search_mutation_contract` | Not specified as a closed contract in merged docs/code; TDI-9.3 (#195) does not authorize |
| `h9_classifier_aggregation_plumbing` | blocked | `h9_classifier_software_transcription` | H9 rules exist in the frozen TDI-9.0 preregistration; software transcription plumbing is not implemented |

## Holdout boundary

No TDI-9.2 runner, seed list, dataset, or result payload. No TDI-7.2 / TDI-8.2 contact.

## Why remaining fields stay unresolved

The one pinned field is the closed `ReferenceRejectionCode` vocabulary (#140).
`PolicyObservation` / `ResourceEnvelope` are closed *types*, not selected
experimental vectors or numeric envelopes. Difficulty, schedules, thresholds,
populations, entropy, and H9 plumbing remain unspecified as freeze values.
Inventing them is forbidden. The readiness / freeze validators fail-closed if
any unauthorized field is pinned, if a pin is missing a non-empty `evidence`
block (PR citation + existing in-repo file), if the pinned count drops below
1, if pin evidence cites `docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md`, or if
`scientific_status` / STATUS is silently upgraded while fields remain
`unresolved_blocking`. TDI-9.3 (#195) does not authorize a 9.1 pin.

Post-#204 scout (after TDI-10.18 on `main`): no newly closed identifier can pin a remaining
field. Holdouts 7.2 / 8.2 / 9.2 stay untouched. Evidence-class inventory + freeze-progress
summary are CI-checked and do not invent values.

Integrity gate: `scripts/check-tdi9.1-readiness.sh` (workflow `tdi9-readiness.yml`).
