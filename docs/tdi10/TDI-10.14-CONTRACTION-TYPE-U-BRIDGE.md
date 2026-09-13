# TDI-10.14 — Exact contraction ↔ Type-U ρ bridge

Status: **EXACT identification of `FrozenToeplitzCavity::contraction` with the
Type-U uniform geometric coefficient of family `F_U`, plus dual-path finite-n
product identity; REFUTED that every frozen Toeplitz symbol yields Type D**

## Scope

TDI-10.13 registered named operator families `F_S` / `F_U` / `F_D` and noted
that `FrozenToeplitzCavity::contraction` (TDI-10.1) is an admissible Type-U
coefficient source. TDI-10.14 does **not** reopen the scalar subunit-product
arc (10.5–10.12). It records the **EXACT** bridge between the frozen Toeplitz
contraction `κ(a,b)` and the Type-U parameter `ρ` of family `F_U`, including
equality / inequality cases already forced by the public APIs, and the
finite-n product identity that the zero-drift cavity path and the algebraic
`ρ^n` path must agree on.

No slowly-varying Jacobi theorem is claimed.

## Standing vocabulary (cited, not reinvented)

From TDI-10.1, for admissible constant symbols with `a > 2|b|`:

- `D = sqrt(a^2 - 4 b^2) > 0`
- `q = (a + D)/2` (positive Schur fixed point)
- `κ = contraction() = b^2 / q^2 = (a - D)/(a + D)`, with `0 <= κ < 1`

From TDI-10.6 / TDI-10.8 / TDI-10.13 family `F_U`:

- Type U: `0 < alpha_k <= ρ < 1` forces `product_{k=1}^n alpha_k <= ρ^n → 0`
- Canonical witness: constant `alpha_k = ρ`
- Operator-family packaging: zero-drift `CavityTransportStep` /
  `CavityTransportChain` realizing those `alpha_k`

## EXACT claim 1 — contraction coefficient is Type-U ρ

Let `(a,b)` be admissible (`a > 2|b|`) and set `κ = FrozenToeplitzCavity::new(a,b).contraction()`.

1. **Boundary case `κ = 0`.** Equivalent to `b = 0`. The open Type-U witness
   requires `ρ ∈ (0,1)`; the vanishing contraction is recorded but is **not**
   promoted to that open witness.
2. **Open case `0 < κ < 1`.** Equivalent to `b ≠ 0` under `a > 2|b|`. Setting
   `ρ := κ` places the constant family `alpha_k = κ` in Type U.
3. **Equality saturation.** For that family, `alpha_k = ρ` for every `k`, so
   the Type-U inequality `alpha_k <= ρ` holds with **equality** at every step.
   The least uniform geometric bound for the constant-`κ` family is exactly
   `ρ = κ`.
4. **Inequality envelope.** Any larger declared envelope `ρ' ∈ [κ, 1)` also
   certifies Type U for the same family (`alpha_k = κ <= ρ' < 1`), with strict
   inequality in the envelope whenever `ρ' > κ`.

These are exact identities / comparisons of already-declared quantities. They
do not invent new coefficients.

## EXACT claim 2 — dual-path finite-n product identity

Fix admissible `(a,b)` with `0 < κ < 1`, set `ρ := κ`, and fix `n >= 1`. Then
the following three finite products are equal (algebraically; floating-point
checks are NUMERICAL EVIDENCE for the implementation):

1. **Algebraic Type-U closed form:** `ρ^n`.
2. **Declared constant `F_U` map:** `product_{k=1}^n family_alpha(F_U, k; ρ)`.
3. **Zero-drift cavity realization:** cumulative transport factor of the
   contiguous `CavityTransportChain` built from `CavityTransportStep::left`
   steps whose transport factors equal `ρ` (within floating-point tolerance),
   as in TDI-10.13.

In particular, the Toeplitz-sourced coefficient path
(`ρ = FrozenToeplitzCavity::contraction(a,b)`) and the zero-drift cavity chain
path **must agree on `ρ^n`**. This is the finite-n product identity for `F_U`
through both declared coefficient sources of TDI-10.13.

## REFUTED: every frozen Toeplitz symbol yields Type D

The naive claim

> every admissible frozen Toeplitz symbol `(a,b)` with `a > 2|b|` induces a
> Type-D cavity family (`alpha_k → 1`, no uniform geometric envelope `ρ < 1`)

is **REFUTED** by any open contraction. Example: `(a,b) = (3,1)` yields
constant `κ = b^2/q^2 ∈ (0,1)` with `alpha_k = κ ↛ 1` and uniform envelope
`ρ = κ`. The canonical classification is Type U (TDI-10.8), not Type D.

(Type U and Type D overlap as *sufficient mechanisms* on some sequences —
constant `ρ` also has linearly divergent remainder — but the Type-D
*canonical* witness requires `alpha_k → 1` with no uniform `ρ < 1`. Frozen
Toeplitz contraction does not force that canonical Type-D regime.)

## Required implementation evidence

The dedicated gate verifies that:

1. for several admissible symbols with `0 < κ < 1`, `κ = b^2/q^2 = (a-D)/(a+D)`
   and `ρ := κ` saturates Type-U equality `alpha_k = ρ`;
2. a strictly larger envelope `ρ' ∈ (κ, 1)` still certifies the inequality case;
3. dual-path products agree on `ρ^n` for several `n` through algebraic map and
   zero-drift cavity chain;
4. at least one frozen symbol **REFUTES** universal Type-D classification;
5. `κ = 0` (`b = 0`) remains the recorded boundary, not the open Type-U witness;
6. documentation retains the EXACT / REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
7. existing `tdi-operator` tests and strict Clippy remain green.

## Explicit non-claims

TDI-10.14 does **not**:

- prove slowly-varying Jacobi asymptotics or soft-edge / double-scaling limits;
- claim finite-section cavity convergence rates to the frozen model;
- attach variable Jacobi coefficients `(a_i,b_i)` beyond the constant frozen
  symbol and the zero-drift cavity realization already used in TDI-10.13;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins or set any execution flag true.
