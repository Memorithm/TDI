# TDI-21.0 — Bounded distributional ANF policy search

Status: **development-only scaffold; not frozen; not confirmatory; no final/holdout material**.

This work follows the v1 predicate identifiability counterexample and the conflict-preserving distributional objective. It is intentionally separate from exact truth-table/function synthesis.

## Question

Given repeated causal predicate states whose counterfactual `ADMIT`/`INHIBIT` outcomes may conflict across episodes, can a bounded deterministic sparse ANF policy minimize a declared Development failure count without erasing those conflicts?

This is a policy-selection engineering question. It is not yet evidence that the selected policy generalizes, improves B3 on a frozen sequence population, or replaces attention.

## Selection input

Selection accepts only `DevelopmentAdmissionSet`. Repeated predicate assignments retain their episode multiplicity through exact outcome counts; they are not collapsed into one Boolean label.

`ValidationAdmissionSet` is accepted only after selection through `evaluate_distributional_validation`. This type split is an API isolation mechanism, not proof against every possible experiment-level leak.

No runtime candidate receives evaluator counterfactual outcomes. Those outcomes are research labels used only to construct Development/Validation objectives.

## Candidate family

The search reuses the bounded `SearchEnvelope`:

- at most six Boolean variables;
- monomial degree at most three;
- at most four monomials;
- at most 65,536 candidate programs.

Before enumeration, the complete candidate count

`2 * sum_{k=0..K} C(m,k)`

is computed from the eligible monomial universe. Search fails closed when the declared candidate budget is insufficient; it does not silently truncate the candidate population.

## Development objective

For every candidate ANF program, the primary selection score is the exact number of failed Development episodes under the conflict-preserving outcome counts.

Ties are deterministic:

1. fewer Development failures;
2. fewer monomials;
3. lower maximum degree;
4. constant `false` before `true`;
5. canonical monomial-mask order.

The objective does **not** subtract the v1 feature-conflict floor before selection. The raw episode-weighted failure count is the operational quantity being optimized. The decomposition is reported separately so interpretation remains honest.

## Provenance and exact decomposition

The selected result retains:

- the complete fitted `SearchEnvelope`;
- Development sample count;
- Development unique observable-state count;
- selected Development failures;
- per-episode hindsight-oracle failure floor;
- best deterministic state-wise failure floor;
- exact feature-conflict lower bound from non-identifying observations;
- selected policy excess above the state-wise floor;
- bounded search work.

The implementation independently re-scores the selected `AnfProgram` with the distributional objective. If the exhaustive enumerator's selected failure count disagrees with that independent scorer, fitting fails with `SelectionScoreMismatch`; the discrepancy is not hidden behind a debug-only assertion.

Consequently a nonzero total error can be attributed separately to:

1. irreducible failure even with per-episode hindsight (`neither` cases);
2. feature non-identifiability (`min(admit_only, inhibit_only)`);
3. selected policy-form/search error above the best deterministic action for each observable state.

## Search work

`DistributionalSearchWork` counts:

- candidate programs evaluated;
- aggregated observable states evaluated during search;
- candidate monomial evaluations over those aggregated states.

These counters describe the search implementation, not runtime policy execution. Runtime ANF term work per original episode is reported by the separate distributional objective scorer. Neither quantity is a measured FLOP, CPU instruction, latency, energy or bandwidth value.

## Qualification targets

The development tests cover:

- the exact v1 two-episode conflict, for which both deterministic constants fail exactly one of two episodes and tie-breaking selects `INHIBIT` without fabricating a perfect classifier;
- retention of Development sample count and unique-state count in result provenance;
- repeated identical states with a 3:1 Development preference, verifying that episode multiplicity affects selection;
- an identifiable two-state distribution recovering the one-term `x0` policy;
- XOR behavior demonstrating policy-form excess under a constant-only envelope and zero excess when the envelope admits two degree-1 terms;
- post-selection Validation whose reversed objective fails without changing the selected policy;
- Validation arity drift rejection;
- candidate-budget rejection before enumeration;
- exact search-work accounting over aggregated states.

These are software fixtures. They do not define the future scientific Development/Validation population.

## Required before interpretation as research evidence

A later TDI-21 stage must preregister, before comparative evidence:

- how sequence episodes generate the counterfactual action outcomes;
- Development/Validation population derivation and disjointness;
- weighting of old-fact preservation versus new-fact retention and any other strata;
- search envelope and stopping/rejection rules;
- primary and secondary metrics;
- memory/compute/training/search accounting;
- treatment of the exact v1 conflict stratum;
- decision rule for retaining v1 versus introducing a richer causal predicate/state representation.

The search must never gain access to Validation labels during selection. A richer representation, if justified, must use present/past causal state rather than the evaluator's future recall.

## Scope guard

Admission/replacement is a qualification surface for ANF routing and causal-state limitations. It must not redefine TDI-21 as a cache-policy project. After this mechanism is qualified, B4 research must return to relational composition/binding tasks and later Boolean state evolution that exercise functions currently served by attention.
