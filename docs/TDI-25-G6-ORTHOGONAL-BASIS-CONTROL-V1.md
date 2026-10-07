# TDI-25 G6 orthogonal-basis control v1 (slice 36)

Status: Phase-D Development/Validation software control. Contract
`tdi25-g6-orthogonal-basis-control-v1`. No training, no protected/final
access, no confirmatory execution and no scientific claim. No configuration
freeze pin is introduced or changed; holdouts are untouched.

## Population, rotations and arms

The control reuses the unchanged matched population
`tdi25-matched-reference-population-v1` and the Stage-C preflight budget
(every required family, seed blocks `0..seed_blocks`, `cases_per_block` cases
per block, canonical order).

The deterministic basis rotations are the four declared probes of the TDI-24
slice-36 prototype (`tdi24-learned-basis-prototype-v1`), consumed unchanged:
identity, gauge `diag(R, R)`, sector mixing in the `(0,4)` plane and a generic
15-plane rotation. Each is an orthogonal `O` applied to the six query and six
key scalars. No new angle, seed or parameter is introduced.

- **G6**: generic six-component dot product (`tdi25-generic6-control-v1`) on
  the unrotated pair and on `(O q, O k)`.
- **C6 contrast**: the unchanged matched C6 score (reused from the matched
  primary run and re-checked bit for bit) on the unrotated pair and on the
  rotated pair.

Capacity is `ParameterReadoutCapacity::reference_g6()` before and after
rotation.

## Checked identities and rejections

- **Identity probe**: G6 and C6 rotated scores equal the reference bit for bit
  (`identity_probe_drift`).
- **G6 basis invariance**: for every probe and case,
  `|g6_rotated - g6_reference| <= 1e-12 * max(1, |g6_reference|)` using the
  upstream `LEARNED_BASIS_TOLERANCE` (`g6_rotation_invariance_drift`).
- **Unchanged probes**: the probe set must equal the upstream declared set
  (`probe_set_drift`, `probe_contract_drift`); upstream probe validation
  (orthogonality, algebra) is re-run on every scoring.
- **Regenerated evidence** (`case_evidence_drift`), contract, population,
  generic-contract and capacity drift, case count, case-major/probe-minor
  order, tampered summaries and access/training/claim flags are rejected.
  Protected/final labels never generate a case.

## Smoke-budget software diagnostics

Smoke budget (2 blocks x 8 cases per family). G6 matches are identical before
and after every rotation. "C6 changed" counts cases whose C6 score moved
beyond the same tolerance:

| Split | Probe | G6 matches (Torsor/Chiral/Mixed/Neutral) | C6 changed (Torsor/Chiral/Mixed/Neutral) |
| --- | --- | --- | --- |
| Development | 0 identity | 0/0/0/0 | 0/0/0/0 |
| Development | 1 gauge | 0/0/0/0 | 0/0/0/0 |
| Development | 2 sector mixing | 0/0/0/0 | 16/16/16/16 |
| Development | 3 generic | 0/0/0/0 | 16/16/16/16 |
| Validation | 0 identity | 2/0/0/0 | 0/0/0/0 |
| Validation | 1 gauge | 2/0/0/0 | 0/0/0/0 |
| Validation | 2 sector mixing | 2/0/0/0 | 16/16/16/16 |
| Validation | 3 generic | 2/0/0/0 | 16/16/16/16 |

## Recorded degeneracies

1. **G6 invariance is a theorem, not evidence.** `q.k` is invariant under any
   orthogonal `O`; the control confirms the software has no privileged G6
   basis. It cannot by itself attribute anything to T6 or C6 structure.
2. **Gauge probe.** `diag(R, R)` commutes with `M` and `J`, so C6 is also
   unchanged within tolerance under probe 1.
3. **G6 rarely matches the family targets** on the smoke budget; these are
   software diagnostics only.

Interpreting these counts as attribution evidence is reserved for the Stage-D
attribution audit (slice 40).
