# TDI-25 slice 45 — mixed adversarial suite (`tdi25-mixed-adversarial-suite-v1`)

Status: experimental, non-final, Development/Validation only (Phase E). No
training, no protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched. Slices 49–50 remain pre-arm/decision
artifacts only.

## Contract

- Both declared transformation classes are combined; no new transformation,
  offset or seed is introduced:
  - mirror/parity class: the slice-44 `MIRROR_STRESS_TRANSFORMS`
    (`(Mq, Mk)`, `(Jq, Jk)`, `(q, Mk)`);
  - translation/origin class: the slice-43 `ORIGIN_STRESS_TRANSFORMS`
    (origin shift, key re-reduction, query re-reduction) at the slice-43
    offsets `[1, 1e3, 1e6]`, with the slice-43 per-case unit directions drawn
    from the slice-43 contract seed (`translation_origin_stress_seed`).
- Composition order is declared: the mirror acts first on the six-carriers,
  then the origin stress acts on the mirrored input
  (`mixed_stress_matched_input`). All 3 x 3 x 3 = 27 compositions are
  reported; none is selected.
- Deterministic oracle: the family common target is recomputed inside the
  evaluator on the composed input. T6 and C6 score the identical composed
  input with matched reference capacities.
- Per (family, seed block, mirror, origin, offset, arm) cell: clean matches,
  stressed matches, flips, max absolute score change, max common-target
  change and max origin-component effect (score on the composed input minus
  score on the mirror-only input).

## Validation

`validate_mixed_adversarial_suite_report` rejects `contract_drift`,
`population_drift`, `seed_drift` (the reused slice-43 seed),
`capacity_mismatch`, `cell_count`, `grid_drift`, `paired_count_drift`,
`score_change_drift`, the structural C6 invariants `c6_origin_shift_drift`
(C6 reads no position, so the origin-shift component must leave its score
bit-for-bit unchanged) and `c6_chiral_target_drift` (C6 must match every
chiral-favorable case under every composition), the protected/training/
claim/non-final flags, and regenerates every cell (`case_evidence_drift`).
The per-case clean score must reproduce the matched primary bit for bit
(`clean_reference_drift`).

## Recorded degeneracies

- The composition inherits both earlier degeneracies: because the target is
  recomputed on the composed input, T6 keeps matching torsor-favorable cases
  and C6 every chiral-favorable case except where f64 cancellation intervenes.
- T6 torsor-favorable flips appear only at offset `1e6` under the origin-shift
  and query re-reduction components (1–4 of 8 per cell, after every mirror
  transformation), never at `<= 1e3` and never under key re-reduction — the
  slice-43 pattern is unchanged by the mirror component.
- A few coincidental single matches (one case per cell, flips into a match)
  appear for Mixed (Development block 1) and torsor-favorable C6 (Validation
  block 1) after `(Jq, Jk)` or `(q, Mk)`; they are reported, not interpreted.
- The suite measures displacement under combined stress; it does not by
  itself establish a robustness gap between arms.
