# TDI-10.12 — Cesàro / mean-remainder rate

Status: **EXACT Cesàro-mean rate companion to TDI-10.11 exponential bounds**

## Scope

TDI-10.7–10.10 established equivalence of remainder divergence, product decay,
and log-sum divergence under `0 < alpha_k <= 1`. TDI-10.11 recorded the
finite-`n` bounds

`product_{k=1}^n alpha_k <= exp( - sum_{k=1}^n x_k )`

(with `x_k = 1 - alpha_k`) and a declared-cutoff lower companion.

TDI-10.12 extracts the **Cesàro / mean-remainder** consequence: a strictly
positive liminf of the averages `(1/n) sum_{k=1}^n x_k` forces an
**exponential** product decay rate. This is stronger than mere product → 0
(which, by TDI-10.10, only needs divergent remainder, possibly with averages
tending to zero). The stage also **REFUTES** that a positive Cesàro limit is
necessary for product decay.

Standing hypotheses throughout: `0 < alpha_k <= 1`, `x_k = 1 - alpha_k` (so
`x_k in [0, 1)`).

## EXACT claim 1 — positive Cesàro liminf implies exponential rate

Assume

`liminf_{n -> infinity} (1/n) sum_{k=1}^n x_k >= lambda > 0`.

Then the product decays at least exponentially with rate arbitrarily close to
`lambda`. Precise finite-`n` form:

> For every `epsilon in (0, lambda)` there exists an integer `N = N(epsilon)`
> such that for every `n >= N`,
>
> `(1/n) sum_{k=1}^n x_k >= lambda - epsilon`,
>
> and therefore, via the TDI-10.11 upper bound,
>
> `product_{k=1}^n alpha_k <= exp( - sum_{k=1}^n x_k )
>   <= exp( - (lambda - epsilon) n )`.

In the compressed asymptotic shorthand this is

`product_{k=1}^n alpha_k <= exp( - lambda n + o(n) )`,

Compact single-line form of the rate bound:

`product_{k=1}^n alpha_k <= exp( - (lambda - epsilon) n )`.

but the gateable statement is the epsilon / N form above. No soft-edge
constants are fitted.

### Concrete witnesses

**Type U (constant remainder).** Fix `lambda in (0, 1)` and set
`alpha_k = 1 - lambda` (so `x_k = lambda`) for every `k`. Then

`(1/n) sum_{k=1}^n x_k = lambda`

exactly for every `n`, and

`product_{k=1}^n alpha_k = (1 - lambda)^n <= exp( - lambda n )`

by TDI-10.11 (already for `epsilon = 0` in the finite-`n` comparison).

**Alternating blocks (liminf strictly positive, not constant).** Fix
`lambda in (0, 1)`. Define

`x_k = lambda` if `k` is odd, `x_k = 0` if `k` is even

(equivalently `alpha_k = 1 - lambda` on odd steps and `alpha_k = 1` on even
steps). Then the Cesàro means satisfy

`liminf_{n -> infinity} (1/n) sum_{k=1}^n x_k = lambda / 2 > 0`

(and the limit exists and equals `lambda/2`). Claim 1 therefore applies with
this liminf value: for every `epsilon in (0, lambda/2)` the product is
eventually bounded by `exp( - (lambda/2 - epsilon) n )`.

## REFUTED: positive Cesàro limit is necessary for product → 0

The naive claim

`(1/n) sum_{k=1}^n x_k -> lambda > 0` is necessary for
`product_{k=1}^n alpha_k -> 0`

is **REFUTED** by the Type D witness `alpha_k = k/(k+1)`, `x_k = 1/(k+1)`:

- `product_{k=1}^n alpha_k = 1/(n+1) -> 0`;
- `sum_{k=1}^n x_k = H_{n+1} - 1 ~ log n`, so
  `(1/n) sum_{k=1}^n x_k -> 0`.

Thus product decay holds while every Cesàro average tends to zero. (Pointwise
`x_k -> 0` as well; the REFUTED claim concerns the Cesàro mean, not a
pointwise liminf of `x_k` alone.)

## EXACT claim 2 — sufficiency for exponential rate (stronger than → 0)

Under the standing hypotheses, the condition

`liminf_{n -> infinity} (1/n) sum_{k=1}^n x_k >= lambda > 0`

is **sufficient** for exponential product decay in the sense of Claim 1. By
TDI-10.10, divergent remainder alone is already sufficient for mere
`product -> 0`; Claim 1 upgrades a positive Cesàro liminf to an explicit
exponential rate via TDI-10.11. Type D shows that this upgrade is strictly
stronger than product → 0: remainder diverges and product → 0, yet the Cesàro
means vanish and the decay is only polynomial (`1/(n+1)`), not exponential.

## Relation to TDI-10.5–10.11

- TDI-10.5 **REFUTED** pointwise `0 < alpha_k < 1` as a decay criterion.
- TDI-10.6 gave uniform geometric decay under `alpha_k <= rho < 1`.
- TDI-10.7–10.10 gave remainder divergence / equivalence for product → 0.
- TDI-10.9 supplied a harmonic comparison rate forcing remainder divergence.
- TDI-10.11 supplied `product <= exp(-sum x)`.
- TDI-10.12 packages the Cesàro-mean hypothesis that turns 10.11 into a
  uniform exponential rate, and **REFUTES** necessity of that hypothesis for
  mere product decay.

Together, TDI-10.5–10.12 form the elementary subunit-product arc. Next
frontiers remain slowly-varying Jacobi hypotheses and soft-edge /
double-scaling statements — not authorized by this numbering alone.

## Required implementation evidence

The dedicated gate verifies that:

1. Type U constant-`x = lambda` realizes Claim 1 with exact average `lambda`
   and `product <= exp(-(lambda - epsilon) n)` for declared `epsilon`;
2. the alternating-block witness has Cesàro liminf `lambda/2 > 0` and
   satisfies the finite-`n` exponential upper bound of Claim 1;
3. Type D has `(1/n) sum x -> 0` while `product -> 0` (**REFUTED** necessity
   of a positive Cesàro limit for product decay);
4. Claim 2's rate upgrade is recorded: positive Cesàro liminf yields
   exponential decay, strictly stronger than Type D's polynomial decay;
5. admissible zero-drift cavity chains realize the Type U / alternating /
   Type D witnesses;
6. documentation retains the EXACT/REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
7. existing `tdi-operator` tests and strict Clippy remain green.

Finite floating-point products and averages are **NUMERICAL EVIDENCE**. The
Cesàro implication via TDI-10.11 and the Type D counterexample are **EXACT**.

## Explicit non-claims

TDI-10.12 does **not**:

- weaken or replace the TDI-10.10 equivalence theorem;
- claim that positive Cesàro liminf is necessary for product → 0;
- identify which general Jacobi families realize positive Cesàro liminf beyond
  the named witnesses;
- introduce soft-edge asymptotics, double scaling, or fitted constants;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins.
