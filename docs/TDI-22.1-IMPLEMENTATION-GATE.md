# TDI-22.1 → TDI-22.2 Implementation Gate

TDI-22.2 bounded non-final evaluator implementation/execution may begin only after all of the following are true on `main`:

1. `docs/TDI-22.1-PREREGISTRATION.md` is merged.
2. `docs/TDI-22.1-ARM-CONTRACT.md` is merged.
3. `docs/TDI-22.1-FREEZE.md` is merged.
4. `docs/TDI-22.1-SEED-CONTRACT.md` is merged.
5. `docs/TDI-22.1-GENERATOR-CONTRACT.md` is merged.
6. `docs/TDI-22.1-RECORD-CONTRACT.md` is merged.
7. `docs/TDI-22.1-STATUS.md` states bounded non-final authorization.
8. `scripts/check-tdi22-freeze.sh`, `scripts/check-tdi22.1-preregistration.sh`, and `scripts/check-tdi22.1-freeze.sh` all pass.
9. No TDI-22 final/confirmatory runner, dataset, seed list or result payload exists.
10. Protected TDI-7.2/TDI-8.2 surfaces remain untouched.

## Authorized TDI-22.2 scope

After the gate is satisfied, agents may implement and execute only the frozen development/validation surfaces:

- deterministic P1–P5 episode generation;
- G0/G1/G2/G3 geometry exactly as frozen;
- T0, T1, T3 and T4 reference semantics;
- evaluator-owned F1/F2/F3 target generation;
- top-1 retrieval, target margin, T1 value reconstruction and typed rejection;
- full semantic resource ledger and canonical `tdi22-eval-record-v1` serialization;
- deterministic replay tests;
- direct-vs-factorized T3 differential checks against the Stage-0 oracle.

## Still forbidden

T2 execution, learned geometry, result-conditioned protocol tuning, final/confirmatory material or execution, FLAT-ATTENTION kernel/routing changes, and latency/bandwidth/energy/KV/asymptotic or universal replacement claims remain forbidden.
