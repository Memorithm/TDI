# TDI-7.1 — bounded evaluator specification

Status: implementation specification for the non-holdout TDI-7.1 software-oracle gate.

This document records concrete evaluator choices made after TDI-7.0 preregistration and before any TDI-7.2 final-holdout execution.

## Scope

TDI-7.1 is limited to training, development, and validation seeds. It must not generate, read, fit against, classify, or report the TDI-7.2 final holdout.

## Deterministic reference semantic

Identifier: `deterministic_local_row_stochastic_v1`.

For a sequence of length `n`, each row mixes its center and immediate neighbors. Interior rows use weights `[side, center, side]`, with:

- `spread = retrieval_distance / (n + 1)` clamped to `[0,1]`;
- `side = 0.15 + 0.10 * spread`;
- `center = 1 - 2 * side`.

Boundary mass is folded onto the boundary token so every row sums to one. Accumulation uses Rust `f64` scalar arithmetic. This is an attention-like deterministic sequence mixer, not StandardSoftmax and not a FLAT production kernel.

## Task populations

Confirmatory task families remain exactly those frozen by TDI-7.0:

- associative recall;
- copy.

TDI-7.1 bounded population sizes per task are:

- training: 96 generator seeds;
- development: 48 generator seeds;
- validation: 48 generator seeds.

Each generator seed is evaluated at both preregisterable intervention sites, so the predictive record count is twice the generator count. Bootstrap resampling remains at generator level, with both site records kept together.

Non-holdout seed starts:

- training: `7_100_000_000`;
- development: `7_100_010_000`;
- validation: `7_100_020_000`.

No final-holdout range is embedded in the bounded end-to-end evaluator.

## Interventions

Two single-site balanced perturbations are retained:

- early site: add at token index 1 and subtract at index 0;
- late site: add at token index `len-2` and subtract at `len-1`.

Amplitude: `0.25` in normalized scalar activation units.

The generated task token sequence is never mutated by the intervention. The perturbation is applied once before downstream dynamics; reference and perturbed trajectories then use identical dynamics.

## Early recovery block

Observable: complete scalar token state.

Recovery metric: reciprocal L-infinity recovery already implemented in `tdi-ai`:

`R = 1 / (1 + ||x_ref - x_perturbed||_inf)`.

Frozen early observation depths for this evaluator: `1` and `2`.

No late target enters this feature block.

## Static/task baseline block

B0 contains:

- sequence length;
- distractor count;
- retrieval distance;
- mean row Shannon entropy;
- mean normalized row entropy;
- mean row maximum weight;
- mean row L2 concentration;
- mean entropy-derived effective support;
- Frobenius norm;
- intervention-site indicator.

Entropy-derived support is not effective rank.

## Late retrieval deficit

Target depth: `5`, strictly after both frozen early recovery depths.

At the task's declared retrieval position, let

`d = |reference_value - perturbed_value|`.

The scalar target is the bounded transform

`deficit = d / (1 + d)`.

Properties:

- `deficit = 0` iff no degradation is observed at the declared scalar retrieval position;
- larger values mean larger perturbation-induced degradation;
- the target is finite for finite `d` and lies in `[0,1)`.

This target definition is frozen here before TDI-7.2.

## Model ladder

Both arms use the same ridge-linear model class with an intercept and the same lambda grid:

`[0, 1e-6, 1e-3, 1e-1]`.

- B0: static/task baseline block only;
- B1: B0 plus the two early recovery values.

Lambda selection uses training/development only. Validation is evaluation-only in TDI-7.1.

## Paired uncertainty

Relative MSE reduction remains exactly the TDI-7.0 definition.

Bootstrap:

- 2,000 deterministic replicates;
- seed `0x5444493745324501`;
- generator-level resampling;
- both intervention-site records for a selected generator are resampled together;
- percentile two-sided 95% interval.

## Non-claims

TDI-7.1 validation behavior is a software/mechanistic preflight, not the confirmatory TDI-7.2 result. No validation-set effect size may be presented as evidence that TDI improves Transformers, selects a FLAT semantic, or improves GPU performance.
