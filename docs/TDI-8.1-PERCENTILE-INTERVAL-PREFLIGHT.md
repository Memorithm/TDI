# TDI-8.1 conservative percentile interval preflight

This tranche qualifies one deterministic paired-bootstrap interval construction on bounded, non-final software fixtures. It does **not** freeze an experimental replicate count, bootstrap seed, architecture configuration, population, or TDI-8.2 surface.

## Parent constraints

TDI-8.0 already freezes:

- paired generator-level contrasts;
- exactly nine primary task × horizon cells per hypothesis;
- family-wise `alpha = 0.05`;
- Bonferroni allocation across nine cells;
- a two-sided interval consumed by the frozen four-way primary-cell classifier;
- the exact zero-baseline decision branch.

PR #109 supplies deterministic paired bootstrap replicates but deliberately leaves the interval estimator open.

## Candidate construction

For a bootstrap output whose complete-sample baseline mean is positive and for which **every** bootstrap resample also has positive baseline mean:

1. validate exact replicate accounting;
2. require every relative-effect replicate to be finite;
3. sort effects with Rust `f64::total_cmp`;
4. use no interpolation;
5. exploit the exact frozen tail allocation

   `0.05 / 9 / 2 = 1 / 360`;

6. with `n` defined replicates, set `k = floor(n / 360)`;
7. return order statistics `x[k]` and `x[n - 1 - k]`.

For `n < 360`, `k = 0`, so the candidate returns the complete observed min/max bootstrap range rather than inventing sub-replicate interpolation.

This construction is intentionally simple and deterministic. Qualification here is software evidence only; it does not claim bootstrap coverage beyond the protocol's intended percentile interpretation and does not yet make this the final TDI-8.1 interval choice.

## Degenerate bootstrap policy

A relative-effect bootstrap replicate is undefined when its resampled baseline mean is exactly zero. PR #109 records those cases explicitly as `zero_zero_replicates` or `zero_positive_replicates`.

This candidate rejects interval construction if **either count is non-zero**. It does not:

- discard those replicates;
- replace them with finite sentinels;
- condition the interval on only positive-baseline resamples;
- reinterpret a zero/zero resample as an ordinary relative effect.

The complete-sample zero-baseline case also rejects relative interval construction because the already-frozen TDI-8 classifier handles that case directly without division.

The purpose is to prevent a favorable interval from being manufactured by silently deleting undefined bootstrap outcomes.

## Provenance

The candidate output carries through the caller-selected:

- requested replicate count;
- resampling seed;
- number of order statistics dropped per tail.

No defaults are introduced.

## Qualification tests

The bounded software oracle verifies:

- the exact rational tail denominator `360` matches the frozen Bonferroni allocation;
- sorting/order statistics are deterministic and non-interpolated;
- fewer than 360 replicates use the full observed range;
- a constant paired relative effect produces an exact point interval;
- zero-baseline bootstrap replicates fail closed;
- complete-sample zero baseline uses the non-relative branch;
- non-finite effects are rejected defensively.

## Scientific boundary

This tranche does not:

- freeze percentile bootstrap as the final interval estimator;
- choose the bootstrap replicate count or seed;
- execute T1/T2/T3 through A0/A1/A2/A3;
- define recurrent symbolic readout;
- define late retrieval deficit;
- emit any H8-A/H8-B verdict;
- create or access a TDI-8.2 holdout surface;
- access or reinterpret TDI-7.2 evidence.

A later bounded TDI-8.1 decision must compare/qualify this candidate and explicitly freeze the interval method, replicate count, seed, and degenerate policy before any TDI-8.2 protocol can exist.
