# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: active — foundation and bounded associative-memory reference implemented; evaluator incomplete
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- TDI-8.1 foundation merge: `7cfe1b66e11f1eb5d67a5890b07e6f41fa175670` (PR #89)
- TDI-8 post-foundation integrity merge: `9ee32002603942b9f4152cad47f7fb59331f8c7a` (PR #90)
- Frozen TDI-8.0 preregistration blob: `fe80e7053d89824a77ef6790794f6930d1b424e2`
- Final holdout: does not exist
- Confirmatory runner: does not exist
- Human confirmation token: does not exist
- TDI-8.2 seed range: does not exist
- TDI-7.2 interaction: forbidden

## Merged foundation

The merged TDI-8.1 foundation provides the typed A0/A1/A2/A3 reference-arm
vocabulary and exact component-wise memory accounting in `tdi-ai`.

It validates:

- non-zero recurrent state for A1/A2/A3;
- non-zero associative payload for A2/A3;
- non-zero VSA workspace for A3;
- exact matched total dynamic-memory accounting for A1/A2/A3;
- inclusion of temporary working storage in that matched budget;
- separate reporting of A0 cumulative history and static parameters;
- fail-closed overflow handling.

These contracts are infrastructure only. They are not evidence that A2 is
better than A1 or that A3 is better than A2.

## Bounded associative-memory reference

The current TDI-8.1 implementation adds a deterministic bounded direct-mapped
associative-memory oracle under `tdi_ai::associative_memory`.

The primitive has explicit and testable:

- integer address projection;
- empty, hit and collision-miss read states;
- insert, same-key update and collision-replacement write states;
- full pre-mutation validation for payload width and finite binary64 values;
- exact declared payload, metadata and static-constant accounting;
- deterministic clear/reset behavior.

The concrete slot counts, payload widths and projection seeds used by its unit
tests are synthetic oracle fixtures and are not frozen experimental choices.

## Next permitted implementation work

The next bounded TDI-8.1 tranche may connect the associative-memory oracle to a
deterministic recurrent reference step for A2, with an explicit retrieval/state
fusion rule and matched accounting against A1. A3 VSA workspace semantics remain
a separate later oracle tranche.

It must not create any TDI-8.2 runner, token, seed range or result surface.
Concrete experimental dimensions, budgets, horizons, non-final seed ranges,
sample counts, paired interval implementation/replicate count and typed
rejection taxonomy remain to be frozen during TDI-8.1 before any TDI-8.2 stage
can exist.
