# TDI-10.7 — Divergent remainder decay lemma

Status: **EXACT algebraic companion to TDI-10.5 and TDI-10.6**

## Scope

TDI-10.5 showed that pointwise `0 < alpha_k < 1` alone does **not** force the
boundary transport product to vanish. TDI-10.6 showed that a uniform geometric
bound `0 < alpha_k <= rho < 1` is sufficient. TDI-10.7 records the classical
remainder criterion that sits between those two statements:

> If `0 < alpha_k <= 1` for every finite step `k` and
>
> `sum_{k=1}^{infinity} (1 - alpha_k) = infinity`,
>
> then `product_{k=1}^n alpha_k -> 0` as `n -> infinity`.

This is **EXACT** real analysis under the stated hypotheses. It is not a soft-edge
theorem, not a slowly-varying Jacobi theorem, and not a Riemann claim.

## Elementary proof under the declared hypotheses

Write `x_k = 1 - alpha_k`, so `x_k in [0, 1)` and `alpha_k = 1 - x_k`.
For every `x in [0, 1)` the elementary inequality

`log(1 - x) <= -x`

holds, because `f(x) = -x - log(1-x)` satisfies `f(0) = 0` and
`f'(x) = x / (1-x) >= 0`. Therefore

`sum_{k=1}^n log alpha_k <= - sum_{k=1}^n (1 - alpha_k)`.

If the remainder series diverges, the partial sums of `log alpha_k` tend to
`-infinity`, and the finite products tend to zero.

Terms with `alpha_k = 1` contribute nothing to the remainder sum and multiply
the product by one. The hypothesis excludes `alpha_k = 0`, which would make the
product vanish in a single step.

## Exact telescoping witness

For integer `k >= 1`, define

`alpha_k = k / (k + 1)`.

Then `0 < alpha_k < 1`, `alpha_k -> 1`, and

`1 - alpha_k = 1 / (k + 1)`.

The remainder partial sums are the shifted harmonic numbers
`H_{n+1} - 1`, which diverge. The product telescopes:

`product_{k=1}^n alpha_k = 1 / (n + 1) -> 0`.

The same zero-drift cavity construction used in TDI-10.5 / TDI-10.6 realizes
this family through public `CavityTransportStep` instances.

## Relation to TDI-10.5 and TDI-10.6

- The TDI-10.5 family `alpha_k = 1 - 1/(k+1)^2` has a **summable** remainder:
  `1 - alpha_k = 1/(k+1)^2 < 1/(k(k+1))` and `sum 1/(k(k+1)) = 1`, so
  `sum (1 - alpha_k) < infinity`. Its product tends to `1/2`, consistent with
  the missing divergence hypothesis of TDI-10.7.
- TDI-10.6 is a special case: a uniform bound `alpha_k <= rho < 1` forces
  `1 - alpha_k >= 1 - rho > 0`, so the remainder diverges at least linearly.

TDI-10.7 does **not** claim that remainder divergence is necessary for every
operator family, nor identify which Jacobi families realize a divergent
remainder.

## REFUTED naive criterion

The same telescoping witness **REFUTES** the naive claim

`(alpha_k -> 1) => product stays bounded away from zero`.

Here `alpha_k -> 1` while `product_{k=1}^n alpha_k = 1/(n+1) -> 0`.
Approaching one is therefore not a decay-prevention criterion.

## Required implementation evidence

The dedicated gate verifies that:

1. the harmonic-remainder product matches `1/(n+1)` within floating-point
   tolerance;
2. `log(1-x) <= -x` holds on a declared sample of `x in [0, 1)`;
3. an admissible zero-drift cavity chain realizes the decaying witness;
4. documentation retains the EXACT/lemma boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
5. existing `tdi-operator` tests and strict Clippy remain green.

Finite floating-point products are **NUMERICAL EVIDENCE** for the
implementation. The telescoping identities and the logarithm inequality are
**EXACT algebra / real analysis**.

## Explicit non-claims

TDI-10.7 does **not**:

- prove decay for the TDI-10.5 counterexample family;
- establish necessary and sufficient hypotheses for cavity-boundary decay;
- identify operator families that realize a uniform `rho` or a divergent
  remainder;
- introduce soft-edge asymptotics, double scaling, or fitted constants;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface.
