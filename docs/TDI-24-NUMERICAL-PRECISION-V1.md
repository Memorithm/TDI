# TDI-24 slice 44 — numerical precision study (`tdi24-numerical-precision-v1`)

Status: experimental, non-final, Development/Validation only. No training, no
protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched.

## Contract

- Paired arms `PRECISION_ARMS`: C6 `(1, 0, 1)` and direct-only `(1, 0, 0)`,
  matched zero-trainable capacity (`reference_c6`).
- Reference: the unchanged f64 `chiral_score` (fail-closed products and
  sums). Candidate: `chiral_score_f32`, which rounds the six query, six key
  components and the weights to f32 and evaluates `alpha*s + beta*m + gamma*chi`
  in f32 with the reference term order.
- Explicit tolerance, declared before any run: a finite f32 score is within
  tolerance iff `|score_f32 - score_f64| <= PRECISION_BOUND_FACTOR *
  f32::EPSILON * condition`, with `PRECISION_BOUND_FACTOR = 32` and
  `condition` the sum of the absolute weighted products entering the score
  (a standard forward-error bound for short f32 dot products with rounded
  inputs, with margin).
- Case groups: the four Stage-C task families (bounded preflight stream) and
  the slice-43 reflection adversarial set (both members of every pair across
  its declared grid).
- Failure accounting per (group, arm): every case is exactly one of within
  tolerance, tolerance failure or non-finite f32 (overflow) — none is
  dropped — plus decision sign flips versus f64, maximum absolute error and
  maximum error-to-bound ratio.

## Validation

`validate_numerical_precision_report` rejects `contract_drift`,
`tolerance_drift`, `capacity_mismatch`, `cell_count`, `grid_drift`,
`failure_accounting_drift` (a dropped or double-counted case),
`error_statistic_drift` (non-finite statistics, or a failure count that
disagrees with the error-to-bound ratio), the protected/training/claim/
non-final flags, and regenerates every cell (`case_evidence_drift`).

## Recorded degeneracies

- At the smoke budget (4 pairs per family) every case on both splits is
  within the declared bound, with no overflow and no decision flip; the
  largest error-to-bound ratio is about `0.02` and the largest absolute
  error below `1e-6`. The bound is therefore loose by roughly 50x on this
  population; it is a declared guard, not a fitted tolerance.
- The Stage-C reflection-discriminative fixtures are dyadic, so f32 is
  exact there (error `0`).
- The adversarial near-boundary ratios `0.99`/`1.01` and the near-achiral
  scale `1e-3` keep a decision margin far above f32 rounding at these
  magnitudes; f32 flips would require margins near `1e-7 * condition`.
- Only the reference arms are studied: there is no trained arm and no
  mixed-precision accumulation strategy.
