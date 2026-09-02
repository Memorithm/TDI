# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: active — reference foundation merged, evaluator incomplete
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- TDI-8.1 foundation merge: `7cfe1b66e11f1eb5d67a5890b07e6f41fa175670` (PR #89)
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

## Next permitted implementation work

The next bounded TDI-8.1 tranche may implement deterministic associative-memory
reference semantics, including explicit address, write, read, collision and
replacement behavior with exact metadata accounting and adversarial oracle
tests.

It must not create any TDI-8.2 runner, token, seed range or result surface.
Concrete experimental dimensions, budgets, horizons, non-final seed ranges,
sample counts, paired interval implementation/replicate count and typed
rejection taxonomy remain to be frozen during TDI-8.1 before any TDI-8.2 stage
can exist.
