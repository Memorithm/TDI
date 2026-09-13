# TDI-10.16 — Exact κ(a,b) domain / monotonicity + frozen factorization Type-U link

Status: **EXACT domain calculus and monotonicity for
`FrozenToeplitzCavity::contraction` κ(a,b); EXACT identification of the
TDI-10.3 transport factor at the matched frozen Toeplitz fixed point with
Type-U ρ; REFUTED that κ → 0 as the symbol approaches the positivity
boundary; REFUTED that increasing the diagonal alone forces smaller κ
without fixing |b|**

## Scope

TDI-10.14 identified `κ = FrozenToeplitzCavity::contraction(a,b)` with the
Type-U parameter `ρ` of family `F_U`. TDI-10.15 recorded named-family
concatenation. TDI-10.16 does **not** reopen the scalar subunit-product arc
(10.5–10.12) or the concatenation calculus (10.15). It records the **EXACT**
admissible-domain calculus of `κ(a,b)` (scale invariance, evenness in `b`,
ratio reduction, monotonicity in the radial coordinate) and the **EXACT**
bridge from TDI-10.3 drift/transport factorization at the matched frozen
fixed point into that Type-U coefficient.

No slowly-varying Jacobi theorem is claimed.

## Standing vocabulary (cited, not reinvented)

From TDI-10.1, for admissible constant symbols with `a > 2|b|`:

- `D = sqrt(a^2 - 4 b^2) > 0`
- `q = (a + D)/2`
- `κ(a,b) = contraction() = b^2 / q^2 = (a - D)/(a + D)`, with `0 <= κ < 1`

From TDI-10.3: for a transport step with local reference edge `b_i`,

- `alpha = normalized_edge_square * cavity_correction`
- at a matched frozen Toeplitz fixed point (`a_i = a`, `e = b`,
  `C_j = q_j = q_i = q`, `b_i = b`), all drift components vanish and
  `cavity_correction = 1`.

From TDI-10.14: open `0 < κ < 1` places the constant family `alpha_k = κ` in
Type U with equality saturation `ρ := κ`.

Define the radial coordinate on the open positive-symbol cone:

`r(a,b) := 2|b|/a ∈ [0, 1)` whenever `a > 2|b|`.

## EXACT claim 1 — domain calculus for κ(a,b)

On the admissible domain `a > 2|b|`:

1. **Evenness in the edge.** `κ(a,b) = κ(a,-b)` (depends on `|b|` only).
2. **Positive homogeneity.** For every `λ > 0`, `(λa, λb)` is admissible iff
   `(a,b)` is, and `κ(λa, λb) = κ(a,b)`.
3. **Ratio reduction.** Writing `r = 2|b|/a` and `s = sqrt(1 - r^2)`,

   `κ(a,b) = (1 - s)/(1 + s)`.

   In particular `κ` is a function of `r` alone on the admissible cone.
4. **Endpoint values of the reduced map.** As `r → 0+`, `κ → 0`. As
   `r → 1-` (equivalently `a ↓ 2|b|+` with `b ≠ 0`), `κ → 1-`. At `b = 0`,
   `κ = 0` exactly (TDI-10.14 boundary case).

These are exact identities of already-declared algebraic expressions.

## EXACT claim 2 — monotonicity in the radial coordinate

Let `κ_*(r) = (1 - sqrt(1-r^2))/(1 + sqrt(1-r^2))` for `r ∈ [0,1)`. Then
`κ_*` is **strictly increasing** on `[0,1)`.

Consequences on the cone `a > 2|b|`:

1. **Fixed nonzero edge.** If `b ≠ 0` and `a_1 > a_2 > 2|b|`, then
   `κ(a_1,b) < κ(a_2,b)`.
2. **Fixed diagonal.** If `a > 0` and `0 ≤ |b_1| < |b_2| < a/2`, then
   `κ(a,b_1) < κ(a,b_2)`.

Floating-point checks of ordered samples are **NUMERICAL EVIDENCE** for the
implementation; the identities above are **EXACT**.

## EXACT claim 3 — TDI-10.3 factorization at the matched frozen point is Type-U ρ

Fix admissible `(a,b)` with `0 < κ < 1`. Let `q` be the frozen cavity and
construct the zero-drift matched transport step

`CavityTransportStep::left(a, b, q, q, q)`

together with `CavityDriftFactorization::new(step, b)`. Then exactly
(algebraically; floating-point checks are NUMERICAL EVIDENCE):

1. `reference_defect = edge_drift = reference_drift = 0` and `drift = 0`;
2. `cavity_correction = 1`;
3. `normalized_edge_square = κ`;
4. `reconstructed_transport_factor = step.transport_factor() = κ`.

Hence the TDI-10.3 transport factor at this matched frozen point **is** the
TDI-10.14 Type-U coefficient `ρ := κ`. Constant repetition of that factor is
family `F_U(ρ)` (TDI-10.13). This is the honest drift-factorization link into
family classification: factorization supplies the coefficient; the Type-U
placement is the already-declared 10.14 bridge, not a new trichotomy lemma.

## REFUTED: κ → 0 as the symbol approaches the positivity boundary

The naive claim

> as an admissible symbol approaches the positivity boundary
> (`a ↓ 2|b|+` with `b ≠ 0`), the frozen contraction κ tends to 0
> (vanishing / strongest contraction)

is **REFUTED**. Claim 1(4) gives `κ → 1-` on that approach: the local map
derivative tends to the open unit circle from below, not to 0. Example:
`(a,b) = (2.0001, 1.0)` has `κ` close to 1, while `(a,b) = (10.0, 1.0)` has
much smaller `κ`.

## REFUTED: increasing the diagonal alone forces smaller κ without fixing |b|

The naive claim

> for any two admissible symbols, the one with larger diagonal `a` has
> strictly smaller contraction κ (stronger uniform Type-U envelope),
> regardless of the edge

is **REFUTED**. Counterexample: `(a,b) = (3,1)` versus `(10,4)`. Both are
admissible, the second has larger diagonal, yet `κ(10,4) = 0.25 > κ(3,1)`.
Monotonicity in `a` holds only after fixing `|b|` (Claim 2); the radial
coordinate `r = 2|b|/a` is the controlling quantity.

## Required implementation evidence

The dedicated gate verifies that:

1. evenness, positive homogeneity, and ratio reduction hold for several
   admissible symbols;
2. ordered radial samples realize strict monotonicity of `κ_*`, and the
   fixed-`b` / fixed-`a` consequences hold on explicit tuples;
3. matched frozen factorization reconstructs `transport_factor = κ` with
   vanishing drift components for at least one open contraction;
4. an explicit near-boundary symbol **REFUTES** `κ → 0` at `a ↓ 2|b|+`;
5. the `(3,1)` vs `(10,4)` pair **REFUTES** diagonal-only monotonicity;
6. documentation retains the EXACT / REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
7. existing `tdi-operator` tests and strict Clippy remain green.

## Explicit non-claims

TDI-10.16 does **not**:

- prove slowly-varying Jacobi asymptotics or soft-edge / double-scaling limits;
- claim finite-section cavity convergence rates to the frozen model;
- assert uniform contraction over variable `(a_i,b_i)` rows;
- invent a new trichotomy beyond TDI-10.8 / 10.13–10.14;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins or set any execution flag true.
