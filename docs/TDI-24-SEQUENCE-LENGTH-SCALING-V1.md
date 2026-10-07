# TDI-24 slice 39 — sequence-length scaling (`tdi24-sequence-length-scaling-v1`)

Status: experimental, non-final, Development/Validation only. Software
semantics only: no training, no length selection, no protected/final/holdout
access, no scientific claim. Holdouts TDI-7.2/8.2/9.2 are untouched.

## Contract

- Lengths `SEQUENCE_LENGTHS = [2, 4, 8]`, preregistered; every length is
  reported, none is selected.
- Mask policies: `MaskPolicy::Full` and `MaskPolicy::Causal`, both through the
  shared `tdi24_attention` normalizer (`NORMALIZER_CONTRACT`,
  `MASKING_CONTRACT`).
- Arms: C6 `(1, 0, 1)` and its direct-only arm `(1, 0, 0)`, fixed weights,
  matched zero-trainable capacity (`TrainableCapacity::reference_c6()`).
- Per family, the bounded Stage-C case stream is cut into non-overlapping
  windows of length `L`; the remainder is counted as `unwindowed_cases`.
- Cost accounting per cell is exact: `windows * L + unwindowed = cases`,
  `score_evaluations = windows * L^2`, `normalizer_calls = windows * L`,
  `masked_entries = windows * L(L-1)/2` under causal masking and `0` under full.
- Each normalized row must sum to 1 within `1e-12`.

## Validation

`validate_sequence_length_scaling_report` regenerates the evidence and rejects
`contract_drift`, `normalizer_contract_drift`, `length_set_drift`,
`capacity_mismatch`, `cell_count`, `cost_accounting_drift`, `row_sum_drift`,
`case_evidence_drift`, and any set protected/training/claim flag.

## Recorded degeneracies

- Under the causal mask the first row of every window has a single admissible
  key, so its self-retrieval is trivial; causal self-retrieval counts are
  therefore inflated relative to full masking and are not comparable across
  policies.
- Direct-only full-mask self-retrieval is zero on the chiral families in the
  smoke budget (the direct score does not prefer the diagonal there); this is
  recorded, not tuned.
- Row-sum errors are at the `1e-16` level; the `1e-12` tolerance is a
  software guard, not a numerical claim.
