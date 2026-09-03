# TDI-7.3 through TDI-7.10 evaluator implementation

Status: **development/validation implementation; final holdout NOT AUTHORIZED**.

This document describes the executable evaluator foundation added after the audit of PR #93. It does not amend any frozen preregistration, decision record, historical result, or holdout artifact.

## Scientific boundary

The implementation is deliberately fail-closed. It accepts only training, development, and validation split labels. It contains no final-holdout execution path and no authorization secret. A confirmatory run remains governed by the frozen stage-specific decision records and explicit final-run guard.

No evaluator may manufacture a stage verdict. Negative, null, inconclusive, divergent, fragile, or failed-transfer evidence is first-class input.

## TDI-7.3 — heterogeneity and coupling

Implemented primitives:

- explicit task, seed, depth, intervention sites, and magnitudes;
- bounded recovery-overlap validation;
- single-site deficits derived from measured overlaps;
- joint deficit derived from the measured coupled intervention;
- excess coupling = joint deficit - single-site deficit A - single-site deficit B;
- aggregate mean and maximum absolute excess coupling.

The implementation rejects zero depths, duplicate intervention sites, invalid magnitudes, non-finite values, and overlaps outside `[0, 1]`.

## TDI-7.4 — joint information

Implemented primitive: empirical discrete mutual information from explicit contingency counts, reported in bits for static-only, recovery-only, and joint predictors. The implementation also reports the joint information increment over the stronger single predictor.

This quantity is **not automatically labelled synergy**. Confirmatory synergy/redundancy language requires a separately frozen estimator/decomposition and frozen discretization policy. This prevents the ad-hoc proxy behaviour removed during the PR #93 audit.

## TDI-7.5 — FLAT semantic discrimination

A semantic is not admitted by name. Admission requires non-empty references for:

- frozen mathematical specification;
- scalar/reference oracle;
- numerical policy;
- invariances;
- failure modes.

The implemented recovery-distance primitive compares aligned measured recovery profiles. It does not claim semantic superiority and does not substitute for the static-spectral insufficiency gate.

## TDI-7.6 — evidence-justified ablations

The evaluator exposes evidence-justified comparison deltas only when a motivating evidence reference is supplied. The caller must preserve the frozen ablation taxonomy and stage-specific evidence lineage.

## TDI-7.7 — cross-architecture transfer

Implemented protocol arithmetic:

- forward transfer efficiency ratio;
- reverse transfer efficiency ratio;
- asymmetry index;
- joint-training benefit.

The evaluator rejects zero source denominators and non-finite scores. It does not convert transfer arithmetic into a neural-transfer claim.

## TDI-7.8 — evidence-justified extensions

The same evidence-lineage guard used for ablations is used for extension comparisons. Extended horizons, multi-site interventions, cross-length transfer, and composite features still require their preregistered motivating evidence and architecture-specific bounds.

## TDI-7.9 — calibration and robustness

Implemented primitives:

- bit-exact re-execution comparison;
- tolerance-bound comparison;
- divergent-run detection;
- maximum absolute numerical sensitivity.

Changing the observation horizon is not treated as a numerical-precision test. Precision robustness must compare actual numerical executions under the frozen calibration policy.

## TDI-7.10 — programme synthesis

The synthesis layer consumes evidence records for exactly TDI-7.3 through TDI-7.9. Every input record must carry an evidence reference, provenance reference, and explicit outcome (`Positive`, `Negative`, or `Inconclusive`). Missing or duplicate stages fail closed.

Contradictions are supplied explicitly and unresolved contradictions are counted. TDI-7.10 cannot be supplied as an input to itself. The implementation deliberately does **not** invent `PASS`, `robust`, or `coherent` stage outcomes.

## Current implementation surface

Executable and unit tests:

`tdi-ai/src/bin/tdi7-followup-evaluator.rs`

The binary prints only implementation readiness and the fact that final holdout execution is not authorized. Its unit tests exercise data-derived coupling, an XOR contingency-table information fixture, semantic admission guards, measured recovery-profile distance, evidence lineage, bidirectional transfer arithmetic, re-execution classes, and archive completeness.

## Next construction tranche

The next tranche should connect these primitives to deterministic TDI-7.1 task/intervention generation and produce development/validation evidence envelopes for TDI-7.3 first. TDI-7.4 and later stages must consume frozen upstream evidence rather than synthetic stage verdicts.
