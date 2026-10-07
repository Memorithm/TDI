# TDI-25 slice 39 — data-volume scaling (`tdi25-data-volume-scaling-v1`)

Status: experimental, non-final, Development/Validation only. Software
semantics only: no training, no count selection, no protected/final/holdout
access, no scientific claim. Holdouts TDI-7.2/8.2/9.2 are untouched.

## Contract

- Population: `tdi25-matched-reference-population-v1`; every required
  synthesis family and every seed block of the `StageCPreflightBudget`.
- Preregistered per-block sample counts `DATA_VOLUME_COUNTS = [8, 16, 32]`;
  all reported. The budget's `cases_per_block` is not consumed; only
  `seed_blocks` is.
- No adaptive arm-specific allocation: T6 and C6 always score the same cases
  at the same count (`n_cases = seed_blocks * count` for both arms).
- Nesting: the cases at a smaller count are verified to be exact prefixes of
  the largest count (sealed outcomes compared; `nesting_drift`).
- Reference capacities unchanged (`reference_t6`, `reference_c6`).

## Validation

`validate_data_volume_scaling_report` rejects `contract_drift`,
`population_drift`, `count_set_drift`, `capacity_mismatch`, `cell_count`,
`cell_order`, `allocation_drift`, the protected/training/claim/non-final
flags, and regenerates every cell (`case_evidence_drift`).

## Recorded degeneracies

- Because volumes are nested, match counts are monotone in volume by
  construction; this is a software property, not a learning curve (there is
  no training).
- The fixed, non-trained matched arms do not change with volume; only the
  sampling resolution of the reported counts changes.
- The largest count (32) stays below `MAX_CASES_PER_RUN = 64`.
