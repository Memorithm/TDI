# TDI-10.1 — positive frozen Toeplitz cavity reference

Scientific status: **EXACT** for the declared constant two-sided Jacobi model. Numerical quadrature checks are **NUMERICAL EVIDENCE** for the implementation only.

Tracking issue: #141.

## Object

Consider the constant real two-sided Jacobi operator with diagonal coefficient `a` and signed nearest-neighbor coefficient `b`. Its Fourier symbol is

`p(theta) = a + 2 b cos(theta)`.

The model used by TDI-10.1 is restricted to the strictly positive-symbol regime

`a > 2 |b|`.

This condition is deliberate. The weaker condition `a^2 - 4 b^2 > 0` is not sufficient for a **positive** frozen operator: for example `a=-3, b=1` has positive discriminant but a strictly negative symbol.

## Exact identities

Under `a > 2|b|`, define

`D = sqrt(a^2 - 4 b^2) > 0`

and

`q = (a + D)/2`.

Then `q` is the positive stable fixed point of the Schur cavity map

`F(x) = a - b^2/x`,

so exactly

`q = a - b^2/q`.

The local derivative at that fixed point is

`kappa = F'(q) = b^2/q^2`,

with

`0 <= kappa < 1`.

For the constant two-sided operator, the selected Green entries are

`G_00 = 1/D`

and

`G_01 = -b G_00/q`.

They satisfy the row identity

`a G_00 + 2 b G_01 = 1`.

These are exact identities for the constant infinite model. They are not a theorem about a finite slowly-varying Jacobi matrix.

## Numerical evaluation

`FrozenToeplitzCavity` evaluates `D` in scaled form:

`D = a sqrt((1-r)(1+r))`, where `r=2|b|/a`.

This avoids explicitly forming `a^2` and `b^2` and therefore avoids an unnecessary overflow path for large finite coefficients. The implementation fails closed if the exact positive model maps to Green quantities outside the finite `f64` range.

This numerical design does not change the mathematical identity or constitute a soft-edge error theorem.

## Independent implementation check

Tests compare the closed-form `G_00` and `G_01` to a deterministic midpoint quadrature of the independent Fourier formulas

`G_00 = (1/2pi) integral 1/p(theta) dtheta`

and

`G_01 = (1/2pi) integral cos(theta)/p(theta) dtheta`.

Agreement of floating-point implementations is **NUMERICAL EVIDENCE**, not a mathematical proof. The algebraic formulas above have status **EXACT** from the declared constant-symbol calculation.

## Explicit non-claims

TDI-10.1 does not establish:

- convergence of finite-section cavities to `q`;
- a rate of convergence to the frozen model;
- a slowly-varying approximation bound;
- uniformity as `a-2|b| -> 0`;
- summability of cavity drift;
- cumulative contraction estimates;
- any asymptotic in `(i,m,t)`;
- any statement about Riemann, zeta zeros, prolate coefficients, or RH.

Those questions require separate hypotheses and separate TDI-10 increments.
