# TDI-8.1 A3 atomic store gate

Status: **bounded implementation guard — no A3 task policy selected**

Tracks issue #132 and the frozen TDI-8.0 programme.

## Why this gate exists

The merged A3 reference deliberately exposes two individually fail-closed operations:

- `A3Reference::step_routed(...)`, which advances the embedded A2 recurrent/associative state;
- `A3Reference::store_vsa(...)`, which mutates only the bounded VSA workspace.

PR #127 correctly separated VSA-read routing from A2 associative-read routing, but it did not create an event-level transaction spanning an A2 transition and a VSA store.

A concrete task adapter must not implement a write event as a naive two-call sequence. `step_routed` followed by `store_vsa` can leave A2 committed if the VSA store rejects. Reversing the order can leave VSA committed if the A2 transition rejects.

Cloning the complete A3 state for rollback is also not an acceptable hidden default: it duplicates recurrent state, the associative table and the VSA workspace and would change the actual temporary-memory/operation cost of the reference path unless explicitly accounted.

## Required property before concrete A3 adapter qualification

Before a concrete A3 `SymbolicTaskAdapter` is admitted, the reference layer must expose a reviewed event-level mechanism with these properties:

1. the prospective VSA bundle is fully prepared and validated before persistent mutation;
2. the A2 transition executes only after preparation succeeds;
3. an A2 failure leaves persistent VSA state unchanged;
4. a successful A2 transition is followed by an infallible commit of the already-prepared VSA state;
5. a store event does not simultaneously require a second keyed VSA-read temporary unless the larger simultaneous peak is explicitly accounted;
6. regression tests prove both failure directions cannot create a partially committed A3 event.

A natural bounded implementation is a prepare/commit VSA bundle path plus an A3 store-event transition that uses `A3VsaReadRoute::Skip` for the same event. The exact API name is not frozen by this document.

## Candidate semantics deliberately not selected here

This gate does **not** choose:

- what payload is stored in VSA;
- which task events store;
- role/key mapping;
- query cleanup/similarity policy;
- recurrent/VSA dimensions;
- associative capacity or projection seed;
- VSA seed or fusion gain;
- matched memory budget;
- horizons, populations, deficits, intervals or intervention sites.

Those remain later bounded TDI-8.1 decisions. No H8-B evidence is produced.

## Machine enforcement

`scripts/check-tdi8.1-a3-atomic-store-gate.sh` permits the current state with no concrete A3 adapter. If a future branch introduces the canonical `A3Adapter`/A3-adapter preflight surface, the gate requires a reviewed atomic combined-store primitive and a VSA prepare/commit substrate to be present first.

The gate is intentionally conservative. A materially different correct implementation may update the structural matcher in the same reviewed PR, but it may not remove the cross-mechanism atomicity requirement.

## Holdout boundary

- TDI-8.2 executable: absent;
- TDI-8.2 seed/result surface: absent;
- final holdout: does not exist;
- TDI-7.2 interaction: forbidden.
