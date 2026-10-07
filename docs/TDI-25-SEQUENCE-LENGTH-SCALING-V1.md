# TDI-25 slice 38 — sequence-length scaling (`tdi25-sequence-length-scaling-v1`)

Status: experimental, non-final, Development/Validation only. Software
semantics only: no training, no length selection, no protected/final/holdout
access, no scientific claim. Holdouts TDI-7.2/8.2/9.2 are untouched.

## Contract

- Population: `tdi25-matched-reference-population-v1`, bounded by
  `StageCPreflightBudget`; every required synthesis family and seed block.
- Lengths `SEQUENCE_SCALING_LENGTHS = [2, 4, 8]`, preregistered; all reported.
- Mask policies `[Full, Causal]`, consumed unchanged through the shared
  TDI-24 normalizer via `normalize_arm_row` (`NORMALIZER_CONTRACT`,
  `MASKING_CONTRACT`); no arm-specific normalization branch.
- Arms: matched T6 and matched C6 (fixed weights `(1, 0, 1)`), reference
  capacities unchanged at every length.
- Each block is cut into non-overlapping windows of `L` consecutive cases.
  Row `i` of a window holds the arm score of query `i` (with its query
  position) against key `j` (with its key position) for every `j` in the
  window. The diagonal must reproduce the matched primary score bit for bit
  (`diagonal_reference_drift`).
- Exact accounting per (family, seed block, length, policy, arm) cell:
  `windows * L + unwindowed = cases_per_block`, `unwindowed < L`,
  `score_evaluations = windows * L^2`, `normalizer_calls = windows * L`,
  `masked_entries = windows * L(L-1)/2` (causal) or `0` (full); masked
  probabilities must be exactly zero (`mask_drift`). T6 and C6 consume
  identical resources.
- Each normalized row sums to 1 within `1e-12`.

## Validation

`validate_sequence_length_scaling_report` rejects `contract_drift`,
`population_drift`, `normalizer_contract_drift`, `length_set_drift`,
`capacity_mismatch`, `cell_count`, `cell_order`, `cost_accounting_drift`,
`row_sum_drift`, the protected/training/claim/non-final flags, and
regenerates every cell (`case_evidence_drift`).

## Recorded degeneracies

- The first causal row of every window has one admissible key, so causal
  self-retrieval is inflated and not comparable with full-mask rows.
- Self-retrieval (diagonal argmax) is a software diagnostic, not a target:
  the matched target is defined per case, not per window, so these counts
  carry no accuracy meaning.
- With the smoke budget (8 cases per block) every length divides the block,
  so the unwindowed remainder is zero; the remainder path is exercised in
  the tests with 6 cases per block.
- Row-sum errors are at the `1e-16` level; the tolerance is a software guard.
