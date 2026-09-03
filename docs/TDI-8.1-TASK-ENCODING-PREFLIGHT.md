# TDI-8.1 leakage-safe task-encoding preflight

This bounded tranche implements and tests a concrete lossless binary64 encoding candidate behind the merged leakage-safe symbolic execution contract. It is evaluator-local preflight infrastructure only.

It does not run H8-A/H8-B, select a final population, construct a late-retrieval deficit, choose a paired interval estimator, or create any TDI-8.2 surface.

## Why this replaces PR #111's first design

The first task-adapter design had two structural problems:

1. it added `tdi-ai` as a dependency of `tdi-bench`, changing `tdi-bench/Cargo.toml`, which is protected by the frozen TDI-6.8 scientific-code integrity chain;
2. its recurrent query frames encoded the exact query target and generator provenance fields, which would let an arm bypass delayed retrieval.

The corrected design changes neither `tdi-bench/Cargo.toml` nor `Cargo.lock`. It also sits strictly behind `tdi_ai::task_execution::SymbolicTaskAdapter`, whose method signatures already exclude exact query targets, association source indices, and T3 collision-class annotations.

## Qualification status

The candidate lives in `tdi-ai/src/task_encoding.rs` and is compiled only by the bounded `tdi8-task-encoding-preflight` binary. It is deliberately **not** exported from `tdi-ai/src/lib.rs` yet.

That placement means:

- normal Rust/Clippy/MSRV CI compiles the candidate;
- the dedicated preflight can test it against merged TDI-8.1 primitives;
- downstream crates cannot silently treat the candidate as a frozen public API;
- later bounded development can reject or revise the encoding without pretending a scientific parameter was already frozen.

Promotion into the public library, if justified, is a separate reviewed step after non-final qualification.

## Exact integer encoding

Every `u64` is represented by two finite binary64 coordinates:

- high 32-bit limb divided by `2^32`;
- low 32-bit limb divided by `2^32`.

Both coordinates are exact binary fractions in `[0,1)`. The mapping is injective over the complete `u64` domain and avoids lossy integer-to-`f64` conversion above `2^53`.

The decoder rejects non-finite coordinates, values outside `[0,1)`, and values that are not on the exact `2^-32` limb grid.

## Arm-facing recurrent frame

The encoder mirrors only the merged `SymbolicTaskAdapter` arguments:

- `associate(key_code, value)`;
- `payload(value)`;
- `distractor(token)`;
- `query_association(key_code)`;
- `query_payload(position)`.

The maximum exposed stimulus therefore needs five coordinates:

1. event-kind tag;
2. two exact `u64` limbs for the first arm-visible symbolic argument;
3. two exact limbs for the association value when present.

The minimum candidate width is therefore `5`, not the earlier `9`. Larger caller-selected widths are deterministic zero padding.

Crucially:

- query targets are not parameters of query encoders;
- association source indices are not parameters of any encoder;
- T3 collision classes are not parameters of any encoder;
- T2 payload source positions are not supplied to payload inputs; payload order is reconstructed only from chronological calls.

The requested T2 query position remains visible because it is the symbolic query itself, exactly as required by the merged symbolic execution contract.

## A0 support

A0 uses exact namespaced keys:

- association namespace + exact association key code;
- payload namespace + chronological payload position;
- distractor namespace + exact distractor code.

Exact task values use the same two-limb integer representation. Association and payload query helpers construct only the query key; they receive no exact target.

A future concrete A0 `SymbolicTaskAdapter` can maintain its payload position from call order and use these helpers without receiving hidden generator provenance.

## A2/A3 logical memory keys

Association memory keys use the stable symbolic key code directly.

T2 payload keys are deterministic domain-separated functions of chronological payload position. `PayloadKeyCursor` reconstructs those positions from payload call order, matching the leakage-safe symbolic executor where payload source position is not an arm argument.

A deterministic per-instance distractor read key can be selected outside the complete logical write-key set. This helper is runner-side configuration; it contains no query target.

No associative slot count, projection seed, payload width, fusion gain, VSA seed, or matched dynamic-memory budget is selected here.

## Physical collision audit

`audit_associative_projection` is a runner-side diagnostic over an immutable generated task and one concrete `DirectMappedAssociativeMemory::address_for` rule.

It records separately:

- planned logical writes;
- distinct occupied physical addresses;
- physical replacement collisions;
- query hits;
- query collision misses;
- query reads into empty addresses;
- generator-side T3 collision-class reuse;
- physical replacements where the replaced and replacing associations also share a generator collision class.

Generator collision metadata is used only for this analysis record. It is never inserted into an arm-facing binary64 frame.

The audit does not mutate associative payloads, recurrent state, or VSA state and therefore cannot manufacture task success or an H8 verdict.

## What is not frozen

This preflight does not freeze:

- a production/public encoding API;
- recurrent state width;
- concrete recurrent input width above the lossless minimum;
- A1/A2/A3 recurrent parameters;
- associative table capacity or projection seed;
- fusion gain;
- VSA width or seed;
- matched A1/A2/A3 dynamic-memory budget;
- short/medium/long numeric horizons;
- train/development/validation/final seed ranges;
- generator sample counts;
- recurrent output/readout semantics;
- late-retrieval deficit construction;
- operation accounting taxonomy;
- paired interval method, replicate count, or seed;
- a TDI-8.2 runner, token, seed range, holdout, or result.

## Next boundary

If this encoding survives bounded qualification, the next implementation step is to build concrete A0/A1/A2/A3 `SymbolicTaskAdapter` implementations around it. That step still requires explicit recurrent-arm readout semantics and exact per-generator operation/provenance accounting before any primary-cell evidence can be produced.
