# TDI-23.0 — Global Reduction of the Dagger Carrier

Status: **development-only Stage-0 formalization; not frozen; no confirmatory execution authorized**.

## Purpose

This note defines the first bounded notion of a **global reduction** for the TDI-23 real finite-dimensional dagger scaffold. The purpose is not to claim a quotient category or a production compression method. The purpose is to expose, with explicit equations and executable controls, which structures survive an object-wise reduction and where information loss appears.

For Stage 0 every object uses a declared orthonormal coordinate basis. A reduction of an object `H = R^n` keeps an ordered, duplicate-free subset of basis coordinates.

Let

`E_H : R^r -> R^n`

be the coordinate isometry whose columns are the retained standard-basis vectors. Then

`E_H^dagger E_H = I_r`

and

`P_H = E_H E_H^dagger`

is the coordinate projector onto the retained ambient subspace.

## Reduced morphism

For a linear morphism

`f : H -> K`,

the Stage-0 reduced morphism is

`R(f) = E_K^dagger f E_H`.

With coordinate isometries this is exactly the row/column submatrix selected by the retained coordinates of `K` and `H`.

The corresponding lift is

`L(R(f)) = E_K R(f) E_H^dagger = P_K f P_H`.

The lift therefore preserves the selected block and zeroes all omitted rows and columns. It is an audit reconstruction, not an inverse in general.

## Exact dagger compatibility

The first structure-preservation target is exact:

`R(f^dagger) = R(f)^dagger`

provided the source and target reductions are swapped in the adjoint direction.

Indeed,

`R(f^dagger) = E_H^dagger f^dagger E_K`

and

`R(f)^dagger = (E_K^dagger f E_H)^dagger = E_H^dagger f^dagger E_K`.

The Stage-0 Rust scaffold tests this identity on deterministic finite fixtures using coordinate selection only.

## Composition is deliberately not assumed to survive

Let

`f : A -> B`

and

`g : B -> C`.

Reducing the composite gives

`R(g o f) = E_C^dagger g f E_A`.

Composing the reduced maps gives

`R(g) o R(f) = E_C^dagger g E_B E_B^dagger f E_A`

or equivalently

`R(g) o R(f) = E_C^dagger g P_B f E_A`.

In exact real algebra the difference is therefore

`D_comp = E_C^dagger g (I_B - P_B) f E_A`.

This term is the contribution carried by omitted intermediate coordinates. In general it is non-zero. Consequently the Stage-0 coordinate reduction is **not claimed to be a functor on arbitrary morphisms**.

The implementation evaluates the omitted-path term on the right-hand side directly, in ambient intermediate-coordinate order, and reports its `max_abs` value. It deliberately does **not** compute `R(g o f) - R(g) o R(f)` by subtracting two separately accumulated `f64` matrix products. That subtraction can produce a non-zero value solely from IEEE-754 reassociation when the retained intermediate coordinates are a permutation of the full basis, even though `P_B = I_B` and no path was removed.

Accordingly, the Stage-0 metric is a numerical evaluation of the declared **structural omitted-path defect**. It is kept separate from floating-point execution residuals. A later frozen evaluator may introduce a second numerical-equivalence metric with an explicit tolerance and common execution order, but Stage 0 does not pin such a tolerance.

## Reconstruction residual

For one map the reduce/lift residual is

`D_lift = f - P_K f P_H`.

The initial implementation records

`||D_lift||_max = max_ij |(D_lift)_ij|`.

This is only the first deterministic diagnostic. A later frozen evaluator may add Frobenius, spectral, application-weighted, attention-score, or task-level errors, but none of those metrics is frozen by Stage 0.

## Global interpretation

A global reduction candidate consists of one declared reduction per participating object. It is therefore an object-wise family

`{E_H}`

used consistently across every morphism touching those objects.

The research problem is to identify conditions under which this family preserves enough structure to be useful. Candidate sufficient conditions to investigate later include, without being frozen here:

- the omitted intermediate term `(I - P_B) f E_A` is zero;
- `f` maps the retained source subspace into the retained target subspace;
- `g` annihilates omitted intermediate components relevant to the retained output;
- a controlled approximate version of these conditions holds under an explicitly frozen error budget.

Stage 0 does not claim these conditions are necessary, optimal, or sufficient for attention quality.

## Relevance to FLAT-ATTENTION

The direct relevance is structural. A future attention graph may contain maps for query/key/value projections, head-local transforms, state transforms, or other linear components. TDI-23 can ask whether a common family of reduced object spaces preserves:

1. dagger semantics;
2. pairings such as `k^dagger q`;
3. composition to within a declared structural-defect budget;
4. enough task semantics to justify a downstream FLAT-ATTENTION experiment.

A positive TDI-23 result would still not establish lower latency, lower memory, better model quality, or a valid FLAT kernel. Those remain separate downstream qualification questions.

## Stage-0 executable controls

The development scaffold must prove at least the following deterministic properties:

- invalid coordinate selections fail closed;
- coordinate reduction commutes with dagger on finite fixtures;
- reduce/lift/reduce is idempotent on the selected block;
- retaining the complete intermediate basis yields zero structural composition defect, including when that complete basis is represented in a noncanonical order;
- deliberately dropping a contributing intermediate path yields a non-zero, predicted structural composition defect;
- reduce/lift residual reports discarded entries rather than silently treating the lift as lossless.

## Deferred generalizations

The following are deliberately deferred until a later explicit freeze decision:

- arbitrary orthonormal-basis isometries rather than coordinate selectors;
- learned or data-dependent subspaces;
- low-rank SVD/PCA-style reductions;
- complex-valued `FdHilb` carriers;
- approximate dagger preservation under quantization;
- tensor-product-aware reduction;
- direct-sum/head-aware reduction;
- search over reduction families;
- FLAT-ATTENTION integration.

This staged restriction keeps the first reduction mechanism exact enough to audit and small enough to falsify.