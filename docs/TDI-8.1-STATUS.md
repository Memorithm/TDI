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

## Integrity (this slice)

Post-#201 scout (after TDI-10.15 family composition on `main`, with TDI-10.16 κ-monotonicity / matched-factorization in flight) found **no** newly closed identifier that can pin a remaining field. Existing authorized pins stay 3/17. Readiness continues to fail-close on a pinned-count floor of **≥3**, STATUS↔freeze JSON pin-count cross-check, missing pin-evidence blocks, unauthorized pins, and silent STATUS / `scientific_status` upgrades (#199).

## Remaining TDI-8.1 work

1. Resolve the remaining **14** fields from admissible Development/Validation evidence (`docs/TDI-8.1-FREEZE-RESOLUTION-PLAN.md`).
2. Content-address / CI-qualify `frozen_nonfinal` on exact head.
3. Keep readiness green while refusing any TDI-8.2 surface.

## Holdout boundary

TDI-8.2 remains future human-only. No autonomous confirmation token or confirmatory run. No TDI-7.2 contact.

## Series orthogonality

TDI-10.x through TDI-10.16 (operator-family chapter: 10.13–10.15 families/composition + 10.16 κ(a,b) domain/monotonicity and matched TDI-10.3 → Type-U link) remains orthogonal: operator-research advances must not invent freeze pins here or contact TDI-8.2 / TDI-9.2 surfaces. #199 readiness floors are unchanged; STATUS↔JSON pin-count cross-check is additive.
