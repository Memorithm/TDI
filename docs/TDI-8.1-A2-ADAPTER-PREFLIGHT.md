# TDI-8.1 A2 symbolic-adapter preflight

Status: bounded software qualification only; **not H8-A evidence and not a final TDI-8.1 architecture/configuration freeze**.

## Purpose

The merged A2 reference exposes deterministic lookup-before-write recurrent + associative-memory semantics, but `SymbolicTaskAdapter` does not itself define which memory key should be read or written for each symbolic event. That mapping is architecture semantics and must therefore be explicit and reviewable.

This tranche qualifies one bounded A2 event policy after the A0/A1 adapter preflight. A3/VSA remains out of scope.

## Candidate event policy

The adapter is constructed for one immutable task instance with a deterministic **neutral read key** produced by `distractor_read_key_for_instance`. That helper guarantees the neutral logical key is absent from the complete set of association/payload write keys for the instance.

The policy is:

- association: encode `(key, value)`, read the neutral key, then write the fused recurrent state under `association_memory_key(key)`;
- payload: encode the value, derive the next payload key solely from chronological payload calls, read the neutral key, then write under that payload key;
- distractor: encode the distractor, read the neutral key, perform no write;
- association query: encode only the queried key, read `association_memory_key(key)`, perform no write, then apply the exact recurrent readout;
- payload query: encode only the requested position, read `payload_memory_key(position)`, perform no write, then apply the exact recurrent readout.

A neutral read may project onto an occupied physical slot, but because its tag was never written it can only be `Empty` or `CollisionMiss`; a neutral `Hit` is a fail-closed adapter error. This prevents a nominal write/distractor event from silently becoming a successful memory retrieval.

## Why writes do not read their own key

`A2Reference::step` requires a `read_key` on every event. Passing the same key for both read and write would create implicit read-modify-write behavior when a key is repeated. That is not implied by the symbolic association/payload event contract.

The neutral-read policy keeps the roles separate:

- non-query events can update recurrent state and optionally write memory;
- query events alone may retrieve a resident association.

Any future alternative policy must be separately reviewed rather than inferred from this fixture.

## Physical diagnostics

The preflight uses two independent views of the direct-mapped table:

1. runner-side `audit_associative_projection`, which replays only logical writes/queries against the concrete layout/projection;
2. runtime `A2StepReport` values emitted by the actual adapter execution.

The bounded collision-free fixture requires agreement on:

- zero physical replacement writes;
- zero query collision misses;
- zero empty query reads;
- one physical query hit per declared query.

Generator-side T3 collision classes are never supplied to the adapter and are not used as a proxy for these physical measurements.

## Software-oracle retrieval fixture

The fixture recurrent parameters are deliberately artificial. For T1 association frames they copy only the two exact encoded **value** limbs into the two recurrent-state coordinates. Query frames contribute zero to those coordinates. With associative fusion gain `1.0`, a memory hit therefore restores the exact stored symbol into recurrent state, allowing the already-qualified exact readout to recover the target.

This proves the complete `encoding -> recurrent write -> bounded associative table -> query read/fusion -> exact readout -> TaskPrediction` path. It is not a learned model and establishes no comparative A2 advantage.

## Deliberately not selected

This tranche does **not** freeze:

- final table capacity or projection seed;
- final recurrent state width, parameters or readout coordinates;
- matched A1/A2/A3 dynamic-memory budget;
- T2 retrieval quality or a universal task-family parameterization;
- collision policy alternatives beyond the existing direct-mapped oracle;
- A3 VSA store/read/fusion policy;
- horizons, populations or train/development/validation seed ranges;
- late retrieval deficit or operation-count formulas;
- final interval replicate count/seed/promotion;
- intervention/recovery sites;
- any TDI-8.2 runner, token, seed range or result surface.

## Qualification target

The tranche passes only if the bounded T1 fixture is collision-free under the explicitly supplied software-fixture layout/seed, every declared query is a physical memory hit, exact target predictions are recovered, non-query neutral reads never hit, and the standard TDI-8 integrity gates remain green.
