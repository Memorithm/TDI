# TDI-23.2 — Bounded Local Rewrite Calculus

Status: **CANDIDATE PREPARATION / DRAFT PR / BLOCKED ON TDI-23.1 FREEZE / NOT FROZEN / NO REWRITE SEARCH / NO CONFIRMATORY EXECUTION**

## Purpose

This branch prepares a deliberately tiny candidate local rewrite calculus over the TDI-23.1 typed IR. It does **not** promote TDI-23.2 to the active stage: the canonical TDI-23 programme still blocks TDI-23.2 on a stable TDI-23.1 grammar. The purpose of this candidate is to establish that explicit local rewrite rules can be represented, pattern-matched, independently validated, and verified against the existing exact semantic oracle before any broader rewrite engine exists.

Versioned candidate contract:

`tdi23.2-local-rewrite-calculus-v1`

Implementation:

`tdi-ai/src/tdi23_rewrite.rs`

## Admitted rules

Exactly four rules are admitted in this first candidate slice:

1. left identity: `id_B o f -> f`;
2. right identity: `f o id_A -> f`;
3. dagger of identity: `id_A^dagger -> id_A`;
4. double dagger: `(f^dagger)^dagger -> f`.

No other rule is implied by the existence of this module.

## Exactness classes

The verifier distinguishes two guarantees.

### Finite-value exact

Left- and right-identity elimination require ordinary finite `f64` equality for every matrix entry. This deliberately does not claim IEEE-754 bit identity because the Stage-0 matrix-composition implementation may normalize signed zero during multiply-accumulate evaluation. For example, `+0.0 == -0.0` under ordinary `f64` equality while their bit patterns differ.

### Bitwise exact

Dagger-of-identity and double-dagger elimination require identical IEEE-754 bit patterns for every scalar entry.

The exactness class is part of the rule contract; it is not inferred post hoc from a favorable fixture.

## Validation sequence

For one explicitly requested rule and one explicitly supplied root:

1. independently validate the rooted subgraph with the TDI-23.1 provenance validator;
2. pattern-match the requested local rule exactly;
3. select an already-existing node in the same IR as the replacement;
4. invoke the TDI-23.1 exact equivalence oracle on original and replacement;
5. reject any nonlinear/algebraic boundary encountered by that oracle;
6. require the rule's declared exactness class;
7. return the verification report with both finite-value and bit-identity diagnostics.

The function does not mutate the graph.

## Explicitly excluded rewrites

This candidate does **not** authorize:

- associativity or reassociation of composition under floating-point arithmetic;
- commutation or reordering of arbitrary maps;
- tensor-product reassociation, symmetry, fusion, distribution, or contraction rewrites;
- direct-sum reassociation, fusion, distribution, or head-concatenation transformations;
- conversion between direct sum and tensor product;
- softmax, Boolean, `F2`, ANF/Zhegalkin, or max-plus rewrites;
- reduction push-through rules;
- approximate or tolerance-based equivalence;
- recursive rewriting;
- rewrite enumeration;
- rewrite search, ranking, cost modeling, or planning;
- FLAT-ATTENTION integration;
- confirmatory or final execution.

## Boundary discipline

An identity surrounding an opaque nonlinear boundary is *not* rewritten by this candidate even when an abstract category could admit an identity law there. The verifier delegates semantic acceptance to the Stage-0 linear oracle, which fails closed at softmax/Boolean/`F2`/ANF/max-plus boundaries. Cross-domain rewrite semantics require a later explicit interface contract.

Coordinate-reduction annotations are not interpreted as a permission to push a rewrite through a reduced execution plan. The current verifier establishes only unreduced Stage-0 linear semantics. Any interaction between rewrite rules and staged/global reduction requires a separate TDI-23 contract.

## Deterministic controls

The development tests cover at least:

- contract versioning;
- left- and right-identity elimination;
- explicit signed-zero distinction between value equality and bit identity;
- dagger-of-identity elimination;
- double-dagger elimination including a signed-zero carrier entry;
- pattern mismatch rejection;
- fail-closed identity elimination across a softmax boundary;
- refusal to manufacture an identity rule merely because distinct endpoint objects share the same dimension.

## Promotion gate

This candidate may not be promoted from draft preparation to active TDI-23.2 until the required TDI-23.1 grammar/equivalence foundation is explicitly frozen and qualified on the exact repository state used by the candidate.

After promotion, additional rewrite rules may be proposed only one rule family at a time. Each family must declare:

- exact structural preconditions;
- endpoint/object-identity requirements;
- nonlinear-boundary behavior;
- exactness class;
- whether the rule changes evaluation order;
- deterministic positive and negative fixtures;
- an explicit termination or application bound before any recursive rewriting is allowed.

Rewrite **search** remains unauthorized until the rule set, exactness classes, and global application bound are separately frozen.
