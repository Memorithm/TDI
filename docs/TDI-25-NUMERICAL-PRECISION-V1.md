# TDI-25 slice 46 — numerical precision study (`tdi25-numerical-precision-v1`)

Status: experimental, non-final, Development/Validation only (Phase E). No
training, no protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched. Slices 49–50 remain pre-arm/decision
artifacts only.

## Contract

- Arms: T6 (`tdi25-matched-t6-evaluator-v1`, factorized score
  `(v + Q x omega).R + omega.(M + P x R)`) and C6 (`(1, 0, 1)`), matched
  reference capacities.
- Reference: the unchanged f64 matched evaluator. Candidate: f32 kernels
  that round every input to f32 and keep the reference operation order —
  `t6_score_f32` (new, this slice) and, for C6, the upstream TDI-24 slice-44
  `chiral_score_f32` (TDI-25 does not redefine the chiral contract).
- Tolerance, reused unchanged from TDI-24 slice 44 (no new parameter): a
  finite f32 score is within tolerance iff
  `|score_f32 - score_f64| <= PRECISION_BOUND_FACTOR * f32::EPSILON * condition`,
  `PRECISION_BOUND_FACTOR = 32`, where `condition` is the sum of the absolute
  elementary products of the score expansion (for T6:
  `sum_i (|v_i| + |Q x omega|abs_i) |R_i| + |omega_i| (|M_i| + |P x R|abs_i)`,
  with `|a x b|abs_i = |a_j b_k| + |a_k b_j|`).
- Input classes, all reported: the clean matched input, and every slice-43
  translation/origin stress (origin shift, key re-reduction, query
  re-reduction) at the slice-43 offsets `[1, 1e3, 1e6]`, with the slice-43
  per-case directions and contract seed.
- Complete failure accounting per (family, block, input class, arm): within
  tolerance + tolerance failures + non-finite f32 = cases (none dropped),
  plus sign flips versus f64, max absolute error and max error-to-bound
  ratio.

## Validation

`validate_numerical_precision_report` rejects `contract_drift`,
`population_drift`, `seed_drift`, `tolerance_drift` (bound factor differs
from the TDI-24 pin), `capacity_mismatch`, `cell_count`, `grid_drift`,
`failure_accounting_drift`, `error_statistic_drift` (non-finite statistics
or a failure count inconsistent with the error-to-bound ratio), the
protected/training/claim/non-final flags, and regenerates every cell
(`case_evidence_drift`).

## Recorded degeneracies

- Clean matched inputs are small half-integers: both f32 kernels reproduce
  the f64 primary exactly (error `0`).
- Under every slice-43 stress, on both splits, every case of both arms is
  within the declared bound: no tolerance failure and no f32 overflow. The
  largest error-to-bound ratio is about `0.03`, so the bound is loose by
  roughly 30x on this population (a declared guard, not a fitted
  tolerance). Absolute errors reach about `0.5` (T6) and `0.74` (C6) at
  offset `1e6`, where carrier/position magnitudes are about `1e6`.
- C6 reads no position, so its f32 error under the origin shift is `0`.
- T6 shows one sign flip in 16 stressed cells (offsets `1`, `1e3`, `1e6`):
  these are cases whose f64 score is zero or near zero, where any rounding
  changes the sign; C6 shows none. Flips are reported, not interpreted.
- Only the fixed reference arms are studied: no trained arm, no
  mixed-precision accumulation strategy.
