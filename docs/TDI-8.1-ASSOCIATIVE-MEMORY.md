# TDI-8.1 — Deterministic bounded associative-memory reference

Status: **BOUNDED IMPLEMENTATION — NON-FINAL DATA ONLY**

Parent TDI-8.0 preregistration blob:
`fe80e7053d89824a77ef6790794f6930d1b424e2`

## Purpose

This TDI-8.1 tranche implements an explicit bounded associative-memory software
oracle for the A2/A3 reference mechanisms. It establishes deterministic table
semantics that can be tested independently before recurrent-state fusion or any
mechanistic experiment is run.

It does not establish that A2 improves on A1 and does not freeze experimental
slot counts, payload widths or projection seeds.

## Reference table semantics

The current reference primitive is a direct-mapped bounded table:

1. each association has an exact `u64` key tag and a fixed-width binary64
   payload;
2. the address is produced deterministically by a fixed SplitMix64 integer
   mixer over `key + projection_seed`, followed by modulo `slot_count`;
3. a read from an empty projected slot returns `Empty`;
4. a read whose resident tag matches the requested key returns `Hit` and the
   stored payload;
5. a read whose projected slot contains a different tag returns the observable
   `CollisionMiss` state;
6. a write to an empty slot returns `Inserted`;
7. a write of the resident key replaces its payload in place and returns
   `Updated`;
8. a write whose projected slot contains another key deterministically replaces
   that resident association and returns `ReplacedCollision` with the evicted
   key tag;
9. reads never mutate the table;
10. the complete payload is width-checked and finite-checked before any write
    mutation, so a rejected write cannot partially modify memory.

This policy is deliberately simple enough to act as a transparent software
oracle. Retaining it as the primary TDI-8.1 evaluator policy, or replacing it
with another deterministic policy, requires reviewed bounded evidence before
any TDI-8.2 surface can exist.

## Exact declared storage accounting

For `S` slots and payload width `W`, the primitive reports separately:

- payload: `S * W * 64` bits;
- per-slot metadata: 8 occupancy bits + 64 key-tag bits;
- layout metadata: two explicit `u64` dimensions = 128 bits;
- static projection constants: one `u64` seed plus three fixed `u64` mixing
  constants = 256 bits.

The payload and associative metadata are the quantities available to later A2/A3
`MemoryAccounting` construction. Projection constants are reported separately
as static parameter/constant bits.

These are architecture-state representation quantities. Rust allocator and
container headers are implementation overhead and are not turned into a runtime
or hardware-memory claim. Any later real-device memory/performance statement
requires separate measurement.

## Numerical and fail-closed rules

- address projection uses exact wrapping integer arithmetic;
- integer indexing and tag comparison are exact;
- stored payload values are IEEE-754 binary64;
- a non-finite payload is rejected before mutation;
- zero slot count and zero payload width are rejected;
- host-index and accounting overflow conditions fail closed;
- every backing vector is byte-capacity checked against the host `Vec` limit
  before any allocation is attempted;
- backing-vector allocation uses fallible exact reservation, so an allocator
  refusal is returned as a typed error instead of intentionally relying on an
  infallible `vec!` capacity path.

## Deliberately not frozen by this tranche

The values used in unit tests are synthetic oracle fixtures only. This tranche
does not freeze:

- experimental slot count;
- experimental payload width;
- the projection seed used by train/development/validation runs;
- recurrent-state dimension;
- the A2/A3 recurrent-memory fusion gate;
- VSA dimension or VSA operations;
- task horizon values;
- non-final seed ranges;
- sample counts;
- paired interval implementation or replicate count;
- typed final rejection taxonomy;
- any TDI-8.2 seed, runner, token or result surface.

## Required oracle tests

The merged primitive must retain tests proving:

- oversized vector capacities fail closed before allocation;
- deterministic bounded address projection;
- empty → insert → hit behavior;
- same-key update behavior;
- deliberate collision replacement and collision-miss observability;
- no partial mutation after a rejected non-finite write;
- fail-closed payload-width mismatch;
- exact declared payload/metadata/static accounting;
- clear/reset behavior without projection drift.
