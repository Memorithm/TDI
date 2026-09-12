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
| `recurrent_dimension_and_parameters` | blocked | Dev/Val selection under matched-budget discipline; fixture values are not experimental freezes |
| `readout_coordinates` | blocked | Same; target-blind readout coords must be chosen without leakage |
| `a2_capacity_and_projection` | blocked | Occupancy/collision pressure under concrete projection on Dev/Val |
| `a3_vsa_width_role_seed_and_fusion` | blocked | Width/seed/fusion independent of the already-pinned event policy |
| `matched_dynamic_memory_budget` | blocked | Exact accounting equality A1/A2/A3 after concrete layouts |
| `short_medium_long_numeric_horizons` | blocked | Numeric `HorizonPlan` values; labels alone are insufficient |
| `late_retrieval_deficit_definition` | blocked | Pre-target observable deficit formula + orientation |
| `intervention_sites_and_recovery_observable` | blocked | Task-label-preserving intervention set + early recovery descriptor |
| `paired_interval_method` | blocked | Qualify final method beyond the PR #113 candidate |
| `paired_resampling_replicate_count` | blocked | Dev/Val power/coverage choice under `alpha = 0.05 / 9` |
| `paired_resampling_seed` | blocked | Explicit Domains seed, disjoint from any future final domain |
| `degenerate_replicate_policy` | blocked | Explicit handling of zero-baseline / undefined relative effects |
| `development_population_domain_and_sample_count` | blocked | Domain identity + N; no final seeds |
| `validation_population_domain_and_sample_count` | blocked | Domain identity + N; disjoint from development |
| `final_population_domain_and_sample_count` | blocked | Domain **rules** only; no final seed list or result payload |

## Next bounded slices (suggested)

1. Dev/Val matched-budget layout proposal for recurrent + A2 + A3 widths (still non-holdout).
2. Numeric Short/Medium/Long horizons + population domain contracts.
3. Deficit + intervention/recovery freeze from early observables only.
4. Final paired-interval method + replicate/seed/degenerate policy.
5. Readiness gate promotion to `frozen_nonfinal` only after all pins land and exact-head CI is green.

## Holdout boundary

TDI-8.2 remains absent. This plan must not create confirmatory runners, tokens, seed lists, or result payloads.
