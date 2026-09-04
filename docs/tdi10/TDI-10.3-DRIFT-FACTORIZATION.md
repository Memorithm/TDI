# TDI-10.3 — Exact drift and transport-factor factorization

## Scope

TDI-10.3 refines the exact TDI-10.2 cavity-error identity without adding an asymptotic claim.

For one finite transport step,

`C_i = a_i - e^2/C_j`,

with positive references `q_j,q_i`, TDI-10.2 gives

`E_i = alpha E_j + delta`,

`alpha = e^2/(C_j q_j)`,

`delta = a_i - e^2/q_j - q_i`.

TDI-10.3 additionally accepts an explicit finite local reference edge `b_i`. It does not define how `b_i` is constructed.

## Exact drift factorization

The drift decomposes exactly as

`delta = rho + eta_edge + eta_reference`,

where

`rho = a_i - b_i^2/q_i - q_i`

is the **reference defect**,

`eta_edge = (b_i^2-e^2)/q_i`

is the edge mismatch contribution, and

`eta_reference = e^2(1/q_i-1/q_j)`

is the change in the supplied positive reference cavity.

The explicit `rho` term is essential. If a chosen local reference does not satisfy its own frozen equation, TDI must expose that defect rather than silently reinterpret it as coefficient drift.

For a true constant Toeplitz reference from TDI-10.1 with matching `(a,b,q)`, `rho=0`. This is a property of that declared reference, not a generic assumption.

## Exact transport-factor factorization

The TDI-10.2 multiplier decomposes exactly as

`alpha = nu * chi`,

where

`nu = (e/q_j)^2`

and

`chi = q_j/C_j`.

The implementation calls these quantities `normalized_edge_square` and `cavity_correction`.

Neither quantity is named or treated as a contraction certificate. In particular, TDI-10.3 does not assert `nu<1`, `chi<1`, `alpha<1`, or any uniform bound over rows, shifts, dimensions, or operator families.

## Scientific status

The displayed decompositions are **EXACT algebra** under the finite-value and positive-cavity/reference conditions inherited from TDI-10.2 plus a finite local reference edge.

Floating-point reconstruction tests are **NUMERICAL EVIDENCE** for implementation correctness only.

## Explicit reference metadata

The local reference edge is caller-supplied metadata. TDI-10.3 does not promote:

- arithmetic averaging of neighboring edges;
- geometric averaging;
- interpolation in `i/m`;
- a slowly-varying coefficient class;
- a soft-edge scaling rule.

Any later reference-construction policy must be explicit and separately qualified.

## Non-claims

TDI-10.3 does **not establish**:

- pointwise or uniform contraction;
- cumulative decay of boundary error;
- a bound on any drift component from coefficient derivatives;
- finite-to-frozen convergence;
- soft-edge uniformity;
- a theorem for slowly-varying Jacobi operators;
- any Riemann or RH consequence.
