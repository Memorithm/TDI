# TDI-7.1 completion checklist

TDI-7.1 is complete only when every item below is satisfied on the exact stacked head intended for merge.

## Protocol fidelity

- [x] TDI-7.0 preregistration hash verifies.
- [x] Associative-recall and copy remain the only confirmatory task families.
- [x] Training, development, validation and final-holdout seed ranges are structurally disjoint.
- [x] The bounded evaluator generates only training/development/validation populations.
- [x] No final-holdout range, authorization variable or token appears in ordinary tests/examples.

## Deterministic semantics

- [x] Reference semantic is identified as `deterministic_local_row_stochastic_v1`.
- [x] Every generated mixer is finite, non-negative and row stochastic.
- [x] Intervention amplitude and both sites match `TDI-7.1-EVALUATOR-SPEC.md`.
- [x] Interventions mutate mechanism state once and never task tokens/target.
- [x] Reference and perturbed trajectories use identical downstream dynamics.

## Feature construction

- [x] B0 contains task-local controls and the frozen static diagnostics.
- [x] Entropy-derived effective support is never labelled effective rank.
- [x] B1 is exactly B0 plus recovery values from early depths 1 and 2.
- [x] No late target or validation/holdout label enters the TDI feature block.
- [x] Target depth 5 is strictly after all early observation depths.

## Target and model

- [x] Retrieval deficit uses `d/(1+d)` at the declared retrieval position.
- [x] `0` means no observed degradation and valid targets lie in `[0,1)`.
- [x] B0 and B1 use the identical ridge-linear model class.
- [x] Both arms use the same lambda grid `[0, 1e-6, 1e-3, 1e-1]`.
- [x] Lambda choice uses training/development only.
- [x] Validation is evaluation-only.

## Paired uncertainty

- [x] Relative MSE reduction matches the TDI-7.0 formula.
- [x] Bootstrap uses 2,000 deterministic replicates.
- [x] Resampling is generator-level and keeps both intervention-site records paired.
- [x] Non-finite or non-positive baseline MSE fails closed.

## Reproducibility and provenance

- [x] `scripts/reproduce-tdi7.1-preflight.sh` runs all bounded stages.
- [x] The end-to-end evaluator and its tests are included in that preflight.
- [x] Provenance records commit SHA, semantic, populations, intervention, features, model, bootstrap, numerical policy and `final_holdout_status=NOT_ACCESSED`.
- [x] A dedicated TDI-7.1 workflow validates the bounded preflight without modifying historical frozen workflows.
- [x] Historical TDI manifests continue to verify unchanged paths.

## Scientific boundary

- [x] Validation-set behavior is not reported as the TDI-7.2 result.
- [x] No claim is made about Transformer transfer, FLAT semantic superiority or GPU performance.
- [x] TDI-7.2 remains blocked until TDI-7.1 is merged and CI-valid.

A checked box is evidence of software/protocol readiness only. It is not evidence that H-AI-1 is Beneficial.

## Verification record

All items above were verified on commit `74846ab` (merge of PR #81, `feat/tdi7-early-recovery-features` head `aed3233`):

- `bash scripts/check-tdi7.1-readiness.sh` returned exit 0 (`TDI-7.1 readiness gate: PASS`), covering the TDI-7.0 preregistration hash, historical TDI-6.8 integrity, specification surfaces, TDI-7.2 surface exclusion, preflight holdout-refusal and the complete bounded preflight.
- `cargo test --workspace`: 1,134 passed / 0 failed (local, cargo 1.98.0).
- Direct source verification: nested B0/B1 ridge construction (`tdi-ai/examples/tdi7_end_to_end.rs`), lambda grid `[0, 1e-6, 1e-3, 1e-1]`, `BOOTSTRAP_REPLICATES = 2_000`, deficit `distance / (1.0 + distance)`, fail-closed `b0_mse > 0.0`, `EARLY_DEPTHS = 2 < TARGET_DEPTH = 5`, row-stochastic mixer validation (`tdi-ai/src/toy_attention.rs`), `final_holdout_status=NOT_ACCESSED` provenance (`tdi-bench/src/bin/tdi-attention-v71-provenance.rs`), dedicated workflow `.github/workflows/tdi71-ci.yml` (CI check `TDI-7.1 preflight`: success).
- Self-hosted jetson checks remain queued (jetson-tdi-01..04 offline); the public hosted workflow and all TDI-7 gates were green on the exact head.
