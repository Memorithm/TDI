# TDI-25 slice 40 — Stage-D attribution audit (`tdi25-stage-d-attribution-audit-v1`)

Status: experimental, non-final, Development/Validation only. No training, no
protected/final/holdout access, no scientific claim. Holdouts
TDI-7.2/8.2/9.2 are untouched. This closes Phase D.

## Question

Which structure-specific claims (torsor transport, chirality, generic G6)
remain admissible after the Phase-C/D ablations and controls?

## Contract

- Audited registry `ATTRIBUTION_AUDITED_SLICES`, in campaign order: Stage-C
  bounded preflight (30), torsor reduction-point ablation (31), direct vs
  factorized torsor bridge (32), chiral `gamma=0` ablation (33), chiral
  parity-shuffle control (34), torsor structure-shuffle control (35), G6
  orthogonal-basis control (36), position-geometry ablation (37),
  sequence-length scaling (38), data-volume scaling (39), each with its frozen
  contract pin.
- Every slice is regenerated on the same split and bounded budget; each
  runner validates its own report, and any rejection aborts the audit.
- Per entry: the report's own `protected_or_final_access`,
  `scientific_claim`, `experimental_non_final` and (where present)
  `training_executed` flags, and an FNV-1a 64 digest of the regenerated
  report's `Debug` rendering.
- Admissibility is derived, never asserted: `SoftwareSemanticsOnly` iff the
  report validated and all flags hold; otherwise `Withheld`.
- `scientific_attribution_admissible` is always false.

## Outcome (Development and Validation, bounded budget)

All ten slices validate and are `SoftwareSemanticsOnly`. They support
statements about contracts, matched populations and capacities, exact
accounting and fail-closed behavior. None supports attributing a
performance difference to torsor transport, chirality or the generic basis;
such a claim needs the Phase-E evidence gate and a preregistered
confirmatory decision, which no Phase-D slice provides.

## Validation

`validate_stage_d_attribution_audit_report` rejects `contract_drift`,
`registry_drift`, `admissibility_drift`, `withheld_entry`,
`scientific_attribution_admissible`, the protected/training/claim/non-final
flags, and regenerates every entry (`case_evidence_drift`, including split,
budget and digest drift).

## Recorded degeneracies

- The Stage-C preflight report (slice 30) has no `training_executed` field
  because it has no training path; the audit records `false` for it
  structurally rather than reading a flag.
- The digest binds to the `Debug` rendering under the pinned toolchain
  (1.97.1); a toolchain change may change digests without any semantic
  change, and is then caught as `case_evidence_drift`, fail-closed.
- `Withheld` cannot occur in a valid report: the audit is a gate, not a
  ranking. It adds no new evidence; it only re-reads slices 30 to 39.
