# TDI-24 slice 42 — input-noise robustness (`tdi24-input-noise-robustness-v1`)

Status: experimental, non-final, Development/Validation only. No training, no
protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched.

## Contract

- Declared perturbation families `NOISE_FAMILIES`: `Isotropic` (all six
  chiral components), `EvenOnly` (mirror-even components), `OddOnly`
  (mirror-odd components). Declared amplitudes
  `NOISE_AMPLITUDES = [1e-3, 1e-2, 1e-1]`. The full grid is reported.
- Deterministic perturbation: each component of the query (operand 0) and
  key (operand 1) of case `i` receives a value in `[-a, a]` from a splitmix64
  stream keyed by the case index, operand and component. The stream seed is
  the FNV-1a 64 hash of the contract pin (`input_noise_seed`); there is no
  free seed constant.
- Paired arms: C6 `(1, 0, 1)` and direct-only `(1, 0, 0)` score the identical
  perturbed inputs; the perturbation never depends on the arm. Matched
  zero-trainable capacity (`reference_c6`).
- Metric: label-free decision stability. The sealed task oracle stays inside
  the evaluator, so the study counts cases whose perturbed score keeps the
  clean score's sign, plus the maximum absolute score change, per (family,
  amplitude, arm, task family). The clean C6 score must reproduce the
  Stage-C reference bit for bit (`clean_reference_drift`).

## Validation

`validate_input_noise_robustness_report` rejects `contract_drift`,
`seed_drift`, `capacity_mismatch`, `cell_count`, `grid_drift`,
`paired_count_drift`, `score_change_drift`, the protected/training/claim/
non-final flags, and regenerates every cell (`case_evidence_drift`).

## Recorded degeneracies

- Stability is measured against the clean decision, not against the oracle:
  this is a robustness diagnostic, not accuracy under noise.
- At the smoke budget (4 pairs per family, 8 cases per task family) every
  cell is fully sign-stable except direct-only on reflection-nuisance at
  amplitude `1e-1` (isotropic 6/8, odd-only 7/8 on Development). Score
  changes scale roughly linearly with amplitude. Descriptive only.
- Amplitudes are absolute, not relative to each case's norm.
