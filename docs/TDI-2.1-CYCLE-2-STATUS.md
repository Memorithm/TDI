# TDI-2.1 — implementation cycle 2 status

Date: 2026-09-16

## Scope

This document closes the second autonomous TDI-2.1 cycle. The cycle moved the
preregistered intuition programme from research contracts to an opt-in scalar
Rust reference implementation under `tdi-ai`'s `experimental` feature.

Historical TDI-2 results remain unchanged.

## Merged implementation slices before this status PR

1. #311 — typed intuition state primitives;
2. #312 — validated experiential template IR;
3. #313 — crisp Boolean template matching;
4. #314 — empirical template reliability;
5. #315 — numeric experience weighting;
6. #316 — relational experiential template IR;
7. #317 — role-preserving structural transfer;
8. #318 — bounded read-only experiential memory;
9. #319 — deterministic applicable-candidate ranking;
10. #320 — explicit `InsufficientExperience` inference outcome;
11. #321 — fail-closed invalid-weight hardening;
12. #322 — weighted multi-template numeric aggregation;
13. #323 — explicit post-outcome consolidation records;
14. #324 — canonical inference provenance trace;
15. #325 — deterministic motif/context task fixtures;
16. #326 — B0/B1/B2 matched baselines;
17. #327 — composed scalar intuition reference engine;
18. #328 — cross-module integration tests;
19. #329 — dedicated TDI-2.1 intuition CI gate.

This status PR is slice 20/20.

## Implemented reference path

The current experimental path is:

```text
BooleanState
  -> exact Boolean applicability
  -> bounded ExperienceStore
  -> empirical ReliabilityEvidence
  -> numeric ExperienceWeightPolicy
  -> deterministic candidate ranking
  -> Selected | InsufficientExperience
  -> canonical InferenceTrace
```

Relational templates additionally support a complete injective mapping from
abstract roles to novel concrete entities while preserving relation identity.

Consolidation is explicitly outside inference:

```text
inference -> observed outcome -> validation -> evidence update
```

Generalization, specialization and template creation remain review proposals;
they are not silently applied by the reference engine.

## Algorithmic boundary

Runtime speed is not an input, objective or decision variable of the intuition
algorithm. Latency, throughput, memory traffic and SIMD effects remain later
engineering measurements.

The reference engine performs one bounded control-flow pass, but its work still
depends on the number and size of stored templates. No total `O(1)` complexity
claim is made.

## Scientific status

No TDI-2.1 confirmatory result exists yet.

In particular, this cycle does **not** establish that:

- the implementation is an adequate model of human intuition;
- experience-conditioned templates outperform matched baselines;
- structural transfer generalizes out of distribution;
- the confidence/ranking signals are calibrated;
- consolidation improves continual performance;
- the method is faster than neural, associative-memory or symbolic baselines.

No TDI-2.1 protected holdout was opened during this cycle.

## Qualification status

PR #329 installs a dedicated path-scoped workflow covering formatting, clippy
with warnings denied, all-feature `tdi-ai` tests, and Rust 1.85 all-targets
checking. Workflow completion is evidence to collect after merge; the existence
of the workflow itself is not a green-CI claim.

The repository-wide existing `TDI AI experimental contracts` workflow also
covers `tdi-ai --all-targets --all-features` at the declared experimental MSRV.

## Concurrent PR handling

Before and during this cycle, compatible concurrent TDI work was merged to
minimize branch divergence. PR #296 remains intentionally unmerged because its
own integration contract states that it depends on Memorithm/scirust#1452
passing required CI/review and recording its final source pin. That external
scientific/provenance gate is not bypassed merely to make the TDI PR queue empty.

## Next gate

Before any TDI-2.1 protected holdout is opened:

1. collect green CI evidence for the exact final implementation revision;
2. fix any compiler, clippy or test failure without changing frozen scientific
   criteria;
3. freeze concrete Development/Validation populations from the deterministic
   task families;
4. implement the remaining matched B3/deliberative comparison path if required
   by the preregistration;
5. execute only Development/Validation campaigns;
6. calibrate thresholds exclusively there;
7. record immutable artifacts and hashes;
8. open the PrimaryHoldout exactly once under the frozen holdout policy.

The next cycle therefore begins with qualification and Development/Validation
execution, not with performance optimization or confirmatory claims.
