# TDI-2.1 — Data and split contract

Date frozen: 2026-09-16

## Required partitions

Every TDI-2.1 experiment must use disjoint logical partitions:

1. `Development`: fitting, feature design and debugging.
2. `Validation`: threshold selection and model selection permitted only when explicitly preregistered.
3. `PrimaryHoldout`: opened once for the primary confirmatory evaluation.
4. `TransferHoldout`: structurally shifted confirmatory population.

## Record schema

Each case must expose only information available at decision time:

- `case_id`: stable opaque identifier;
- `family_id`: preregistered task/generator family;
- `split`: one of the four partitions;
- `x`: observation vector or canonical observation record;
- `g`: optional context record;
- `target`: protected outcome, inaccessible to the candidate before scoring;
- `feedback`: optional post-decision information governed by the consolidation policy;
- `generator_revision` or source-data digest;
- deterministic seed/provenance fields where applicable.

## Split isolation

No case identifier, target, target-derived statistic or threshold fitted on either holdout may enter training, memory construction, feature design, calibration or arbitration-policy selection.

## Experience construction

The experience store for a tested holdout case may contain only records explicitly allowed by the frozen protocol. The primary confirmatory arm uses Development-derived experience unless a later preregistered transfer experiment states otherwise.

## Duplicate and near-duplicate policy

Exact duplicate observations with conflicting split assignments are forbidden. If a task family can generate near duplicates, its generator specification must define a structural-equivalence key and enforce split separation at that level.

## Missing or invalid values

Non-finite numeric inputs, invalid dimensions, missing required context and malformed targets fail closed before scoring. The rejection count is reported by split and arm.

## Artifact binding

A run is interpretable only when its result record identifies the exact split manifest and source/generator revision. Reconstructed or manually edited split lists cannot be substituted after holdout opening.
