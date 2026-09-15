# TDI-22.0 — Torsor Attention Stage-0 Scope

Status: **active bootstrap; not frozen; no confirmatory execution authorised**.

## Purpose

TDI-22.0 establishes the mathematical and software boundary required before any torsor-attention experiment compares task outcomes. The only executable scientific surface authorised in Stage 0 is deterministic algebraic scaffolding for the declared three-dimensional torsor/twist convention.

## Allowed work

Stage 0 may define finite three-dimensional vectors, torsor/wrench-like and twist-like data types; implement the declared change-of-reduction-point law; derive the origin-reduced moment `C`; implement direct and factorized dual pairings; test reduction-point invariants and coordinate-origin translation consistency; reject non-finite arithmetic; document later control arms; and add bootstrap integrity checks.

## Deferred work

Quality comparisons between vector, hybrid and torsor attention arms are deferred to later frozen stages. TDI-22.0 does not authorize final datasets or final result material, learned geometry selection, FLAT-ATTENTION runtime changes, or claims about quality, memory, complexity, latency, bandwidth or throughput.

## Exact convention

For a torsor reduced at `P`:

```text
M(Q) = M(P) + (P - Q) x R
C    = M(P) + P x R
M(Q) = C - Q x R
```

For query twist `(v, omega)` at `Q`:

```text
s_direct     = v . R + omega . M(Q)
s_factorized = (v + Q x omega) . R + omega . C
```

The Stage-0 implementation must make the two score forms agree within a tolerance attributable to floating-point roundoff. No learned parameters are present in this test.

## Required control before evaluation

A later TDI-22 protocol must include a matched six-component non-torsor control. Otherwise an apparent full-torsor advantage cannot distinguish torsor structure from increased representational width.

Later matched protocols must separately account for parameter count, dynamic state/cache bits, native score operations, geometry-generation cost, normalization choice, and training/evaluation compute when learning is introduced.

## Stage transition

TDI-22.1 remains blocked until the Stage-0 programme/scope are merged, exact algebra tests are green, the bootstrap check passes, TDI-22.0 is explicitly frozen by a reviewed change, and the TDI ecosystem roadmap is reread before Stage-1 preregistration.

Passing Stage 0 establishes implementation consistency only. It is not a positive torsor-attention result.
