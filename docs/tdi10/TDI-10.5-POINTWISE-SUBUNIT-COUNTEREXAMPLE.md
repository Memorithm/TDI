# TDI-10.5 — Pointwise-subunit counterexample

## Scope

TDI-10.5 is a falsification slice built directly on the exact finite-chain
identity qualified in TDI-10.4. It asks one deliberately narrow question:

> Does `0 < alpha_k < 1` at every finite step, by itself, force the boundary
> transport product to vanish along an increasing chain?

The answer is **no**. This stage records one explicit counterexample. It does
not promote a replacement convergence theorem.

## Exact counterexample

For integer `k >= 1`, define

`alpha_k = 1 - 1/(k+1)^2 = k(k+2)/(k+1)^2`.

Every factor satisfies exactly

`0 < alpha_k < 1`.

For a chain of length `n`, the product telescopes:

`product_{k=1}^n alpha_k`

`= product_{k=1}^n [k/(k+1)] product_{k=1}^n [(k+2)/(k+1)]`

`= (n+2)/(2(n+1))`.

Hence

`lim_{n -> infinity} product_{k=1}^n alpha_k = 1/2`,

not zero.

Therefore pointwise subunit transport factors alone do **not** imply decay of
the boundary coefficient in the TDI-10.4 affine unrolling.

## Realization by admissible TDI cavity steps

The witness is not merely an abstract scalar product. It can be realized by
`CavityTransportStep` instances satisfying the existing positivity contracts.

Choose the supplied reference cavities

`q_k = 1`

for every `k`, with initial cavity/error

`C_0 = 2`, `E_0 = C_0 - q_0 = 1`.

Given positive `C_{k-1}`, set

`e_k^2 = alpha_k C_{k-1}`

and

`a_k = 1 + e_k^2`.

Then the TDI-10.2 drift is exactly

`delta_k = a_k - e_k^2/q_{k-1} - q_k = 0`,

while

`C_k = a_k - e_k^2/C_{k-1}`

`= 1 + alpha_k(C_{k-1} - 1)`.

Thus

`E_k = alpha_k E_{k-1}`

and therefore

`E_n = [(n+2)/(2(n+1))] E_0 -> E_0/2`.

All cavities remain strictly positive. The Rust test constructs this family
through the public TDI-10.2 API and feeds it through `CavityTransportChain`.
Floating-point agreement with the closed form is **NUMERICAL EVIDENCE** for the
implementation; the telescoping identities above are **EXACT algebra**.

## Scientific consequence

This counterexample falsifies only the implication

`(forall k, 0 < alpha_k < 1) => boundary product tends to zero`.

It does not establish necessary or sufficient hypotheses for decay. In
particular, TDI-10.5 makes no claim about operator families satisfying a
uniform bound `alpha_k <= rho < 1`, summability/divergence criteria for
`1-alpha_k`, drift accumulation, slowly varying Jacobi matrices, soft-edge
limits, or Riemann-specific coefficients.

Those require separate hypotheses and separate qualification.

## Required implementation evidence

The dedicated gate verifies that:

1. every realized TDI transport factor is positive and strictly smaller than
   one;
2. the finite products match `(n+2)/(2(n+1))` numerically;
3. the realized chain has zero drift up to floating-point roundoff;
4. the observed final error remains bounded away from zero for long finite
   witnesses;
5. the existing `tdi-operator` tests and strict Clippy remain green;
6. documentation retains the COUNTEREXAMPLE/falsification boundary and does not
   promote a general convergence theorem.
