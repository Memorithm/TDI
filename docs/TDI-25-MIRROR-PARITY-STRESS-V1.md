# TDI-25 slice 44 — mirror/parity stress suite (`tdi25-mirror-parity-stress-v1`)

Status: experimental, non-final, Development/Validation only (Phase E). No
training, no protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched. Slices 49–50 remain pre-arm/decision
artifacts only.

## Contract

- Declared chiral-relevant transformations `MIRROR_STRESS_TRANSFORMS` on
  `tdi25-matched-reference-population-v1` inputs, built from the upstream
  TDI-24 involutions `M(x+, x-) = (x+, -x-)` and `J(x+, x-) = (x-, -x+)` on
  the query and key six-carriers; reduction points are never modified:
  - `SimultaneousMirror`: `(q, k) -> (Mq, Mk)`; `q.k` invariant,
    `chi(q, k)` negated.
  - `ComplexStructure`: `(q, k) -> (Jq, Jk)`; `q.k` and `chi(q, k)` both
    invariant (`J` orthogonal and commuting with itself).
  - `KeyMirror`: `(q, k) -> (q, Mk)`; one-sided reflection, neither pairing
    preserved in general.
- The transformation depends on the declared transform only: no arm, score,
  target, label or seed enters it, so no new seed material is introduced.
- T6 `(tdi25-matched-t6-evaluator-v1)` and C6 `(1, 0, 1)` score the identical
  transformed input with matched reference capacities. The family common
  target is recomputed inside the evaluator on the transformed input.
- Per (family, seed block, transformation, arm) cell: clean matches,
  stressed matches, flips, label-free maximum absolute score change and
  maximum absolute common-target change. Every transformation is reported;
  none is selected.

## Validation

`validate_mirror_parity_stress_report` rejects `contract_drift`,
`population_drift`, `capacity_mismatch`, `cell_count`, `grid_drift`,
`paired_count_drift`, `score_change_drift`, the structural C6 invariants
`c6_complex_structure_drift` (C6 score must be bit-for-bit unchanged under
`(Jq, Jk)`) and `c6_chiral_target_drift` (C6 must match every chiral-favorable
case under every transformation), the protected/training/claim/non-final
flags, and regenerates every cell (`case_evidence_drift`). The per-case
clean score must reproduce the matched primary bit for bit
(`clean_reference_drift`).

## Recorded degeneracies

- Because the common target is recomputed on the transformed input, each
  arm keeps matching the family whose target it computes: T6 matches every
  torsor-favorable case and C6 every chiral-favorable case under all three
  transformations, with zero flips. The suite therefore measures score and
  target displacement, not a robustness gap between arms.
- C6 is bit-for-bit invariant under `(Jq, Jk)` on the dyadic matched
  population (score change `0`), as required.
- Neutral targets `|q - k|^2` are invariant under `(Mq, Mk)` and `(Jq, Jk)`
  (target change `0`); under those two transformations T6 is also unchanged
  on chiral-favorable and neutral inputs, whose reduction points are at the
  origin (the key-only mirror does move it).
- At the smoke budget on Validation one torsor-favorable C6 case matches
  coincidentally after `(Jq, Jk)` (one flip into a match); no other cross
  family/arm match is observed. Mixed targets are matched by neither arm,
  clean or stressed.
