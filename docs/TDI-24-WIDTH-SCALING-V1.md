# TDI-24 width scaling v1 (slice 38)

Status: Phase-D Development/Validation software scaling study. Contract
`tdi24-width-scaling-v1`. No training, no width selection, no protected/final
access, no confirmatory execution and no scientific claim. No configuration
freeze pin is introduced or changed; holdouts are untouched.

## Preregistered matched widths

`WIDTH_SCALING_WIDTHS = [2, 4, 6]`: every even carrier width `2n`,
`n = 1..=3`, derived from the six-slot carrier. All widths are evaluated and
reported together; nothing is chosen adaptively. At width `2n` the query and
key carriers are restricted to the first `n` slots of each sector (the other
slots are zero), so `M` and `J` act on the matched sub-carrier
(`restrict_carrier_to_width`). Unregistered widths are rejected
(`width_not_registered`).

## Arms

- **C6** with the unchanged reference weights `(1, 0, 1)`.
- **Direct-only** matched arm `(1, 0, 0)` (`direct_only_weights`, slice 33).

Both arms use `TrainableCapacity::reference_c6()` (zero trainable parameters)
at every width; the same restricted carriers feed both arms.

## Checked identities and rejections

- **Full-width reproduction**: width 6 equals the Stage-C C6 score bit for bit
  (`full_width_reference_drift`).
- Width-set, weight, capacity and contract drift, case count, tampered
  summaries, regenerated evidence (`case_evidence_drift`) and
  access/training/claim flags are rejected. Protected/final labels never
  generate a case.

## Smoke-budget software diagnostics

Budget `StageCPreflightBudget::bounded(8, 0)`, Development (16 cases per
family), correct counts C6 / direct-only:

| Width | ReflectionDiscriminative | ReflectionNuisance | DirectionReversal | NonChiralControl |
| ---: | --- | --- | --- | --- |
| 2 | 16 / 8 | 12 / 16 | 16 / 8 | 8 / 16 |
| 4 | 16 / 8 | 12 / 8 | 16 / 8 | 8 / 16 |
| 6 | 16 / 8 | 12 / 8 | 16 / 8 | 8 / 16 |

## Recorded degeneracies

1. **Development equals Validation on this budget**, inherited from the
   Stage-C case stream (slice 30).
2. **Restriction, not re-generation.** Narrow widths project the same six-wide
   cases; they are not independently generated narrow tasks, so task
   difficulty is not matched across widths.
3. **Direct-only cannot see handedness** at any width (the direct channel is
   reflection-even), so its discriminative/direction counts stay at chance by
   construction.

Interpreting these counts as attribution evidence is reserved for the Stage-D
attribution audit (slice 40).
