# TDI-9.1 status

- Scientific series: TDI-9.x
- Stage: TDI-9.1 bounded autonomous adaptive-inference evaluator
- Status: **active** — policy/accounting foundation, P1/P2/P3 generators, solver/verifier/checkpoint/replay execution, C0/C1/C2/C3 reference policies, **composed non-final evaluator** (#135), and **machine-readable rejection records** (#140) are merged. Freeze progress: **1/14 pinned** (`closed_rejection_taxonomy`); remaining scientific freezes are `unresolved_blocking`.
- Parent TDI-9.0 merge: `bb13c59aa91e3e5e2e6a480f4ae12adfe168221b` (PR #122)
- TDI-9.1 policy/accounting foundation: PR #123
- Deterministic P1/P2/P3 generators: PR #126
- Reference execution: PR #128 (`6a4b7decdb044166ee3a6193108fed423913499e`)
- Reference policies: PR #129 (`d5ba650e2f0abdadbe44856698a8e44d33117cb0`)
- Reference evaluator integration: PR #135
- Reference rejection records: PR #140
- Frozen TDI-9.0 preregistration blob: `babad0a4e309e67e57820281a0f31284ba1e5da0`
- Configuration freeze contract: `docs/tdi9.1-configuration-freeze.json` (schema `tdi9.1-configuration-freeze-v1`)
- Freeze resolution ledger: `docs/TDI-9.1-FREEZE-RESOLUTION-PLAN.md`
- Integrity readiness gate: `scripts/check-tdi9.1-readiness.sh` (workflow `tdi9-readiness.yml`)
- TDI-9.2 runner / final seed list / dataset / result payload: **absent**
- Human confirmation token: intentionally absent from TDI-9
- TDI-7.2 / TDI-8.2 interaction: **forbidden**

## What is merged (infrastructure only — not H9-A/H9-B evidence)

- C0–C3 identities, action legality, PolicyObservation, envelopes, accounting (#123)
- P1/P2/P3 generators with evaluator-only metadata boundary (#126)
- Deterministic solver/verifier/checkpoint/replay execution (#128)
- Bounded C0/C1/C2/C3 reference policies (#129)
- Composed `adaptive_evaluator` non-final integration (#135)
- `evaluate_generated_task_recorded` + `ReferenceRejectionCode` vocabulary (#140)

## Remaining TDI-9.1 work

1. Follow `docs/TDI-9.1-FREEZE-RESOLUTION-PLAN.md` to resolve remaining freeze fields from Dev/Val evidence only. `PolicyObservation` remains a closed type, not a pinned experimental observation vector.
2. Define agent-search-safe policy mutation/evaluation for development/validation only.
3. Implement deterministic paired primary-cell evidence + frozen H9 classifier plumbing.
4. Freeze future public-entropy source/event/encoding and final-seed derivation **before** that public value is knowable.
5. Prove fail-closed absence of any TDI-9.2 final surface before the entropy gate.

No item authorizes TDI-9.2 execution.
