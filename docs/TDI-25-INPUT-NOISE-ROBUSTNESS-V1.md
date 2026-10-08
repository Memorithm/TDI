# TDI-25 slice 42 — input/noise robustness (`tdi25-input-noise-robustness-v1`)

Status: experimental, non-final, Development/Validation only (Phase E). No
training, no protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched. Slices 49–50 remain pre-arm/decision
artifacts only.

## Contract

- Declared perturbation families `INPUT_NOISE_FAMILIES` on the 18 shared
  input scalars of `tdi25-matched-reference-population-v1`: `Isotropic` (all
  18), `CarrierOnly` (the 12 query/key carrier scalars), `PositionOnly` (the
  6 key/query position scalars). Declared absolute amplitudes
  `INPUT_NOISE_AMPLITUDES = [1e-3, 1e-2, 1e-1]`. The full grid is reported;
  nothing is selected.
- Deterministic perturbation (`perturb_matched_input`): scalar slot `s` of
  case `c` in seed block `b` receives a value in `[-a, a]` from a splitmix64
  stream keyed by the seed, the case key `(b << 32) | c` and the slot. The
  seed is the FNV-1a 64 hash of the contract pin (`input_noise_seed`); there
  is no free seed constant. The masked families reuse the isotropic draws
  slot for slot.
- Identical perturbation distributions by arm: T6 and C6 score the identical
  perturbed `MatchedInput`; the draw never depends on the arm. Reference
  capacities `reference_t6` / `reference_c6` are unchanged.
- Per (family, seed block, noise family, amplitude, arm): clean matches of
  the matched primary, matches on the perturbed input against the common
  target recomputed by the evaluator from that same perturbed input, match
  flips, and the label-free maximum absolute score change. The clean arm
  score must reproduce the matched primary bit for bit
  (`clean_reference_drift`).

## Validation

`validate_input_noise_robustness_report` rejects `contract_drift`,
`population_drift`, `seed_drift`, `capacity_mismatch`, `cell_count`,
`grid_drift`, `paired_count_drift` (flip bounds), `score_change_drift`, the
protected/training/claim/non-final flags, and regenerates every cell
(`case_evidence_drift`).

## Recorded degeneracies

- At the smoke budget (2 seed blocks × 8 cases per family, 64 paired cases
  per split) the match-flip metric is fully degenerate: **0 flips in every
  cell** on Development and Validation. Clean and perturbed match counts are
  identical in every cell: T6 16/16 on TorsorFavorable and 0/16 on every
  other family; C6 16/16 on ChiralFavorable and 0/16 on every other family
  (Mixed and Neutral are 0/16 for both arms). Because the
  common target is recomputed from the perturbed input and the favorable arm
  score equals that target identically, this measures preservation of the
  by-construction identity under noise, not robustness of accuracy. It is
  not evidence of architecture superiority.
- C6 never reads the position scalars: under `PositionOnly` its score change
  is exactly 0 at every amplitude. T6 reads them through the transport term
  and moves even on ChiralFavorable/Neutral cases whose clean positions are
  zero.
- The label-free maximum score change scales linearly with amplitude: about
  3× to 11× the amplitude across cells for both arms (for example
  Development TorsorFavorable isotropic: T6 7.47e-3 / 7.47e-2 / 7.55e-1,
  C6 6.56e-3 / 6.57e-2 / 6.62e-1). Descriptive only.
- Amplitudes are absolute, not relative to each case's norm.
