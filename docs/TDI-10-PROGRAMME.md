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
- **TDI-10.8** — **EXACT** witness trichotomy: Type S (summable remainder; product ↛ 0), Type U (uniform geometric; product → 0), Type D (divergent remainder; product → 0); **REFUTED** that `alpha_k -> 1` implies Type S; **REFUTED** that divergent remainder is automatic for every `0 < alpha_k < 1` family
- **TDI-10.9** — **EXACT** harmonic remainder-rate lemma: `1 - alpha_k >= c/k` eventually forces divergent remainder hence product → 0 (Type D sufficient rate); closed-form witness calculus for Types S/U/D; **REFUTED** that a superharmonic lower bound `1 - alpha_k >= c/k^{1+epsilon}` is sufficient for decay
- **TDI-10.10** — **EXACT** remainder / product / log-sum equivalence under `0 < alpha_k <= 1`: `sum (1 - alpha_k) = infinity` iff `product -> 0` iff `sum (-log alpha_k) = infinity`; **REFUTED** necessity of divergent remainder once `alpha_k = 0` is allowed; **REFUTED** that `alpha_k -> 1` is required for the equivalence
- **TDI-10.11** — **EXACT** quantitative product / exponential bounds: `product <= exp(-sum x)` from `log(1-x) <= -x`; declared-cutoff lower bound `product >= exp(-C_K - 2 sum_{k>=K} x)` once `x_k <= 1/2`; Type D sandwich `exp(-2(H_{n+1}-1)) <= 1/(n+1) <= exp(-(H_{n+1}-1))`; **REFUTED** that the upper bound is equality-sharp for all sequences
- **TDI-10.12** — **EXACT** Cesàro / mean-remainder rate: `liminf (1/n) sum x_k >= lambda > 0` implies `product <= exp(-(lambda-epsilon)n)` eventually via TDI-10.11; **REFUTED** that `(1/n) sum x -> lambda > 0` is necessary for product → 0 (Type D: averages → 0 yet product → 0)
- **TDI-10.13** — **EXACT** operator-family witnesses (pivot): named families `F_S` / `F_U` / `F_D` induce the trichotomy through `CavityTransportStep` / `CavityTransportChain` (and `FrozenToeplitzCavity::contraction` for Type U); **REFUTED** that every cavity family forces product decay; FORMAL pointer only for future slowly-varying Jacobi hypotheses
- **TDI-10.14** — **EXACT** contraction ↔ Type-U ρ bridge: `FrozenToeplitzCavity::contraction(a,b)` identifies with `F_U` parameter ρ (equality saturation / inequality envelopes); dual-path finite-n product identity (algebraic `ρ^n` and zero-drift cavity chain agree); **REFUTED** that every frozen Toeplitz symbol yields Type D
- **TDI-10.15** — **EXACT** operator-family composition: finite concatenation `F_U` then `F_D` / `F_S` with closed products `ρ^m/(n+1)` and `ρ^m·(n+2)/(2(n+1))` (Toeplitz-sourced ρ allowed); **REFUTED** that a Type-S suffix erases Type-U prefix decay uniformly in the prefix length `m`
- **TDI-10.16** — **EXACT** κ(a,b) domain / monotonicity (evenness, homogeneity, ratio reduction, radial monotone); **EXACT** matched TDI-10.3 transport factor = κ = Type-U ρ; **REFUTED** that κ → 0 as `a ↓ 2|b|+`; **REFUTED** that larger diagonal alone forces smaller κ without fixing `|b|`

The elementary subunit-product arc **TDI-10.5–10.12** is closed (pointwise counterexample → uniform geometric → remainder divergence → trichotomy → harmonic rate → equivalence → exp bounds → Cesàro rate). **TDI-10.13** opens the operator-family chapter; **TDI-10.14** bridges the frozen Toeplitz contraction to Type-U `ρ`; **TDI-10.15** records exact concatenation calculus for named families; **TDI-10.16** records exact κ(a,b) domain/monotonicity and the matched TDI-10.3 factorization → Type-U link without inventing slowly-varying or soft-edge theorems.

## Next frontiers (not yet authorized by numbering alone)

- further operator / coefficient classes that induce `alpha_k` beyond zero-drift cavity maps, frozen contractions, named-family concatenations, and the κ(a,b) domain calculus;
- slowly varying Jacobi hypotheses with explicit remainder windows (see TDI-10.13 FORMAL pointer — not a theorem);
- soft-edge / double-scaling statements labelled FORMAL ASYMPTOTIC or PROVED UNDER DECLARED ASSUMPTIONS as appropriate.

## Series boundary

TDI-10.x is orthogonal to the historical TDI-1…9 lineages. It does not modify, rerun, reinterpret, or authorize any TDI-7.2, TDI-8.2, or TDI-9.2 surface.

## Ownership boundary with RiemannBench

TDI owns reusable statements for a declared class of Jacobi/resolvent problems. RiemannBench may later prove that its particular operator satisfies TDI hypotheses and apply the result.

TDI must not import `W+`/`W-` parity semantics, model-specific prolate/Riemann normalizations, Phase-4 coefficients, spectral-crossing interpretation, or any RH-dependent premise.
