# TDI-25 slice 43 — translation/origin stress suite (`tdi25-translation-origin-stress-v1`)

Status: experimental, non-final, Development/Validation only (Phase E). No
training, no protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched. Slices 49–50 remain pre-arm/decision
artifacts only.

## Contract

- Declared torsor-relevant transformations `ORIGIN_STRESS_TRANSFORMS` on
  `tdi25-matched-reference-population-v1` inputs, each leaving the physical
  twist/torsor pairing `v.R + omega.M(Q)` exactly invariant in real
  arithmetic:
  - `OriginShift`: rigid frame translation, key and query reduction points
    both move by `d`; carriers unchanged;
  - `KeyReduction`: the key torsor is re-reduced at `P + d` through the
    unchanged upstream `Torsor3::transport` (`M(P + d) = M(P) - d x R`);
  - `QueryReduction`: the query twist is re-reduced at `Q + d`
    (`v' = v + omega x d`, `omega` unchanged).
- Declared offset magnitudes `ORIGIN_STRESS_OFFSETS = [1, 1e3, 1e6]`, all
  reported, none selected. The direction of `d` is a unit vector drawn per
  case from a splitmix64 stream keyed by the FNV-1a 64 hash of the contract
  pin (`translation_origin_stress_seed`) and the case key only — never from
  an arm, a score or a target.
- T6 and C6 score the identical transformed matched input with identical
  reference capacities. Per (family, seed block, transformation, offset,
  arm): clean matches (reproducing the matched primary), stressed matches
  against the common target recomputed from the transformed input inside the
  evaluator, flips, label-free maximum absolute score change and maximum
  absolute change of the recomputed target. Clean scores must reproduce the
  matched primary bit for bit (`clean_reference_drift`).

## Validation

`validate_translation_origin_stress_report` rejects `contract_drift`,
`population_drift`, `seed_drift`, `capacity_mismatch`, `cell_count`,
`grid_drift`, `paired_count_drift`, `score_change_drift`, the structural
invariant `c6_origin_shift_drift` (C6 reads no position, so a rigid origin
shift must leave its score bit-for-bit unchanged), the
protected/training/claim/non-final flags, and regenerates every cell
(`case_evidence_drift`).

## Recorded degeneracies

- On the torsor-favorable family T6 matches every clean case and every
  transformed case at offsets `1` and `1e3` on both splits (score and
  recomputed-target drift `<= 1e-12`). At `|d| = 1e6` rounding of the
  transformed inputs reaches `~5e-10`, beyond the shared `1e-12` relative
  match tolerance: under `OriginShift` the factorized T6 form drifts, under
  `QueryReduction` the direct-pairing target drifts, and 3–4 of 8 cases per
  block flip. `KeyReduction` never flips at any offset on this
  population. This is floating-point conditioning of the declared offsets,
  not a modelling result.
- Key and query re-reduction change the raw carrier scalars that C6 reads,
  so C6 scores change by `O(|d|)`; on the chiral-favorable family the
  recomputed chiral target moves with the same scalars and C6 keeps every
  match, while T6 (zero clean matches there) is unaffected. These are
  consequences of the declared targets, not robustness claims.
- Mixed and neutral families have zero clean and stressed matches for both
  arms at this budget, as in slices 41–42.
