# TDI-8.0 → TDI-8.1 implementation gate

TDI-8.1 may begin only after the TDI-8.0 preregistration is merged and treated
as frozen design evidence.

Before implementation starts, an agent must verify all of the following:

1. `docs/TDI-8.0-ASSR-PREREGISTRATION.md` exists on `main` and matches the pinned
   Git blob recorded by `docs/TDI-8.0-ASSR-PREREGISTRATION.gitblob`.
2. TDI-7.2 final-holdout status remains untouched by TDI-8 work.
3. No TDI-8.2 runner, confirmation token, final-holdout seed range or final
   result surface exists.
4. A0/A1/A2/A3 names retain their reference meanings.
5. Primary contrasts remain A2 vs A1 and A3 vs A2.
6. A1/A2/A3 use the same declared total dynamic-memory budget in the primary
   comparison.
7. A0 remains a contextual full-history reference rather than a hidden
   matched-budget baseline.
8. The three frozen task families remain associative recall, delayed copy and
   interference recall, each with short, medium and long primary horizon strata.
9. Reference execution remains deterministic binary64 with fixed operation
   order unless a separately reviewed preregistration amendment changes it
   before any final holdout is created.
10. Each hypothesis retains exactly nine primary task × horizon cells.
11. The primary relative-effect margin remains `delta = 0.02`.
12. For non-zero-baseline cells, the four-way classifier remains: `Beneficial`
    iff `L > +delta`; `Harmful` iff `U < -delta`; `Equivalent` iff
    `L >= -delta` and `U <= +delta`; `Inconclusive` otherwise.
13. Zero-baseline cells never divide by zero: a zero/zero deficit pair is
    `Equivalent`; a zero/positive pair is `Harmful`; neither produces a finite
    relative effect.
14. Each hypothesis retains at least 95% family-wise coverage across its nine
    primary cells using the frozen Bonferroni allocation `alpha = 0.05 / 9`.
15. The hypothesis-level aggregation gate remains conservative: all nine cells
    must be Beneficial/Equivalent with at least one Beneficial for a Beneficial
    hypothesis verdict; the symmetric Harmful rule applies; all Equivalent is
    Equivalent; every other pattern is Inconclusive.
16. TDI-8.1 must freeze concrete dimensions, budgets, horizon values, seed
    ranges, sample counts, paired interval implementation/replicate count and
    typed rejection policy before TDI-8.2 can be armed. It must not alter the
    frozen margin, classifier, primary-cell set, family-wise coverage rule or
    hypothesis-level aggregation gate.

Failure of any item blocks TDI-8.1 implementation until the inconsistency is
resolved through a reviewed preregistration change made before any TDI-8.2
runner, seed range or result surface exists.
