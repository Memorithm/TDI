# TDI-25 position-geometry ablation v1 (slice 37)

Status: Phase-D Development/Validation software ablation. Contract
`tdi25-position-geometry-ablation-v1`. No training, no geometry selection, no
protected/final access, no confirmatory execution and no scientific claim. No
configuration freeze pin is introduced or changed; holdouts are untouched.

## Arms

On the unchanged matched population (`tdi25-matched-reference-population-v1`,
Stage-C preflight budget), T6 (`(v + Q x omega).R + omega.C`) is scored with
the six query and six key scalars unchanged under each geometry arm
(`POSITION_GEOMETRY_ABLATION_ARMS`, fixed order):

| Arm | Geometry |
| ---: | --- |
| 0 | generated matched geometry (reference) |
| 1 | registry `linear` (`tdi25-geometry-linear-abscissa-v1`) |
| 2 | registry `helical` (`tdi25-geometry-helical-octagon-v1`) |
| 3 | registry `learned` frozen table (`tdi25-geometry-learned-table-v1`) |

For registry arms the reduction point is `P = point(arm, 2c)` and the query
point `Q = point(arm, 2c + 1)` for case index `c`, via the frozen
`tdi25-position-geometry-arm-v1` registry. Every arm is always reported; no
arm is privileged or selected. Capacity is
`ParameterReadoutCapacity::reference_t6()` under every arm. Targets are minted
from the generated case.

## Checked identities and rejections

- **Matched reproduction**: arm 0 reproduces the matched primary T6 score and
  match bit exactly (`matched_reference_drift`).
- **Registry consistency**: positions equal the registry points (tested), and
  any drift is caught by regeneration (`case_evidence_drift`).
- Contract, population, geometry-contract, torsor-contract, arm-set and
  capacity drift, case count and case-major/arm-minor order, tampered
  summaries and access/training/claim flags are rejected; unregistered arms
  fail (`arm_not_registered`). Protected/final labels never generate a case.

## Smoke-budget software diagnostics

Smoke budget (2 blocks x 8 cases per family). Matches / non-zero transport
term (out of 16):

| Split | Arm | TorsorFavorable | ChiralFavorable | Mixed | Neutral |
| --- | --- | --- | --- | --- | --- |
| Development | 0 generated | 16 / 16 | 0 / 0 | 0 / 16 | 0 / 0 |
| Development | 1 linear | 0 / 12 | 0 / 15 | 0 / 16 | 0 / 14 |
| Development | 2 helical | 0 / 16 | 0 / 15 | 0 / 16 | 0 / 16 |
| Development | 3 learned | 0 / 16 | 0 / 16 | 0 / 16 | 0 / 16 |
| Validation | 0 generated | 16 / 14 | 0 / 0 | 0 / 15 | 0 / 0 |
| Validation | 1 linear | 0 / 15 | 0 / 13 | 0 / 14 | 0 / 14 |
| Validation | 2 helical | 0 / 16 | 0 / 16 | 0 / 13 | 0 / 16 |
| Validation | 3 learned | 0 / 15 | 0 / 15 | 0 / 15 | 0 / 16 |

## Recorded degeneracies

1. **TorsorFavorable target is the generated-geometry T6 score**, so arm 0
   matches by construction and every other geometry loses those matches; this
   restates the target construction and is not attribution evidence.
2. **ChiralFavorable/Neutral cases are generated with `P = Q = 0`**; registry
   arms introduce transport where the generated geometry has none.
3. **`External` excluded.** It requires caller-supplied coordinates; no
   external geometry is invented here.
4. **Index mapping** `2c`, `2c + 1` is a declared rule of this ablation, not
   tuned and not a freeze pin.

Interpreting these counts as attribution evidence is reserved for the Stage-D
attribution audit (slice 40).
