# TDI-7.3 bounded evaluator implementation

Status: **development/validation evaluator tranche; confirmatory final run NOT AUTHORIZED**.

This document records the software implementation added for the TDI-7.3 H-AI-2 preregistration. It does not amend `docs/TDI-7.3-PREREGISTRATION.md`, does not modify the frozen population/selection/rejection decisions, and does not produce a confirmatory verdict.

## Purpose

The first executable tranche answers a narrower engineering question before any predictive B0/B1 layer is armed:

> Can the exact TDI-7.1 deterministic task and intervention mechanics be reused to generate reproducible per-location and joint recovery trajectories, with explicit location-difference and coupling diagnostics, on non-holdout development/validation seeds?

## Shared mechanics

`tdi-bench/src/attention_v7.rs` is now the reusable source for the task and intervention mechanics previously embedded only in the TDI-7.1 binaries.

The TDI-7.1 task and intervention binaries consume that shared source. This prevents the TDI-7.3 evaluator from silently cloning or drifting from the bounded mechanics already tested in TDI-7.1.

The shared surface provides:

- deterministic associative-recall generation;
- deterministic copy generation;
- the frozen early-token and late-token intervention locations;
- the same single-site additive activation perturbation;
- the same deterministic local diffusion update;
- task/target preservation checks;
- deterministic joint application of the two distinct site interventions;
- reciprocal L-infinity recovery over the same evolved mechanistic states.

No final-run confirmation variable or token is present in this shared surface.

## Bounded seed scope

The bounded evaluator uses only small inherited non-holdout windows:

- development starts at `7100010000`;
- validation starts at `7100020000`;
- 64 generators per task and split.

The public frozen TDI-7.3 final population range `7100040000..=7100049999` is referenced only as an exclusion boundary. A runtime guard proves that neither bounded window overlaps it. The final range is never iterated by this executable.

## Task families

Exactly the two preregistered task families are evaluated:

- `associative_recall`;
- `copy`.

## Intervention design

For every generator the evaluator constructs three measured trajectories from the same reference state:

1. early-token single-site intervention;
2. late-token single-site intervention;
3. joint early-token + late-token intervention.

The single-site amplitude is `0.25`, matching the bounded TDI-7.1 intervention fixture. Task tokens and targets remain unchanged.

## Recovery trajectory

Four downstream recovery depths are measured. The bounded score is

`R(x, y) = 1 / (1 + ||x - y||_inf)`.

This is the bounded deterministic recovery score already used by the TDI-AI toy attention contract. It is not a probability and is not represented as historical finite-state TDI overlap.

## Site heterogeneity diagnostics

At every downstream depth the evaluator records:

- early-site recovery;
- late-site recovery;
- absolute site difference `|R_early - R_late|`;
- symmetric relative site difference
  `|R_early - R_late| / max(|R_early|, |R_late|, epsilon)`.

The relative-difference formula is explicit here so a later result cannot redefine it post hoc.

## Coupling diagnostic

At every downstream depth:

- `D_early = 1 - R_early`;
- `D_late = 1 - R_late`;
- `D_joint = 1 - R_joint`;
- `excess_coupling = D_joint - D_early - D_late`.

The implementation reports the population mean excess coupling and maximum absolute excess coupling. It does not automatically label either sign as beneficial, harmful, synergistic, or mechanistically important.

## Deterministic output envelope

For each of the four task/split combinations, the evaluator emits one canonical line containing:

- split;
- task;
- generator count;
- trajectory-point count;
- mean/max absolute site difference;
- mean/max relative site difference;
- mean/max-absolute excess coupling.

Floating-point quantities are serialized by their exact `f64::to_bits()` representation for deterministic provenance comparison.

## Explicitly not implemented in this tranche

The executable prints:

- `confirmatory_verdict=NOT_COMPUTED`;
- `predictive_B0_B1_layer=NOT_YET_IMPLEMENTED`.

Therefore this tranche does **not** satisfy the full TDI-7.3 required reporting contract yet. In particular, it does not claim B0/B1 MSE, relative MSE reduction, paired 95% intervals, or a frozen H-AI-2 verdict.

Those are the next evaluator tranche and must consume these measured trajectories plus the frozen TDI-7.1 feature/model discipline rather than fabricated fixture scores.

## CI gate

`.github/workflows/tdi7-v73-bounded.yml`:

- checks formatting with Rust 1.97.1;
- runs Clippy with warnings denied;
- runs the shared-mechanics and bounded-evaluator tests;
- executes the evaluator;
- requires exactly four canonical bounded summary records;
- requires explicit non-holdout/non-confirmatory status lines;
- proves the final-run authorization token and variable are absent from the new evaluator surfaces.
