# TDI-10.2 — Exact cavity-error transport

## Scope

This increment isolates a generic finite algebraic identity for positive Schur cavities. It does not choose a slowly-varying class, a row-freezing convention, or a soft-edge scaling regime.

Let a finite left or right cavity step be written uniformly as

`C_i = a_i - e^2 / C_j`,

where `j=i-1` for a left step and `j=i+1` for a right step. Let `q_j>0` and `q_i>0` be arbitrary caller-supplied reference cavities and define

`E_j = C_j-q_j`, `E_i=C_i-q_i`.

Then

`E_i = alpha E_j + delta`

with

`alpha = e^2/(C_j q_j)`

and

`delta = a_i - e^2/q_j - q_i`.

This follows by direct subtraction and the identity

`1/q_j - 1/C_j = (C_j-q_j)/(C_j q_j)`.

## Scientific status

The finite transport decomposition above is **EXACT** under the declared finite-value and positive-cavity/reference conditions.

The Rust regression tests that compare the transport reconstruction against independently computed finite `SchurCavities` are **NUMERICAL EVIDENCE** for implementation correctness. They are not an asymptotic theorem.

## Deliberate abstraction boundary

TDI-10.2 does not define how `q_i` is selected. This is intentional. A constant Toeplitz fixed point from TDI-10.1 is one admissible reference, but a variable-coefficient local-freezing rule is an additional modelling choice that must be declared and justified separately.

This keeps the exact identity independent of:

- arithmetic or geometric edge averaging;
- any slowly-varying coefficient hypothesis;
- any particular boundary window;
- any soft-edge scaling;
- any Riemann/prolate coefficient family.

## Fail-closed numerical contract

The implementation rejects non-finite inputs, non-positive finite/reference cavities, non-positive derived current cavities, and non-finite derived quantities. These checks preserve the positive shifted-Schur regime used by the TDI-10 finite operator core.

## Non-claims

This increment does **not establish**:

- `alpha < 1` uniformly;
- cumulative contraction along a finite chain;
- a bound on `delta` from coefficient variation;
- finite-to-frozen convergence;
- soft-edge uniformity;
- a theorem for slowly-varying Jacobi operators;
- any Riemann or RH consequence.

Those require additional hypotheses and separate proof or counterexample work.
