# TDI-12.0 — Exact ordinal ranking primitives

Status: **EXACT finite combinatorics / elementary algebra for Stage-0
bootstrap; not a confirmatory ordinal-transport theorem**

## Scope

This note records the **EXACT** ranking objects landed in
`tdi-operator::ordinal` for TDI-12 Stage 0. They reuse generic TDI-10 Green /
Jacobi primitives only as **candidate** response extractors. No population,
metric, or split is frozen by this document.

## EXACT claim 1 — average ranks for ties

Given a finite sequence of finite reals, sort indices by value. For each tied
block occupying 1-based sorted positions `p..=q`, every member receives the
midrank `(p+q)/2`. The assignment depends only on the multiset of values.

## EXACT claim 2 — Spearman ρ

Spearman ρ is the Pearson sample correlation of the two average-rank vectors.
It is undefined (fail-closed) when either rank vector is constant.

## EXACT claim 3 — Kendall τ-b companion

Kendall τ-b counts concordant minus discordant pairs over the `n(n-1)/2`
unordered pairs, with classical pair-tie denominators
`sqrt((P+Q+T)(P+Q+U))`. Full ties fail closed.

## EXACT claim 4 — strictly increasing affine invariance

If `scale > 0` and `shift` are finite, the map `x ↦ scale·x + shift` preserves
average ranks exactly; therefore Spearman and Kendall against any paired
sequence are invariant.

## EXACT claim 5 — identity and dimension-only controls

On a nondegenerate sample, Spearman/Kendall of a sequence against itself equal
1. The dimension-only key equals `JacobiMatrix::len` as `f64` and does not read
Green values. Distinct finite values in reverse order yield Spearman =
Kendall = −1.

## EXACT claim 6 — GreenBands wiring identity

For each `CandidateResponseObservable` variant, `evaluate(matrix, shift)` equals
`from_green_bands(GreenBands::compute(matrix, shift)?)` and equals the
corresponding public TDI-10 `GreenBands` extractor (mid-diagonal entry, diagonal
trace, or mean absolute off-diagonal). Empty operators fail closed. On 1×1
operators the mean-abs off-diagonal observable is exactly 0. Identifiers match
the Stage-0 template `non_authorizing_candidates` list and do **not** freeze
`response_observable_registry`.

## EXACT claim 7 — coefficient-only control keys

`coefficient_frobenius_norm_key` is `sqrt(∑ a_i² + ∑ b_j²)` from Jacobi
coefficients only. `gershgorin_dominance_margin_key` is
`min_i (a_i − |b_{i−1}| − |b_i|)` (missing edges 0). Both ignore Green values
and do **not** freeze `control_battery`.

## EXACT claim 8 — deterministic shuffle control

`deterministic_shuffle` is a fixed-seed in-place Knuth shuffle. On a strictly
increasing length≥3 sample, shuffling one side with a fixed seed yields
Spearman/Kendall strictly less than 1, and the same seed reproduces the same
permutation. This scaffolds `shuffled_family` without freezing split discipline.


## EXACT claim 9 — constant-Toeplitz Frobenius closed form

For a constant Jacobi symbol with diagonal `a`, edge `b`, and width `n ≥ 1`,
the coefficient Frobenius key equals
`sqrt(n a² + (n − 1) b²)` (no off-diagonal term when `n = 1`). When
`a² + b² > 0` the key is strictly increasing in `n`. On any strictly increasing
width ladder the Frobenius keys therefore have Spearman = Kendall = 1 against
the dimension-only keys.

## EXACT claim 10 — constant-Toeplitz Gershgorin width invariance

For the same constant symbol the Gershgorin dominance margin equals `a` when
`n = 1`, `a − |b|` when `n = 2`, and `a − 2|b|` when `n ≥ 3`. In particular the
margin is width-invariant on every ladder contained in `{n : n ≥ 3}`, so
Spearman / Kendall against dimension fail closed with degenerate ranks. This
**REFUTES** treating the Stage-0 Gershgorin control as a covert dimension key
on constant Toeplitz families. It does **not** freeze `control_battery` or
`operator_population_families`.

## EXACT claim 11 — rank-normalize and negate-response scaffolding

`rank_normalize` replaces values by average ranks. On a nondegenerate sample,
Spearman / Kendall of the rank-normalized vector against the original equal 1.
`negate_values` multiplies by −1; on pairwise-distinct finite values, Spearman /
Kendall against the negated sample equal −1. These implement the Stage-0
`normalization_contract` non-authorizing candidates and do **not** pin that
field.

## EXACT claim 12 — tie-heavy adversarial midranks

`tie_heavy_adversarial_sample(n)` (`n ≥ 3`) yields endpoints `{0, 2}` with an
interior tied block of `1`s of length `n − 2`. Endpoints receive midranks `1`
and `n`; the interior block occupies positions `2..=(n − 1)` and receives
midrank `(n + 1) / 2`. Ranks are non-constant, so identity Spearman / Kendall
equal 1. This scaffolds `tie_heavy_adversarial` without freezing
`control_battery`.

## EXACT claim 13 — observable ladder evaluation

`evaluate_observable_ladder` maps a non-empty finite matrix list through one
`CandidateResponseObservable` at a fixed shift, fail-closed on empty ladders or
Green / resolvent failure. Candidate population identifiers
`PositiveConstantToeplitzWidthLadder` and `DiagonalOnlyWidthLadder` match the
Stage-0 template and do **not** pin `operator_population_families`.

## REFUTED (Stage-0 boundary)

The existence of these ranking primitives does **not** imply that ordinal
transport beats absolute calibration out of family. That comparison is deferred
to later gated stages.

## Non-claims

No soft-edge / double-scaling statement. No slowly-varying Jacobi theorem. No
Riemann / RH claim. No TDI-8.1 / TDI-9.1 / TDI-11.2 pin. No TDI-7.2 / 8.2 / 9.2
contact. Confirmatory execution remains unauthorized.
