# TDI-10.18 — Exact operator-family finite hypothesis checklist

Status: **EXACT finite comparison packaging of TDI-10.9 as an operator-family
hypothesis checklist; EXACT TDI-10.2 one-step identity on family-realized cavity
steps; REFUTED that informal `alpha_k → 1` / “slow variation” alone meets the
Type-D checklist item; REFUTED that a finite-window harmonic lower bound alone
forces infinite product decay**

## Scope

TDI-10.13 registered named families `F_S` / `F_U` / `F_D` and left a **FORMAL**
pointer for future slowly-varying Jacobi hypotheses. TDI-10.17 specialized the
TDI-10.2/10.4 affine unrolling onto those families (including constant drift).

TDI-10.18 does **not** reopen κ(a,b) (10.16), concatenation calculus (10.15), or
constant-drift `B_n` formulae (10.17). It records an **EXACT**, elementary
**operator-family hypothesis checklist** that turns the TDI-10.9 harmonic
comparison into a finite, auditable family-classification gate, and it wires the
TDI-10.2 one-step transport identity onto family-realized `CavityTransportStep`
chains through the public APIs already used in TDI-10.13–10.17.

No slowly-varying Jacobi asymptotic theorem is claimed. No soft-edge /
double-scaling statement is claimed. No new physical parameters are invented:
family alphas are those of TDI-10.13; the one-step identity is TDI-10.2; the
harmonic comparison is TDI-10.9.

## Standing vocabulary (cited, not reinvented)

From TDI-10.2: one-step cavity-error transport

`E_i = alpha E_j + delta`,

with `alpha = e^2 / (C_j q_j)` and `delta = a_i - e^2/q_j - q_i`, under the
declared positivity / finite-value conditions. Public realization:
`CavityTransportStep` (`transport_factor`, `drift`, `reconstructed_error`).

From TDI-10.9: if `0 < alpha_k ≤ 1` and there exist `L > 0`, `K ≥ 1` such that

`1 - alpha_k ≥ L / k` for all `k ≥ K`,

then `sum (1 - alpha_k) = ∞` and `product alpha_k → 0` (Type-D sufficient rate
via TDI-10.7).

From TDI-10.13 (block indices start at 1):

- `F_U(ρ)`: `alpha_k = ρ` for fixed `ρ ∈ (0,1)`.
- `F_D`: `alpha_k = k/(k+1)`.
- `F_S`: `alpha_k = 1 - 1/(k+1)^2`.

Realization: contiguous zero-drift `CavityTransportStep` / `CavityTransportChain`
as in TDI-10.13–10.17.

## Operator-family finite hypothesis checklist (EXACT bookkeeping)

For a declared operator family producing `alpha_k ∈ (0,1]` (and an admissible
cavity-transport realization), the finite checklist is:

### Item D — harmonic remainder lower bound (cites TDI-10.9)

Exists `L > 0` and integer `K ≥ 1` such that for every `k ≥ K`,

`x_k := 1 - alpha_k ≥ L / k`.

**Consequence (EXACT via TDI-10.9):** remainder diverges and
`product_{k=1}^n alpha_k → 0`. This is a **sufficient** decay gate. It does not
by itself distinguish Type U from classical Type D (constant-gap Type U also
meets Item D eventually, since `1 - ρ ≥ L/k` for large `k`).

### Item U — uniform geometric bound (cites TDI-10.6)

Exists `ρ ∈ (0,1)` such that `alpha_k ≤ ρ` for every `k`. Then
`product ≤ ρ^n → 0` (Type U).

### Item S — summable-remainder non-decay witness (cites TDI-10.5 / 10.8)

If `sum x_k < ∞` with `0 < alpha_k ≤ 1`, product need not vanish. Named family
`F_S` is the closed witness: product → `1/2`.

### Item T — TDI-10.2 transport identity on realized steps

Every realized step must satisfy the public one-step identity

`current_error = transport_factor · neighbor_error + drift`,

with `transport_factor` matching the declared family `alpha_k` (within floating-
point tolerance) and `drift = 0` for the zero-drift family witnesses.

Meeting Items D/U/S/T is **finite algebraic / elementary comparison
bookkeeping**. It is **not** a slowly-varying Jacobi theorem and does **not**
close the TDI-10.13 FORMAL pointer (coefficient class + modulus of slow
variation + envelope estimate on a stated window remain future work).

## EXACT claim 1 — Item D classifies via TDI-10.9

Assume `0 < alpha_k ≤ 1`. If the family meets checklist Item D for some
`L > 0` and `K ≥ 1`, then by TDI-10.9 the family is in the product-decay
regime (`product → 0`).

Named family `F_D` meets Item D: `1 - alpha_k = 1/(k+1)`, and for every
`L ∈ (0, 1/2]` one has `1/(k+1) ≥ L/k` for all `k ≥ 1` (since
`k/(k+1) ≥ 1/2`). Named family `F_U(ρ)` also meets Item D eventually with any
`L ∈ (0, 1-ρ]`. Named family `F_S` fails Item D for every `L > 0` (ratio
`k · x_k → 0`).

