# TDI-24 Stage-C bounded preflight (campaign slice 30)

Status: **candidate pending exact-head qualification, material review, and merge**.

This slice closes Phase C by running the merged Stage-C machinery
(campaign slices 21–29) end-to-end as one bounded smoke campaign. It is a
software qualification of the evaluation pipeline, not an experiment.

`tdi24_eval::run_stage_c_preflight(split, StageCPreflightBudget)`:

- accepts only `DataSplit::Development` / `DataSplit::Validation`;
  `run_stage_c_preflight_for_label` rejects `protected`, `final` and any
  unknown label with `ProtectedOrFinalSplit` before any case is generated;
- caps the budget at 8 pairs per family × 2 members × 4 Phase-B families
  = 64 cases per arm (`MAX_CASES_PER_RUN`), rejecting empty, oversized,
  overflowing or contract-drifted budgets fail-closed;
- matches V6/C6 trainable capacity (slice 23), initialization (slice 24)
  and optimizer/update budget (slice 25) on the non-trained reference path,
  reusing the Slice-18 registered seed — no new seed material;
- opens both evaluators (slices 21/22) bound to paired provenance
  envelopes (slice 29) under the pinned metric registry (slice 26);
- evaluates the same sealed synthetic cases on both arms in the same order
  through the protected-label API;
- retains every failure in per-arm failure ledgers (slice 28); a paired
  uncertainty summary (slice 27) is produced only when no failure was
  retained, so a summary can never hide dropped cases;
- returns a `StageCPreflightReport` whose validator rejects
  `protected_or_final_access`, `training_executed` or `scientific_claim`
  set to true, unaccounted/dropped cases, unpaired provenance and summary
  mismatches.

The candidate manifest is `docs/tdi24-stage-c-preflight.yaml`. It authorizes
only the bounded Development/Validation smoke run; training, confirmatory
execution, protected/final access, scientific and performance claims remain
`false`. It pins no configuration-freeze value: the smoke budget is a
software cap, not a frozen population.

No scientific outcome, V6-vs-C6 superiority, novelty, speedup, hardware
behaviour or downstream FLAT-ATTENTION readiness follows from this preflight.
Holdouts TDI-7.2 / TDI-8.2 / TDI-9.2 are not touched.
