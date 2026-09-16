# TDI-23.1 Status

Status: **ACTIVE DEVELOPMENT — typed IR and rooted provenance/validation merged; exact equivalence/reduction-contract slice active; not frozen; confirmatory execution unauthorized**

## Qualified foundations

TDI-23.1 now has two merged development foundations behind the `experimental` feature:

1. the typed categorical-attention IR in `tdi-ai/src/tdi23_ir.rs`;
2. deterministic rooted provenance and independent rooted validation in `tdi-ai/src/tdi23_ir_provenance.rs`.

The rooted provenance slice is merged and qualified; it is no longer merely “in qualification”. Its manifest remains construction-order-sensitive provenance, not graph-isomorphism canonicalization, a cryptographic digest, or a globally unique identifier.

## Active bounded slice

The current development slice is `tdi-ai/src/tdi23_ir_equivalence.rs`.

It adds:

- versioned exact boundary-aware comparison contract `tdi23.1-boundary-aware-exact-equivalence-v1`;
- versioned explicit reduction summary contract `tdi23.1-reduction-contract-summary-v1`;
- independent rooted validation before comparison;
- exact source/target object identity checks;
- fail-closed rejection of nonlinear/algebraic boundaries through Stage-0 lowering;
- separate ordinary finite-value equality and IEEE-754 bit-identity diagnostics;
- explicit reachable-object reduction summaries with no implicit annotation filling.

## Scientific authorization state

- TDI-23.1 remains **not frozen**.
- TDI-23.0 freeze fields remain unresolved unless separately and explicitly frozen.
- `confirmatory_execution_authorized` remains **false**.
- `final_execution_authorized` remains **false**.
- No rewrite search, approximate-equivalence policy, learned reduction, attention-quality comparison, FLAT integration, device lowering, or hardware-performance claim is authorized by this slice.

## Why this remains TDI-23.1

TDI-23.2 is reserved for an explicit rewrite calculus. This slice deliberately does not enumerate, apply, rank, or search rewrite rules. It only creates a stricter verifier and an explicit reduction-contract surface that future rewrite fixtures can call.

## Next gate

After this slice is merged and qualified, the next bounded work may define a small versioned TDI-23.2 rewrite rule set (for example identity elimination and double-dagger elimination) with deterministic legality checks. Rewrite **search** must remain disabled until rule termination/bounds and exactness classes are explicit.
