# TDI-12.0 Status

Status: **ACTIVE STAGE-0 BOOTSTRAP — not frozen; confirmatory execution unauthorized**

## Current state

TDI-12.x was introduced as a programme stub and is now bootstrapped as Stage 0.

Primary surfaces:

- `docs/TDI-12-PROGRAMME.md`;
- `docs/TDI-12.0-SCOPE.md`;
- `docs/TDI-12.0-STATUS.md`;
- `docs/tdi12/tdi12.0-stage0-freeze.template.json`;
- `scripts/check-tdi12.0-freeze-template.py`;
- `tdi-operator/src/ordinal.rs`;
- `tdi-operator/tests/ordinal_stage0_bootstrap.rs`;
- `scripts/check-tdi12-stage0-bootstrap.sh`.

## What Stage 0 lands (exact / engineering, non-pinning)

1. **EXACT** average-rank assignment for ties (1-based midranks).
2. **EXACT** Spearman ρ via Pearson correlation of average ranks, fail-closed on full ties.
3. **EXACT** Kendall τ-b companion with pair-tie adjustment, fail-closed on full ties.
4. **EXACT** strictly-increasing affine invariance of Spearman and Kendall.
5. **EXACT** identity-ordering control on nondegenerate samples; reverse-order
   unit anticorrelation on distinct values.
6. Candidate (non-frozen) Green-band response observables over TDI-10 `GreenBands`:
   mid-diagonal Green, Green trace, mean-abs off-diagonal Green, with **EXACT**
   wiring identity to the public `GreenBands` extractors.
7. Dimension-only ranking key that does not consult response observables.
8. **EXACT** coefficient-only Stage-0 control keys: Frobenius-norm baseline and
   Gershgorin dominance-margin baseline (do not freeze `control_battery`).
9. **EXACT** deterministic shuffle control that destroys ρ = τ = 1 on a
   nondegenerate length≥3 sample (`shuffled_family` scaffolding only).
10. Machine-readable Stage-0 freeze **template** with every scientific field
    `unresolved_blocking` and both execution flags `false`, plus a dedicated
    fail-closed freeze-template validator that refuses invented pins.
11. **EXACT** constant-Toeplitz Frobenius closed form
    `sqrt(n a²+(n−1)b²)` with width monotonicity and ρ = τ = 1 vs dimension on
    a positive width ladder (`norm_baseline` / population scaffolding).
12. **EXACT** constant-Toeplitz Gershgorin margin `a−2|b|` for all `n≥3`
    (width-invariant); Spearman/Kendall vs dimension fail closed — REFUTES
    covert dimension keying (`spectral_gap_baseline` scaffolding).
13. **EXACT** `rank_normalize` / `negate_values` normalization scaffolding and
    `tie_heavy_adversarial_sample` midranks; observable-ladder evaluation;
    non-authorizing candidate ids for populations / normalization / split /
    typed rejection / provenance / derivation — fields remain
    `unresolved_blocking`.
14. **EXACT** constant diagonal-only Frobenius `√(n a²)` with width monotonicity
    and ρ = τ = 1 vs dimension (`DiagonalOnlyWidthLadder` / `norm_baseline`
    scaffolding; edge-zero specialization of item 11).
15. **EXACT** constant diagonal-only Gershgorin margin `= a` for **all** `n ≥ 1`
    (stronger width-invariance than Toeplitz n≥3); Spearman/Kendall vs dimension
    fail closed — REFUTES covert dimension keying.
16. **EXACT** diagonal-only Green closed forms under `a + shift > 0`:
    MidDiagonalGreen `= 1/(a+t)` width-invariant (REFUTES as dimension key);
    GreenTrace `= n/(a+t)` with ρ = τ = 1 vs dimension; MeanAbsOffDiagonalGreen
    `≡ 0` (REFUTES as dimension key). Does not freeze
    `response_observable_registry`.

## Authorization state

- TDI-12.0 is **not frozen**.
- `confirmatory_execution_authorized` and `final_execution_authorized` remain **false**.
- TDI-12.1 evaluator work beyond the Stage-0 exact primitives may begin only after
  `scripts/check-tdi12-stage0-bootstrap.sh` passes on the relevant branch/`main`
  and any evaluator-required freeze fields are explicitly resolved by a later
  gated change.
- No TDI-12 confirmatory population, final seed list, or final result payload is
  authorized.

## Pin scout (orthogonal series)

Stage 0 invents **no** TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins. Existing
authorized pins stay 3/17, 1/14, 0/12. No TDI-7.2 / TDI-8.2 / TDI-9.2 contact.
TDI-9.3.0 does not authorize TDI-9.1 / TDI-9.2. TDI-10.19 reverse/cross family
composition is orthogonal Stage-10 EXACT work and invents no TDI-12.0 pins.
DiagonalOnlyWidthLadder closed forms remain non-authorizing candidates.
Post-#215 TDI-11.2 resource-accounting scaffolding is orthogonal and likewise
invents no Stage-0 freeze pins.

## Next engineering slice after DiagonalOnly / Toeplitz Stage-0 controls

Still Stage-0 / non-confirmatory: review and (only with explicit evidence)
resolve freeze fields among the declared non-authorizing candidates for
operator populations, ranking metric, tie policy, normalization, and split
discipline **without** authorizing confirmatory execution or setting execution
flags true. Only after explicit field resolutions may TDI-12.1 synthetic
evaluator fixtures expand under those resolutions.
