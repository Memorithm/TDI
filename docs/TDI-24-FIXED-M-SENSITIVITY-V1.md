# TDI-24 fixed-M sensitivity v1 (slice 35)

Status: Phase-D Development/Validation software sensitivity study. Contract
`tdi24-fixed-m-sensitivity-v1`. No training, no basis selection, no
protected/final access, no confirmatory execution and no scientific claim. No
configuration freeze pin is introduced or changed; holdouts are untouched.

## Frozen rule

The alternative fixed mirror bases are **all** choices of three of the six
carrier slots as the parity-even sector `H+`, the complementary three slots
forming `H-`. Both sectors are listed in ascending slot order and the `i`-th
`H+` slot is paired with the `i`-th `H-` slot by the complex structure. The
family is enumerated in lexicographic order of the `H+` slots, so the
canonical basis `H+ = {0,1,2}` is rank 0 and its sector exchange
`H+ = {3,4,5}` is rank 19. The family size `C(6,3) = 20`
(`FIXED_MIRROR_BASIS_COUNT`) is derived from the carrier width; nothing is
sampled, filtered, tuned or selected, and every basis is always reported.

Scoring a carrier under basis `b` is scoring the relabelled carrier
`(x[H+], x[H-])` with the canonical `M`/`J`, i.e. using the effective operators
`M' = P^T M P` and `J' = P^T J P`.

## Checked identities and rejections

- **Exact algebra per basis**: `M'^2 = I`, `J'^T = -J'`, `J'^2 = -I` and
  `M' J' M' = -J'`, checked on exact signed integer matrices
  (`basis_algebra_failure`). Every basis is therefore a legitimate fixed
  mirror basis, unlike the slice-34 parity shuffle.
- **Complete ordered family**: wrong size, wrong rank or a non-partition is
  rejected (`basis_family_incomplete`, `basis_order_drift`,
  `basis_not_a_sector_partition`).
- **Canonical reproduction**: the rank-0 basis must reproduce the Stage-C C6
  evaluator score bit for bit on every case (`canonical_reference_drift`).
- **Complement antisymmetry**: for every case, the parity-odd observable under
  basis `b` is exactly the negation of that under its sector exchange `19 - b`
  (`complement_antisymmetry`).
- **Regenerated evidence**: the validator regenerates every case from the
  report's split and budget and requires exact equality
  (`case_evidence_drift`).

Validation also rejects contract, weight or capacity drift, case-count or
case-major/basis-minor order drift, tampered summaries, and any
protected/final access, training or scientific-claim flag. Protected/final
split labels never generate a case.

Weights are the unchanged C6 reference `(alpha, beta, gamma) = (1, 0, 1)` and
capacity is `TrainableCapacity::reference_c6()` under every basis.

## Smoke-budget software diagnostics

Budget `StageCPreflightBudget::bounded(8, 0)` (64 cases per basis, 16 per
family). Correct counts per basis and family:

| Basis | `H+` slots | ReflectionDiscriminative | ReflectionNuisance | DirectionReversal | NonChiralControl |
| ---: | --- | ---: | ---: | ---: | ---: |
| 0 | `{0,1,2}` (canonical) | 16/16 | 12/16 | 16/16 | 8/16 |
| 1 | `{0,1,3}` | 8/16 | 16/16 | 8/16 | 16/16 |
| 2 | `{0,1,4}` | 8/16 | 15/16 | 8/16 | 8/16 |
| 3 | `{0,1,5}` | 8/16 | 16/16 | 8/16 | 16/16 |
| 4 | `{0,2,3}` | 8/16 | 14/16 | 8/16 | 16/16 |
| 5 | `{0,2,4}` | 16/16 | 8/16 | 8/16 | 16/16 |
| 6 | `{0,2,5}` | 8/16 | 12/16 | 13/16 | 8/16 |
| 7 | `{0,3,4}` | 0/16 | 8/16 | 8/16 | 8/16 |
| 8 | `{0,3,5}` | 8/16 | 12/16 | 8/16 | 16/16 |
| 9 | `{0,4,5}` | 8/16 | 8/16 | 16/16 | 8/16 |
| 10 | `{1,2,3}` | 8/16 | 12/16 | 0/16 | 16/16 |
| 11 | `{1,2,4}` | 8/16 | 12/16 | 8/16 | 13/16 |
| 12 | `{1,2,5}` | 8/16 | 16/16 | 8/16 | 16/16 |
| 13 | `{1,3,4}` | 8/16 | 12/16 | 3/16 | 8/16 |
| 14 | `{1,3,5}` | 8/16 | 16/16 | 8/16 | 16/16 |
| 15 | `{1,4,5}` | 16/16 | 8/16 | 8/16 | 8/16 |
| 16 | `{2,3,4}` | 16/16 | 8/16 | 8/16 | 8/16 |
| 17 | `{2,3,5}` | 16/16 | 8/16 | 8/16 | 16/16 |
| 18 | `{2,4,5}` | 8/16 | 8/16 | 8/16 | 8/16 |
| 19 | `{3,4,5}` | 0/16 | 12/16 | 0/16 | 16/16 |

## Recorded degeneracies

1. **Development equals Validation on this budget.** The Stage-C case stream
   derives case values from the pair id alone and only relabels the split, so
   identical budgets give identical counts on both splits. This is inherited
   from the landed Stage-C preflight (slice 30), not introduced here; it is
   recorded for the Stage-D attribution audit (slice 40), which must use
   disjoint pair-id ranges for any Development/Validation comparison.
2. **Sector exchange flips handedness.** Rank 19 negates `chi`, so the
   handedness- and direction-signed families flip from 16/16 to 0/16; this is
   the exact antisymmetry above, not a new effect.
3. **Beta = 0.** The C6 reference has `beta = 0`, so only the parity-odd
   channel depends on the basis; `s` changes at most by summation order.

## What is recorded

Per case and basis: score, parity-odd observable and correctness. Per basis
and family: case count and correct count. The canonical basis is not
privileged in reporting and no basis is chosen from these counts; interpreting
them as attribution evidence is reserved for the Stage-D attribution audit
(slice 40).
