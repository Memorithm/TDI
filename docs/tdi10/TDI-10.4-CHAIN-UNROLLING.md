# TDI-10.4 — Exact finite cavity-error chain unrolling

## Scope

TDI-10.4 composes the exact one-step TDI-10.2 relation

`E_k = alpha_k E_{k-1} + delta_k`

across a finite contiguous sequence. It adds no new model for constructing the
reference cavities and no slowly-varying coefficient hypothesis.

For `k=1,...,n`, repeated substitution gives exactly

`E_n = A_n E_0 + B_n`,

with

`A_n = product_{k=1}^n alpha_k`

and

`B_n = sum_{k=1}^n delta_k product_{r=k+1}^n alpha_r`.

The implementation evaluates the equivalent recurrence

`A_k = alpha_k A_{k-1}`

`B_k = alpha_k B_{k-1} + delta_k`

from `A_0=1`, `B_0=0`.

## Scientific status

The displayed identities are **EXACT finite algebra** under the positivity and
finite-value conditions already required by TDI-10.2.

Floating-point reconstruction tests are **NUMERICAL EVIDENCE** for the Rust
implementation only. Accumulated products and sums can round, underflow, or
overflow. Non-finite accumulation is rejected explicitly.

The chain constructor also checks exact metadata continuity between adjacent
floating-point steps. This is a provenance/integrity guard; it is not an
analytic assumption about an operator family.

## Deliberate non-claims

TDI-10.4 does **not** claim:

- any pointwise factor is smaller than one;
- any cumulative transport product is smaller than one;
- uniform contraction or boundary-error decay;
- a summable or asymptotically small drift term;
- a slowly-varying Jacobi theorem;
- finite-section convergence to a frozen model;
- soft-edge uniformity or a double-scaling law;
- any Riemann-specific consequence or RH implication.

A later stage may investigate bounds or counterexamples only after declaring a
specific operator class and proving the hypotheses needed for those bounds.

## Required implementation evidence

The dedicated gate must verify:

1. one-step composition reduces to the existing TDI-10.2 identity;
2. multi-step left and right propagation reproduce the closed affine unrolling;
3. discontinuous chain metadata is rejected;
4. non-finite cumulative arithmetic is rejected;
5. the complete `tdi-operator` test suite and strict Clippy remain green;
6. source/docs retain the no-uniform-bound scientific boundary.
