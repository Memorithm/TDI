# TDI-7.2 — final-holdout arming protocol

Status: **unarmed design contract**. This document does not authorize or execute the final holdout.

TDI-7.2 may only be armed after the entire TDI-7.1 stack is merged to `main`, all required CI is green on the merged head, and the exact evaluator/specification surfaces are frozen.

## Preconditions

Before any final-holdout process exists, all of the following must be true:

1. TDI-7.0 preregistration integrity verifies.
2. TDI-7.1 evaluator specification is merged and unchanged.
3. TDI-7.1 completion/readiness gate passes on the exact merged `main` commit.
4. The historical `Rust validation`, the additive `TDI-7 hosted software validation`, and the dedicated TDI-7.1 bounded-preflight workflow are green on that commit.
5. The worktree used for the final run is clean and points at that exact merged commit.
6. No evaluator, feature, model, intervention, seed, target, bootstrap, numerical-policy or decision-rule change is pending.
7. The exact final-holdout generator count is frozen in `docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml` by a separately reviewed decision, and the machine validator accepts that record as `FROZEN`.
8. The deterministic rule selecting concrete seeds from the frozen final range is separately reviewed and frozen. `docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml` must no longer be `UNRESOLVED`, and its validator must contain an explicit machine-recognized implementation of that reviewed rule.
9. The final run is explicitly authorized by a human-supplied confirmation value that CI, tests and examples never provide.

If any precondition fails, TDI-7.2 remains blocked.

The hosted software gate is additive evidence only. Its success must never be used to relabel a queued, failed or cancelled historical self-hosted `Rust validation` run as green. Its exact scope and non-claims are defined in `docs/TDI-7-HOSTED-SOFTWARE-GATE.md`.

## Final population decision record

The final seed range was frozen by TDI-7.0, but the exact number of generators selected from that range was not. TDI-7.2 therefore carries a separate machine-readable population decision record rather than inferring a population size from training, development or validation splits.

The population record has two valid decision states:

- `UNRESOLVED`: no `final_holdout_generator_count` field may exist and `decision_reference` must remain `UNRESOLVED`;
- `FROZEN`: an exact positive `final_holdout_generator_count` must exist, must fit within the frozen final seed range, and `decision_reference` must identify the reviewed decision that froze it.

In both states `authorization_state` must remain `NOT_AUTHORIZED`. Freezing the population size is not authorization to access the holdout. The record is validated by `tdi-ai/examples/tdi7_arming_decision.rs`, which contains no final-run confirmation value and cannot generate or consume holdout data.

## Final seed-selection decision record

TDI-7.0 requires exact split ranges and population sizes, but it does not state how a population smaller than the final seed-range capacity selects concrete seeds from that range. The implementation must not silently infer `first N`, random sampling, striding, hashing or any other convention.

`docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml` therefore records this independent decision. Its initial state is `UNRESOLVED`. The current validator, `tdi-ai/examples/tdi7_seed_selection_decision.rs`, intentionally rejects `FROZEN`: a future freeze must update the validator in the same separately reviewed change with an explicit machine-recognized selection rule. This prevents a prose-only or arbitrary string from becoming an executable seed-selection policy.

The seed-selection record also remains `NOT_AUTHORIZED` and cannot itself enable a final run.

The initial decision records are anchored to the exact post-#76 `main` commit on which the dedicated TDI-7.1 bounded preflight and TDI-7 pre-arm integrity gates succeeded. Later population-size and seed-selection decisions do not retroactively rewrite TDI-7.1 provenance.

## Single-use scientific boundary

The final holdout is not a development split. Once accessed under an armed TDI-7.2 runner:

- no tuning is allowed;
- no feature may be added or removed;
- no task/intervention record may be selectively dropped because of its result;
- no margin, bootstrap rule, model class or regularization grid may change;
- invalid records may only be rejected for reasons frozen before access and every rejection reason must be reported;
- a code or protocol change requires a new preregistration and a fresh disjoint holdout.

## Required final output

For each confirmatory task family, the run must emit at minimum:

- final-holdout record count;
- invalid/rejected count and reasons;
- B0 MSE;
- B1 MSE;
- relative MSE reduction;
- paired 95% bootstrap interval;
- task verdict;
- intervention-location summaries.

The aggregate TDI-7 verdict must follow the frozen TDI-7.0 multi-task gate exactly.

The result artifact must also include full provenance: repository commit, evaluator specification identity, semantic identifier, task-generator version, final seed range, frozen population count, frozen seed-selection rule, intervention definition, observation depths, feature schema, model configuration, bootstrap configuration, numerical policy and classifier margins.

## What may be implemented before arming

Before all preconditions are satisfied, development may add only **fail-closed arming checks**, population/seed-selection decision validation, additive software validation and reporting schemas that cannot generate or consume the final holdout.

A real final-holdout generator/runner must not be added until the population count and seed-selection rule are separately frozen and the arming transition is reviewed.

## Non-claims

Arming TDI-7.2 does not imply a positive result. Beneficial, Equivalent, Harmful and Inconclusive are all valid outcomes and must remain recorded.
