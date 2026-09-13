# TDI-10.19 — Exact reverse and cross operator-family composition

Status: **EXACT finite concatenation identities for `F_D`/`F_S` then `F_U` and
for `F_D`↔`F_S` cross products, including Toeplitz-sourced ρ; REFUTED that a
Type-S prefix blocks Type-D suffix decay; REFUTED that a finite Type-D prefix
forces composed liminf 0 under a Type-S suffix; REFUTED that `F_D`/`F_S` block
order is immaterial**

## Scope

TDI-10.15 recorded **EXACT** products for `F_U` then `F_D` / `F_S`. TDI-10.18
packaged the finite family checklist. TDI-10.19 does **not** reopen κ(a,b)
(10.16), affine unrolling (10.17), or the checklist (10.18). It records the
**complementary** concatenation calculus:

1. reverse Type-U suffixes after Type-D / Type-S prefixes;
2. cross products between Type-D and Type-S blocks (both orders).

No slowly-varying Jacobi theorem is claimed. No soft-edge / double-scaling
statement is claimed. No Riemann / RH result is asserted.

## Standing vocabulary (cited, not reinvented)

From TDI-10.13 (declared maps; indices restart at 1 inside each block):

- `F_U(ρ)`: `alpha_k = ρ` for fixed `ρ ∈ (0,1)`; closed product over `n` steps
  is `ρ^n`.
- `F_D`: `alpha_k = k/(k+1)`; closed product over `m` steps is `1/(m+1)`.
- `F_S`: `alpha_k = 1 - 1/(k+1)^2`; closed product over `m` steps is
  `(m+2)/(2(m+1))`.

From TDI-10.14: any admissible frozen symbol with open contraction may supply
`ρ := FrozenToeplitzCavity::contraction(a,b) ∈ (0,1)`.

From TDI-10.15: block indices for the second family restart at 1 after the
prefix (declared concatenation bookkeeping; not a new physical parameter).

Realization: contiguous `CavityTransportStep` / `CavityTransportChain`
(TDI-10.2 / 10.4) with zero drift, as in TDI-10.13–10.15.

## EXACT claim 1 — F_D then F_U product identity

Fix `ρ ∈ (0,1)`, integers `m ≥ 1`, `n ≥ 1`. Let the composed map be

1. `m` steps of `F_D` with indices restarting at 1, then
2. `n` steps of `F_U(ρ)`.

Then the finite product is exactly

`P_{D→U}(m,n;ρ) = 1/(m+1) · ρ^n`.

In particular:

- for fixed `m`, `P → 0` as `n → ∞` (Type-U suffix);
- for fixed `n`, `P → 0` as `m → ∞` (Type-D prefix);
- the zero-drift cavity chain cumulative transport factor must agree with this
  closed form.

When `ρ` is sourced from `FrozenToeplitzCavity::contraction`, the same identity
holds with that Toeplitz coefficient (TDI-10.14 bridge reused, not reinvented).

## EXACT claim 2 — F_S then F_U product identity

Under the same concatenation convention,

`P_{S→U}(m,n;ρ) = (m+2)/(2(m+1)) · ρ^n`.

In particular:

- for fixed `m`, `P → 0` as `n → ∞` (Type-U suffix forces decay);
- for fixed `n`, as `m → ∞`, `P → (1/2) · ρ^n` (Type-S prefix saturates);
- cavity-chain cumulative transport must agree with the closed form.

## EXACT claim 3 — F_D then F_S product identity

Fix integers `m ≥ 1`, `n ≥ 1` (no ρ). Then

`P_{D→S}(m,n) = 1/(m+1) · (n+2)/(2(n+1))`.

In particular:

- for fixed `m`, `lim_{n→∞} P_{D→S}(m,n) = 1/(2(m+1)) > 0`;
- for fixed `n`, `P → 0` as `m → ∞`;
- cavity-chain cumulative transport must agree with the closed form.

## EXACT claim 4 — F_S then F_D product identity

Under the same convention,

`P_{S→D}(m,n) = (m+2)/(2(m+1)) · 1/(n+1)`.

In particular:

- for fixed `m`, `P → 0` as `n → ∞` (Type-D suffix forces decay despite a
  Type-S prefix);
- for fixed `n`, as `m → ∞`, `P → 1/(2(n+1))`;
- cavity-chain cumulative transport must agree with the closed form.

## REFUTED: Type-S prefix blocks Type-D suffix decay

The naive claim

> a Type-S prefix prevents the composed product from tending to 0 when a
> Type-D suffix lengthens

is **REFUTED**. For any fixed `m ≥ 1`,

`lim_{n→∞} P_{S→D}(m,n) = 0`

because `P_{S→D}(m,n) = (m+2)/(2(m+1)) · 1/(n+1)`. A Type-S prefix does **not**
block Type-D suffix decay.

## REFUTED: finite Type-D prefix forces composed liminf 0 under Type-S suffix

The naive claim

> every finite Type-D prefix forces
> `lim_{n→∞} P_{D→S}(m,n) = 0`

is **REFUTED**. For each finite `m ≥ 1`,

`lim_{n→∞} P_{D→S}(m,n) = 1/(2(m+1)) > 0`.

Only sending the Type-D prefix length `m → ∞` drives that liminf to 0. A
finite Type-D prefix alone does **not** force composed liminf 0 under a
Type-S suffix.

## REFUTED: F_D / F_S block order is immaterial

The naive claim

> for all `m,n ≥ 1`, `P_{D→S}(m,n) = P_{S→D}(m,n)`
> (block order does not matter)

is **REFUTED**. Explicit counterexample: `m = 2`, `n = 1` yields

- `P_{D→S}(2,1) = 1/3 · 3/4 = 1/4`,
- `P_{S→D}(2,1) = 4/6 · 1/2 = 1/3`,

and `1/4 ≠ 1/3`. Concatenation order is part of the declared bookkeeping.

(Note: the special case `m = n = 1` happens to agree at `3/8`; agreement on a
single pair does not restore order-independence.)

## Required implementation evidence

The dedicated gate verifies that:

1. closed-form `P_{D→U}`, `P_{S→U}`, `P_{D→S}`, and `P_{S→D}` match algebraic
   map products for several `(m,n,ρ)` (ρ unused for D↔S);
2. contiguous zero-drift cavity chains realize all four concatenations and
   agree on the cumulative transport factor;
3. at least one reverse composition sources `ρ` from
   `FrozenToeplitzCavity::contraction`;
4. explicit examples **REFUTE** Type-S-prefix blocking of Type-D decay,
   finite-Type-D-prefix liminf-0 under Type-S, and D/S order-independence;
5. documentation retains the EXACT / REFUTED boundary and does not claim a
   slowly-varying Jacobi asymptotic, soft-edge / double-scaling limit, or Riemann / RH result.

## Series boundary

TDI-10.19 is orthogonal to TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins and does
not contact TDI-7.2 / TDI-8.2 / TDI-9.2 surfaces. It does not authorize
TDI-12 confirmatory execution and does not invent TDI-12.0 freeze pins.
TDI-9.3.0 does not authorize TDI-9.1 / TDI-9.2.