## EXACT claim 2 — TDI-10.2 one-step identity on family-realized steps

Fix any named family `F ∈ {F_S, F_U(ρ), F_D}` and integer `n ≥ 1`. Realize `F`
as a contiguous zero-drift cavity chain exactly as in TDI-10.13. Then every
step satisfies Item T:

- `transport_factor = alpha_k` (declared family map),
- `drift = 0`,
- `reconstructed_error = transport_factor · neighbor_error`,
- and these agree with the public `CavityTransportStep` accessors under the
  already-required positivity / finite-value conditions.

Floating-point checks are **NUMERICAL EVIDENCE** for the Rust realization; the
identity itself is **EXACT** finite algebra (TDI-10.2).

## EXACT claim 3 — checklist is finite; FORMAL pointer remains open

The checklist above is an **EXACT** packaging of already-proved elementary
comparisons (TDI-10.6 / 10.9) and the TDI-10.2 identity. It does **not** assert
any of the TDI-10.13 FORMAL pointer items (declared `(a_i,b_i)` class, proved
alpha↔coefficient relation, windowed envelope). Those remain future work labelled
**PROVED UNDER DECLARED ASSUMPTIONS** or **FORMAL ASYMPTOTIC** when earned.

## REFUTED: informal `alpha_k → 1` / “slow variation” alone meets Item D

The candidate claim

> every cavity-transport family with `alpha_k → 1` meets checklist Item D
> (hence is Type D / product → 0)

is **REFUTED** by family `F_S`. One has `alpha_k → 1` and
`x_k = 1/(k+1)^2`, so for every `L > 0` the inequality `x_k ≥ L/k` fails for
all sufficiently large `k`, while

`product_{k=1}^n alpha_k = (n+2)/(2(n+1)) → 1/2 ≠ 0`.

Informal slogans (“slowly varying”, “coefficients almost constant”,
“`alpha_k → 1`”) are **not** checklist Item D.

## REFUTED: finite-window harmonic lower bound alone forces product → 0

The candidate claim

> if there exist `L > 0` and integers `1 ≤ K ≤ N` such that
> `1 - alpha_k ≥ L/k` for all `k ∈ [K, N]`, then
> `product_{k=1}^n alpha_k → 0` as `n → ∞`

is **REFUTED**. A harmonic lower bound on a **finite** window does not control
the infinite tail. Explicit counterexample: take the concatenated family with
prefix length `N ≥ 1`,

- `alpha_k = k/(k+1)` for `1 ≤ k ≤ N` (meets a harmonic bound on `[1,N]`),
- `alpha_k = 1 - 1/(k+1)^2` for `k > N` (Type-S tail),

realized as a contiguous zero-drift cavity chain. The infinite product equals
the finite Type-D prefix product `1/(N+1)` times the Type-S tail product from
index `N+1`, which tends to a **strictly positive** limit as `n → ∞`
(explicitly: prefix `1/(N+1)` times
`∏_{k=N+1}^n (1 - 1/(k+1)^2) = (N+1)(n+2)/((N+2)(n+1)) → (N+1)/(N+2)`,
so the full product → `1/(N+2) > 0`). Checklist Item D requires the harmonic
lower bound **eventually forever**, not on a finite inspection window.

## Required implementation evidence

The dedicated gate verifies that:

1. named family `F_D` meets Item D for an explicit `L > 0`, and zero-drift
   cavity products agree with `1/(n+1) → 0`;
2. named family `F_S` fails Item D for every tested `L > 0` while product → `1/2`;
3. named family `F_U(ρ)` meets Item D eventually and Item U, with product `ρ^n`;
4. every named-family zero-drift step satisfies the TDI-10.2 one-step identity
   (Item T) through public `CavityTransportStep` accessors;
5. an explicit finite-window + Type-S-tail concatenation **REFUTES** infinite
   product decay from a finite harmonic window alone;
6. documentation retains the EXACT / REFUTED boundary and does not claim
   soft-edge, slowly-varying Jacobi theorems, or Riemann conclusions;
7. existing `tdi-operator` tests and strict Clippy remain green.

## Explicit non-claims

TDI-10.18 does **not**:

- prove slowly-varying Jacobi asymptotics or soft-edge / double-scaling limits;
- close the TDI-10.13 FORMAL pointer (coefficient class / modulus / envelope);
- attach variable Jacobi coefficients `(a_i,b_i)` beyond zero-drift cavity maps
  and the frozen contraction coefficient already used in TDI-10.13–10.17;
- claim a new κ(a,b) identity (that is TDI-10.16);
- claim new concatenation or constant-drift `B_n` formulae (those are
  TDI-10.15 / 10.17);
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins or set any execution flag true.
