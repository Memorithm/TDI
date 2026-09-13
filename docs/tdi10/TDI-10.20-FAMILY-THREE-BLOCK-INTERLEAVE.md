# TDI-10.20 — Exact three-block family concatenation and interleaved schedules

Status: **EXACT finite three-block concatenation identities for named cavity
families `F_U` / `F_D` / `F_S`, including same-family products under index
restart, mixed `{U,D,S}` permutation closed forms, and interleaved pair-schedule
products; REFUTED that three-block `{U,D,S}` order is immaterial; REFUTED that
interleaving vs blocking is immaterial; REFUTED that index-restart bookkeeping
is immaterial for Type-D / Type-S; REFUTED that a Type-S middle block erases
Type-U prefix decay uniformly in the prefix length; REFUTED that a finite
Type-D prefix plus Type-S middle forces composed liminf 0 under a Type-U suffix
of fixed length**

## Scope

TDI-10.15 recorded **EXACT** products for `F_U` then `F_D` / `F_S`. TDI-10.19
recorded the complementary reverse / cross two-block table. TDI-10.20 does
**not** reopen κ(a,b) (10.16), affine unrolling (10.17), or the checklist
(10.18). It records the **next** concatenation calculus listed as a TDI-10
frontier after 10.19:

1. three-block named-family concatenations with closed products;
2. interleaved pair-schedules versus blocked concatenations of the same
   family step counts;
3. the index-restart bookkeeping consequence that concatenated short blocks of
   Type D / Type S are **not** a single longer block of that family.

No slowly-varying Jacobi theorem is claimed. No soft-edge / double-scaling
statement is claimed. No Riemann / RH result is asserted.

## Standing vocabulary (cited, not reinvented)

From TDI-10.13 (declared maps; indices restart at 1 inside each block):

- `F_U(ρ)`: `alpha_k = ρ` for fixed `ρ ∈ (0,1)`; closed product over `k` steps
  is `ρ^k`.
- `F_D`: `alpha_k = k/(k+1)`; closed product over `k` steps is `1/(k+1)`.
- `F_S`: `alpha_k = 1 - 1/(k+1)^2`; closed product over `k` steps is
  `(k+2)/(2(k+1))`.

From TDI-10.14: any admissible frozen symbol with open contraction may supply
`ρ := FrozenToeplitzCavity::contraction(a,b) ∈ (0,1)`.

From TDI-10.15 / 10.19: block indices restart at 1 after each block (declared
concatenation bookkeeping; not a new physical parameter). Realization:
contiguous `CavityTransportStep` / `CavityTransportChain` (TDI-10.2 / 10.4)
with zero drift.

Write `P_A(k)` for the one-block closed product of family `A` of length `k`.

## EXACT claim 1 — Three-block multiplicative product identity

Fix families `A,B,C ∈ {F_U(ρ), F_D, F_S}` and integers `ℓ,m,n ≥ 1`. Let the
composed map be `ℓ` steps of `A`, then `m` steps of `B`, then `n` steps of
`C`, each block restarting its local index at 1.

Then the finite product is exactly

`P_{A→B→C}(ℓ,m,n) = P_A(ℓ) · P_B(m) · P_C(n)`.

In particular the product is associative at the closed-form level:

`(P_A(ℓ) · P_B(m)) · P_C(n) = P_A(ℓ) · (P_B(m) · P_C(n))`.

The zero-drift cavity chain cumulative transport factor must agree with this
closed form.

## EXACT claim 2 — Mixed `{U,D,S}` permutation closed forms

Under the same convention, the six distinct orderings of one block of each
family are

