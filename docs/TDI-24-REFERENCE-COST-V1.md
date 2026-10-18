# TDI-24 slice 46 — CPU reference cost study (`tdi24-reference-cost-v1`)

Status: experimental, non-final, Development/Validation only (Phase E). No
training, no protected/final/holdout access, no scientific claim, no
production, GPU, hardware or performance claim. Holdouts TDI-7.2/8.2/9.2 are
untouched. Slice 45 (trained arms) remains blocked pending human training
parameters, so slices 47+ are not executed by this slice.

## Contract

- Paired arms `REFERENCE_COST_ARMS`: V6 vector control and C6 `(1, 0, 1)`,
  at their matched reference capacities (`reference_v6` / `reference_c6`).
- Per-pair accounting is the unchanged slice-09 contract
  `tdi24-reference-accounting-v3`: V6 `6M + 6A`, 12 validity predicates;
  C6 `21M + 26A`, 41 validity predicates; both read 12 carrier scalars
  (96 logical f64 bytes) per pair score.
- `run_reference_cost` actually executes both unchanged reference scorers
  over the identical bounded Stage-C stream (measured software execution:
  every scored case and its finite score are counted), then multiplies the
  per-pair accounting by the cases scored. The C6 diagonal score must
  reproduce the Stage-C reference bit-for-bit.
- No wall-clock timing is measured: `timing_measured = false`,
  `timing_status = timing_not_measured`. Counts describe the bounded Rust
  reference algorithms; they are not CPU instructions, latency, allocator
  peak or resident memory.

At the smoke budget (4 pairs per family, 32 cases per arm):

| Arm | Mult. | Add. | Predicates | Carrier bytes |
| --- | ---: | ---: | ---: | ---: |
| V6 | 192 | 192 | 384 | 3072 |
| C6 | 672 | 832 | 1312 | 3072 |

## Validation

`validate_reference_cost_report` rejects `contract_drift`,
`accounting_contract_drift`, `capacity_mismatch`, `population_size_drift`,
`timing_status_drift` (any measured timing or other status),
`cell_count`, `grid_drift`, `accounting_drift` (per-pair counts differ from
slice 09), `failure_accounting_drift` (a scored case dropped or non-finite),
`totals_drift`, the performance/protected/training/scientific/non-final
flags, and regenerates the scored population (`case_evidence_drift`).

## Recorded degeneracies

- The counts are static per pair score: totals scale exactly linearly with
  the population, so the study carries no data-dependent cost signal.
- C6 is charged for its zero-weight mirror channel because the reference
  computes it; a fused kernel would cost less, but none is evaluated.
- V6 and C6 read identical logical carrier bytes; C6 performs more
  arithmetic and validity predicates. Neither is a runtime statement.
- No row/normalizer accounting is reported in this slice (slice 09 retains
  it separately).
