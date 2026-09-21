# TDI-26.0 -> TDI-26.1 Implementation Gate

TDI-26.1 evaluator work may begin only after all conditions below are true on `main`:

1. TDI-26.0 Stage-0 bootstrap is merged.
2. A later `docs/TDI-26.0-FREEZE.md` pins the exact Stage-0 payload.
3. A freeze-integrity script passes on `main`.
4. The exact merged SciRust V888 import/graph contract to be consumed is pinned.
5. TDI-26.1 separately freezes development/validation tasks, topology construction, split derivation, metrics, rejection rules and provenance.
6. No final or confirmatory data/result surface exists.

## Invariants that later stages may not weaken in response to results

- BANC v888 is the only connectome source;
- raw BANC data remains outside TDI;
- random, degree, degree+reciprocity, and degree+reciprocity+modularity sparse controls remain available;
- sparse topology arms use exact declared matched budgets;
- active runtime work is accounted separately from static edge budget;
- negative/null/harmful results are retained;
- final/confirmatory execution requires a distinct later authorization.

This gate authorizes no scientific result by itself.
