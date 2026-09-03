# TDI-8.1 — frozen primary decision rules

Status: **SOFTWARE TRANSCRIPTION OF FROZEN TDI-8.0 RULES — NO SCIENTIFIC RESULT**

This tranche implements the TDI-8.0 primary-cell classifier and nine-cell hypothesis aggregation gate exactly as preregistered. It does not estimate uncertainty and does not access any holdout.

## Frozen constants

- primary cells per hypothesis: exactly 9;
- equivalence/decision margin: `delta = 0.02`;
- primary verdicts: `Beneficial`, `Equivalent`, `Harmful`, `Inconclusive`.

## Non-zero-baseline cells

For baseline mean deficit `B > 0` and candidate mean deficit `C >= 0`, the relative effect is:

`R = (B - C) / B`.

Given a valid two-sided interval `[L,U]`, classification uses this exact precedence:

1. `Beneficial` iff `L > +0.02`;
2. `Harmful` iff `U < -0.02`;
3. `Equivalent` iff `L >= -0.02` and `U <= +0.02`;
4. `Inconclusive` otherwise.

Thus touching either equivalence boundary remains `Equivalent`, while an interval that crosses zero but extends outside the equivalence band is `Inconclusive`.

A missing interval for otherwise valid `B>0` deficits is represented as `Inconclusive`; it can never be promoted to a favorable verdict.

## Zero-baseline branch

The implementation never evaluates the relative statistic with a zero denominator:

- `B == 0` and `C == 0` => `Equivalent`, relative effect undefined;
- `B == 0` and `C > 0` => `Harmful`, relative effect undefined, absolute degradation `C-B` reported;
- zero-baseline cells can never be `Beneficial`.

Non-finite or negative deficits are typed rejections, not scientific verdicts.

## Nine-cell hypothesis gate

The API accepts exactly `[PrimaryCellDisposition; 9]`, preventing accidental eight- or ten-cell aggregation.

- `Beneficial` iff all nine cells are `Beneficial` or `Equivalent` and at least one is `Beneficial`;
- `Harmful` iff all nine are `Harmful` or `Equivalent` and at least one is `Harmful`;
- `Equivalent` iff all nine are `Equivalent`;
- `Inconclusive` otherwise.

Any missing/rejected primary cell produces `Inconclusive`. A mixture containing both `Beneficial` and `Harmful` also produces `Inconclusive`.

## Explicit non-scope

This tranche does not:

- construct paired intervals;
- choose the Bonferroni implementation or replicate count;
- select tasks, horizons, dimensions, budgets or populations;
- run T1/T2/T3 or A0/A1/A2/A3;
- access TDI-7.2;
- create a TDI-8.2 seed range, runner, confirmation token or result surface;
- emit an H8-A or H8-B result.

The paired interval implementation remains a separate TDI-8.1 tranche and must satisfy the frozen family-wise coverage rule `alpha = 0.05 / 9` before confirmatory arming can ever be considered.
