# TDI-10.10 — Remainder / product / log-sum equivalence

Status: **EXACT necessity-and-sufficiency companion to TDI-10.7–10.9**

## Scope

TDI-10.7 proved that, under `0 < alpha_k <= 1`, divergence of the remainder
series `sum (1 - alpha_k)` is **sufficient** for `product alpha_k -> 0`.
TDI-10.8 classified witnesses S/U/D. TDI-10.9 gave a harmonic comparison rate
that forces remainder divergence.

TDI-10.10 upgrades the sufficiency lemma to an **EXACT equivalence** under the
same standing hypotheses: for sequences with `0 < alpha_k <= 1`,

> `sum_{k=1}^{infinity} (1 - alpha_k) = infinity`
> `  <=>  product_{k=1}^n alpha_k -> 0`
> `  <=>  sum_{k=1}^{infinity} (-log alpha_k) = infinity`.

Finite products of strictly positive factors are never zero; vanishing of the
infinite product is equivalent to divergence of `sum (-log alpha_k)` by the
classical infinite-product criterion. The remainder series is tied to that
logarithm sum by the elementary inequalities below. This is **EXACT** real
analysis. It is not a soft-edge
theorem, not a slowly-varying Jacobi theorem, and not a Riemann claim.

## Elementary inequalities

Write `x_k = 1 - alpha_k`, so `x_k in [0, 1)` and `alpha_k = 1 - x_k` under
`0 < alpha_k <= 1`. For every `x in [0, 1)` the sandwich

`x <= -log(1 - x) <= x / (1 - x)`

holds:

1. `log(1 - x) <= -x` (equivalently `-log(1 - x) >= x`), because
   `f(x) = -x - log(1-x)` satisfies `f(0) = 0` and `f'(x) = x/(1-x) >= 0`
   (already used in TDI-10.7);
2. `-log(1 - x) <= x/(1-x)`, because `g(x) = x/(1-x) + log(1-x)` satisfies
   `g(0) = 0` and `g'(x) = x/(1-x)^2 >= 0`.

## EXACT equivalence theorem

Assume `0 < alpha_k <= 1` for every finite step `k`. Then the following are
equivalent:

1. `sum_{k=1}^{infinity} (1 - alpha_k) = infinity`;
2. `product_{k=1}^n alpha_k -> 0` as `n -> infinity`;
3. `sum_{k=1}^{infinity} (-log alpha_k) = infinity`.

### Proof sketch

**(1) => (2).** From `log(1-x) <= -x`,

`sum_{k=1}^n log alpha_k <= - sum_{k=1}^n (1 - alpha_k)`.

Divergent remainder forces the log partial sums to `-infinity`, hence the
finite products tend to zero (TDI-10.7).

**(2) => (3).** Under `alpha_k > 0`, the infinite product `product alpha_k`
converges to a strictly positive limit if and only if `sum log alpha_k`
converges in `(-infinity, 0]`. Therefore `product -> 0` if and only if
`sum log alpha_k = -infinity`, i.e. `sum (-log alpha_k) = infinity`.

**(3) => (1).** From `-log(1-x) >= x`,

`sum_{k=1}^n (-log alpha_k) >= sum_{k=1}^n (1 - alpha_k)`.

Divergence of the log-sum forces divergence of the remainder.

Combining (1)=>((2) and, via (2)=>(3)=>(1), the three statements are
equivalent.

Terms with `alpha_k = 1` contribute nothing to either series and leave the
product unchanged. The standing hypothesis `alpha_k > 0` excludes one-step
annihilation from the equivalence class treated here (see REFUTED section).

## Near-one regime (optional rate companion)

Under the additional regime `alpha_k -> 1` (equivalently `1 - alpha_k -> 0`),
the sandwich becomes a local rate comparison. Once eventually
`1 - alpha_k <= 1/2`, one has `1/(1-x_k) <= 2`, so

`sum (1 - alpha_k) <= sum (-log alpha_k) <= 2 sum (1 - alpha_k)`

on that tail. Thus, once factors approach one, remainder divergence and
log-sum divergence are comparable with explicit constants — consistent with,
but not required for, the global equivalence already proved under
`0 < alpha_k <= 1` alone.

## Witness consistency (Types S / U / D)

| Type | Factor | Remainder | Product | Equivalence instance |
|------|--------|-----------|---------|----------------------|
| S | `1 - 1/(k+1)^2` | summable | `-> 1/2 != 0` | all three fail together |
| U | constant `rho in (0,1)` | linearly divergent | `rho^n -> 0` | all three hold together |
| D | `k/(k+1)` | harmonic divergent | `1/(n+1) -> 0` | all three hold together |

TDI-10.5 does **not** contradict the theorem: its product does not tend to
zero, and its remainder is summable.

## REFUTED: necessity without the hypothesis `alpha_k > 0`

The naive claim

`(alpha_k in [0,1] for all k and product_{k=1}^n alpha_k -> 0)
  => sum (1 - alpha_k) = infinity`

is **REFUTED** once a zero factor is allowed. Take

`alpha_1 = 0` and `alpha_k = 1` for every `k >= 2`.

Then the product vanishes at step 1, while the remainder series equals `1`
(only the first term is nonzero). TDI-10.7–10.10 exclude `alpha_k = 0`; under
that exclusion, necessity holds.

## REFUTED: `alpha_k -> 1` is required for the equivalence

The naive claim

`remainder divergence <=> product -> 0` only under the extra hypothesis
`alpha_k -> 1`

is **REFUTED** by the Type U witness `alpha_k = rho` with fixed
`rho in (0,1)`: factors stay bounded away from one, yet remainder divergence,
log-sum divergence, and product decay all hold simultaneously. The
near-one sandwich is a rate refinement, not a prerequisite for equivalence.

## Relation to TDI-10.7–10.9

- TDI-10.7 supplied **sufficiency** of divergent remainder (and REFUTED
  `alpha_k -> 1` as a decay-prevention criterion).
- TDI-10.8 organized Types S/U/D.
- TDI-10.9 supplied a harmonic comparison rate forcing remainder divergence.
- TDI-10.10 supplies **necessity** of divergent remainder for product decay
  under the same `0 < alpha_k <= 1` hypotheses, identifies the three-way
  equivalence with `sum (-log alpha_k)`, and REFUTES careless weakenings of
  those hypotheses.

## Required implementation evidence

The dedicated gate verifies that:

1. the sandwich `x <= -log(1-x) <= x/(1-x)` holds on a declared sample of
   `x in [0,1)`;
2. Type S has summable remainder, bounded `-log` sum, and product bounded
   away from zero;
3. Type D / Type U have divergent remainder, divergent `-log` sum proxies,
   and product → 0;
4. the zero-factor counterexample has product zero with finite remainder
   (**REFUTED** necessity without `alpha > 0`);
5. Type U shows equivalence without `alpha_k -> 1`;
6. admissible zero-drift cavity chains realize the S/U/D witnesses;
7. documentation retains the EXACT/REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
8. existing `tdi-operator` tests and strict Clippy remain green.

Finite floating-point products and partial sums are **NUMERICAL EVIDENCE** for
the implementation. The inequalities and the equivalence theorem are
**EXACT algebra / real analysis**.

## Explicit non-claims

TDI-10.10 does **not**:

- weaken or replace TDI-10.7's elementary `log(1-x) <= -x` argument;
- identify which general Jacobi families realize divergent remainders beyond
  the named witnesses;
- introduce soft-edge asymptotics, double scaling, or fitted constants;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins.