- `P_{U→D→S}(ℓ,m,n;ρ) = ρ^ℓ · 1/(m+1) · (n+2)/(2(n+1))`
- `P_{U→S→D}(ℓ,m,n;ρ) = ρ^ℓ · (m+2)/(2(m+1)) · 1/(n+1)`
- `P_{D→U→S}(ℓ,m,n;ρ) = 1/(ℓ+1) · ρ^m · (n+2)/(2(n+1))`
- `P_{D→S→U}(ℓ,m,n;ρ) = 1/(ℓ+1) · (m+2)/(2(m+1)) · ρ^n`
- `P_{S→U→D}(ℓ,m,n;ρ) = (ℓ+2)/(2(ℓ+1)) · ρ^m · 1/(n+1)`
- `P_{S→D→U}(ℓ,m,n;ρ) = (ℓ+2)/(2(ℓ+1)) · 1/(m+1) · ρ^n`

Cavity-chain cumulative transport must agree. When `ρ` is sourced from
`FrozenToeplitzCavity::contraction`, the same identities hold with that
Toeplitz coefficient (TDI-10.14 bridge reused, not reinvented).

## EXACT claim 3 — Same-family three-block products under index restart

- `P_{U→U→U}(ℓ,m,n;ρ) = ρ^{ℓ+m+n}` (equals one `F_U` block of total length,
  because `alpha` is constant).
- `P_{D→D→D}(ℓ,m,n) = 1/(ℓ+1) · 1/(m+1) · 1/(n+1)`.
- `P_{S→S→S}(ℓ,m,n) = (ℓ+2)/(2(ℓ+1)) · (m+2)/(2(m+1)) · (n+2)/(2(n+1))`.

The Type-D / Type-S products are **not** the one-block products of total
length `ℓ+m+n`. That distinction is Claim 3 plus the REFUTED bookkeeping
claim below; it is a consequence of the declared index-restart convention,
not a new family.

## EXACT claim 4 — Interleaved pair-schedule products

Fix `k ≥ 1`. Write `(AB)^k` for the length-`2k` schedule that concatenates
`k` blocks of `A` of length 1 with `k` blocks of `B` of length 1, alternating
and restarting the local index at 1 in **every** block.

Then

- `P_{(UD)^k}(ρ) = (ρ/2)^k`
- `P_{(US)^k}(ρ) = (3ρ/4)^k`
- `P_{(DS)^k} = (3/8)^k`

because a length-1 `F_U` block contributes `ρ`, a length-1 `F_D` block
contributes `1/2`, and a length-1 `F_S` block contributes `3/4`.

By contrast, the **blocked** concatenations of the same family step counts
are the two-block products of TDI-10.15 / 10.19:

- `P_{U^k → D^k}(ρ) = ρ^k / (k+1)`
- `P_{U^k → S^k}(ρ) = ρ^k · (k+2)/(2(k+1))`
- `P_{D^k → S^k} = 1/(k+1) · (k+2)/(2(k+1))`

Cavity-chain cumulative transport must agree with both the interleaved and
the blocked closed forms.

## REFUTED: three-block `{U,D,S}` order is immaterial

The naive claim

> for all `ℓ,m,n ≥ 1` and `ρ ∈ (0,1)`, the six mixed products of Claim 2
> are equal (block order does not matter)

is **REFUTED**. Explicit counterexample: `ℓ = 2`, `m = 2`, `n = 1`,
`ρ = 1/2` yields

- `P_{U→D→S}(2,2,1;1/2) = 1/16`
- `P_{U→S→D}(2,2,1;1/2) = 1/12`
- `P_{D→S→U}(2,2,1;1/2) = 1/9`

and `1/16 ≠ 1/12 ≠ 1/9`. Concatenation order is part of the declared
bookkeeping.

## REFUTED: interleaving vs blocking is immaterial

The naive claim

> only the multiset of family step counts matters, so an interleaved
> pair-schedule equals the blocked two-block concatenation of the same
> counts

is **REFUTED**. Explicit counterexample: `k = 2`, `ρ = 1/2` yields

- `P_{(UD)^2}(1/2) = (1/4)^2 = 1/16`
- `P_{U^2 → D^2}(1/2) = (1/2)^2 / 3 = 1/12`

and `1/16 ≠ 1/12`. Interleaving is not a rewrite of blocked concatenation.

## REFUTED: index-restart bookkeeping is immaterial for Type-D / Type-S

The naive claim

