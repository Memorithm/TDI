# TDI-10.6 — Uniform subunit decay lemma

Status: **EXACT algebraic companion to the TDI-10.5 counterexample**

## Scope

TDI-10.5 showed that pointwise `0 < alpha_k < 1` alone does **not** force the
boundary transport product to vanish. TDI-10.6 records the elementary positive
statement that *does* force decay:

> If there exists `rho` with `0 < rho < 1` such that `0 < alpha_k <= rho` for
> every finite step `k`, then for every finite chain length `n`,
>
> `0 < product_{k=1}^n alpha_k <= rho^n`,
>
> and therefore `product_{k=1}^n alpha_k -> 0` as `n -> infinity`.

This is **EXACT** real analysis under the stated uniform bound. It is not a soft-edge
theorem, not a slowly-varying Jacobi theorem, and not a Riemann claim.

## Relation to TDI-10.5

The TDI-10.5 family `alpha_k = 1 - 1/(k+1)^2` satisfies `0 < alpha_k < 1` but
**fails** any uniform bound `alpha_k <= rho < 1`, because `alpha_k -> 1`.
Its product tends to `1/2`, consistent with the missing hypothesis of TDI-10.6.

Thus TDI-10.5 and TDI-10.6 together separate:

1. pointwise subunit factors (insufficient);
2. a uniform geometric bound (sufficient for product decay).

Neither stage claims necessity of the uniform bound, nor identifies which
operator families realize a uniform `rho`.

## Required implementation evidence

The dedicated gate verifies that:

1. for several fixed `rho in (0,1)` and chain lengths, the constant-factor product
   equals `rho^n` within floating-point tolerance;
2. the product is strictly decreasing toward zero under the uniform bound;
3. documentation retains the EXACT/lemma boundary and does not claim soft-edge,
   slowly-varying, or Riemann conclusions;
4. existing `tdi-operator` tests and strict Clippy remain green.

## Explicit non-claims

TDI-10.6 does **not**:

- prove decay for the TDI-10.5 counterexample family;
- establish necessary and sufficient hypotheses for cavity-boundary decay;
- introduce soft-edge asymptotics, double scaling, or fitted constants;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface.
