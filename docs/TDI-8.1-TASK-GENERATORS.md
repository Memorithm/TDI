# TDI-8.1 — deterministic symbolic task generators

Status: **BOUNDED SOFTWARE-ORACLE TRANCHE — NO TDI-8.2 SURFACE**

This tranche implements the frozen TDI-8.0 task-family vocabulary without yet choosing architecture-specific binary64 encodings or concrete experimental dimensions.

## 1. Scientific boundary

The generator layer is deliberately symbolic. A task instance is created once before any architecture executes and can then be consumed identically by A0, A1, A2 and A3 through a later frozen adapter.

This preserves the TDI-8.0 rule that queried keys and targets are generated before execution and are identical across arms.

This tranche does **not** freeze:

- concrete short/medium/long horizon values;
- recurrent, associative-memory, VSA or A0 key/value dimensions;
- matched dynamic-memory budgets;
- architecture-specific symbol encodings;
- train/development/validation/final seed ranges;
- population sizes or sample counts;
- interval replicate count or resampling seed;
- final rejection taxonomy;
- any TDI-8.2 runner, confirmation token, seed range or result surface.

Synthetic unit-test values are software-oracle fixtures only.

## 2. Horizon vocabulary

`HorizonStratum` exposes exactly the frozen labels:

- `Short`;
- `Medium`;
- `Long`.

`HorizonPlan` accepts three caller-supplied positive values and requires `short < medium < long`. It intentionally has no defaults and therefore does not silently turn unit-test values into scientific choices.

Later bounded development work must select and freeze the concrete horizon values before any TDI-8.2 surface can exist.

## 3. T1 — associative recall

A T1 instance contains:

1. at least two deterministic symbolic key/value associations;
2. at least one association that is not queried, providing a distractor association;
3. a strictly positive sequence of irrelevant distractor events;
4. one or more delayed keyed queries whose exact targets were generated with the original associations.

The queried subset is a deterministic seed-selected cyclic window. Because `query_count < association_count`, at least one introduced association remains unqueried.

T1 keys come from a domain-separated SplitMix64 stream. The internal state advances by an odd additive constant modulo `2^64`, and the output mixing operations are bijective over `u64`; therefore distinct stream positions remain distinct until the full period is exhausted. This avoids accidental duplicate-key overwrite semantics in the bounded host-representable task sizes.

## 4. T2 — delayed copy

A T2 instance contains:

1. a non-empty ordered symbolic payload;
2. a strictly positive irrelevant-input delay;
3. ordered payload queries whose targets exactly equal the corresponding original payload symbols.

The generator preserves payload order explicitly through stable positions.

## 5. T3 — interference recall

A T3 instance contains multiple competing associations followed by delayed queries that always include both:

- the oldest association (`source_index = 0`);
- the most recent association (`source_index = association_count - 1`).

Every T3 key shares a configured high-bit prefix. Its low suffix is produced by an odd affine permutation over the available suffix space, giving exact distinct key codes whenever the validated configuration fits that space.

The generator also assigns reused `collision_class` labels to create explicit interference groups.

### Collision-class limitation

`collision_class` is generator-side metadata only. It is **not** evidence that the concrete A2/A3 direct-mapped projection places equal-class keys in the same physical slot. The later arm adapter/evaluator must explicitly measure or construct actual occupancy/collision pressure under its frozen associative layout and projection seed. A generator-side class must never be reported as a measured associative-memory collision.

## 6. Determinism and allocation discipline

All task randomness is deterministic and caller-seeded. Independent semantic domains use distinct SplitMix64 streams/constants so changes to one generated field do not implicitly consume another field's stream.

The generator validates platform-independent counts before host conversion, checks event-count overflow, checks vector byte capacity against `isize::MAX`, and uses fallible exact reservation. Invalid configurations fail closed before task execution.

## 7. Next bounded tranche

After this symbolic layer is merged, TDI-8.1 still needs an architecture adapter/evaluator layer that:

- freezes one explicit mapping from symbolic events into deterministic binary64 arm inputs;
- gives all four arms the same generated task instance and target;
- verifies actual A2/A3 associative occupancy/collision behavior rather than trusting symbolic classes;
- records exact task success/deficit and rejected-record reasons;
- keeps A1/A2/A3 under the same declared dynamic-memory budget;
- leaves concrete horizons, budgets and population decisions to bounded development until those choices are separately frozen.

No scientific H8-A/H8-B verdict is produced by the generator layer itself.
