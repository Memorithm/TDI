# TDI-21.0 — Conflict-preserving B4 distributional objective

Status: **development-only objective scaffold; not a fitted policy; not frozen; not confirmatory**.

This increment follows the exact v1 predicate identifiability counterexample. The purpose is to represent the resulting tradeoff honestly instead of forcing one Boolean label onto an observable state that can have opposite hindsight-optimal actions in different episodes.

## Why an exact truth-table label is insufficient

The v1 counterexample produces the same candidate-visible assignment `0b011001` in two episodes. One episode uniquely favors `INHIBIT`; the other uniquely favors `ADMIT` for their evaluator-owned next recall.

An exact Boolean-function dataset must therefore not resolve that conflict by silently overwriting one label, dropping one episode, or selecting a majority label before accounting. `DevelopmentSet` / `ValidationSet` in the exact sparse-ANF search remain appropriate for genuine function synthesis and continue to reject duplicate assignments.

The distributional surface is separate.

## Counterfactual sample representation

Each `AdmissionOutcomeSample` contains:

- one causal predicate assignment;
- whether `ADMIT` succeeds under the declared evaluator probe;
- whether `INHIBIT` succeeds under the same probe.

Repeated assignments are retained and aggregated exactly into four counts:

1. both actions succeed;
2. only `ADMIT` succeeds;
3. only `INHIBIT` succeeds;
4. neither action succeeds.

No observation is converted into a single label during aggregation.

The current safety envelope admits at most six predicate variables and 4,096 evaluator-only samples per Development or Validation set. Development and Validation use distinct Rust types. This module scores a supplied ANF program; it does not choose or mutate one.

## Exact loss decomposition

For one observable state with counts

- `B` = both succeed,
- `A` = admit only,
- `I` = inhibit only,
- `N` = neither succeeds,

choosing `ADMIT` deterministically causes

`I + N`

failures, while choosing `INHIBIT` causes

`A + N`.

A per-episode evaluator with hindsight cannot avoid the `N` failures. Therefore

`hindsight_oracle_failures = N`.

The best deterministic action allowed to depend on the observable state alone incurs

`N + min(A, I)`.

The term

`min(A, I)`

is the exact extra failure lower bound caused by collapsing episodes with conflicting preferred actions onto the same observable state. The implementation reports this as `deterministic_conflict_lower_bound`.

For an actual ANF policy, the additional quantity

`selected_failures - deterministic_state_oracle_failures`

is reported as `policy_excess_failures`. It separates limitation of the chosen policy form/parameters from limitation of the available predicates.

## Exact v1 witness

Aggregating the two episodes from the identifiability counterexample at assignment `0b011001` gives:

- `A = 1`;
- `I = 1`;
- `B = 0`;
- `N = 0`.

Hence every deterministic local policy must fail at least

`min(1,1) = 1`

of the two episodes, while the per-episode hindsight oracle fails zero. An always-ADMIT and an always-INHIBIT policy both achieve exactly one success and one failure on this two-episode set.

This is an exact property of the constructed distribution and v1 observation map. It is not a claim about an unknown real-world distribution.

## Runtime-work boundary

The scorer may evaluate an ANF once per aggregated state for efficiency. `runtime_anf_term_evaluations` instead reports the term evaluations implied by executing the policy once per original sample. It is semantic runtime work, not measured CPU instructions, latency, FLOPs or energy.

The scoring dataset itself is evaluator-side research material. Its counterfactual action outcomes must never become runtime candidate features.

## Development / Validation discipline

`DevelopmentAdmissionSet` and `ValidationAdmissionSet` are distinct types. This objective layer does not fit a policy at all, so Validation cannot alter selection here.

A later search layer may minimize a declared Development loss, but it must:

- accept Development only for selection;
- evaluate Validation after selection;
- preserve exact conflict counts;
- freeze weighting/metrics before comparative evidence;
- retain negative strata rather than tune them away;
- report both feature-conflict lower bound and policy-excess error.

## Qualification fixtures

Tests cover:

- the exact two-episode v1 conflict and its one-failure deterministic floor;
- preservation of all four counterfactual outcome classes;
- a one-bit ANF that makes the correct action on two identifiable states;
- separation between policy-form error and feature-conflict error;
- aggregation of repeated assignments rather than rejection/overwrite;
- distinct Development and Validation scoring;
- program/dataset arity rejection;
- empty, excessive, over-wide and out-of-range inputs.

These tests establish software semantics only. They do not select a policy, establish a task distribution, demonstrate generalization, replace attention, or measure hardware performance.
