# TDI-10.x — Operator / Resolvent Research

Status: **active generic operator-research line**

Tracking issue: #141.

## Mission

TDI-10.x studies generic finite and asymptotic questions for real symmetric tridiagonal/Jacobi operators, with emphasis on shifted resolvents, Schur cavities, Green functions, slowly varying coefficients, and soft-edge regimes.

TDI-10.x is scientifically autonomous. It does not contain or assume the Riemann hypothesis, zeta-zero identification, generalized-prolate crossing semantics, or any model-specific conclusion from `riemann_ndim_bench`.

## Evidence vocabulary

Every TDI-10 research result must carry one of these statuses:

- **EXACT** — finite algebraic identity or algorithm whose equality follows directly from declared definitions;
- **PROVED UNDER DECLARED ASSUMPTIONS** — theorem with explicit hypotheses and constants/domain;
- **FORMAL ASYMPTOTIC** — formal expansion without a proved uniform remainder sufficient for the stated limit;
- **NUMERICAL EVIDENCE** — reproducible computation only;
- **CONJECTURE** — unproved mathematical claim;
- **REFUTED** — explicit counterexample or contradiction to a declared candidate claim.

A numerical fit is never proof. A formal asymptotic is never promoted to a uniform theorem without a remainder estimate on the declared window.

## Current stage map

- **TDI-10.0** — audit / scope (`docs/tdi10/TDI-10.0-AUDIT.md`)
- **TDI-10.1** — exact frozen Toeplitz cavity reference
- **TDI-10.2** — exact finite cavity-error transport
- **TDI-10.3** — cavity drift factorization without contraction claims
- **TDI-10.4** — exact finite-chain affine unrolling
- **TDI-10.5** — **REFUTED** implication: pointwise `0 < alpha_k < 1` alone forces product → 0
- **TDI-10.6** — **EXACT** companion: uniform bound `0 < alpha_k <= rho < 1` forces product ≤ `rho^n` → 0
- **TDI-10.7** — **EXACT** remainder criterion: `sum (1 - alpha_k) = infinity` with `0 < alpha_k <= 1` forces product → 0; `alpha_k -> 1` alone is **REFUTED** as a decay-prevention criterion

## Next frontiers (not yet authorized by numbering alone)

- operator families that realize a uniform `rho` or a divergent remainder (or fail to);
- slowly varying Jacobi hypotheses with explicit remainder windows;
- soft-edge / double-scaling statements labelled FORMAL ASYMPTOTIC or PROVED UNDER DECLARED ASSUMPTIONS as appropriate.

## Series boundary

TDI-10.x is orthogonal to the historical TDI-1…9 lineages. It does not modify, rerun, reinterpret, or authorize any TDI-7.2, TDI-8.2, or TDI-9.2 surface.

## Ownership boundary with RiemannBench

TDI owns reusable statements for a declared class of Jacobi/resolvent problems. RiemannBench may later prove that its particular operator satisfies TDI hypotheses and apply the result.

TDI must not import `W+`/`W-` parity semantics, model-specific prolate/Riemann normalizations, Phase-4 coefficients, spectral-crossing interpretation, or any RH-dependent premise.
