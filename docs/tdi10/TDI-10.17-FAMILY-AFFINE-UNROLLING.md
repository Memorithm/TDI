# TDI-10.17 — Exact family ↔ affine-unrolling link (10.2/10.4 into F_S/F_U/F_D)

Status: **EXACT zero-drift collapse of TDI-10.4 unrolling onto named-family
closed products; EXACT constant-drift F_U / F_D accumulated-drift closed forms
via the 10.2/10.4 affine recurrence; REFUTED that Type-U product decay alone
forces cavity-error → 0 under nonzero constant drift**

## Scope

TDI-10.13–10.15 realized named families `F_S` / `F_U` / `F_D` as **zero-drift**
cavity chains and recorded closed transport products (including `F_U` then
`F_D`/`F_S` concatenation). TDI-10.4 already gives the exact affine unrolling

`E_n = A_n E_0 + B_n`,

with `A_n = ∏_{k=1}^n alpha_k` and
`B_n = Σ_{k=1}^n delta_k ∏_{r=k+1}^n alpha_r`.

TDI-10.17 does **not** reopen the κ(a,b) domain calculus (10.16) or the
subunit-product arc (10.5–10.12). It records the **EXACT** specialization of
the 10.2/10.4 identities to already-declared family alpha maps, including the
nonzero constant-drift case that zero-drift family witnesses deliberately
excluded.

No slowly-varying Jacobi theorem is claimed. No new physical parameters are
introduced: constant drift `δ` is the already-declared one-step TDI-10.2
`delta` held fixed across steps; family alphas are those of TDI-10.13.

## Standing vocabulary (cited, not reinvented)

From TDI-10.2 / 10.4: one-step `E_i = alpha E_j + delta`; finite unrolling
`E_n = A_n E_0 + B_n` via `CavityTransportChain`.

From TDI-10.13 (block indices start at 1):

- `F_U(ρ)`: `alpha_k = ρ` for fixed `ρ ∈ (0,1)`; closed product `A_n = ρ^n`.
- `F_D`: `alpha_k = k/(k+1)`; closed product `A_n = 1/(n+1)`.
- `F_S`: `alpha_k = 1 - 1/(k+1)^2`; closed product `A_n = (n+2)/(2(n+1))`.

From TDI-10.14: `ρ` may be sourced from
`FrozenToeplitzCavity::contraction(a,b) ∈ (0,1)`.

Realization: contiguous `CavityTransportStep` / `CavityTransportChain` with
either zero drift (family product witnesses) or constant nonzero drift
(closed `B_n` formulae below).

## EXACT claim 1 — zero-drift family chains collapse 10.4 onto the product

Fix any named family `F ∈ {F_S, F_U(ρ), F_D}` and integer `n ≥ 1`. Realize the
family as a contiguous zero-drift cavity chain (`delta_k = 0` for all steps)
exactly as in TDI-10.13. Then the TDI-10.4 unrolling collapses to

`E_n = A_n E_0`, `B_n = 0`,

where `A_n` is the closed family product above. Equivalently, the public chain
API must satisfy

- `cumulative_transport_factor = A_n`,
- `accumulated_drift = 0`,
- `reconstructed_final_error = A_n · initial_error`.

Floating-point checks are **NUMERICAL EVIDENCE** for the Rust realization;
the collapse itself is **EXACT** finite algebra under the already-required
positivity / finite-value conditions.

## EXACT claim 2 — constant-drift F_U geometric unrolling

Fix `ρ ∈ (0,1)`, constant drift `δ ∈ ℝ`, and `n ≥ 1`. Let every step use
`alpha_k = ρ` (family `F_U`) and `delta_k = δ`. Then

`A_n = ρ^n`,

`B_n = δ · (1 - ρ^n) / (1 - ρ)`   (for `ρ ≠ 1`; here `ρ < 1` by Type U),

and therefore

`E_n = ρ^n E_0 + δ · (1 - ρ^n) / (1 - ρ)`.

In particular, as `n → ∞`,

`E_n → δ / (1 - ρ)`

whenever the constant-drift / constant-alpha hypotheses hold (the transport
product still vanishes, but the affine particular solution need not).

When `ρ` is sourced from `FrozenToeplitzCavity::contraction`, the same closed
form holds with that Toeplitz coefficient (TDI-10.14 bridge reused).

## EXACT claim 3 — constant-drift F_D accumulated-drift closed form

Fix constant drift `δ` and `n ≥ 1` with `alpha_k = k/(k+1)` (family `F_D`).
Then `A_n = 1/(n+1)` and the suffix products are

`∏_{r=k+1}^n alpha_r = (k+1)/(n+1)`   (empty product `= 1` when `k = n`).

Hence

`B_n = Σ_{k=1}^n δ · (k+1)/(n+1) = δ/(n+1) · ( Σ_{j=2}^{n+1} j )
     = δ/(n+1) · ( (n+1)(n+2)/2 - 1 )`.

So

`E_n = E_0/(n+1) + δ/(n+1) · ( (n+1)(n+2)/2 - 1 )`.

## REFUTED: Type-U product decay alone forces cavity-error → 0 under drift

The candidate claim

> whenever a cavity chain realizes Type-U factors `alpha_k = ρ < 1`, the
> reconstructed cavity error `E_n` must tend to 0 as `n → ∞`

is **REFUTED** under nonzero constant drift. Take any `ρ ∈ (0,1)`, any
`δ ≠ 0`, and the constant-drift `F_U` chain of EXACT claim 2. Then
`A_n = ρ^n → 0`, yet

`lim_{n→∞} E_n = δ / (1 - ρ) ≠ 0`.

Product decay of the homogeneous transport factor does **not** imply error
decay once the TDI-10.2 drift term is allowed to be a nonzero constant.
(Zero-drift Type-U chains remain compatible with `E_n → 0`; the refutation
targets the drift-ignorant claim.)

## Required implementation evidence

The dedicated gate verifies that:

1. zero-drift `F_S` / `F_U` / `F_D` chains satisfy EXACT claim 1 against closed
   products;
2. constant-drift `F_U` chains match `A_n`, `B_n`, and `E_n` of EXACT claim 2,
   including at least one Toeplitz-sourced `ρ`;
3. constant-drift `F_D` chains match EXACT claim 3;
4. an explicit constant-drift `F_U` example **REFUTES** error → 0 while
   `ρ^n → 0`;
5. documentation retains the EXACT / REFUTED boundary and does not claim
   soft-edge, slowly-varying, or Riemann conclusions;
6. existing `tdi-operator` tests and strict Clippy remain green.

## Explicit non-claims

TDI-10.17 does **not**:

- prove slowly-varying Jacobi asymptotics or soft-edge / double-scaling limits;
- attach variable Jacobi coefficients `(a_i,b_i)` beyond the cavity alpha/delta
  maps already used in TDI-10.2–10.15;
- claim a new κ(a,b) identity (that is TDI-10.16);
- claim that every nonzero-drift family forces a nonzero error limit (only the
  constant-drift Type-U case above);
- import RiemannBench coefficients or RH premises;
- authorize any TDI-7.2 / TDI-8.2 / TDI-9.2 surface;
- invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins or set any execution flag true.
