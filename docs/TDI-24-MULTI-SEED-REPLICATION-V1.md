# TDI-24 slice 41 — multi-seed replication (`tdi24-multi-seed-replication-v1`)

Status: experimental, non-final, Development/Validation only (first Phase-E
slice). No training, no protected/final/holdout access, no scientific claim.
Holdouts TDI-7.2/8.2/9.2 are untouched. Slices 49–50 remain pre-arm/decision
artifacts only.

## Contract

- Frozen seed blocks `MULTI_SEED_BLOCKS = [0, 1, 2, 3]`. Block `b` runs the
  Stage-C preflight with `first_pair_id = b * MULTI_SEED_BLOCK_STRIDE`, where
  the stride equals the per-family pair cap `MAX_PREFLIGHT_PAIRS_PER_FAMILY`,
  so no admissible budget can make two blocks overlap.
- Disjointness is also checked on evidence: the case digests of all blocks
  must be pairwise distinct (`block_overlap`).
- The caller chooses `pairs_per_family` only; the base offset is frozen at 0
  (`frozen_offset_drift`). Nothing is selected or reweighted per block.
- Per block: cases per arm, V6/C6 correct counts and the unchanged slice-27
  paired summary. Pooled: summed counts. Descriptive tallies of blocks with
  C6 ahead, V6 ahead or tied; they are not a test and carry no claim.
- Block 0 reproduces the Stage-C preflight on the same budget.

## Validation

`validate_multi_seed_replication_report` rejects `contract_drift`,
`preflight_contract_drift`, `frozen_offset_drift`, `block_set_drift`,
`block_count_drift` (including a summary bound to another split),
`pooled_sum_drift`, `sign_tally_drift`, the protected/training/claim/
non-final flags, and regenerates every block (`case_evidence_drift`).

## Recorded degeneracies

- At the smoke budget (4 pairs per family) all four blocks report identical
  counts (32 cases per arm; V6 20, C6 26 on Development) even though their
  case digests are distinct: correctness on the Phase-B generators is
  determined by family construction rather than by pair id at this budget.
  The replication therefore shows determinism and disjointness, not
  sampling variability; this is recorded, not tuned.
- Sign tallies over four blocks are descriptive only; no inferential rule is
  attached here (the primary contrast is frozen only in slice 48).
