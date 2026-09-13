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

## REFUTED (Stage-0 boundary)

The existence of these ranking primitives does **not** imply that ordinal
transport beats absolute calibration out of family. That comparison is deferred
to later gated stages.

## Non-claims

No soft-edge / double-scaling statement. No slowly-varying Jacobi theorem. No
Riemann / RH claim. No TDI-8.1 / TDI-9.1 / TDI-11.2 pin. No TDI-7.2 / 8.2 / 9.2
contact. Confirmatory execution remains unauthorized.
