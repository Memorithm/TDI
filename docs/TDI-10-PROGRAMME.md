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

## Series boundary

TDI-10.x is orthogonal to the historical TDI-1…9 lineages. It does not modify, rerun, reinterpret, or authorize any TDI-7.2, TDI-8.2, or TDI-9.2 surface.

The initial finite operator core contains no train/validation/final split because it establishes deterministic finite identities. If later TDI-10 work introduces fitted constants, learned surrogates, empirical scale selection, or benchmark-tuned hypotheses, those development/validation choices must be separated before any confirmatory claim.

## Ownership boundary with RiemannBench

TDI owns reusable statements for a declared class of Jacobi/resolvent problems. RiemannBench may later prove that its particular operator satisfies TDI hypotheses and apply the result.

TDI must not import:

- `W+` / `W-` parity semantics;
- model-specific prolate/Riemann normalizations;
- Phase-4 coefficients;
- spectral-crossing interpretation;
- any RH-dependent premise.

## Planned architecture

- `tdi-operator/` — generic finite operator, cavity, Green, frozen-model, transport, and later asymptotic primitives;
- `tdi-bench/` — reproducible TDI-10 benchmark binaries only when a benchmark question is declared;
- `docs/tdi10/` — audit, assumptions, theorem attempts, counterexamples, and evidence-status records.

The first increment intentionally implements only the finite generic core. `frozen`, `transport`, `soft_edge`, and `asymptotic` modules are not created until their assumptions and evidence roles are separately reviewed.
