# TDI-7.4 — bounded source-identifiability audit

Status: **non-holdout qualification evidence; not a confirmatory H-AI-3 result**.

This audit was added after the bounded Gaussian/MMI qualification showed `rank(S) = rank(R) = rank([S,R]) = 3` and effectively zero incremental joint information on development/validation data. Its purpose is to determine whether that observation is merely a bounded null result or whether the currently frozen source blocks are linearly non-identifiable on the available non-holdout populations.

## Scope and guardrails

The audit uses only the existing non-holdout training, development and validation populations for associative recall and copy.

- `S`: exactly the six TDI-7.1 static attention controls.
- `R`: recovery depths 1 through 4.
- Geometry: centered, unit-RMS columns followed by two-pass modified Gram-Schmidt.
- Rank residual scale: `1e-10`.
- Source-space equivalence tolerance: `1e-10` on the maximum bidirectional residual ratio.
- Final TDI-7.4 population: **not accessed**.
- Final-run authorization token/environment surface: absent from the audit implementation and CI.
- Confirmatory H-AI-3 verdict: **not computed**.

Evidence execution: GitHub Actions workflow `TDI-7.4 bounded identifiability audit`, run `33756592977`, on head SHA `5f637f9b893ca70497b1a9f5d459791cb38db970`.

## Executed bounded results

| Population | Task | rank(S) | rank(R) | rank([S,R]) | max R outside span(S) | max S outside span(R) | Equivalent within 1e-10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| training | associative recall | 3 | 3 | 3 | 3.0534801202246685e-13 | 2.6332559370336015e-13 | yes |
| training | copy | 3 | 3 | 3 | 6.9729892248244476e-13 | 8.9146227797172384e-12 | yes |
| development | associative recall | 3 | 3 | 3 | 2.4772394035450728e-13 | 1.1723562734828716e-13 | yes |
| development | copy | 3 | 3 | 3 | 6.2991140035252241e-13 | 3.0265951623704539e-11 | yes |
| validation | associative recall | 3 | 3 | 3 | 2.4361514559509159e-13 | 1.8605224589845546e-13 | yes |
| validation | copy | 3 | 3 | 3 | 5.1050912372293887e-13 | 1.4871875276427808e-11 | yes |
| pooled train+dev+validation | associative recall | 3 | 3 | 3 | 7.914556137618699e-13 | 8.209555579815066e-13 | yes |
| pooled train+dev+validation | copy | 3 | 3 | 3 | 8.116369395939639e-13 | 3.177865336858892e-11 | yes |

The pooled cells are important: the equivalence is not an artifact of separately fitting a different source-space relation inside each split. After concatenating all non-holdout records for a task, the centered static and recovery blocks still span the same rank-3 sample-space subspace within the frozen tolerance.

## Interpretation

Within these bounded non-holdout populations and the current TDI-7.4 source definitions:

1. `S`, `R`, and `[S,R]` provide the same centered linear sample-space directions within tolerance.
2. Appending `R` to `S` does not add a new centered linear direction, and appending `S` to `R` does not add one either.
3. For coordinate-invariant Gaussian information calculations after rank reduction, this explains why static-only, recovery-only and joint information collapse to the same effective source subspace up to numerical error.
4. For unrestricted affine-linear prediction on the audited records, the predictor spaces induced by `S` and `R` are equivalent. A difference produced only by coordinate-dependent ridge regularization would not by itself demonstrate new information in the joint source block.

This is an **identifiability blocker for the current H-AI-3 realization**, not a confirmatory negative H-AI-3 result. The frozen question asks whether static and recovery sources become jointly informative and requires competent single-source controls plus joint predictive value. With the current source blocks, the bounded data do not provide distinct linear source directions from which that incremental joint value can be identified.

## Authorization consequence

TDI-7.4 final holdout must remain `NOT_AUTHORIZED` under the current realization. Executing the frozen final seeds now would spend an untouched holdout on a diagnostic whose source blocks are already non-identifiable in the bounded qualification domain.

This audit does **not** alter the frozen preregistration, horizons, estimators, source definitions, decision records, or final seed range. Any material attempt to restore identifiability must be preregistered separately before evaluation and must use an appropriately disjoint untouched final population. The existing TDI-7.4 record must remain as evidence of the blocked realization rather than being silently rewritten.

## Non-claims

This audit does not establish that recovery observables are universally redundant with static diagnostics, that H-AI-3 is false in other mechanisms, that nonlinear information decompositions cannot distinguish the sources, or that the same relation transfers to Transformers, language models, or FLAT-ATTENTION implementations.
