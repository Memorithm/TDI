# TDI-7.4 — Gaussian/MMI estimator qualification candidate

Status: **BOUNDED QUALIFICATION CANDIDATE — NOT FROZEN FOR FINAL HOLDOUT**

This document refines the implementation machinery for the frozen TDI-7.4 protocol candidate. It does not authorize the TDI-7.4 final population, does not modify the TDI-7.4 confirmatory question, and contains no confirmatory result.

## 1. Purpose

TDI-7.4 asks whether the static attention-diagnostic block `S` and the early intervention-conditioned recovery block `R` jointly encode more information about the later retrieval deficit `T` than either block alone.

The first implementation must not use an ad-hoc information proxy. It reuses and generalizes the Gaussian / Minimum Mutual Information (MMI) partial-information-decomposition discipline already frozen and exercised by TDI-6.3.

## 2. Source blocks

For each generator and intervention site:

- `S` contains exactly the six static attention diagnostics already used by the TDI-7.1 baseline: mean row entropy, mean normalized row entropy, mean max weight, mean L2 concentration, mean entropy-derived support, and Frobenius norm.
- `R` contains reciprocal-L-infinity recovery at depths `1,2,3,4`.
- `T` is the bounded retrieval-position deficit at depth `5`, identical to the TDI-7.1 target construction.

Depth 4 is the maximal bounded observation horizon strictly before the inherited depth-5 target. This choice is **qualification-only** until separately frozen for a final TDI-7.4 execution.

Task metadata and the intervention-site indicator are not members of `S`; they remain provenance/stratification variables rather than information sources.

## 3. Gaussian mutual information

For scalar target `T` and a non-degenerate vector source `X`, the canonical estimate is

`I(T;X) = 0.5 * log2(var(T) * det(Sigma_X) / det(Sigma_[T,X]))`.

TDI-7.4 computes:

- `I_static = I(T;S)`;
- `I_recovery = I(T;R)`;
- `I_joint = I(T;[S,R])`.

The canonical log determinants use a single-threaded fixed-order Cholesky decomposition. A non-positive covariance pivot at or below `1e-12` fails closed.

## 4. Deterministic rank reduction

The deterministic TDI-7 synthetic generator exposes only a small number of geometry states, so the six static diagnostics can be linearly redundant or nearly collinear. Direct full-rank covariance determinants would therefore be an invalid and numerically fragile implementation assumption.

Before covariance estimation, each source block is centered, scaled to unit RMS when non-constant, and scanned in its declared column order with two-pass modified Gram-Schmidt. A candidate direction is retained only when its residual norm after re-orthogonalization exceeds `1e-10 * sqrt(n)`.

- Constant columns are removed.
- Exactly linearly redundant columns are removed.
- No column reordering or data-dependent pivot search is allowed.
- Two MGS passes are used to suppress finite-precision loss of orthogonality for nearly collinear controls.
- The covariance calculations operate on the resulting orthonormal basis of the retained source subspace, not on the original retained coordinates.
- The retained rank is reported with every estimate.
- If a source retains zero dimensions, the estimator fails closed.

Representing the same retained source subspace in an orthonormal basis is an invertible coordinate change on that subspace and therefore preserves Gaussian mutual information. It also avoids manufacturing a tolerance relaxation or ridge regularization merely to compensate for coordinate ill-conditioning.

This `declared_order_reorthogonalized_mgs_v2` rule supersedes the initial qualification-only one-pass implementation, which selected rank using an orthogonal residual but then returned the original retained coordinates. That mismatch was detected by the independent arithmetic cross-check on bounded data before any final-holdout access.

## 5. Independent arithmetic cross-check

Every point estimate is computed twice:

1. canonical covariance log-determinant / Cholesky path;
2. multiple-correlation path `I(T;X) = -0.5 log2(1-R^2)` where `R^2 = c^T Sigma_X^-1 c / var(T)` and the covariance system is solved by partial-pivot Gaussian elimination.

The two methods must agree within `1e-9` bit for `I_static`, `I_recovery`, and `I_joint`. Otherwise the evaluator fails closed. This inherits the TDI-6.3 cross-method tolerance while extending the second method from scalar sources to vector blocks.

The tolerance was not enlarged in response to the bounded-data disagreement discovered during qualification. Numerical conditioning was corrected instead.

## 6. MMI partial information decomposition

With `I_S = I_static`, `I_R = I_recovery`, and `I_SR = I_joint`:

- `Redundancy = min(I_S, I_R)`;
- `Unique_static = I_S - Redundancy`;
- `Unique_recovery = I_R - Redundancy`;
- `Synergy = I_SR - I_S - I_R + Redundancy`.

The implementation verifies the PID identity

`Redundancy + Unique_static + Unique_recovery + Synergy = I_joint`

within `1e-9` bit and rejects materially negative components. Tiny negative round-off within tolerance is clamped to zero and reported only through canonical numeric output.

This is an MMI/Gaussian working-model decomposition. It is not evidence about causality, MMI as a universal PID definition, human cognition, or real-world language models.

## 7. Bootstrap

The bounded qualification evaluator uses a deterministic generator-level bootstrap. Both intervention-site records from a sampled generator are kept together.

- point estimate: all records in the bounded split;
- bootstrap replicates: `4000`;
- seed: `0x5444_4937_3400_4700`;
- interval: two-sided percentile 95%;
- quantities: `I_joint`, redundancy, unique-static, unique-recovery, synergy;
- rank reduction is rerun deterministically inside each replicate;
- rejected degenerate replicates are counted, never silently replaced;
- fewer than 95% accepted replicates is a hard failure.

## 8. Bounded split discipline

Qualification uses only inherited non-final TDI-7 ranges:

- training starts at `7_100_000_000`;
- development starts at `7_100_010_000`;
- validation starts at `7_100_020_000`.

The public TDI-7.4 final range `7_100_050_000..7_100_059_999` is used only for an explicit non-overlap assertion. Its decision record remains `authorization_state = "NOT_AUTHORIZED"`.

PID is reported independently on development and validation for replication. Predictive static-only, recovery-only, and joint layouts use training for fitting, development for lambda selection, and validation for evaluation through the shared TDI-7 predictive engine.

## 9. Qualification criteria

The estimator implementation is software-qualified only if:

- analytic/synthetic oracles pass;
- duplicate-source rank reduction preserves MI;
- a nearly collinear full-rank source remains cross-method stable without ridge regularization;
- PID identity passes;
- both arithmetic paths agree within tolerance;
- bootstrap is bit-deterministic on the reference toolchain;
- development and validation execution remain disjoint from the final range;
- final-run authorization strings are absent from evaluator surfaces;
- workspace, public, hosted, and TDI pre-arm CI remain green.

Passing these criteria qualifies software. It does **not** establish H-AI-3.

## 10. Required bounded reporting

Each task/split report includes source ranks, the three mutual informations, four MMI PID components, cross-method discrepancy, bootstrap intervals/rejection count, and provenance. Predictive reporting separately includes static-only MSE, recovery-only MSE, joint MSE, joint gain over each source alone, and the shared generator-paired uncertainty discipline.

No bounded output may contain a confirmatory H-AI-3 PASS/FAIL classification.
