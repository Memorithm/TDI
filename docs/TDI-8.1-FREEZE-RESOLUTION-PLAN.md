# TDI-8.1 configuration freeze — resolution plan

Status: **operational plan only — not H8-A/H8-B evidence and not a TDI-8.2 authorization**.

Tracks programme issue #87 and contract `docs/tdi8.1-configuration-freeze.json`.
Machine-readable companion: `docs/tdi8.1-blocker-evidence-classes.json`
(aggregated by `docs/tdi-freeze-progress-summary.json`).

## Rules

- Pin a field only from reviewable Development/Validation (non-holdout) evidence already in-repo or produced by existing qualification gates.
- Never invent dimensions, seeds, budgets, horizons, or sample counts to clear `unresolved_blocking`.
- Keep `tdi8_2_execution_authorized`, holdout/runner/token flags hard-false until a future human-only TDI-8.2 stage.
- `scientific_status` stays `unresolved_blocking` until all 17 fields are `pinned`.
- Each unresolved field below names a **required evidence class**. That class is an evidence *kind*, not a scientific value.

## Field ledger

| Field | Pinability now | Required evidence class | Required evidence |
| --- | --- | --- | --- |
| `a3_event_store_read_cleanup_policy` | **pinned** (`tdi8.1-a3-qualified-adapter-v1`) | `qualified_software_policy` | A3 adapter preflight + PRs #138/#172 |
| `closed_rejection_taxonomy` | **pinned** (`SymbolicRejectionCode`) | `closed_rejection_vocabulary` | Symbolic rejections docs + PRs #162/#175 |
| `degenerate_replicate_policy` | **pinned** (`tdi8.1-reject-zero-baseline-bootstrap-replicates-v1`) | `qualified_software_policy` | Percentile preflight + PR #113 |
| `paired_interval_method` | blocked | `comparative_estimator_selection` | PR #113 / `docs/TDI-8.1-PERCENTILE-INTERVAL-PREFLIGHT.md` qualify a conservative percentile *candidate* and explicitly do **not** freeze it as the final estimator. No later merged tranche compares admissible constructions. |
| `paired_resampling_replicate_count` | blocked | `alpha_powered_dev_val_simulation` | Dev/Val under `alpha = 0.05 / 9` |
| `paired_resampling_seed` | blocked | `explicit_domains_seed_declaration` | Explicit Domains seed |
| `recurrent_dimension_and_parameters` | blocked | `matched_budget_dev_val_selection` | Matched-budget Dev/Val selection |
| `readout_coordinates` | blocked | `target_blind_coordinate_selection` | Target-blind coords without leakage |
| `a2_capacity_and_projection` | blocked | `occupancy_collision_under_concrete_projection` | Occupancy/collision under concrete projection |
| `a3_vsa_width_role_seed_and_fusion` | blocked | `independent_vsa_parameter_selection` | Independent of pinned event policy |
| `matched_dynamic_memory_budget` | blocked | `exact_arm_accounting_equality` | Exact A1/A2/A3 accounting equality |
| `short_medium_long_numeric_horizons` | blocked | `numeric_horizon_plan` | Numeric `HorizonPlan` |
| `late_retrieval_deficit_definition` | blocked | `pre_target_observable_deficit` | Pre-target observable deficit |
| `intervention_sites_and_recovery_observable` | blocked | `task_label_preserving_interventions` | Task-label-preserving interventions |
| `development_population_domain_and_sample_count` | blocked | `disjoint_population_domain_and_n` | Domain + N; no final seeds |
| `validation_population_domain_and_sample_count` | blocked | `disjoint_population_domain_and_n` | Disjoint from development |
| `final_population_domain_and_sample_count` | blocked | `final_domain_rules_only` | Domain **rules** only |

## Holdout boundary

TDI-8.2 remains absent.

## Why remaining fields stay unresolved

The three pinned fields are closed software policies already qualified on merged
Dev/Val surfaces. The remaining fourteen require experimental choices
(dimensions, seeds, budgets, horizons, sample counts, or a selected interval
estimator) that do not exist as frozen values in-repo. Inventing them to clear
`unresolved_blocking` is forbidden. The readiness / freeze validators now
fail-closed if any unauthorized field is pinned, if a pin is missing a
non-empty `evidence` block (PR citation + existing in-repo file), if the
pinned count drops below 3, or if `scientific_status` / STATUS is silently
upgraded while fields remain `unresolved_blocking`.

Post-#204 scout (after TDI-10.18 on `main`): no newly closed identifier can pin a remaining
field. Holdouts 7.2 / 8.2 stay untouched. Evidence-class inventory + freeze-progress
summary are CI-checked and do not invent values.
