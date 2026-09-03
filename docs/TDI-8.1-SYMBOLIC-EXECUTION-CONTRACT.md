# TDI-8.1 symbolic execution contract

Status: bounded evaluator infrastructure; **not a vector-encoding freeze and not H8-A/H8-B evidence**.

## Purpose

TDI-8.0 requires the same generator-owned symbolic task instance and exact target to be used across A0/A1/A2/A3. The merged task generators deliberately stop before architecture-specific binary64 encoding. This tranche defines the execution boundary between those two layers without selecting the missing encoding policy.

## Leakage boundary

`tdi-ai::task_execution::SymbolicTaskAdapter` does not receive every field stored in `TaskEvent`.

The adapter may receive only task stimuli:

- association: stable key code + symbolic value;
- T2 payload: symbolic value, with order conveyed by event order;
- distractor: symbolic token;
- association query: stable key code;
- T2 query: requested payload position.

The runner retains and never supplies to the adapter:

- exact query target;
- association `source_index`;
- generator-side `TaskKey::collision_class()` metadata.

This matters especially for T3: the generator-side collision class is a stress annotation, not evidence of an actual A2/A3 address collision and not an input feature that an arm may exploit.

## Runner guarantees

`execute_symbolic_task`:

- resets the adapter before every generator-level instance;
- captures one fixed `ReferenceArm` identity and rejects mid-instance arm drift;
- processes the immutable event slice in exact source order;
- invokes one typed adapter method per event;
- constructs query records only after the adapter returns its prediction;
- copies target/provenance fields directly from the source `TaskInstance` into private immutable record fields;
- fails closed with the exact source event index on adapter failure;
- verifies the processed query count equals the generator-owned declaration;
- reports exact discrete query successes without defining a late-retrieval deficit.

## Deliberately not frozen here

This tranche does not select:

- binary64 key/value/input encoding;
- recurrent dimensions or parameters;
- A0 key/value widths;
- A2/A3 table layout, projection seed or fusion gain;
- A3 VSA width, role seed or fusion gain;
- matched dynamic-memory budget;
- horizon values or population sizes;
- late-retrieval deficit function;
- operation-count formulas;
- paired interval construction, replicate count or resampling seed;
- intervention site or recovery metric;
- any TDI-8.2 runner, token, seed range or result surface.

## Next bounded step

Concrete A0/A1/A2/A3 adapters can now be implemented behind this interface only after one explicit non-final TDI-8.1 encoding/configuration contract is reviewed. That next contract must keep the task target and generator-only annotations outside the arm-visible surface and must report observed A2/A3 addressing/collision behavior rather than inferring it from generator collision classes.
