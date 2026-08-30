# TDI-7.1 completion checklist

TDI-7.1 is complete only when every item below is satisfied on the exact stacked head intended for merge.

## Protocol fidelity

- [ ] TDI-7.0 preregistration hash verifies.
- [ ] Associative-recall and copy remain the only confirmatory task families.
- [ ] Training, development, validation and final-holdout seed ranges are structurally disjoint.
- [ ] The bounded evaluator generates only training/development/validation populations.
- [ ] No final-holdout range, authorization variable or token appears in ordinary tests/examples.

## Deterministic semantics

- [ ] Reference semantic is identified as `deterministic_local_row_stochastic_v1`.
- [ ] Every generated mixer is finite, non-negative and row stochastic.
- [ ] Intervention amplitude and both sites match `TDI-7.1-EVALUATOR-SPEC.md`.
- [ ] Interventions mutate mechanism state once and never task tokens/target.
- [ ] Reference and perturbed trajectories use identical downstream dynamics.

## Feature construction

- [ ] B0 contains task-local controls and the frozen static diagnostics.
- [ ] Entropy-derived effective support is never labelled effective rank.
- [ ] B1 is exactly B0 plus recovery values from early depths 1 and 2.
- [ ] No late target or validation/holdout label enters the TDI feature block.
- [ ] Target depth 5 is strictly after all early observation depths.

## Target and model

- [ ] Retrieval deficit uses `d/(1+d)` at the declared retrieval position.
- [ ] `0` means no observed degradation and valid targets lie in `[0,1)`.
- [ ] B0 and B1 use the identical ridge-linear model class.
- [ ] Both arms use the same lambda grid `[0, 1e-6, 1e-3, 1e-1]`.
- [ ] Lambda choice uses training/development only.
- [ ] Validation is evaluation-only.

## Paired uncertainty

- [ ] Relative MSE reduction matches the TDI-7.0 formula.
- [ ] Bootstrap uses 2,000 deterministic replicates.
- [ ] Resampling is generator-level and keeps both intervention-site records paired.
- [ ] Non-finite or non-positive baseline MSE fails closed.

## Reproducibility and provenance

- [ ] `scripts/reproduce-tdi7.1-preflight.sh` runs all bounded stages.
- [ ] The end-to-end evaluator and its tests are included in that preflight.
- [ ] Provenance records commit SHA, semantic, populations, intervention, features, model, bootstrap, numerical policy and `final_holdout_status=NOT_ACCESSED`.
- [ ] A dedicated TDI-7.1 workflow validates the bounded preflight without modifying historical frozen workflows.
- [ ] Historical TDI manifests continue to verify unchanged paths.

## Scientific boundary

- [ ] Validation-set behavior is not reported as the TDI-7.2 result.
- [ ] No claim is made about Transformer transfer, FLAT semantic superiority or GPU performance.
- [ ] TDI-7.2 remains blocked until TDI-7.1 is merged and CI-valid.

A checked box is evidence of software/protocol readiness only. It is not evidence that H-AI-1 is Beneficial.
