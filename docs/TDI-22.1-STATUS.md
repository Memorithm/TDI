# TDI-22.1 — Status

Status: **preregistration candidate; not frozen; TDI-22.2 execution blocked**.

## Present in this stage

- T0–T4 arm semantics are declared.
- T3 versus T4 is the critical torsor-specific contrast.
- F1 transport-consistent, F2 generic-bilinear and F3 position-nuisance task families are declared.
- deterministic geometry families G0/G1/G2/G3 are named, with learned geometry deferred;
- resource-accounting fields and typed rejection categories are declared;
- development and validation seed-domain labels are separated;
- final/confirmatory material remains absent and unauthorized.

## Still unresolved and execution-blocking

TDI-22.2 must not run until a later TDI-22.1 freeze resolves at least:

- concrete episode counts and per-episode query/candidate bounds;
- maximum coordinate/component magnitudes;
- exact G2 helix constants;
- exact seed-derivation/hash/PRNG algorithm;
- exact T2 `alpha` or an explicit decision to defer T2 from the first execution;
- exact value-aggregation weighting rule for T1;
- tolerance policy for value reconstruction and floating score comparisons;
- exact development/validation population sizes;
- canonical serialization/provenance schema.

No agent may fill these fields at evaluator runtime by fallback/default.

## Forbidden at this point

- TDI-22.2 evaluator or result execution;
- final/confirmatory runner, seeds, dataset or result payload;
- learned latent geometry;
- modification of FLAT-ATTENTION runtime/kernel semantics;
- production memory/performance claims.
