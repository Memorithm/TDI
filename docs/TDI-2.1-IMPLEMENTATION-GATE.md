# TDI-2.1 — Implementation gate

Date frozen: 2026-09-16

This gate defines the coding order after the scientific contracts are present on `main`.

## Gate A — Deterministic scalar reference

Implement in `tdi-ai` a safe Rust reference that provides:

- input/context validation;
- normalization;
- fixed-size or bounded pattern projection/retrieval;
- stable softmax or alternative frozen retrieval transform;
- output synthesis;
- confidence diagnostics;
- no hidden System-2 execution.

All dimensions/configuration must be explicit. Invalid `tau`, non-finite inputs and empty pattern stores fail closed.

## Gate B — Experience-store controls

Implement constructors for valid, shuffled and unrelated experience stores with identical capacity accounting and deterministic seeds.

## Gate C — Calibration and router

Implement metric-only confidence evaluation, Validation-fitted threshold serialization and a router that returns `Fast`, `Slow` or `Abstain` without executing privileged actions.

## Gate D — Consolidation

Implement bounded update rules behind an explicit experiment configuration. Primary confirmatory scoring keeps mutation disabled.

## Gate E — Task generators and baselines

Implement F1–F6 generators plus B0–B3 and S2 reference arms independently from candidate target logic.

## Gate F — Evidence

Emit raw case-level records, configuration/provenance digests, resource accounting and deterministic aggregation inputs.

## Gate G — Qualification

Before PrimaryHoldout opening require:

- `cargo fmt --check`;
- `cargo test --workspace`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` where repository CI supports it;
- deterministic/repeatability tests;
- leakage tests;
- negative controls;
- Validation report with no protected holdout access.

## SIMD/performance gate

Only after scalar scientific equivalence is tested may an optimized implementation be introduced. The optimized path must match the scalar reference within a frozen numerical tolerance before any speed comparison is interpreted.
