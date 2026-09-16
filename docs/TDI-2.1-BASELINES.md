# TDI-2.1 — Baseline contract

Date frozen: 2026-09-16

## B0 — No reusable experience

Uses the same current observation `x`, context `g`, output space, fitting population and comparable parameter budget as the intuition candidate, but cannot retrieve a persistent pattern store.

## B1 — Invalid experience control

Uses the same retrieval machinery and capacity as `I1`, but the experience-to-target association is shuffled within preregistered strata or replaced by structurally unrelated Development experience. This tests whether any gain depends on informative experience rather than added capacity alone.

## B2 — Nearest prototype

A simple deterministic nearest-prototype/centroid rule using the same representation inputs and capacity accounting. This prevents a complex associative mechanism from receiving credit for a benefit obtainable by elementary similarity search.

## B3 — Linear matched predictor

A regularized linear predictor using all permitted instantaneous features. Hyperparameters are frozen from Development/Validation only.

## S2 — Deliberate reference

A separately specified higher-compute reference allowed to perform iterative analysis. `S2` is not automatically the ground truth; its errors are measured against the same protected targets.

## Fairness constraints

For every comparison, record:

- training cases and split access;
- feature inputs;
- parameter/storage budget;
- preprocessing and normalization;
- target access policy;
- number of optimization updates;
- random seeds;
- inference resource accounting.

If budgets cannot be matched exactly, report the difference and prohibit efficiency or superiority wording that depends on the mismatch.

## Success interpretation

`I1 > B0` supports an experience increment only if `B1` and elementary controls do not explain the effect. Performance parity with `B2` is scientifically informative and must not be reframed as a positive associative-memory result.
