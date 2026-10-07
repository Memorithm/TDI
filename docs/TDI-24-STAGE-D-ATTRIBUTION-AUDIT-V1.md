# TDI-24 slice 40 — Stage-D attribution audit (`tdi24-stage-d-attribution-audit-v1`)

Status: experimental, non-final, Development/Validation only. No training, no
protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched.

## Question

Which claims remain admissible after the Phase-C/D ablations and controls?

## Contract

- Audited registry `ATTRIBUTION_AUDITED_SLICES`, in campaign order: Stage-C
  preflight (30), `gamma=0` (31), `beta=0` (32), direct-only collapse (33),
  parity-shuffle control (34), fixed-M sensitivity (35), learned-basis
  prototype (36), head-sharing ablation (37), width scaling (38),
  sequence-length scaling (39), each with its frozen contract pin.
- Every slice is regenerated on the same split and bounded budget; each
  runner validates its own report, and any rejection aborts the audit.
- Per entry: the report's own `protected_or_final_access`,
  `training_executed`, `scientific_claim` and `experimental_non_final` flags,
  and an FNV-1a 64 digest of the regenerated report's `Debug` rendering.
- Admissibility is derived, never asserted: `SoftwareSemanticsOnly` iff the
  report validated and all four flags hold; otherwise `Withheld`.
- `scientific_attribution_admissible` is always false.

## Outcome (Development and Validation, bounded budget)

All ten slices validate and are `SoftwareSemanticsOnly`: they support
statements about contracts, matched capacity, cost accounting and
fail-closed behavior. None supports attributing a performance difference to
the chiral term, the basis, head sharing, width or length; no such claim is
admissible without a trained, preregistered Stage-D execution, which this
campaign does not run.

## Validation

`validate_stage_d_attribution_audit_report` rejects `contract_drift`,
`registry_drift`, `admissibility_drift`, `withheld_entry`,
`scientific_attribution_admissible`, the protected/training/claim/non-final
flags, and regenerates every entry (`case_evidence_drift`, including split,
budget and digest drift).

## Recorded degeneracies

- The digest binds to the `Debug` rendering under the pinned toolchain
  (1.97.1); a toolchain change may change digests without any semantic
  change, and is then caught as `case_evidence_drift`, fail-closed.
- `Withheld` cannot occur in a valid report: any withheld entry is itself a
  rejection, so the audit is a gate, not a ranking.
- The audit adds no new evidence; it only re-reads slices 30 to 39.
