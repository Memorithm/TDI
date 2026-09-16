# TDI-23.1 Status

Status: **ACTIVE DEVELOPMENT — typed IR scaffold implemented; not frozen; confirmatory execution unauthorized**

## Current implementation

TDI-23.1 now has a first bounded graph IR in `tdi-ai/src/tdi23_ir.rs`, exposed only through the `experimental` feature.

Implemented surfaces:

- instance-bound `ObjectId` and `NodeId` handles with explicit rejection of cross-IR handles;
- non-clonable mutable IR ownership in this slice, preventing divergent graphs from sharing one owner token;
- named finite-dimensional real Hilbert objects;
- atomic objects;
- direct sums with additive dimension;
- tensor products with multiplicative dimension;
- concrete typed Stage-0 real linear maps;
- identity nodes;
- typed composition with exact middle-object identity checks;
- dagger nodes restricted to boundary-free linear subgraphs;
- opaque nonlinear/algebraic boundaries for softmax, Boolean, `F2`, ANF/Zhegalkin, and max-plus;
- object-level coordinate-reduction annotations;
- boundary-free evaluation back to `RealLinearMap`;
- reduced evaluation through explicit endpoint reductions;
- composition-defect auditing through the Stage-0 omitted-path structural diagnostic.

## Important semantic properties

1. Handles are valid only in the IR instance that created them. A foreign object/node handle fails closed even if the receiving graph has the same numeric slot.
2. Equal dimensions are not sufficient for composition: the same declared middle object must connect the two nodes.
3. Direct sum and tensor product are distinct IR constructions and produce additive versus multiplicative dimensions respectively.
4. A nonlinear boundary may appear in a typed pipeline, but the IR refuses to reinterpret it as a linear morphism.
5. Dagger construction fails closed if its source subgraph crosses any declared nonlinear/algebraic boundary.
6. Coordinate reductions are annotations, never implicit transformations.
7. Reduced lowering fails if required object annotations are missing.
8. The composition-reduction diagnostic is the direct omitted-middle-path structural term inherited from TDI-23.0; it is not an independently accumulated floating-point residual.
9. The current owner token is process-local and is not a serialization/provenance identifier. Stable cross-process graph identity remains future work.

## Scientific authorization state

- TDI-23.1 is **not frozen**.
- TDI-23.0 freeze fields remain unresolved unless separately and explicitly frozen.
- `confirmatory_execution_authorized` remains **false**.
- `final_execution_authorized` remains **false**.
- No rewrite search, learned reduction, attention-quality comparison, FLAT integration, device lowering, or hardware-performance claim is authorized by this slice.

## Next bounded engineering work

After this slice is qualified and merged, the next TDI-23.1 work should remain development-only and focus on a small legality/equivalence oracle around the existing graph grammar:

- deterministic graph serialization/provenance with explicit stable graph identity distinct from the process-local owner token;
- explicit structural validation independent of construction path;
- boundary-aware equivalence fixtures for linear-only subgraphs;
- explicit reduction-contract summaries attached to candidate graph fragments;
- preparation for TDI-23.2 rewrite rules without yet enabling rewrite search.

TDI-23.2 must not start rewrite enumeration until the TDI-23.1 grammar and legality oracle have a stable versioned contract and deterministic regression corpus.
