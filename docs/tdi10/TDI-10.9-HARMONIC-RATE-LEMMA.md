# TDI-10.9 — Harmonic remainder-rate lemma and closed-form witness calculus

Status: **EXACT rate companion to TDI-10.7–10.8, with closed-form witness identities**

## Scope

TDI-10.7 states that divergence of `sum (1 - alpha_k)` forces the subunit
product to vanish. TDI-10.8 classifies three named witnesses by product-limit
regime. TDI-10.9 records a **finite-rate** sufficient criterion that forces
remainder divergence by elementary harmonic comparison, together with the
**EXACT** finite-`n` closed forms already realized by those witnesses.

This stage does **not** duplicate TDI-10.7's logarithm inequality. It adds a
comparison-rate window that is complementary to the abstract divergence
hypothesis, and packages the witness calculus used to prove sharpness.

## EXACT harmonic rate lemma

Assume `0 < alpha_k <= 1` for every finite step `k`. Suppose there exist
constants `c > 0` and an integer `K >= 1` such that

`1 - alpha_k >= c / k` for all `k >= K`.

Then the remainder series diverges:

`sum_{k=1}^{infinity} (1 - alpha_k) = infinity`,

because the tail dominates a positive multiple of the harmonic series. By the
TDI-10.7 remainder criterion, therefore

`product_{k=1}^n alpha_k -> 0` as `n -> infinity`.

This is a **Type D sufficient rate**: any family whose remainder stays at least
harmonic-order eventually is Type D under the TDI-10.8 classification.

### Elementary comparison proof

For integers `m >= K`,

`sum_{k=K}^m (1 - alpha_k) >= c sum_{k=K}^m 1/k`.

The partial harmonic sums `H_m - H_{K-1}` diverge as `m -> infinity`. Hence the
remainder series diverges, and TDI-10.7 applies.

Terms with `alpha_k = 1` contribute nothing. The hypothesis `alpha_k > 0`
excludes one-step annihilation (already covered as a trivial product-zero case).

## Sharpness of the harmonic window

The Type D witness `alpha_k = k/(k+1)` satisfies

`1 - alpha_k = 1/(k+1)`,

so for every `c in (0, 1]` and all sufficiently large `k`,

`1 - alpha_k >= c / k`.

The rate is therefore **sharp for the comparison lemma**: the canonical Type D
family sits exactly on the harmonic boundary (up to the shift `k -> k+1`), and
its closed product `1/(n+1)` tends to zero. Decay therefore does **not** require
a uniform geometric gap (`alpha_k <= rho < 1`); harmonic remainder rate is enough.

## REFUTED: superharmonic lower rate is sufficient for decay

The naive claim

`exists c > 0, epsilon > 0, K: (k >= K => 1 - alpha_k >= c / k^{1+epsilon})
  => product_{k=1}^n alpha_k -> 0`

is **REFUTED**.

A lower bound by a convergent `p`-series does **not** force remainder
divergence. The Type S witness

`alpha_k = 1 - 1/(k+1)^2`

satisfies `1 - alpha_k = 1/(k+1)^2`, and for `epsilon = 1` and a suitable
`c in (0, 1]` one has `1 - alpha_k >= c / k^{2}` for all large `k`, while

`product_{k=1}^n alpha_k = (n+2)/(2(n+1)) -> 1/2 != 0`.

Thus a polynomial-superharmonic remainder lower bound (`1+epsilon` with
`epsilon > 0`) is **not** a sufficient decay criterion. Harmonic order (`1/k`)
is the elementary comparison threshold used here.

Type S therefore also shows that a `~ 1/k^2` remainder rate is compatible with
product ↛ 0, while Type D shows that a `~ 1/k` rate is compatible with
product → 0.

## Closed-form witness calculus (EXACT finite-n identities)

Under the zero-drift `CavityTransportStep` constructions already used in
TDI-10.5–10.8, the three named witnesses admit **EXACT** finite products:

| Type | Factor `alpha_k` | Finite product `P_n` | Limit |
|------|------------------|----------------------|-------|
| S | `1 - 1/(k+1)^2 = k(k+2)/(k+1)^2` | `(n+2)/(2(n+1))` | `1/2` |
| U | constant `rho in (0,1)` | `rho^n` | `0` |
| D | `k/(k+1)` | `1/(n+1)` | `0` |

Remainder rates:

- Type S: `1 - alpha_k = 1/(k+1)^2` (summable; `p = 2`);
- Type U: `1 - alpha_k = 1 - rho > 0` (uniform; linearly divergent);
- Type D: `1 - alpha_k = 1/(k+1)` (harmonic; comparison-sharp for TDI-10.9).

These identities are **EXACT algebra**. Finite floating-point checks are
**NUMERICAL EVIDENCE** for the implementation only.

## Relation to TDI-10.7 and TDI-10.8

- TDI-10.7 supplies the abstract divergent-sum criterion and the
  `log(1-x) <= -x` inequality.
- TDI-10.8 organizes Types S/U/D.
- TDI-10.9 supplies the **harmonic comparison rate** that implies 10.7's
  hypothesis, proves that rate is sharp on the Type D witness, **REFUTES**
  superharmonic lower-rate sufficiency, and packages the closed-form witness
  table.

## Required implementation evidence

The dedicated gate verifies that:

1. the Type D remainder satisfies `1 - alpha_k >= c/k` on a declared window
   and the product matches `1/(n+1)`;
2. harmonic partial-sum comparison lower-bounds the Type D remainder;
3. the Type S family satisfies a `1/k^2` lower bound on a window while its
   closed product stays at `(n+2)/(2(n+1))` bounded away from zero
   (**REFUTED** superharmonic sufficiency);
4. Type U closed products match `rho^n`;
5. admissible zero-drift cavity chains realize the rate witnesses;
6. documentation retains the EXACT/REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
7. existing `tdi-operator` tests and strict Clippy remain green.

## Explicit non-claims

TDI-10.9 does **not**:

- replace or weaken TDI-10.7's abstract divergence criterion;
- claim that `1/k` is necessary for every Type D family (only that it is a
  sufficient comparison rate, sharp on the named witness);
- identify which general Jacobi families realize harmonic rates;
- introduce soft-edge asymptotics, double scaling, or fitted constants;
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins.
