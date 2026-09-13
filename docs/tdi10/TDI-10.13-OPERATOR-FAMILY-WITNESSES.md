# TDI-10.13 — Exact operator-family witnesses (pivot)

Status: **EXACT classification of cavity-transport operator families by the
subunit-product trichotomy (Types S / U / D)**

## Scope

The elementary subunit-product arc **TDI-10.5–10.12** is closed. It classified
scalar sequences `alpha_k in (0,1]` and recorded exact decay / non-decay
criteria (pointwise counterexample, uniform geometric bound, remainder
divergence, trichotomy, harmonic rate, remainder/product/log-sum equivalence,
exponential bounds, Cesàro rate).

TDI-10.13 is the **pivot** into the operator-family chapter. It does **not**
add a new scalar decay lemma. It registers the already-qualified witnesses as
**named operator families** that induce `alpha_k` sequences for cavity
transport, wires them through the public TDI-10.1–10.4 APIs, and records an
honest boundary: this stage classifies families by the trichotomy; it does
**not** prove slowly-varying Jacobi asymptotics.

## Operator-family interface (EXACT bookkeeping)

An **operator family** (for this stage) is any declared map that produces a
sequence

`alpha_k in (0,1]` for `k = 1,2,...`

together with an admissible zero-drift realization as a contiguous chain of
`CavityTransportStep` values (TDI-10.2) whose transport factors equal those
`alpha_k` (within floating-point tolerance), optionally composed via
`CavityTransportChain` (TDI-10.4).

This interface deliberately does **not** require attaching variable Jacobi
matrix coefficients `(a_i, b_i)` to `alpha_k` beyond the zero-drift cavity
construction already used in TDI-10.5–10.12. Exploring `tdi-operator`
(Jacobi / Toeplitz / Schur / cavity / transport; docs TDI-10.1–10.4) shows
that the public constructions that already induce `alpha_k` sequences are
exactly those zero-drift cavity steps, plus the constant frozen-Toeplitz
contraction of TDI-10.1 as a Type-U coefficient source. No new scientific
physical parameters are invented.

## Named families

### Family `F_S` — Type S (summable remainder; product ↛ 0)

Declared map (TDI-10.5 witness):

`alpha_k = 1 - 1/(k+1)^2 = k(k+2)/(k+1)^2`.

Then `0 < alpha_k < 1`, `sum (1 - alpha_k) < infinity`, and

`product_{k=1}^n alpha_k = (n+2)/(2(n+1)) -> 1/2 > 0`.

Realized as zero-drift `CavityTransportStep::left` chains (TDI-10.2/10.4).

### Family `F_U` — Type U (uniform geometric; product → 0)

Declared map (TDI-10.6 witness), two equivalent coefficient sources already in
the line vocabulary:

1. **Constant factor.** `alpha_k = rho` for fixed `rho in (0,1)`.
2. **Frozen Toeplitz contraction (TDI-10.1).** For any admissible constant
   symbol `(a,b)` with `a > 2|b|`, `FrozenToeplitzCavity::new(a,b)` yields
   `kappa = contraction() = b^2 / q^2 in [0,1)`. Taking `alpha_k = kappa`
   (when `0 < kappa < 1`) is the same Type-U constant family, with the
   coefficient sourced from the public frozen cavity API rather than an
   invented parameter.

Then `0 < alpha_k <= rho < 1` and `product = rho^n -> 0`.

### Family `F_D` — Type D (divergent remainder; product → 0)

Declared map (TDI-10.7 witness):

`alpha_k = k/(k+1)`.

Then `0 < alpha_k < 1`, `alpha_k -> 1`, `sum (1 - alpha_k) = infinity`, and

`product_{k=1}^n alpha_k = 1/(n+1) -> 0`.

Realized as zero-drift `CavityTransportStep` chains.

## EXACT claims

### EXACT claim 1 — named families realize the trichotomy

Under the declared maps above and the standing hypotheses of TDI-10.5–10.8:

1. `F_S` is Type S: liminf of the product equals `1/2 > 0`.
2. `F_U` is Type U: product equals `rho^n -> 0` for the chosen uniform bound.
3. `F_D` is Type D: product equals `1/(n+1) -> 0` with divergent remainder.

Each family admits an admissible zero-drift cavity-transport realization
through `CavityTransportStep` / `CavityTransportChain`. The frozen-Toeplitz
contraction path for `F_U` cites TDI-10.1 and does not invent new coefficients.

### EXACT claim 2 — classification, not slowly-varying asymptotics

TDI-10.13 classifies operator families by the subunit-product trichotomy of
TDI-10.8. It does **not** prove slowly-varying Jacobi asymptotics, soft-edge
limits, or any statement that variable Jacobi coefficients `(a_i,b_i)` force
a particular trichotomy type.

## REFUTED: every cavity family forces product decay

The naive claim

> every admissible cavity-transport family with `0 < alpha_k < 1` forces
> `product alpha_k -> 0`

is **REFUTED** by family `F_S`: the 10.5 witness realized as cavity steps has
product → `1/2`. Not every cavity family forces decay.

(This is the operator-family packaging of the TDI-10.5 / TDI-10.8 Type-S
counterexample; no new scalar mathematics is claimed.)

## FORMAL pointer — future slowly-varying Jacobi hypothesis

A future stage that wants a slowly-varying Jacobi theorem would need, at
minimum (FORMAL pointer only — **not** a theorem here):

1. a declared coefficient class `(a_i, b_i)` with an explicit modulus of
   slow variation on a stated window;
2. a proved relation identifying the cavity transport factors `alpha_k` with
   those coefficients (or with a frozen local symbol) under stated positivity /
   non-resonance hypotheses;
3. a remainder / envelope estimate sufficient to place the induced `alpha_k`
   into Type U, Type D, or another declared regime uniformly on that window;
4. an evidence label **PROVED UNDER DECLARED ASSUMPTIONS** or **FORMAL
   ASYMPTOTIC**, never silently promoted from the named witnesses of this
   stage.

TDI-10.13 does not assert any of (1)–(3).

## Required implementation evidence

The dedicated gate verifies that:

1. named families `F_S`, `F_U`, `F_D` are registered in tests and realize the
   closed-form products of TDI-10.5–10.7;
2. each family is wired through `CavityTransportStep` / `CavityTransportChain`
   with zero drift;
3. at least one `F_U` chain sources its constant factor from
   `FrozenToeplitzCavity::contraction` (TDI-10.1 vocabulary);
4. `F_S` remains the explicit non-decay cavity family (**REFUTED** universal
   decay);
5. documentation retains the EXACT / REFUTED boundary and the FORMAL pointer
   only — does not claim soft-edge, slowly-varying, or Riemann conclusions;
6. existing `tdi-operator` tests and strict Clippy remain green.

Finite floating-point products are **NUMERICAL EVIDENCE** for the algebraic
identities; the identities themselves are **EXACT**.

## Explicit non-claims

TDI-10.13 does **not**:

- prove slowly-varying Jacobi asymptotics or soft-edge / double-scaling limits;
- attach variable Jacobi coefficients to `alpha_k` beyond zero-drift cavity
  maps and the declared frozen-Toeplitz contraction source;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins or set any execution flag true.
