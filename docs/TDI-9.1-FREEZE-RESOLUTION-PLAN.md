# TDI-9.1 configuration freeze — resolution plan

Status: **operational plan only — not H9-A/H9-B evidence and not a TDI-9.2 authorization**.

Tracks programme issue #92 and contract `docs/tdi9.1-configuration-freeze.json`.

## Rules

- Pin a field only from reviewable Development/Validation (non-holdout) evidence already in-repo or produced by existing qualification gates.
- Never invent schedules, thresholds, difficulty values, envelopes, or sample counts to clear `unresolved_blocking`.
- Keep all TDI-9.2 / final seed/dataset/runner/result flags hard-false until the future public-entropy gate is frozen and later satisfied.
- Human confirmation tokens remain intentionally absent from TDI-9.
- `scientific_status` stays `unresolved_blocking` until all registered fields are `pinned`.

## Field ledger

| Field | Pinability now | Required evidence |
| --- | --- | --- |
| `closed_rejection_taxonomy` | **pinned** (`ReferenceRejectionCode`) | Docs + PR #140 |
| `p1_p2_p3_difficulty_parameters` | blocked | Non-final Dev/Val selection |
| `c0_c1_schedules` | blocked | Non-final Dev/Val selection |
| `c2_c3_thresholds_and_verification_cadence` | blocked | Non-final Dev/Val selection |
| `permitted_observation_vector` | blocked | Explicit subset of leakage-safe observations |
| `common_max_compute_memory_envelopes` | blocked | Matched envelopes under accounting |
| `paired_interval_method` / replicate / seed | blocked | Family-wise coverage under frozen TDI-9.0 alpha rules |
| `development_validation_population_domains` | blocked | Disjoint domains; no final seeds |
| `future_public_entropy_source_event_encoding` | blocked | Freeze before the public value is knowable |
| `final_seed_derivation_contract` | blocked | Deterministic map from future entropy |
| `agent_search_safe_policy_mutation_contract` | blocked | Dev/Val-only search with executed evidence authoritative |
| `h9_classifier_aggregation_plumbing` | blocked | Software transcription of frozen H9 rules |

## Holdout boundary

No TDI-9.2 runner, seed list, dataset, or result payload. No TDI-7.2 / TDI-8.2 contact.
