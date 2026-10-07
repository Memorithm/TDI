# TDI-24 structure-preserving learned basis prototype v1 (slice 36)

Status: Phase-D Development/Validation software prototype. Contract
`tdi24-learned-basis-prototype-v1`. No training, no angle fitting or
selection, no protected/final access, no confirmatory execution and no
scientific claim. No configuration freeze pin is introduced or changed;
holdouts are untouched.

## Parameterisation

A learnable basis is restricted to orthogonal transforms of the six-slot
carrier, written as the product of the `C(6,2) = 15` Givens rotations in
lexicographic plane order:

`O(theta) = G(0,1; theta_01) G(0,2; theta_02) ... G(4,5; theta_45)`.

Scoring under `O` is scoring `(O q, O k)` with the canonical `M`/`J`, i.e. using
the effective operators `M' = O^T M O` and `J' = O^T J O`. Because `O` is
orthogonal, the chiral algebra is preserved by construction and the direct
channel is invariant; only `chi` (and `m`, which has weight `beta = 0`) can
change. The subgroup `diag(R, R)`, `R` in `O(3)`, commutes with both `M` and
`J` and is a pure gauge: it cannot change any score.

## Declared probes (not tuned, not selected)

| Probe | Role | Non-zero angles |
| ---: | --- | --- |
| 0 | Identity | none |
| 1 | Gauge `diag(R, R)` | `theta_01 = theta_34 = pi/5`, `theta_12 = theta_45 = pi/7` |
| 2 | Sector mixing | `theta_04 = pi/4` (slots 0 and 4 are not paired by `J`; the `(0,3)` plane would commute with `J` and be a gauge direction) |
| 3 | Generic | `theta_p = (p + 1) pi / 32` for plane rank `p = 0..14` |

The probes are prototype evaluation points of the parameterisation. They are
not freeze pins and no probe is chosen from outcomes; all are always reported.

## Checked identities and rejections

- **Orthogonality**: `||O^T O - I||_inf <= 1e-12` (`orthogonality_failure`);
  scaling or shearing transforms are rejected. Non-finite angles or entries
  are rejected (`non_finite_parameter`).
- **Algebra**: `M'^2 = I`, `J'^T = -J'`, `J'^2 = -I`, `M' J' M' = -J'` within the
  declared tolerance `LEARNED_BASIS_TOLERANCE = 1e-12`
  (`basis_algebra_failure`).
- **Roles**: identity/gauge probes must commute with `M` and `J`
  (`gauge_structure_drift`); sector-mixing/generic probes must not
  (`probe_role_drift`). Probe drift or reordering: `probe_order_drift`,
  `probe_set_incomplete`.
- **Identity reproduction**: probe 0 reproduces the Stage-C C6 evaluator bit
  for bit (`canonical_reference_drift`).
- **Gauge invariance**: probe 1 scores equal the reference within
  `1e-12 * max(1, |reference|)` (`gauge_invariance_drift`).
- **Regenerated evidence**: the validator regenerates every case from the
  report's split and budget and requires exact equality
  (`case_evidence_drift`).

Validation also rejects contract, weight, capacity or parameter-count drift,
case-count or case-major/probe-minor order drift, tampered summaries, and any
protected/final access, training or scientific-claim flag. Protected/final
split labels never generate a case.

Weights are the unchanged C6 reference `(alpha, beta, gamma) = (1, 0, 1)`;
scored capacity is `TrainableCapacity::reference_c6()` under every probe. The
15 basis angles are declared as `basis_parameter_count` but none is trained.

## Smoke-budget software diagnostics

Budget `StageCPreflightBudget::bounded(8, 0)` (64 cases per probe, 16 per
family). Correct counts per probe and family:

| Probe | ReflectionDiscriminative | ReflectionNuisance | DirectionReversal | NonChiralControl |
| ---: | ---: | ---: | ---: | ---: |
| 0 identity | 16/16 | 12/16 | 16/16 | 8/16 |
| 1 gauge | 16/16 | 12/16 | 16/16 | 8/16 |
| 2 sector mixing | 16/16 | 12/16 | 16/16 | 8/16 |
| 3 generic | 8/16 | 12/16 | 8/16 | 16/16 |

## Recorded degeneracies

1. **Development equals Validation on this budget**, inherited from the
   Stage-C case stream (slice 30); the Stage-D audit (slice 40) must use
   disjoint pair-id ranges for any Development/Validation comparison.
2. **Gauge directions are unidentifiable.** `diag(R, R)` leaves every score
   unchanged, so at most `15 - 3 = 12` angles can affect scores; any future
   training must quotient or fix this gauge.
3. **Sector mixing in one plane changes scores but not these counts** on this
   bounded stream (scores differ beyond tolerance on some cases; the sign of
   every score is unchanged). This is a property of the case values, not
   evidence of invariance in general. Rotations in the `J`-paired planes
   `(0,3)`, `(1,4)`, `(2,5)` commute with `J` and leave `chi` invariant.
4. **Beta = 0.** Only the parity-odd channel depends on the basis.

## What is recorded

Per case and probe: score, Stage-C reference score and correctness. Per probe
and family: case count and correct count. Interpreting these counts as
attribution evidence is reserved for the Stage-D attribution audit (slice 40).
