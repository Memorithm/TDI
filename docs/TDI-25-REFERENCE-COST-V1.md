# TDI-25 slice 47 — reference cost study (`tdi25-reference-cost-v1`)

Status: experimental, non-final, Development/Validation only (Phase E). No
training, no protected/final/holdout access, no scientific claim, no
production, GPU, hardware or performance claim. Holdouts TDI-7.2/8.2/9.2 are
untouched. Slices 49–50 remain pre-arm/decision artifacts only.

**Timing non qualifié.** The campaign row asks for "operations, memory and
qualified CPU timing". `docs/TDI-25-PROGRAMME.md` admits runtime costs only
under an explicitly qualified environment, and no such environment is
defined. With the user's approval this slice is delivered at reduced scope:
deterministic operation counts and logical memory only, plus a timing
harness that refuses to measure until a qualified-environment manifest is
frozen by a human.

## Operation and logical-memory accounting

Counts describe the bounded Rust reference algorithms (same definition as
TDI-24 slice 09): scalar multiplications, additions/subtractions and
fail-closed finiteness predicates in the score path after carrier
construction, and logical f64 bytes read. They are not CPU instructions,
latency, allocator peak or resident memory.

| Arm | Contract | Mult. | Add. | Predicates | Scalars read | Logical bytes |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| T6 | `tdi25-t6-reference-accounting-v1` | 18 | 17 | 10 | 18 | 144 |
| C6 `(1, 0, 1)` | `tdi24-reference-accounting-v3` (unchanged) | 21 | 26 | 41 | 12 | 96 |

- T6 factorized pairing `(v + Q x omega).R + omega.(M + P x R)`: two cross
  products (`6M + 3A` each), two vector additions (`3A` each), two dot
  products (`3M + 2A` each) and the final sum (`1A`). Predicates: query
  position (3), factorized resultant dual (3), origin moment (3), final
  pairing (1). T6 reads both reduction points.
- C6 reuses the TDI-24 slice-09 accounting unchanged (TDI-25 does not
  redefine the chiral contract): the reference evaluates all three channel
  pairings and weights, including the zero-weight mirror channel. C6 reads
  the 12 carrier scalars only.
- The shared matched input is 18 scalars (144 B) per case for both arms.

`run_reference_cost` generates and scores the bounded matched population
and reports totals (one pair score per case per arm). At the smoke budget
(4 families x 2 blocks x 8 cases = 64 cases per split):

| Arm | Mult. | Add. | Predicates | Bytes read |
| --- | ---: | ---: | ---: | ---: |
| T6 | 1152 | 1088 | 640 | 9216 |
| C6 | 1344 | 1664 | 2624 | 6144 |

## Timing harness (refuses)

`run_qualified_reference_timing` reads the checked-in constant
`QUALIFIED_TIMING_ENVIRONMENT`. While it is `None`, the harness returns
`timing_environment_not_qualified` without touching the clock. A future
human freeze must pin a `QualifiedTimingEnvironment`
(`tdi25-qualified-timing-environment-v1`): dedicated machine identity,
exact toolchain, warm-up and measured iteration counts. Even then the
harness only returns raw per-pass nanosecond samples for T6 and C6; choosing
a summary statistic and a variance rejection rule is part of that freeze.
A unit test exercises the mechanics with a test-only fixture whose samples
are never reported.

## Validation

`validate_reference_cost_report` rejects `contract_drift`,
`population_drift`, `capacity_mismatch`, `accounting_drift` (per-pair
counts differ from the declared accounting), `population_size_drift`,
`totals_drift`, `timing_status_drift` (any status other than
`timing_non_qualifie`, `timing_measured`, or a set qualified environment),
the protected/training/scientific/performance/non-final flags, and
regenerates the scored population (`case_evidence_drift`).

## Recorded degeneracies

- The counts are static per pair score: totals scale exactly linearly with
  the population, so the study carries no data-dependent cost signal.
- C6 is charged for its zero-weight mirror channel because the reference
  computes it; a fused kernel would cost less, but none is evaluated.
- T6 reads 1.5x the logical bytes of C6 (the two reduction points); C6
  performs more arithmetic and many more validity predicates. Neither is a
  runtime statement.
- No row/normalizer accounting is reported for TDI-25 in this slice.
