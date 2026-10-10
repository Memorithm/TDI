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

## Provenance authority (issue #690, envelope v2)

Issue #690 found that the slice-29 envelope (`tdi24-provenance-envelope-v1`)
recorded semantic contract labels rather than the exact identity of a run:
one run-level seed for a multi-case population, a configuration string that
ignored the actual `EvaluatorConfig`, a data identity that named only the
split, a code identity that named only contracts, a declared (not actual)
compiler, and no requirement that scored cases belong to the envelope.

`tdi24-provenance-envelope-v2` makes the envelope authoritative. It is a
software-provenance repair only: no freeze pin, no new seed material, no
protected/final access and no change to any scoring rule.

- **No scoring without provenance.** `EvaluatorRun::open` still validates a
  configuration, but every `evaluate_v6_binary` / `evaluate_c6_binary` call
  on a run without a bound envelope fails closed with `provenance_required`.
- **Complete configuration.** `stage_c_config_identity_bundle(&EvaluatorConfig)`
  renders arm and arm contract, split, envelope, every budget field
  (`max_cases`, `max_readout_scalars_per_case`, `updates`), the metric
  registry (primary, ordered secondaries, non-final flag) and the matcher,
  uncertainty, failure and provenance contracts under
  `tdi24-evaluator-config-identity-v1`. `open_with_provenance` rejects any
  envelope whose configuration identity differs from the run
  (`config_identity_mismatch`).
- **Exact admitted population.** The envelope carries the ordered
  `admitted_cases` (family, `case_id`, `group_id`, canonical inference-view
  digest, registered seed). `data_identity` is an FNV-1a digest of that list
  under `tdi24-admitted-population-v1`; before scoring, each case must be
  admitted (`case_not_admitted`), match its canonical digest
  (`case_population_mismatch`) and `group_id`, and may be scored once
  (`case_already_evaluated`). Duplicates, empty populations and populations
  larger than the run budget are rejected.
- **Per-case seeds.** Each admitted case is bound to
  `register_seed(domain(split), family, group_id)` (slice 18);
  `seed_identity` digests every case seed under `tdi24-case-seed-binding-v1`.
  Wrong seeds, groups or domains fail closed (`case_seed_mismatch`,
  `seed_domain_split_mismatch`).
- **Code fingerprint.** `code_identity` carries
  `tdi24-source-digest-fnv1a64-v1`: FNV-1a 64 over the path, length and bytes
  of every TDI-24 evaluator source file compiled into the crate
  (`tdi24_{accounting,attention,chiral,eval,tasks,vector}.rs`). It identifies
  the exact source text (equivalent to a tree fingerprint of that surface);
  it is not a cryptographic authentication.
- **Actual compiler.** `tdi-ai/build.rs` records `rustc --version` of the
  compiler Cargo actually uses (`BUILD_RUSTC_VERSION`). The envelope records
  that release and fails closed when it is unavailable or drifts.
  `PROVENANCE_TOOLCHAIN_CHANNEL` / `PROVENANCE_TOOLCHAIN_ID` remain the
  declared CI gate (`1.97.1`), not the recorded identity.
- **Recomputed, not trusted.** `validate_provenance_envelope` recomputes the
  code, data and seed identities; the Stage-C preflight admits its full
  population before scoring, propagates every provenance violation as a hard
  error instead of a retained failure, and its report validator requires
  each arm's envelope to bind the expected configuration and exactly
  `cases_per_arm` admitted cases covering every record.

Negative tests live in `tdi-ai/tests/tdi24_provenance_authority.rs`; the
gate is `scripts/check-tdi24-s30-provenance-authority.sh`, invoked by
`scripts/check-tdi24-s30.sh` so it runs in the slice-30 exact-head workflow.
