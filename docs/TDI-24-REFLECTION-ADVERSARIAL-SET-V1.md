# TDI-24 slice 43 — reflection adversarial set (`tdi24-reflection-adversarial-set-v1`)

Status: experimental, non-final, Development/Validation only. No training, no
protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched.

## Contract

- Hard mirrored pairs are generated from declared geometry only
  (`reflection_adversarial_pairs` takes a split and a budget, never an arm,
  and evaluates no score). Per pair, a query `q` (component magnitudes in
  `[0.5, 1]`) and a nuisance `r` are drawn from a splitmix64 stream keyed by
  the FNV-1a 64 hash of the contract pin (`reflection_adversarial_seed`) and
  the slice-18 registered seed of `(split domain, reflection_discriminative,
  pair_id)`. `r` is Gram–Schmidt orthogonalised against `q` and `Jq` and
  rescaled to `ADVERSARIAL_NUISANCE_RATIO * |q| = 2|q|`.
- Right member: `k = sigma*rho*scale*q - scale*Jq + r`, hence
  `chi(q, k) = scale*|q|^2 > 0` and `s(q, k) = sigma*rho*chi`; the nuisance
  changes neither channel but dominates `|k|`. The generator checks both
  channels against the declared values (`construction_drift`). Left member:
  the exact mirror `(Mq, Mk)` — even channels bit-for-bit equal, `chi`
  bit-for-bit negated. Declared handedness: right `+1`, left `-1`.
- Declared grid, all reported: dominance ratios
  `ADVERSARIAL_DOMINANCE_RATIOS = [0.5, 0.9, 0.99, 1.01, 2.0]` (straddling
  the C6 boundary `rho = 1`), chirality scales
  `ADVERSARIAL_CHIRALITY_SCALES = [1, 1e-3]` (ordinary and near-achiral),
  direct-channel sign `sigma` alternating `+1, -1` by pair.
- Paired arms: C6 `(1, 0, 1)` and direct-only `(1, 0, 0)` score the identical
  pairs with matched zero-trainable capacity (`reference_c6`). Per
  (scale, ratio, arm): pairs, members whose score sign matches the declared
  handedness, and fully discriminated pairs (both members correct).

## Validation

`validate_reflection_adversarial_report` rejects `contract_drift`,
`seed_drift`, `capacity_mismatch`, `cell_count`, `grid_drift`,
`paired_count_drift`, the structural invariant
`direct_only_discrimination` (mirrored members share every even channel, so
the direct-only arm can never separate a pair), the protected/training/
claim/non-final flags, `pair_digest_drift` (FNV-1a 64 over the bit patterns
of every regenerated pair) and `case_evidence_drift` (regenerated cells).

## Recorded degeneracies

- The outcome is analytically determined by construction: C6 separates a
  mirrored pair iff `chi > |s|`, i.e. `rho < 1`; for `rho > 1` exactly one
  member per pair is correct (which one follows `sigma`). The direct-only
  arm always gets exactly one member per pair right and never discriminates.
  At the smoke budget (4 pairs per cell) the observed counts match this on
  both splits and both scales, including the near-boundary ratios
  `0.99`/`1.01` and the near-achiral scale `1e-3`. This qualifies the
  generator and the evaluator path; it is not evidence about learned models.
- Hardness is defined geometrically (`|s|/chi`, `chi/|q|^2`, nuisance
  norm); the boundary coincides with the fixed C6 reference weights
  `(1, 0, 1)`, so the set is adversarial for this reference, not for an
  arbitrary trained arm.
- The mirror-even channel `m` is left uncontrolled (determined by `q` and
  `r`); it does not enter either reference arm (`beta = 0`).
