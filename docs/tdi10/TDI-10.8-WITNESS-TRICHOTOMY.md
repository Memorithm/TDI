# TDI-10.8 — Exact witness trichotomy for subunit-product regimes

Status: **EXACT classification of witness families complementary to TDI-10.5–10.7**

## Scope

TDI-10.5–10.7 already supply three complementary scalar witnesses for cavity
boundary transport products under `0 < alpha_k <= 1`. TDI-10.8 does **not** add
a new decay lemma. It records an **EXACT** trichotomy that organizes those
existing families by product-limit regime and by the mechanism that forces the
limit (when the product vanishes).

The three types are defined on sequences satisfying the declared hypotheses of
the cited stages. They are mutually exclusive on the product-limit behaviour
described below.

## Type S — summable remainder (product not forced to 0)

Witness (TDI-10.5):

`alpha_k = 1 - 1/(k+1)^2 = k(k+2)/(k+1)^2`.

Then `0 < alpha_k < 1`, the remainder `1 - alpha_k = 1/(k+1)^2` is **summable**,
and

`product_{k=1}^n alpha_k = (n+2)/(2(n+1)) -> 1/2`.

Hence `liminf_{n -> infinity} product_{k=1}^n alpha_k = 1/2 > 0`.

## Type U — uniform geometric bound (product -> 0)

Witness (TDI-10.6): constant factors

`alpha_k = rho` for a fixed `rho in (0,1)`.

Then `0 < alpha_k <= rho < 1` and

`product_{k=1}^n alpha_k = rho^n -> 0`.

The same uniform bound applies to any (not necessarily constant) family with
`0 < alpha_k <= rho < 1`. Constant `alpha_k = rho` is the elementary
Toeplitz / constant-coefficient cavity witness already used in TDI-10.6.

## Type D — divergent remainder (product -> 0; may have alpha_k -> 1)

Witness (TDI-10.7):

`alpha_k = k/(k+1)`.

Then `0 < alpha_k < 1`, `alpha_k -> 1`, the remainder series
`sum (1 - alpha_k)` diverges, and

`product_{k=1}^n alpha_k = 1/(n+1) -> 0`.

## Exact mutual-exclusion statement

Under the declared hypotheses of TDI-10.5–10.7 for these three witness families:

1. **Type S** has `liminf product > 0` (here exactly `1/2`).
2. **Type U** and **Type D** both have `product -> 0`, but by different
   **sufficient** mechanisms:
   - Type U: a uniform geometric envelope `alpha_k <= rho < 1`;
   - Type D: divergence of `sum (1 - alpha_k)` (which may hold even when
     `alpha_k -> 1`, so no uniform `rho < 1` exists).

No single sequence can simultaneously be Type S and Type U, or Type S and
Type D, on these product-limit behaviours. Type U and Type D overlap as
sufficient mechanisms on some sequences (every Type-U family has linearly
divergent remainder), but the **canonical witnesses** above are distinct:
constant `rho` is Type U with `alpha_k` bounded away from 1; the harmonic
family is Type D with `alpha_k -> 1` and no uniform `rho < 1`.

This classification is **EXACT** bookkeeping of already-qualified algebraic /
real-analysis content. It is not a soft-edge
theorem, not a slowly-varying Jacobi theorem, and not a Riemann claim.

## REFUTED naive criteria

### REFUTED: alpha_k -> 1 implies Type S

The naive claim

`(alpha_k -> 1) => product stays bounded away from 0 (Type S)`

is **REFUTED** by the Type D witness `alpha_k = k/(k+1)`: here `alpha_k -> 1`
while `product -> 0`.

### REFUTED: divergent remainder is automatic for every cavity family

The naive claim

`0 < alpha_k < 1` for all `k` `=>` `sum (1 - alpha_k) = infinity`

is **REFUTED** by the Type S witness `alpha_k = 1 - 1/(k+1)^2`: the remainder
is summable and the product tends to `1/2`.

## Exact operator-family witness note

Constant `alpha_k = rho in (0,1)` is Type U and corresponds to a
Toeplitz / constant-coefficient cavity transport factor. The same zero-drift
`CavityTransportStep` construction used in TDI-10.5–10.7 realizes each of
Types S, U, and D through the public factor families already cited above.
TDI-10.8 does **not** invent new scientific parameters, dimensions, seeds, or
operator coefficients.

## Required implementation evidence

The dedicated gate verifies that:

1. the Type S telescoping product matches `(n+2)/(2(n+1))` and stays bounded
   away from zero;
2. the Type U constant-`rho` product matches `rho^n` and decays;
3. the Type D harmonic product matches `1/(n+1)` and decays while
   `alpha_n -> 1`;
4. admissible zero-drift cavity chains realize each type;
5. documentation retains the EXACT / REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
6. existing `tdi-operator` tests and strict Clippy remain green.

Finite floating-point products are **NUMERICAL EVIDENCE** for the
implementation. The telescoping identities and the trichotomy statements are
**EXACT algebra / real analysis**.

## Explicit non-claims

TDI-10.8 does **not**:

- introduce a new sufficient decay criterion beyond TDI-10.6 / TDI-10.7;
- claim that Types U and D are disjoint as mechanisms on every sequence;
- identify which general Jacobi families realize each type outside the cited
  witnesses;
- introduce soft-edge asymptotics, double scaling, or fitted constants;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface.
