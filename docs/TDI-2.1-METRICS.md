# TDI-2.1 — Metric contract

Date frozen: 2026-09-16

## Task quality

Each task family declares one primary loss before holdout opening:

- classification: error rate or log loss;
- bounded regression: mean squared error;
- ranking/retrieval: preregistered top-k or rank loss;
- structured tasks: exact task-specific loss defined by the generator.

Secondary metrics may be reported but cannot replace the frozen primary loss after results are known.

## Experience increment

For paired case-level losses `L_B0` and `L_I1`:

`delta_experience = mean(L_B0 - L_I1)`.

Positive values favor the intuition candidate. Relative reductions are reported only when the baseline denominator is well-defined and not near zero.

## Specificity

Compare `I1` to `B1` with the same paired loss difference. A gain over `B0` without a gain over invalid/shuffled experience is insufficient evidence that informative experience caused the effect.

## Confidence/routing

Report calibration and risk-coverage metrics defined in the confidence contract, plus fast-path coverage and slow-path call rate.

## Hybrid quality

Report separately:

- raw primary task loss;
- hybrid utility;
- error among fast accepted cases;
- error among slow-routed cases;
- abstention rate if enabled.

## Consolidation

Report forward loss, replay-audit loss and forgetting as frozen in the consolidation contract.

## Transfer

Report absolute target loss and paired improvement over each matched baseline. Relative advantage does not imply acceptable absolute transfer quality.

## Runtime engineering metrics

Latency, throughput, logical bytes, peak resident memory, CPU cycles, cache events and energy estimates are secondary engineering metrics. They cannot change a scientific pass/fail decision unless an explicit utility contract includes them before holdout opening.
