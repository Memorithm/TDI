# TDI-8.1 symbolic execution contract

Status: bounded evaluator infrastructure; **not a vector-encoding freeze and not H8-A/H8-B evidence**.

## Purpose

TDI-8.0 requires the same generator-owned symbolic task instance and exact target to be used across A0/A1/A2/A3. The merged task generators deliberately stop before architecture-specific binary64 encoding. This tranche defines the execution boundary between those two layers without selecting the missing concrete architecture configuration.

The executor also separates two cases that must not be conflated statistically:

1. a **technical adapter failure**, where the event could not be executed and the executor fails closed with the exact source event index;
2. an **invalid symbolic prediction**, where the query completed but the arm did not produce one valid exact symbol. This remains an evaluated query and is counted as a failure.

This distinction prevents apparent quality from being inflated by conditioning evaluation on only outputs that happen to be decodable.

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

## Evaluable prediction contract

Query methods return `Result<TaskPrediction, AdapterError>`.

`TaskPrediction` has exactly two evaluator-level states:

- `Symbol(TaskSymbol)`: one valid exact symbolic prediction;
- `Invalid`: the query completed technically but no valid exact symbol was produced.

`Invalid` is not an adapter error. It produces a normal `TaskQueryRecord`, contributes to the declared query count, increments `failed_queries()` and `invalid_predictions()`, and makes `all_queries_exact()` false.

This is required for the exact recurrent-state readout qualified by PR #114: a finite, correctly shaped recurrent state with non-canonical designated symbol coordinates must map to an invalid evaluated prediction rather than disappear through an error/rejection path.

Technical failures remain `TaskExecutionError::AdapterReset` or `TaskExecutionError::AdapterEvent` and preserve their exact source event location. These errors represent failure to execute the protocol itself, not model-quality outcomes.

## Runner guarantees

`execute_symbolic_task`:

- resets the adapter before every generator-level instance;
- captures one fixed `ReferenceArm` identity and rejects mid-instance arm drift;
- processes the immutable event slice in exact source order;
- invokes one typed adapter method per event;
- constructs one query record after every technically completed query, including `TaskPrediction::Invalid`;
- copies target/provenance fields directly from the source `TaskInstance` into private immutable record fields;
- fails closed with the exact source event index on technical adapter failure;
- verifies the processed query count equals the generator-owned declaration;
- reports exact discrete query successes, failures and explicit invalid-prediction counts without defining a late-retrieval deficit.

## Deliberately not frozen here

This tranche does not select:

- binary64 key/value/input encoding beyond the separately qualified candidate;
- recurrent dimensions or parameters;
- exact recurrent readout coordinates;
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

Concrete A0/A1/A2/A3 adapters may now be implemented only if they preserve this distinction between model-quality output and technical execution failure. In particular, the recurrent exact-readout candidate must map its valid exact state output to `TaskPrediction::Symbol` and its finite non-canonical output to `TaskPrediction::Invalid`.

The concrete adapter tranche must also keep task targets and generator-only annotations outside the arm-visible surface, preserve exact event order, account all state/static/temporary resources, and report observed A2/A3 addressing/collision behavior rather than inferring it from generator collision classes.
