# TDI-2.1 — Reproducibility contract

Date frozen: 2026-09-16

## Minimum run identity

Every experimental result must record:

- repository commit SHA;
- dirty-tree status, which must be false for confirmatory runs;
- Rust toolchain and target triple;
- operating-system/kernel identity;
- CPU model and available instruction-set features;
- GPU/device identity if used;
- exact command line;
- configuration digest;
- split/generator digest;
- seed set;
- start/end timestamps;
- output artifact SHA-256.

## Determinism classes

Each experiment declares one class:

- `Exact`: repeated execution produces bit-identical scientific outputs;
- `Seeded`: stochasticity is completely seed-bound but floating-point reductions may vary within a frozen tolerance;
- `Statistical`: irreducible nondeterminism is present and repeated-run aggregation is preregistered.

A run may not silently change class.

## Repeat policy

Before holdout opening, candidate implementations must pass at least three repeated Validation executions under the declared determinism class. Confirmatory repetitions, if any, are fixed in advance and all repetitions are reported.

## Provenance

Raw case-level outputs are immutable inputs to aggregation. Summary tables must be regenerable from raw outputs without rerunning the candidate.

## Environment changes

Changing compiler flags, CPU target, dependency versions or hardware between matched arms is prohibited unless the difference is itself the preregistered object of study.

## External dependencies

Network access is disabled for confirmatory scoring unless the task explicitly requires a pinned external service and its exact response provenance can be captured. Unpinned hosted-model responses are not admissible as deterministic TDI-2.1 evidence.
