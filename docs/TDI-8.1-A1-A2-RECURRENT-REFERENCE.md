# TDI-8.1 — Deterministic A1/A2 recurrent reference

Status: **BOUNDED IMPLEMENTATION — NON-FINAL DATA ONLY**

Parent preregistration blob:
`fe80e7053d89824a77ef6790794f6930d1b424e2`

## Purpose

This tranche defines deterministic bounded reference semantics for the first
primary contrast mechanisms:

- **A1:** recurrent state only;
- **A2:** the same recurrent-core family plus the bounded associative-memory
  primitive merged in PR #91.

It is a software-oracle layer. It does not run the nine primary cells and does
not establish a scientific advantage for A2 over A1.

## Recurrent core

The reference recurrent core is parameterized by input width, state width,
row-major input-to-state weights, row-major recurrent weights and bias. No
concrete experimental dimension is fixed by this tranche.

For every state coordinate, accumulation is performed in a fixed order:

1. bias;
2. recurrent-state coordinates from index 0 upward;
3. input coordinates from index 0 upward;
4. deterministic hard-tanh clipping to `[-1, +1]`.

All supplied parameters and step inputs must be finite IEEE-754 binary64
values. Any non-finite intermediate rejects the step before the persistent
recurrent state is committed.

Hard-tanh is used here because its reference behavior depends only on binary64
arithmetic and ordered comparisons; the evaluator does not depend on a
platform-specific transcendental implementation.

## A1 semantics

A1 contains only the recurrent core. One step computes a complete candidate
next-state vector, validates it, and atomically commits it. A rejected input or
non-finite intermediate leaves the persistent state unchanged.

The A1 accounting record includes:

- recurrent state: `state_width * 64` bits;
- temporary next-state working vector: `state_width * 64` bits;
- static recurrent matrices, bias and two declared layout dimensions.

A1 reports zero associative payload, associative metadata and VSA workspace.

## A2 semantics

A2 contains the same recurrent-core family plus a direct-mapped bounded
associative table whose payload width must equal recurrent-state width.

Each A2 step is ordered as follows:

1. validate input and compute a candidate recurrent state;
2. read `read_key` from the associative table;
3. if the read is a hit, fuse each resident payload coordinate using
   `hard_tanh(candidate + fusion_gain * payload)`;
4. if `write_key` is present, write the fused recurrent state under that key;
5. commit the fused recurrent state.

Therefore lookup is explicitly **before** the optional write. Empty and
collision-miss reads do not inject a payload. Collision/replacement outcomes
remain observable through the PR #91 associative-memory contract.

A2 rejects a non-finite fusion gain and rejects a memory layout whose payload
width differs from recurrent-state width.

## Accounting

A2 dynamic accounting includes:

- recurrent-state bits;
- associative payload bits;
- associative metadata/addressing bits;
- temporary next-state working bits.

Static accounting includes recurrent matrices/bias/layout, associative address
projection constants and one binary64 fusion gain.

This tranche does not claim that a particular A1/A2 pair is matched-budget.
Concrete dimensions must later be selected so the frozen A1/A2/A3 matched
budget constraint is exactly satisfied.

## Required software oracles

The implementation retains tests proving:

- bit-identical recurrence for identical starting state/parameters/input;
- rejected non-finite A1 input cannot mutate persistent state;
- A1 accounting validates as recurrent-only;
- A2 payload width must equal state width;
- a stored state can be retrieved and fused on a later A2 read;
- lookup precedes colliding replacement;
- A2 accounting includes associative payload and metadata;
- A1/A2 snapshots expose complete persistent reference state.

## Deliberately not frozen here

This tranche does not freeze:

- state width or input width;
- recurrent matrix values used by the experimental evaluator;
- associative slot count;
- projection seed;
- fusion gain used by the experimental evaluator;
- A3 VSA dimension or operators;
- A0 full-history reference parameters;
- task horizons;
- train/development/validation seed ranges;
- sample counts;
- paired interval implementation or replicate count;
- final rejection taxonomy;
- any TDI-8.2 seed, runner, token or result surface.
