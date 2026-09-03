# TDI-8.1 paired-resampling foundation

Status: bounded software foundation; **not an interval-method freeze**.

## Frozen parent requirements

TDI-8.0 requires the H8-A and H8-B primary contrasts to use paired generator-level observations. For the exactly nine primary cells in each hypothesis, two-sided intervals must later provide at least 95% family-wise coverage through the frozen Bonferroni allocation `alpha = 0.05 / 9` per cell.

TDI-8.0 deliberately does not freeze the concrete resampling/interval implementation, replicate count, or deterministic resampling seed. Those values must be selected and frozen in TDI-8.1 using only non-final data before any TDI-8.2 surface exists.

## This tranche

`tdi-bench::paired_resampling_v8` adds only the reusable deterministic substrate needed before that later choice:

- validated finite, non-negative baseline/candidate deficit pairs;
- the frozen relative mean-deficit point statistic `R=(B-C)/B` when `B > 0`;
- the exact zero-baseline branch without division by zero;
- caller-supplied replicate count and seed with no defaults;
- deterministic SplitMix64 index generation with rejection sampling rather than modulo-biased bounded draws;
- bootstrap draws that resample baseline and candidate deficits together as indivisible pairs;
- fixed-order finite accumulation checks;
- explicit counts for `B == 0 && C == 0` and `B == 0 && C > 0` bootstrap replicates;
- exact accounting proving every requested replicate lands in one output category;
- public access to the frozen family alpha, per-cell Bonferroni alpha, and corresponding two-sided tail probability.

The replicate relative effects are intentionally returned unsorted. No percentile, basic, studentized, BCa, normal-approximation, or other interval construction is selected by this tranche.

## Scientific boundary

This work does not:

- choose or freeze a TDI-8.1 interval method;
- choose or freeze a replicate count or resampling seed;
- choose dimensions, budgets, horizons, population sizes, or seed ranges;
- execute A0/A1/A2/A3 on T1/T2/T3;
- produce an H8-A or H8-B scientific result;
- create a TDI-8.2 runner, token, seed range, result record, or authorization surface;
- access or reinterpret TDI-7.2.

A later TDI-8.1 non-final qualification tranche must compare admissible interval constructions, select one explicit deterministic procedure, freeze its replicate count/seed and degenerate-replicate policy, and then connect the resulting interval to the already-frozen `decision_v8` classifier.
