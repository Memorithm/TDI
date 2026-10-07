# TDI-25 slice 41 — multi-seed replication (`tdi25-multi-seed-replication-v1`)

Status: experimental, non-final, Development/Validation only (first Phase-E
slice). No training, no protected/final/holdout access, no scientific claim.
Holdouts TDI-7.2/8.2/9.2 are untouched. Slices 49–50 remain pre-arm/decision
artifacts only.

## Contract

- Frozen paired seed blocks `REPLICATION_SEED_BLOCKS = [0, 1, 2, 3, 4, 5, 6, 7]`
  of `tdi25-matched-reference-population-v1`, every required family. Only the
  budget's `cases_per_block` is consumed.
- Pairing: T6 and C6 outcomes of each case must carry the same canonical case
  digest and case id (`pairing_drift`); blocks must be disjoint by digest
  (`block_overlap`).
- Per (block, family): cases, T6/C6 correct counts and paired discordance
  (T6-only, C6-only). The paired identity
  `t6_matches - t6_only == c6_matches - c6_only` is enforced.
- Pooled sums and descriptive tallies of blocks (pooled over families) with
  C6 ahead, T6 ahead or tied. No test or decision rule is attached.

## Validation

`validate_multi_seed_replication_report` rejects `contract_drift`,
`population_drift`, `block_set_drift`, `cell_count`, `cell_order`,
`paired_count_drift`, `pooled_sum_drift`, `sign_tally_drift`, the
protected/training/claim/non-final flags, and regenerates every cell
(`case_evidence_drift`).

## Recorded degeneracies

- At the smoke budget (8 cases per block, 256 paired cases per split) the
  arms are almost perfectly complementary: Development T6 66 / C6 65 correct
  with only 2 concordant-correct cases (T6-only 64, C6-only 63); Validation
  66 / 66 (64 / 64). This reflects the family design (torsor-favorable vs
  chiral-favorable targets), not a performance claim.
- Block tallies are mostly ties (Development 0 / 1 / 7, Validation 1 / 1 / 6
  for C6 ahead / T6 ahead / tied); they are descriptive only. The primary
  contrast and its decision rule are frozen only in slice 49.
