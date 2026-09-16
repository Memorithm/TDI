# TDI-2.1 — Statistical analysis plan

Date frozen: 2026-09-16

## Unit of analysis

The primary unit is the held-out case. Comparisons between arms use paired case-level outcomes whenever the same case is scored by both arms.

## H1 primary test

Let `d_i = L_B0(i) - L_I1(i)`. Report:

- mean and median `d_i`;
- a deterministic paired bootstrap 95% interval for the mean difference;
- the fraction of cases improved/tied/worsened;
- standardized effect size where meaningful.

H1 requires a positive mean improvement and a strictly positive lower 95% bootstrap bound on PrimaryHoldout.

## H2–H7

Secondary hypotheses use the corresponding paired contrasts. They are reported separately and are not pooled into a single score.

## Bootstrap

- 10,000 paired replicates for confirmatory analyses;
- fixed seed `0x5444_4932_494E_5455`;
- resample case indices with replacement;
- preserve paired arm observations within each sampled index;
- percentile interval unless a different method is frozen before holdout opening.

## Stratification

Report prespecified task-family and ambiguity strata descriptively. Subgroup intervals are secondary unless explicitly promoted before holdout opening.

## Multiple comparisons

H1 is the sole primary hypothesis. Secondary p-values, if computed, are descriptive and must not be used to retroactively redefine the programme as successful. Confidence intervals and effect sizes remain the principal reporting tools.

## Missing/rejected cases

A case rejected by one arm is not silently dropped from paired analysis. The protocol must either assign the preregistered failure loss or report a sensitivity analysis with rejection counts. The choice is frozen before execution.

## Transfer

TransferHoldout results are confirmatory for direction and magnitude but are interpreted with absolute loss and calibration. A positive paired difference alone cannot establish acceptable transfer.
