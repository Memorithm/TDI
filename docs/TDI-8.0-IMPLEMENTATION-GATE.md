# TDI-8.0 → TDI-8.1 implementation gate

TDI-8.1 may begin only after the TDI-8.0 preregistration is merged and treated
as frozen design evidence.

Before implementation starts, an agent must verify all of the following:

1. `docs/TDI-8.0-ASSR-PREREGISTRATION.md` exists on `main`.
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
   interference recall.
9. Reference execution remains deterministic binary64 with fixed operation
   order unless a separately reviewed preregistration amendment changes it
   before any final holdout is created.
10. TDI-8.1 must freeze concrete dimensions, budgets, horizon strata, seed
    ranges, sample counts, paired uncertainty configuration and typed rejection
    policy before TDI-8.2 can be armed.

Failure of any item blocks TDI-8.1 implementation until the inconsistency is
resolved through a reviewed preregistration change.
