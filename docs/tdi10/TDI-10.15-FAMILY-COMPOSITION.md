# TDI-10.15 — Exact operator-family composition (F_U then F_D / F_S)

Status: **EXACT finite concatenation identities for named cavity families
`F_U`∘`F_D` and `F_U`∘`F_S`, including Toeplitz-sourced ρ; REFUTED that a
Type-S suffix erases Type-U prefix decay uniformly in the prefix length**

## Scope

TDI-10.13 registered named families `F_S` / `F_U` / `F_D`. TDI-10.14 identified
`FrozenToeplitzCavity::contraction` with the Type-U parameter `ρ` of `F_U`.
TDI-10.15 does **not** reopen the scalar subunit-product arc (10.5–10.12). It
records the **EXACT** finite product calculus for **concatenating** already-
declared families as contiguous zero-drift cavity chains, with emphasis on
chaining an `F_U` prefix then an `F_D` or `F_S` remainder block.

No slowly-varying Jacobi theorem is claimed.

## Standing vocabulary (cited, not reinvented)

From TDI-10.13 (declared maps; indices restart at 1 inside each block):

- `F_U(ρ)`: `alpha_k = ρ` for fixed `ρ ∈ (0,1)`; closed product over `m` steps
  is `ρ^m`.
- `F_D`: `alpha_k = k/(k+1)`; closed product over `n` steps is `1/(n+1)`.
- `F_S`: `alpha_k = 1 - 1/(k+1)^2`; closed product over `n` steps is
  `(n+2)/(2(n+1))`.

From TDI-10.14: any admissible frozen symbol with open contraction may supply
`ρ := FrozenToeplitzCavity::contraction(a,b) ∈ (0,1)`.

Realization: contiguous `CavityTransportStep` / `CavityTransportChain`
(TDI-10.2 / 10.4) with zero drift, as in TDI-10.13–10.14. Block indices for
the second family restart at 1 after the prefix (declared concatenation
bookkeeping; not a new physical parameter).

## EXACT claim 1 — F_U then F_D product identity

Fix `ρ ∈ (0,1)`, integers `m ≥ 1`, `n ≥ 1`. Let the composed map be

1. `m` steps of `F_U(ρ)`, then
2. `n` steps of `F_D` with indices restarting at 1.

Then the finite product is exactly

`P_{U→D}(m,n;ρ) = ρ^m · 1/(n+1)`.

In particular:

- for fixed `n`, `P → 0` as `m → ∞` (Type-U prefix);
- for fixed `m`, `P → 0` as `n → ∞` (Type-D remainder);
- the zero-drift cavity chain cumulative transport factor must agree with this
  closed form (floating-point checks are NUMERICAL EVIDENCE).

When `ρ` is sourced from `FrozenToeplitzCavity::contraction`, the same identity
holds with that Toeplitz coefficient (TDI-10.14 bridge reused, not reinvented).

## EXACT claim 2 — F_U then F_S product identity

Under the same concatenation convention,

`P_{U→S}(m,n;ρ) = ρ^m · (n+2)/(2(n+1))`.

In particular:

- for fixed `n`, `P → 0` as `m → ∞`;
- for fixed `m`, `P → ρ^m / 2 > 0` as `n → ∞` (Type-S remainder saturates);
- cavity-chain cumulative transport must agree with the closed form.

## REFUTED: Type-S suffix erases Type-U prefix decay uniformly in m

The naive claim

> appending a Type-S remainder block after any Type-U prefix erases the
> Type-U decay, so that the composed liminf (as remainder length → ∞) is
> bounded away from 0 by a positive constant independent of the Type-U
> prefix length `m`

is **REFUTED**. For any `ε > 0` choose `m` with `ρ^m < ε`. Then for every
remainder length `n`,

`P_{U→S}(m,n;ρ) = ρ^m · (n+2)/(2(n+1)) ≤ ρ^m < ε`,

and `lim_{n→∞} P_{U→S}(m,n;ρ) = ρ^m / 2 < ε`. A Type-S suffix does **not**
uniformly erase Type-U prefix decay.

(Equivalently: the composed liminf depends on `m` through the exact factor
`ρ^m / 2`; it is not an `m`-independent positive constant.)

## Required implementation evidence

The dedicated gate verifies that:

1. closed-form `P_{U→D}` and `P_{U→S}` match algebraic map products for several
   `(m,n,ρ)`;
2. contiguous zero-drift cavity chains realize both concatenations and agree
   on the cumulative transport factor;
3. at least one composition sources `ρ` from
   `FrozenToeplitzCavity::contraction`;
4. an explicit example **REFUTES** `m`-uniform Type-S rescue of Type-U decay;
5. documentation retains the EXACT / REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
6. existing `tdi-operator` tests and strict Clippy remain green.

## Explicit non-claims

TDI-10.15 does **not**:

- prove slowly-varying Jacobi asymptotics or soft-edge / double-scaling limits;
- attach variable Jacobi coefficients `(a_i,b_i)` beyond zero-drift cavity maps
  and the frozen contraction coefficient already used in TDI-10.13–10.14;
- claim a new scalar decay lemma outside the named-family concatenation
  calculus;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins or set any execution flag true.
