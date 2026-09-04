# TDI-10.0 Phase 0 audit

Status: **EXACT repository audit / no mathematical theorem claim**

Tracking issue: #141.

## TDI architecture audit

Current TDI workspace members before TDI-10 were:

- `tdi-core` — exact finite-state primitives and historical TDI core semantics;
- `tdi-bench` — reproducible TDI benchmark surfaces;
- `tdi-ai` — bounded architecture/adaptive-inference research for later series.

None is an appropriate owner for a reusable Jacobi/resolvent mathematics layer:

- placing it in `tdi-core` would blur the historical finite-state ownership boundary;
- placing it in `tdi-ai` would incorrectly frame operator theory as an AI evaluator primitive;
- placing algorithms directly in `tdi-bench` would make the benchmark crate own mathematical semantics.

Decision: introduce a separate `tdi-operator` workspace crate and let future `tdi-bench` binaries depend on it when benchmark questions are explicitly opened.

TDI reproducibility constraints remain unchanged: exact-head CI before merge, frozen historical evidence is not rewritten, and TDI-7.2/TDI-8.2/TDI-9.2 boundaries are unaffected.

## RiemannBench audit

The recent generic/operator-relevant lineage was reviewed from merged PRs #18 and #20–#31 and the default-branch operator sources.

### Generic operator theory — transferable by re-derivation

| RiemannBench lineage | Generic content | TDI-10 decision | Evidence status |
| --- | --- | --- | --- |
| #23 | left/right Schur cavities and selected Green bands | transfer now as generic finite identities | EXACT |
| #24 | positive frozen Toeplitz cavity fixed point `q=(a+sqrt(a^2-4b^2))/2` and `kappa=b^2/q^2` | later TDI-10 increment after finite core | EXACT for frozen model |
| #25 | finite-to-frozen cavity error transport identity | later generic transport module | EXACT |
| #26 | factorization of transport and drift terms | later, after generic local-reference contract | EXACT algebra once definitions are declared |

These ideas are reimplemented from generic formulas rather than copied with Riemann-specific constructors.

### Generic numerical algorithms — transferable

| RiemannBench lineage | Algorithm | TDI-10 decision | Evidence status |
| --- | --- | --- | --- |
| #20 | O(m) tridiagonal LDL and selected inversion of diagonal/first off-diagonal resolvent bands | transfer now in `tdi-operator` | EXACT finite algorithm |
| #18 | eigenvalues-only tridiagonal path and pairwise centered finite differences | retain as later candidate benchmark/numerical tool; not required by first core | NUMERICAL ALGORITHM |
| #21 | row-resolved trace from selected inverse bands and a tridiagonal weight operator | generic identity is reusable, but weighted-trace semantics are outside first core | EXACT finite identity |

### Reusable independent tests/oracles

- #23 validates cavity Green bands against a dense self-adjoint spectral reconstruction at small dimension. TDI-10 keeps the principle of an independent dense oracle but implements a different Gauss-Jordan inversion oracle in tests so the production recurrence is not validated by itself.
- #20 validates O(m) selected inversion against a spectral oracle. TDI-10 independently compares both selected-LDL bands and cavity-reconstructed bands against the dense test oracle.
- deterministic finite dimension/shift sweeps are reusable as a validation pattern.

### Riemann-specific — do not transfer

The following stay in `riemann_ndim_bench`:

- `W+` / `W-` sectors and parity sign corrections;
- `build_k0`, `build_kprime_closed`, and all coefficients derived from the semilocal prolate construction;
- sign-corrected Phase-4 weighted traces and their normalizations;
- generalized-prolate spectral-crossing interpretation;
- Riemann/zeta consequences;
- #30 exact zero-shift left-cavity closed form;
- #31 exact zero-shift shift derivative and universal logarithmic derivative.

The last two are exact finite identities for the specific semilocal coefficient family, not generic Jacobi theorems.

### Not mature enough to transfer as theorem

PR #29 derives a model-specific formal soft-edge expansion with local frozen contraction approaching one and a formal polynomial cumulative law. It explicitly does **not** provide the uniform remainder/control needed for the full variable-coefficient transport.

TDI-10 therefore transfers only the research question:

> when local `kappa_i -> 1`, what hypotheses imply useful bounds on cumulative products of the actual transport multipliers?

No asymptotic coefficient or decay law from #29 is imported as a generic result.

## First-core architecture

The first TDI-10 increment implements:

1. `JacobiMatrix` with exact dimension and finiteness invariants;
2. positive shifted `LDL^T` factorization in O(m);
3. selected inverse diagonal and first off-diagonal bands in O(m);
4. exact left/right Schur cavities;
5. exact selected Green bands from both finite boundaries;
6. an independent dense Gauss-Jordan oracle for small test matrices only.

The implementation accepts any finite shift for which all shifted LDL/Schur pivots remain strictly positive. This is slightly more general than restricting to `t >= 0` for an already-positive operator, while preserving the positive-factorization contract.

## Explicit non-claims

This first increment does not establish:

- slow variation of any coefficient sequence;
- a local frozen approximation error;
- a uniform soft-edge window;
- cumulative contraction estimates;
- asymptotic resolvent bounds;
- a theorem for infinite Jacobi operators;
- any statement about RH.

Its mathematical outputs are finite **EXACT** identities under the declared positive-pivot condition. Floating-point tests are **NUMERICAL EVIDENCE** that the implementation matches an independent finite oracle; they are not proofs of future asymptotic statements.
