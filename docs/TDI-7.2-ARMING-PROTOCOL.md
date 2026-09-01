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
9. The exact final-record rejection/invalidity policy is separately reviewed and frozen. `docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml` must no longer be `UNRESOLVED`, and its validator must encode the allowed reason codes and their applicability conditions rather than accepting free-form post-hoc reasons.
10. The final run is explicitly authorized by a human-supplied confirmation value that CI, tests and examples never provide.

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

## Final rejection-policy decision record

TDI-7.0 requires the final report to contain invalid/rejected counts and reasons, and requires any rejected record to be excluded only for reasons frozen before final-holdout access. TDI-7.1 freezes the deterministic evaluator but does not define an exhaustive final-record invalidity taxonomy.

That omission matters scientifically: a free-form reason supplied after observing a difficult or unfavorable record would make the final population mutable. `docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml` therefore records a third independent pre-holdout decision.

Its initial state is `UNRESOLVED`. The current validator, `tdi-ai/examples/tdi7_rejection_policy_decision.rs`, intentionally rejects `FROZEN` and rejects arbitrary reason-code fields. A future freeze must update code and record together, defining a closed set of reason codes and deterministic applicability conditions before any final record is generated. The evidence handoff may count frozen reasons, but it must not invent them.

The rejection-policy record remains `NOT_AUTHORIZED` and cannot enable final execution.

The initial decision records are anchored to the exact post-#76 `main` commit on which the dedicated TDI-7.1 bounded preflight and TDI-7 pre-arm integrity gates succeeded. Later population-size, seed-selection and rejection-policy decisions do not retroactively rewrite TDI-7.1 provenance.

## Single-use scientific boundary

The final holdout is not a development split. Once accessed under an armed TDI-7.2 runner:

- no tuning is allowed;
- no feature may be added or removed;
- no task/intervention record may be selectively dropped because of its result;
- no margin, bootstrap rule, model class or regularization grid may change;
- invalid records may only be rejected for the closed reason set frozen before access, with the applicability condition and every rejection reported;
- a code or protocol change requires a new preregistration and a fresh disjoint holdout.

## Required final output

For each confirmatory task family, the run must emit at minimum:

- final-holdout record count;
- invalid/rejected count and frozen reason-code counts;
- B0 MSE;
- B1 MSE;
- relative MSE reduction;
- paired 95% bootstrap interval;
- task verdict;
- intervention-location summaries.

The aggregate TDI-7 verdict must follow the frozen TDI-7.0 multi-task gate exactly.

The result artifact must also include full provenance: repository commit, evaluator specification identity, semantic identifier, task-generator version, final seed range, frozen population count, frozen seed-selection rule, frozen rejection policy, intervention definition, observation depths, feature schema, model configuration, bootstrap configuration, numerical policy and classifier margins.

## What may be implemented before arming

Before all preconditions are satisfied, development may add only **fail-closed arming checks**, population/seed-selection/rejection-policy decision validation, additive software validation and reporting schemas that cannot generate or consume the final holdout.

A real final-holdout generator/runner must not be added until the population count, seed-selection rule and rejection policy are separately frozen and the arming transition is reviewed.

## Non-claims

Arming TDI-7.2 does not imply a positive result. Beneficial, Equivalent, Harmful and Inconclusive are all valid outcomes and must remain recorded.
