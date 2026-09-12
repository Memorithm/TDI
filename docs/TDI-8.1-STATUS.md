# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: **active** — substrate merged; freeze progress: **3/17 pinned** (A3 event policy, closed rejection taxonomy, degenerate bootstrap replicate policy); **14** fields remain `unresolved_blocking`. No experimental dimensions/budgets/horizons/populations have been selected.
- Parent programme issue: #87
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- Frozen TDI-8.0 preregistration blob: `fe80e7053d89824a77ef6790794f6930d1b424e2`
- Configuration freeze contract: `docs/tdi8.1-configuration-freeze.json` (schema `tdi8.1-configuration-freeze-v1`)
- Freeze resolution ledger: `docs/TDI-8.1-FREEZE-RESOLUTION-PLAN.md`
- Integrity readiness gate: `scripts/check-tdi8.1-readiness.sh` (workflow `tdi8-readiness.yml`)
- Final holdout / confirmatory runner / human token / TDI-8.2 surfaces: **absent**
- `tdi8_2_execution_authorized`: **false** (hard)
- TDI-7.2 interaction: **forbidden**

## Authorized freeze pins

- `a3_event_store_read_cleanup_policy` → `tdi8.1-a3-qualified-adapter-v1` (#138/#172)
- `closed_rejection_taxonomy` → `SymbolicRejectionCode` (#162/#175)
- `degenerate_replicate_policy` → `tdi8.1-reject-zero-baseline-bootstrap-replicates-v1` (#113)

Still unresolved (must not be guessed): recurrent/A2/A3 dimensions and fusion, matched budget, numeric horizons, deficit/interventions, paired interval method/count/seed, population domains.

`paired_interval_method` remains a qualified percentile *candidate* only (#113). It is not a closed final choice.

`scientific_status` remains `unresolved_blocking` until all 17 fields are `pinned`.
The readiness gate fail-closes if any field outside the three authorized pins is marked `pinned`.

## Remaining TDI-8.1 work

1. Resolve the remaining **14** fields from admissible Development/Validation evidence (`docs/TDI-8.1-FREEZE-RESOLUTION-PLAN.md`).
2. Content-address / CI-qualify `frozen_nonfinal` on exact head.
3. Keep readiness green while refusing any TDI-8.2 surface.

## Holdout boundary

TDI-8.2 remains future human-only. No autonomous confirmation token or confirmatory run. No TDI-7.2 contact.
