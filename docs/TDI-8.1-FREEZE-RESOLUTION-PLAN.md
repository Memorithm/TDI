# TDI-8.1 configuration freeze — resolution plan

Status: **operational plan only — not H8-A/H8-B evidence and not a TDI-8.2 authorization**.

Tracks programme issue #87 and contract `docs/tdi8.1-configuration-freeze.json`.

## Rules

- Pin a field only from reviewable Development/Validation (non-holdout) evidence already in-repo or produced by existing qualification gates.
- Never invent dimensions, seeds, budgets, horizons, or sample counts to clear `unresolved_blocking`.
- Keep `tdi8_2_execution_authorized`, holdout/runner/token flags hard-false until a future human-only TDI-8.2 stage.
- `scientific_status` stays `unresolved_blocking` until all 17 fields are `pinned`.

## Field ledger

| Field | Pinability now | Required evidence |
| --- | --- | --- |
| `a3_event_store_read_cleanup_policy` | **pinned** (`tdi8.1-a3-qualified-adapter-v1`) | A3 adapter preflight + PRs #138/#172 |
| `closed_rejection_taxonomy` | **pinned** (`SymbolicRejectionCode`) | Symbolic rejections docs + PRs #162/#175 |
| `degenerate_replicate_policy` | **pinned** (`tdi8.1-reject-zero-baseline-bootstrap-replicates-v1`) | Percentile preflight + PR #113 |
| `paired_interval_method` | blocked | Final choice beyond the PR #113 candidate |
| `paired_resampling_replicate_count` | blocked | Dev/Val under `alpha = 0.05 / 9` |
| `paired_resampling_seed` | blocked | Explicit Domains seed |
| `recurrent_dimension_and_parameters` | blocked | Matched-budget Dev/Val selection |
| `readout_coordinates` | blocked | Target-blind coords without leakage |
| `a2_capacity_and_projection` | blocked | Occupancy/collision under concrete projection |
| `a3_vsa_width_role_seed_and_fusion` | blocked | Independent of pinned event policy |
| `matched_dynamic_memory_budget` | blocked | Exact A1/A2/A3 accounting equality |
| `short_medium_long_numeric_horizons` | blocked | Numeric `HorizonPlan` |
| `late_retrieval_deficit_definition` | blocked | Pre-target observable deficit |
| `intervention_sites_and_recovery_observable` | blocked | Task-label-preserving interventions |
| `development_population_domain_and_sample_count` | blocked | Domain + N; no final seeds |
| `validation_population_domain_and_sample_count` | blocked | Disjoint from development |
| `final_population_domain_and_sample_count` | blocked | Domain **rules** only |

## Holdout boundary

TDI-8.2 remains absent.
