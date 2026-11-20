# TDI-10 — Conditional block Schur coercivity

Status: **PROVED UNDER DECLARED ASSUMPTIONS** for the real/self-adjoint
operator inequality; binary64 implementation is a diagnostic, not an
outward-rounded formal proof.

Code: [tdi-operator/src/block_schur.rs](../../tdi-operator/src/block_schur.rs).

For a self-adjoint block form

    M = [A B; B* D],

assume A >= alpha I with alpha>0, D >= delta I and the cross
coupling satisfies ||B|| <= beta. The identity

    M >= 0  iff  A >= 0 and D - B* A^-1 B >= 0

holds for the compatible, bounded-domain block decomposition with strictly
positive invertible A; the useful sufficient inequality is

    D - B* A^-1 B >= (delta - beta^2/alpha) I.

Consequently, alpha>0 and delta-beta^2/alpha>0 imply strict positivity
of the quadratic form, subject to the stated operator domain assumptions.

The Rust function \`conditional_block_schur_margin(alpha, delta, beta)\`
calculates a conditional margin and returns either
\`StrictlyPositiveConditional\` or \`Undetermined\`.
An undetermined result is **not** a negative witness.

All three inputs must be independently established valid bounds. A
single floating-point number, a fitted constant, or differences between
numerical resolutions do not establish certified bounds. All arithmetic in
the helper is binary64 without directed rounding. It returns a
computational diagnostic, not a machine-checked certificate.

To prevent a known sign-changing floating-point failure, the helper rejects
a nonzero coupling norm when its computed square is zero or subnormal, with
UnderflowedCouplingSquare. For example, alpha=1e-300, delta=1e-150,
beta=1e-200 would otherwise wrongly appear positive when beta squared
underflows, although beta^2/alpha is about 1e-100 and exceeds delta.
This guard is necessary but does not replace certified interval arithmetic.

## Cross-project application

This generic inequality is reusable in the Riemann-specific
\`Memorithm/riemann_ndim_bench\` Schur gate (PR #75, November 2026),
which separately estimates two finite block lower bounds with
entrywise-error assumptions and a Frobenius upper bound on the coupling.

Do not import zeta zeros, Sonin/prolate projections, or claims of global
Weil positivity into TDI: its status here is a generic block-operator
inequality. Future uses must supply their own operator decomposition,
coercivity and domain proofs.

## Test commands

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test -p tdi-operator block_schur

The tests cover strict positive margins, undecided conditions,
zero coupling and invalid/nonfinite inputs.
