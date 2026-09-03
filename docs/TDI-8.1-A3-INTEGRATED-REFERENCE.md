# TDI-8.1 — Integrated bounded A3 / ASSR-H reference

Status: **BOUNDED SOFTWARE-ORACLE IMPLEMENTATION — NON-FINAL ONLY**

Parent preregistration blob:
`fe80e7053d89824a77ef6790794f6930d1b424e2`

## Purpose

This tranche integrates the already-merged deterministic A2 recurrent + bounded
associative-memory reference with the already-merged bounded VSA workspace.
It implements the missing A3 software-reference mechanism required by TDI-8.1.

It does not run T1/T2/T3, choose a final architecture configuration, access a
holdout, or establish H8-B evidence.

## Integrated A3 operation order

`A3Reference::step(input, read_key, write_key)` uses one explicit order:

1. read/unbind the current VSA workspace with `read_key`;
2. fuse the resulting readout coordinate-wise with the external input as
   `input[i] + vsa_fusion_gain * readout[i]`, in ascending coordinate order;
3. reject before A2 mutation if the runtime width is wrong or any input/fused
   coordinate is non-finite;
4. pass the complete fused input to the unchanged A2 `step` implementation;
5. A2 then performs its already-reviewed recurrent computation, associative
   lookup-before-write fusion and optional direct-mapped write.

The VSA workspace width is required to equal the recurrent input width so that
this coordinate-wise integration has no hidden projection or unaccounted
adapter.

The VSA fusion gain is an explicit finite constructor parameter. This tranche
does not select or freeze its experimental value.

## VSA writes remain explicit

`A3Reference::store_vsa(key, payload)` is intentionally separate from the A3
read/fuse/A2 transition. It delegates to the merged VSA candidate-before-commit
bundling oracle.

This separation gives a simple atomicity boundary:

- failed VSA retrieval/fusion cannot mutate A2;
- failed VSA store cannot mutate A2 or partially mutate the workspace;
- an A3 read/fuse step does not silently decide a task-level VSA write policy.

The later bounded evaluator must define when and what task representation is
stored. That policy is not inferred by this reference primitive.

## Exact memory accounting

A3 reports:

- the actual A2 recurrent-state bits;
- the actual A2 associative payload bits;
- the actual A2 associative metadata bits;
- the VSA persistent workspace bits;
- integrated temporary working storage;
- all static parameters/constants.

During an integrated step the VSA readout/fused-input vector remains live while
A2 computes its recurrent candidate vector. Therefore A3 temporary working
storage is the sum of the existing A2 temporary vector and the VSA width-sized
temporary vector. The explicit VSA fusion gain is also included as 64 static
bits.

No runtime allocator overhead is converted into an architecture-level claim;
the accounting follows the same representation-level policy already frozen for
A1/A2 and the standalone VSA oracle.

A software-oracle test provides one synthetic partition in which A1, A2 and A3
have exactly equal declared dynamic-memory totals. Those fixture dimensions are
only a proof that the accounting contract can represent a valid matched budget;
they are not TDI-8.1 experimental choices.

## Software-oracle coverage

The tranche tests that:

- constructor width and finite-gain guards fail closed;
- an empty VSA workspace preserves A2 step semantics bit-for-bit;
- a stored VSA record changes the integrated recurrent input as declared;
- rejected integrated steps cannot mutate A2 or VSA persistent state;
- rejected VSA stores remain atomic;
- A3 accounting includes VSA persistent, temporary and static storage;
- a synthetic A1/A2/A3 partition passes exact `MatchedDynamicBudget` validation;
- A3 snapshots contain both A2 and VSA persistent state;
- reset clears both mechanisms without changing layouts/seeds;
- a downstream integration test consumes the public A3 API.

## Deliberately not frozen here

This tranche does not freeze:

- recurrent input/state dimensions;
- recurrent matrices or bias;
- associative slot count or payload width;
- associative projection seed or fusion gain;
- VSA workspace width or role seed;
- VSA fusion-gain value;
- task encoding or VSA storage policy;
- A1/A2/A3 matched experimental budget;
- T1/T2/T3 horizon strata;
- train/development/validation/final seed ranges;
- sample counts;
- paired interval implementation, seed or replicate count;
- rejection taxonomy for the evaluator;
- any TDI-8.2 runner, token, seed range or result surface.

## Scientific boundary

This is software-reference infrastructure only. It does not claim that A3 is
better than A2, that ASSR-H is novel, that the mechanism replaces Transformers
or Mamba, or that it has favorable runtime, memory, GPU, energy or language
model performance.
