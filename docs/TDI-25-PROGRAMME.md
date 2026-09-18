# TDI-25.x — Torsor vs Chiral Attention Research Programme

Status: **Stage-0 bootstrap; consumes the merged TDI-24 chiral foundation; experimental; non-confirmatory**.

Tracker: #392.

## Research question

At matched six-component representation capacity, matched training/evaluation budgets and frozen task/split discipline, how does the TDI-22 torsor/twist attention candidate compare with the **same** chiral representation contract used by TDI-24?

The primary comparison is deliberately narrow:

```text
T6 — torsor/twist representation
vs
C6 — mirror-coupled chiral representation.
```

A generic six-component arm `G6` is permitted only as an attribution control. It is not a third primary hypothesis.

## Reuse rule

TDI-25 does not redefine either side.

- **T6** reuses the TDI-22 Stage-0 torsor/twist convention and dual-pairing implementation.
- **C6** reuses the TDI-24 Stage-0 `M/J` chiral contract.

Any later change to either source contract must be versioned and the TDI-25 comparison must record the exact versions used.

## Torsor arm

TDI-22 defines a wrench-like torsor reduced at `P`

```text
T(P) = (R, M(P))
M(Q) = M(P) + (P-Q) × R
C = M(P) + P × R
```

and a twist-like query

```text
xi = (v, omega).
```

The factorized six-component pairing is

```text
score_T = (v + Q × omega) · R + omega · C.
```

TDI-25 consumes this contract; it does not copy or fork the algebra.

## Chiral arm

TDI-24 defines

```text
C6 = (x+,x-) ∈ R^3 ⊕ R^3
M(x+,x-) = (x+,-x-)
J(x+,x-) = (x-,-x+)
```

with

```text
M²=I, Jᵀ=-J, J²=-I, M J M=-J
chi(q,k)=qᵀJk
chi(Mq,Mk)=-chi(q,k).
```

The scalar candidate family is versioned upstream by TDI-24. TDI-25 may choose coefficients or normalization only through a preregistered, capacity-matched protocol shared with its controls.

## Generic six-component diagnostic arm

`G6` uses six finite components and a generic bilinear/dot-product score without Varignon transport, torsor invariants, parity sectors, mirror involution or a distinguished complex structure. Its purpose is attribution:

- T6 vs G6 asks whether torsor structure contributes beyond six-dimensional capacity.
- C6 vs G6 asks whether chiral structure contributes beyond six-dimensional capacity.
- **T6 vs C6 remains the primary contrast.**

## Task-family matrix

The campaign must not choose only tasks favorable to one geometry. It therefore contains four declared families:

1. **Torsor-favorable geometry** — change of reduction point, translation and transported moment semantics where the target is invariant/equivariant under the TDI-22 convention.
2. **Chirality-favorable geometry** — mirrored pairs where parity/handedness is target-relevant.
3. **Mixed geometry** — tasks where a transported geometric relation and a parity-sensitive relation are both present.
4. **Neutral controls** — matched six-component information with neither torsor nor chirality privileged by construction.

Results must be stratified by family before any pooled summary is interpreted.

## Matching requirements

T6 and C6 must be matched, or discrepancies explicitly accounted for, across:

- six-component carrier budget;
- trainable parameter count;
- optimization/update budget;
- sequence length/masks;
- data population and seeds;
- readout capacity;
- precision;
- operation accounting;
- resident/peak memory accounting;
- evaluation protocol.

A physical 3D coordinate is not automatically meaningful for ordinary tokens. Any position geometry is an experimental arm with explicit provenance, never an implicit assumption.

## Primary outcomes

A result is reported by task family and paired seed block. Permitted outcomes include positive, equivalent, harmful, unstable and inconclusive. A pooled win cannot erase a family-specific reversal.

The campaign must report, where applicable:

- paired primary task metric difference;
- torsor transport/reduction-point identity error;
- chiral mirror-swap/parity identity error;
- G6 attribution contrast;
- calibration/stability diagnostics;
- operation/memory/runtime costs under explicitly qualified environments.

## No premature hybrid

A `torsor + chiral` combined architecture is **not** part of the primary TDI-25 campaign. Combining the two before isolating their effects would make attribution ambiguous. A later programme may study cooperation only after TDI-24 and TDI-25 establish reproducible evidence or a scientifically informative contrast.

## Scientific interpretation

TDI-25 can establish evidence about the declared candidates on frozen tasks and budgets. It cannot by itself establish universal replacement of vector attention, novelty, language-model superiority, asymptotic efficiency, lower KV memory, FLAT-ATTENTION readiness or real-hardware speedup.

## Campaign execution

The first bounded campaign contains **50 substantive PR slices** in `docs/TDI-25-CAMPAIGN-50.md`. Every slice has a concrete deliverable and dependency gate. Empty PR generation is forbidden.
