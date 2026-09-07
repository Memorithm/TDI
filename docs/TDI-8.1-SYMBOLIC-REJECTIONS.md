# TDI-8.1 symbolic rejection provenance

Status: **bounded deterministic software qualification only — not H8-A/H8-B evidence and not a TDI-8.2 authorization surface**.

Tracks #161 and the TDI-8 programme issue #87.

## Purpose

TDI-8.0 requires technical rejection, quality outcomes and provenance to remain distinguishable. The qualified symbolic executor already preserves completed invalid predictions as ordinary evaluated failures. This tranche adds a machine-readable wrapper for the other side of that boundary: execution paths that fail technically and therefore cannot yield a complete evaluable task record.

The design follows the already-qualified TDI-9.1 convention of stable numeric categorization plus retention of the original typed error, but it does not depend on TDI-9 policy/task types.

## Completed versus rejected

`RecordedSymbolicTaskOutcome<E>` has exactly two branches:

- `Completed(TaskExecutionRecord)`: the symbolic task completed technically. Incorrect predictions and `TaskPrediction::Invalid` remain inside this record and therefore remain in the quality denominator.
- `Rejected(SymbolicTaskRejectionRecord<E>)`: the architecture-neutral executor could not produce a complete evaluable record because a technical execution condition failed.

A rejection is not assigned a task-success value. An invalid or incorrect prediction is not converted into a rejection.

## Stable rejection vocabulary

`SymbolicRejectionCode` is `#[repr(u16)]` with explicit non-renumberable values:

| Code | Meaning |
|---|---|
| `0x0101` | generated query count cannot fit the host index type |
| `0x0102` | exact query-record reservation failed |
| `0x0201` | adapter reset failed |
| `0x0202` | one adapter event failed technically |
| `0x0301` | adapter arm identity changed during execution |
| `0x0302` | observed completed query count disagreed with the generator declaration |

Future rejection reasons must consume unused codes; existing numeric values must not be reassigned.

## Provenance retained on rejection

Every rejection record contains only evaluator-side execution provenance required to identify the rejected trajectory:

- reference arm captured before reset/execution;
- generated task family;
- exact generator seed;
- stable rejection code;
- original `TaskExecutionError<E>`, including its adapter-specific typed error payload.

The wrapper does not inspect or copy exact query targets, source answers or oracle values. Target-bearing events remain encapsulated by the already-qualified symbolic executor.

## Qualification-local boundary

`task_rejections.rs` intentionally remains outside the stable `tdi-ai` module API in this tranche. The concrete A0/A1/A2/A3 adapters still reside in qualification preflight binaries. Promoting a rejection API before those concrete evaluator adapters have a reusable reviewed surface would freeze an incomplete adapter-specific error taxonomy.

The generic executor-level codes are therefore frozen first. Adapter-specific sub-codes can be considered only when the adapters themselves are promoted without changing their reviewed TDI-8.1 semantics.

## Scientific boundary

This tranche does not select or freeze:

- recurrent dimensions or weights;
- associative capacity, seed or fusion gain;
- VSA width, role seed or gain;
- matched dynamic-memory budget;
- Short/Medium/Long numeric horizons;
- development/validation/final populations or sample counts;
- late-retrieval deficit or intervention/recovery definitions;
- interval parameters;
- TDI-8.2 seeds, runner, result payload, confirmation token or final holdout.

No technical rejection is silently removed from provenance and no rejection count is presented as a quality score.

## Reproducibility gate

Run:

```bash
bash scripts/check-tdi8.1-symbolic-rejections.sh
```

The gate verifies stable code coverage, original typed-error retention, the invalid-prediction quality boundary, provenance fields, absence of target/oracle reads in the rejection layer, qualification-local API status and TDI-8.2 absence.
