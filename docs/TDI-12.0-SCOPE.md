# TDI-12.0 — Stage-0 Scope and Stage Gate

Status: **ACTIVE STAGE-0 BOOTSTRAP / NON-FINAL**

## Purpose

TDI-12.0 bootstraps the ordinal-universality programme without freezing
scientific choices or authorizing confirmatory runs.

It lands:

- the programme map and Stage-0 status/scope surfaces;
- a machine-readable freeze **template** whose fields remain
  `unresolved_blocking` until separately evidenced;
- a dedicated Stage-0 freeze-template validator that refuses invented pins and
  keeps both execution flags false;
- exact finite ordinal ranking primitives in `tdi-operator`;
- candidate Green-band response extractors that consume only generic TDI-10
  primitives, with EXACT wiring tests to public `GreenBands`;
- coefficient-only Stage-0 control keys (norm / Gershgorin margin / shuffle);
- closed-form constant-Toeplitz Frobenius / Gershgorin identities and
  rank-normalize / negate-response / tie-heavy / observable-ladder scaffolding;
- non-authorizing candidate declarations for remaining freeze-template fields
  (populations, normalization, split, typed rejection, provenance, derivation)
  while every scientific field stays `unresolved_blocking`;
- integrity CI that refuses confirmatory/final execution flags and forbidden
  holdout contact.

## Exact Stage-0 claims

| Claim | Status |
| --- | --- |
| Average ranks for ties are unique midranks for a finite multiset | **EXACT** |
| Spearman ρ is Pearson correlation of those ranks; undefined on full ties | **EXACT** |
| Kendall τ-b companion with classical pair-tie denominators | **EXACT** |
| Strictly increasing affine maps preserve Spearman and Kendall | **EXACT** |
| Identity ordering of a nondegenerate sample against itself yields ρ = τ = 1 | **EXACT** |
| Dimension-only keys equal operator length and ignore Green values | **EXACT** |
| Reverse order of distinct values yields Spearman = Kendall = −1 | **EXACT** |
| Candidate observables wire identically to public TDI-10 `GreenBands` extractors | **EXACT** |
| Coefficient Frobenius-norm and Gershgorin-margin keys ignore Green values | **EXACT** |
| Fixed-seed shuffle destroys ρ = τ = 1 on a nondegenerate length≥3 sample | **EXACT** |
| Candidate Green observables are finite on declared positive Toeplitz tests | **EXACT** finite evaluation under TDI-10 pivot conditions |
| Stage-0 freeze-template validator refuses invented pins / execution flags | **EXACT** integrity (engineering) |
| Constant-Toeplitz Frobenius closed form; width-monotone; ρ=τ=1 vs dimension | **EXACT** |
| Constant-Toeplitz Gershgorin width-invariant for n≥3 (degenerate vs dimension) | **EXACT** (REFUTES covert dimension key) |
| Rank-normalize preserves ρ=τ=1; negate-response yields ρ=τ=−1 on distinct values | **EXACT** |
| Tie-heavy adversarial midranks with non-degenerate identity correlations | **EXACT** |
| Observable ladder evaluation fail-closed; population candidate ids non-pinning | **EXACT** integrity / scaffolding |

## Non-claims / still forbidden

Stage 0 does **not**:

- freeze operator populations, thresholds, ranking metric choice, normalization,
  or split discipline;
- authorize TDI-12.2/12.3/12.4 transport experiments or confirmatory populations;
- pin any TDI-8.1 / TDI-9.1 / TDI-11.2 freeze field;
- contact TDI-7.2 / TDI-8.2 / TDI-9.2 surfaces;
- claim ordinal universality, soft-edge theorems, or Riemann consequences;
- treat TDI-10.19 reverse/cross composition as a TDI-12 pin or confirmatory authorization.

## Stage gate toward TDI-12.1

TDI-12.1 non-final synthetic evaluator work may expand only when:

1. `docs/TDI-12-PROGRAMME.md`, `docs/TDI-12.0-SCOPE.md`, and
   `docs/TDI-12.0-STATUS.md` are merged;
2. `docs/tdi12/tdi12.0-stage0-freeze.template.json` remains present with both
   execution flags `false` until a later explicit authorization change;
3. `scripts/check-tdi12.0-freeze-template.py` and
   `scripts/check-tdi12-stage0-bootstrap.sh` pass;
4. any evaluator-required freeze fields are resolved by an explicit later PR
   (no silent upgrades).

## Required Stage-0 controls (implemented)

- identity ordering;
- dimension-only ordering;
- monotone rescaling (strictly increasing affine);
- tie-heavy full-tie fail-closed rejection;
- reverse-order unit anticorrelation;
- coefficient Frobenius-norm baseline (`norm_baseline` scaffolding);
- Gershgorin dominance-margin baseline (`spectral_gap_baseline` scaffolding);
- deterministic shuffled-family control (`shuffled_family` scaffolding).

These control keys do **not** resolve or freeze the `control_battery` template
field. Confirmatory ordinal-transport comparison remains unauthorized.
