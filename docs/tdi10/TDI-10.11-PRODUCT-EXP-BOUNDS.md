# TDI-10.11 — Quantitative product / exponential bounds

Status: **EXACT quantitative companion to TDI-10.10 equivalence**

## Scope

TDI-10.7–10.10 established that, under `0 < alpha_k <= 1`,

> `sum (1 - alpha_k) = infinity`
> `  <=>  product alpha_k -> 0`
> `  <=>  sum (-log alpha_k) = infinity`.

TDI-10.11 records the **finite-n quantitative** consequences of the same elementary
sandwich used there. Write `x_k = 1 - alpha_k`, so `x_k in [0, 1)` and
`alpha_k = 1 - x_k`. For every `x in [0, 1)`,

`x <= -log(1 - x) <= x / (1 - x)`.

This stage does **not** restate the infinite-product equivalence. It upgrades the
partial-sum / partial-product comparison to explicit exponential bounds, gives a
declared-cutoff lower bound once factors are eventually near one, packages the
Type D sandwich identities, and **REFUTES** sharpness of the upper bound as an
equality for all sequences.

## EXACT claim 1 — product upper bound

From `log(1 - x) <= -x`, for every finite `n >= 1`,

`sum_{k=1}^n log alpha_k <= - sum_{k=1}^n x_k`,

hence

`product_{k=1}^n alpha_k <= exp( - sum_{k=1}^n x_k )`.

This is the finite-`n` form of the TDI-10.7 upper estimate that drives
sufficiency of divergent remainder.

## EXACT claim 2 — declared-cutoff product lower bound

From `-log(1 - x) <= x/(1 - x)`, once `x <= 1/2` one has `1/(1 - x) <= 2`, so

`-log(1 - x) <= 2x`.

**Declared cutoff.** Fix an integer `K >= 1` such that `x_k <= 1/2` for every
`k >= K` (equivalently `alpha_k >= 1/2` on that tail). Then for every
`n >= K`,

`sum_{k=K}^n (-log alpha_k) <= 2 sum_{k=K}^n x_k`,

and therefore

`product_{k=K}^n alpha_k = exp( - sum_{k=K}^n (-log alpha_k) )
  >= exp( - 2 sum_{k=K}^n x_k )`.

Writing `P_{K-1} := product_{k=1}^{K-1} alpha_k` (with the convention
`P_0 = 1`) and `C_K := - log P_{K-1}` (so `C_K` depends only on the finite
prefix before the cutoff),

`product_{k=1}^n alpha_k
  = P_{K-1} * product_{k=K}^n alpha_k
  >= exp( - C_K - 2 sum_{k=K}^n x_k )`.

This is the precise finite-`n` form of the “`exp(-C - 2 sum x)` style” lower
bound advertised in TDI-10.10's near-one regime. The constant `C_K` is
**declared** by the cutoff `K`; no fitted soft-edge constants appear.

## EXACT claim 3 — Type D sandwich identities

For the Type D witness `alpha_k = k/(k+1)` one has `x_k = 1/(k+1)` and the
closed forms

`product_{k=1}^n alpha_k = 1/(n+1)`,
`sum_{k=1}^n x_k = H_{n+1} - 1`,

where `H_m` is the `m`-th harmonic number. Claim 1 therefore yields the
elementary upper comparison

`1/(n+1) <= exp( -(H_{n+1} - 1) )`.

For the lower bound, declare cutoff `K = 1`: every Type D term satisfies
`x_k = 1/(k+1) <= 1/2`, so Claim 2 with `C_1 = 0` gives

`1/(n+1) >= exp( - 2 (H_{n+1} - 1) )`.

Together,

`exp( - 2 (H_{n+1} - 1) ) <= 1/(n+1) <= exp( -(H_{n+1} - 1) )`.

These are **EXACT** algebraic / elementary-analytic relations for the named
witness; finite floating-point checks are **NUMERICAL EVIDENCE** only.

## REFUTED: product upper bound is sharp as equality for all sequences

The naive claim

`product_{k=1}^n alpha_k = exp( - sum_{k=1}^n x_k )
  for every sequence with 0 < alpha_k <= 1`

is **REFUTED**. Equality in `log(1 - x) = -x` holds if and only if `x = 0`
(because `f(x) = -x - log(1-x)` has `f(0) = 0` and `f'(x) = x/(1-x) > 0` for
`x in (0,1)`). Therefore, whenever some `x_j > 0`,

`sum log alpha_k < - sum x_k`,

and the product inequality is **strict**:

`product_{k=1}^n alpha_k < exp( - sum_{k=1}^n x_k )`.

The Type D witness (`x_1 = 1/2 > 0`) and the Type U witness
(`alpha_k = rho in (0,1)`, so `x_k = 1 - rho > 0`) both demonstrate the gap
numerically and exactly via the closed forms above.

## Relation to TDI-10.7–10.10

- TDI-10.7 supplied sufficiency via `log(1-x) <= -x`.
- TDI-10.8 organized Types S/U/D.
- TDI-10.9 supplied the harmonic comparison rate.
- TDI-10.10 upgraded sufficiency to three-way equivalence and noted the
  near-one sandwich as a rate companion.
- TDI-10.11 extracts the **finite quantitative** upper/lower exponential
  bounds, the declared-cutoff constant `C_K`, the Type D sandwich, and
  **REFUTES** equality-sharpness of the upper bound.

## Required implementation evidence

The dedicated gate verifies that:

1. Claim 1 holds on declared sample sequences (including Types S/U/D);
2. Claim 2 holds for a declared cutoff `K` with `x_k <= 1/2` on the tail
   (Type D with `K = 1`, and a mixed sample with a nontrivial prefix);
3. Type D closed forms match `1/(n+1)` and `H_{n+1}-1`, and the sandwich
   `exp(-2(H_{n+1}-1)) <= 1/(n+1) <= exp(-(H_{n+1}-1))` holds;
4. Type D / Type U show a strict gap `product < exp(-sum x)` (**REFUTED**
   equality-sharpness);
5. admissible zero-drift cavity chains realize the Type D / Type U witnesses;
6. documentation retains the EXACT/REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
7. existing `tdi-operator` tests and strict Clippy remain green.

## Explicit non-claims

TDI-10.11 does **not**:

- weaken or replace the TDI-10.10 equivalence theorem;
- identify which general Jacobi families realize divergent remainders beyond
  the named witnesses;
- introduce soft-edge asymptotics, double scaling, or fitted constants;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins.
