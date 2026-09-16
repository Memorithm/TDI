# TDI-2.1 — Cycle 1 status

Date: 2026-09-16
Status: preregistration foundation complete; implementation not yet qualified.

## Completed in cycle 1

The first autonomous TDI-2.1 cycle established the scientific control plane for experience-conditioned intuition:

1. programme and boundaries;
2. reference register;
3. frozen hypotheses;
4. operational definitions;
5. data/split contract;
6. task families;
7. experience-store contract;
8. matched baselines;
9. confidence/calibration contract;
10. fast/slow arbitration contract;
11. online consolidation protocol;
12. ablation matrix;
13. holdout opening policy;
14. metric contract;
15. statistical analysis plan;
16. falsification ledger;
17. reproducibility contract;
18. resource accounting;
19. implementation gate.

These artifacts were merged through the cycle's preceding PRs: #289, #290, #291, #292, #293, #294, #295, #297, #298, #299, #300, #301, #302, #303, #304, #305, #306, #307 and #308.

## What has *not* been established

No TDI-2.1 scientific result exists yet. In particular, this cycle does not establish:

- that the intuition candidate beats matched baselines;
- that entropy is calibrated confidence;
- that associative memory is the best representation;
- that fast/slow routing improves utility;
- that online consolidation is stable;
- that transfer succeeds;
- that SIMD/alignment improves performance;
- any biological equivalence or novelty claim.

## Next implementation cycle

The next autonomous cycle starts with Gate A from `TDI-2.1-IMPLEMENTATION-GATE.md`:

1. audit reusable `tdi-ai::associative_memory` and adaptive-inference primitives before adding new code;
2. implement the deterministic scalar TDI-2.1 reference with fail-closed validation;
3. implement capacity-matched valid/shuffled/unrelated experience stores;
4. add F1/F2 task generators and B0/B1/B2 controls;
5. add calibration/risk-coverage metrics without routing authority;
6. add Validation-only threshold serialization and router;
7. add deterministic evidence/provenance records;
8. qualify on Development/Validation only;
9. keep PrimaryHoldout closed until every gate is green.

## Stop condition

Cycle 1 is complete when this status artifact itself is merged. The next cycle may begin from `main` without changing the frozen cycle-1 hypotheses unless a new TDI-2.xx series is deliberately opened.