> concatenating three `F_D` (resp. `F_S`) blocks of lengths `ℓ,m,n` equals
> one `F_D` (resp. `F_S`) block of length `ℓ+m+n`

is **REFUTED**. Explicit counterexample: `ℓ = m = n = 2` yields

- three `F_D` blocks: `1/3 · 1/3 · 1/3 = 1/27`
- one `F_D` block of length 6: `1/7`

and `1/27 ≠ 1/7`. Likewise three `F_S` blocks of length 2 give
`(2/3)^3 = 8/27`, while one `F_S` block of length 6 gives `4/7`, and
`8/27 ≠ 4/7`.

Type-U is the exception (constant `alpha`): three `F_U` blocks of lengths
`ℓ,m,n` **do** equal one `F_U` block of total length. That agreement does
not restore restart-immateriality for Type D / Type S.

## REFUTED: Type-S middle block erases Type-U prefix decay uniformly in `ℓ`

The naive claim

> inserting a Type-S middle block after any Type-U prefix erases the
> Type-U decay, so that the composed liminf (as middle and/or suffix
> lengthen) is bounded away from 0 by a positive constant independent of
> the Type-U prefix length `ℓ`

is **REFUTED**. For any `ε > 0` choose `ℓ` with `ρ^ℓ < ε`. Then for every
middle length `m` and suffix length `n`,

`P_{U→S→D}(ℓ,m,n;ρ) = ρ^ℓ · (m+2)/(2(m+1)) · 1/(n+1) < ε`

because `(m+2)/(2(m+1)) ≤ 3/4 < 1` and `1/(n+1) ≤ 1`. A Type-S middle does
**not** produce a positive lower bound independent of `ℓ`.

## REFUTED: finite Type-D prefix plus Type-S middle forces liminf 0 under Type-U of fixed `n`

The naive claim

> every finite Type-D prefix forces
> `lim_{m→∞} P_{D→S→U}(ℓ,m,n;ρ) = 0`
> for fixed suffix length `n`

is **REFUTED**. For each finite `ℓ ≥ 1` and `n ≥ 1`,

`lim_{m→∞} P_{D→S→U}(ℓ,m,n;ρ) = 1/(2(ℓ+1)) · ρ^n > 0`.

Only sending the Type-D prefix length `ℓ → ∞` (or the Type-U suffix length
`n → ∞`) drives that limit to 0. A finite Type-D prefix plus a lengthening
Type-S middle alone does **not** force composed liminf 0 under a Type-U
suffix of fixed length.

## Required implementation evidence

The dedicated gate verifies that:

1. closed-form three-block products (Claim 1, including the six mixed
   permutations of Claim 2 and the same-family products of Claim 3) match
   algebraic map products for several `(ℓ,m,n,ρ)`;
2. contiguous zero-drift cavity chains realize those concatenations and
   agree on the cumulative transport factor;
3. interleaved pair-schedules `(UD)^k`, `(US)^k`, `(DS)^k` match Claim 4
   closed forms, and the corresponding blocked two-block products match
   TDI-10.15 / 10.19;
4. at least one three-block composition sources `ρ` from
   `FrozenToeplitzCavity::contraction`;
5. explicit examples **REFUTE** three-block order-independence, interleaving
   vs blocking, Type-D / Type-S restart-immateriality, Type-S-middle
   erasure of Type-U prefix decay, and finite-Type-D-prefix liminf-0 under
   a lengthening Type-S middle with fixed Type-U suffix;
6. documentation retains the EXACT / REFUTED boundary and does not claim a
   slowly-varying Jacobi asymptotic, soft-edge / double-scaling limit, or
   Riemann / RH result.

## Series boundary

TDI-10.20 is orthogonal to TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins and does
not contact TDI-7.2 / TDI-8.2 / TDI-9.2 surfaces. It does not authorize
TDI-12 confirmatory execution and does not invent TDI-12.0 freeze pins.
TDI-9.3.0 does not authorize TDI-9.1 / TDI-9.2. Execution flags remain
hard-false.
