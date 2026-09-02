# TDI-8.1 — Reference architecture foundation

Status: **BOUNDED IMPLEMENTATION — NON-FINAL DATA ONLY**

Parent TDI-8.0 merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`

Frozen preregistration blob:
`fe80e7053d89824a77ef6790794f6930d1b424e2`

## Purpose

This first TDI-8.1 implementation tranche establishes the common typed
reference vocabulary required by later A0/A1/A2/A3 evaluator code. It does not
run an experiment and does not select any TDI-8.1 scientific hyperparameter.

The implementation introduces:

- the frozen A0/A1/A2/A3 architecture identifiers;
- exact `u128` bit accounting through `StorageBits`;
- separate accounting for recurrent state, associative payload, associative
  metadata, VSA workspace, temporary work, A0 history and static parameters;
- arm-specific fail-closed validation of components that are not permitted by
  the frozen architecture ladder;
- exact validation that A1/A2/A3 use the same budgeted dynamic-memory total;
- framework-independent reference snapshots carrying arm identity and memory
  accounting.

## Budget interpretation

For the primary matched A1/A2/A3 comparison, `budgeted_dynamic_bits` is the
exact sum of:

1. recurrent-state bits;
2. associative-memory payload bits;
3. associative metadata/addressing bits;
4. VSA workspace bits.

Temporary working storage is reported separately. A0 cumulative history is
reported separately. Static parameters/constants are reported separately.
This follows the component separation and A1/A2/A3 partition rule frozen by
TDI-8.0; it does not imply a runtime, bandwidth or total-process-memory claim.

## Deliberately not frozen by this tranche

The integer values used by unit tests are synthetic accounting fixtures only.
They are **not** experimental choices.

This tranche does not freeze or introduce:

- recurrent-state dimension;
- associative-memory slot count or payload width;
- address projection, collision or replacement policy;
- VSA dimension or binding representation;
- short/medium/long horizon values;
- train/development/validation seed ranges;
- sample counts;
- paired interval implementation or replicate count;
- final rejection taxonomy;
- any TDI-8.2 seed, token, runner or result surface.

Those TDI-8.1 choices require separate bounded implementation/validation and
must be frozen before TDI-8.2 can exist.

## Scientific boundary

This code is reference infrastructure. Passing its unit tests establishes only
that the typed accounting contracts behave as specified. It does not establish
that A2 improves on A1, that A3 improves on A2, or that ASSR/ASSR-H improves on
any external architecture.
