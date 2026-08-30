# TDI-7.2 — final-holdout arming protocol

Status: **unarmed design contract**. This document does not authorize or execute the final holdout.

TDI-7.2 may only be armed after the entire TDI-7.1 stack is merged to `main`, all required CI is green on the merged head, and the exact evaluator/specification surfaces are frozen.

## Preconditions

Before any final-holdout process exists, all of the following must be true:

1. TDI-7.0 preregistration integrity verifies.
2. TDI-7.1 evaluator specification is merged and unchanged.
3. TDI-7.1 completion/readiness gate passes on the exact merged `main` commit.
4. Rust validation and the dedicated TDI-7.1 bounded-preflight workflow are green on that commit.
5. The worktree used for the final run is clean and points at that exact merged commit.
6. No evaluator, feature, model, intervention, seed, target, bootstrap, numerical-policy or decision-rule change is pending.
7. The final run is explicitly authorized by a human-supplied confirmation value that CI, tests and examples never provide.

If any precondition fails, TDI-7.2 remains blocked.

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

The result artifact must also include full provenance: repository commit, evaluator specification identity, semantic identifier, task-generator version, final seed range, intervention definition, observation depths, feature schema, model configuration, bootstrap configuration, numerical policy and classifier margins.

## What may be implemented before arming

Before TDI-7.1 is fully merged and CI-valid, development may add only **fail-closed arming checks** and reporting schemas that cannot generate or consume the final holdout.

A real final-holdout generator/runner must not be added to the stacked pre-merge branches.

## Non-claims

Arming TDI-7.2 does not imply a positive result. Beneficial, Equivalent, Harmful and Inconclusive are all valid outcomes and must remain recorded.
