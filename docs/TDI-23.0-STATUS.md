# TDI-23.0 Status

Status: **ACTIVE STAGE-0 BOOTSTRAP — not frozen; confirmatory execution unauthorized**

## Current state

TDI-23.x is initialized as a categorical/dagger-attention research line with a first explicit global-reduction research axis.

Primary Stage-0 surfaces:

- `docs/TDI-23-PROGRAMME.md`;
- `docs/TDI-23.0-SCOPE.md`;
- `docs/TDI-23.0-STATUS.md`;
- `docs/TDI-23.0-GLOBAL-REDUCTION.md`;
- `docs/tdi23/tdi23.0-stage0-freeze.template.json`;
- `scripts/check-tdi23.0-freeze-template.py`;
- `scripts/check-tdi23-stage0-bootstrap.sh`;
- `tdi-ai/src/tdi23_categorical.rs`, exposed only through the `experimental` feature;
- `tdi-ai/src/tdi23_reduction.rs`, exposed only through the `experimental` feature.

## What Stage 0 establishes

1. A bounded real finite-dimensional Hilbert-space analogue is available as development scaffolding.
2. Linear maps carry explicit domain and codomain dimensions.
3. The real dagger is implemented as matrix transpose under declared orthonormal bases.
4. Dagger involution and reversed-composition laws are tested on deterministic finite fixtures.
5. Query/key kets can be composed as `k^dagger o q`, and the resulting scalar is checked against the Euclidean dot product.
6. Dimension mismatches and non-finite inputs fail closed.
7. A coordinate-subspace reduction can be assigned object by object as a first auditable global-reduction family.
8. For that Stage-0 reduction, `R(f^dagger) = R(f)^dagger` is tested directly.
9. Reduce/lift/reduce idempotence is tested on the retained coordinate block, while a max-absolute reconstruction residual records discarded entries.
10. Composition is explicitly **not** assumed to be preserved: the scaffold compares `R(g o f)` with `R(g) o R(f)` and exposes the resulting defect.
11. Deterministic controls include both zero defect with a fully retained intermediate basis and a predicted non-zero defect when a contributing intermediate path is removed.
12. Softmax and the Boolean / `F2` / ANF / max-plus research domains remain explicitly outside the Stage-0 linear carrier.
13. Comparative scientific fields remain unresolved and both execution-authorization flags remain false.

## Authorization state

- TDI-23.0 is **not frozen**.
- `confirmatory_execution_authorized` is **false**.
- `final_execution_authorized` is **false**.
- No reduction-selection policy, approximate-error budget, attention-quality benchmark, model-training campaign, hardware benchmark, FLAT kernel integration, or final population is authorized.
- TDI-23.1 may not silently resolve Stage-0 freeze fields.

## Immediate next slice after bootstrap

After the Stage-0 integrity gate passes, the next non-final engineering slice is TDI-23.1: specify a minimal typed Categorical Attention IR grammar with explicit domain/codomain objects, composition, dagger, identity, nonlinear boundaries, and an object-reduction annotation whose preservation/defect contract is explicit. Tensor product and direct-sum constructions must remain distinct.

No rewrite search, reduction search, learned subspace, approximate tolerance claim, or FLAT lowering is authorized until that grammar has deterministic legality and equivalence/defect fixtures.
