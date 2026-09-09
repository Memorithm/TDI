# TDI-11.4 — Decomposed Entailment Hallucination Study Preregistration

Status: NON-FINAL PREREGISTRATION ONLY

This document does not authorize model execution, access to any protected/final holdout, or scientific claims. Execution remains blocked by the TDI-11.2 model/observation freeze gate.

## Motivation and external prior art

Recent work by Oukelmoun, Semmar and De Chalendar, *Decomposed Entailment for Factuality Checking and Hallucination Detection* (arXiv:2608.05823, 2026-08-06), proposes a black-box detector that decomposes generated text into atomic claims, scores claim support/contradiction against source chunks with an entailment model, and aggregates claim-level evidence. Their reported results motivate a testable Memorithm hypothesis; they do not constitute evidence for TDI.

This study also complements the already-preregistered TDI-11.4 uncertainty-relevance and claim-level calibration/selective-abstention studies. It must remain analytically separate so that improvements from source-grounded verification are not conflated with improvements from model-internal uncertainty or calibrated abstention.

## Research question

For source-grounded generation under the frozen TDI-11 controlled evaluation contract, does claim decomposition plus source entailment provide non-redundant predictive information about unsupported claims beyond competent response-level and model-internal baselines?

## Primary hypothesis H11-DE

On non-final Development/Validation data, a frozen claim-decomposition + source-entailment detector will improve unsupported-claim discrimination relative to a matched response-level entailment baseline at comparable source access, while preserving an explicitly reported coverage/cost profile.

The hypothesis is falsified if the claim-level method fails to improve the preregistered discrimination metric, or if any gain disappears under matched source access / compute accounting, or if the gain is confined to a single task stratum without passing the declared cross-stratum robustness rule.

## Secondary hypothesis H11-DE-COMP

Claim-level source-entailment evidence and the separately frozen TDI-11.4 uncertainty signal may provide complementary predictive information. This secondary hypothesis may be tested only after each component is frozen independently. A combined model must not be tuned on final data.

## Experimental arms

The exact implementations, model artifacts and thresholds remain unresolved until the TDI-11.2 freeze is complete. Before execution, all arms must be bound to immutable identities.

- B0: no-detector reporting baseline.
- B1: response-level source-entailment score using the same admissible source material as the claim-level arm.
- B2: claim-decomposition + source-entailment detector.
- B3: separately preregistered model-internal uncertainty detector, when available and admissible.
- B4: frozen combination of B2 and B3, only for the secondary complementarity analysis.

No arm may receive additional source text, hidden oracle truth, final labels, answer-bearing filenames, or retrieval metadata unavailable to the others under the matched comparison.

## Claim decomposition contract

Before any scored run, freeze:

1. the decomposition implementation and immutable artifact/revision;
2. the exact definition of an atomic scored claim;
3. deterministic handling of conjunctions, qualifications, quantities and coreference;
4. handling of non-factual or unverifiable spans;
5. maximum claims per response and overflow behavior;
6. typed rejection rules for decomposition failure;
7. whether decomposition is rule-based, model-based or hybrid;
8. all prompts/templates if a model is used;
9. stochasticity settings and seed policy;
10. claim-to-response aggregation rule.

Decomposition errors must be measured explicitly rather than silently relabelled as factuality errors.

## Source-entailment contract

Before execution, freeze:

- entailment model/checkpoint and tokenizer identities;
- source chunking algorithm and chunk-size/overlap parameters;
- candidate-chunk selection rule;
- entailment labels and score interpretation;
- contradiction handling;
- maximum source tokens inspected per response;
- batching/runtime policy;
- deterministic tie-breaking;
- missing-source and no-evidence behavior;
- resource accounting.

Source retrieval/selection must be causally available and must not use hidden complete-world truth or final annotations.

## Data discipline

- Use only Development and Validation populations authorized by the TDI-11.2 freeze.
- Preserve all existing TDI-7.2/TDI-8.2 protected boundaries.
- No final dataset, final seed list, final result payload or confirmatory evaluation may be created by this study.
- Threshold selection, aggregation rules and combination weights are Development-only. Validation is reserved for the preregistered one-shot evaluation of the frozen configuration.
- Any materially changed detector after observing Validation results receives a new versioned hypothesis identifier.

## Primary metrics

Report at claim level and response level where mathematically defined:

- AUROC for unsupported-claim discrimination;
- AUPRC with the unsupported-claim prevalence reported;
- Brier score or another preregistered proper scoring rule when the detector exposes probabilities;
- calibration error with binning policy frozen before use;
- false-negative rate at one or more Development-frozen operating points;
- false-positive rate / false-abstention impact when used for selective control;
- answer/claim coverage;
- useful-task performance;
- regression of originally supported/correct claims;
- source tokens inspected, model calls, wall-clock time and peak memory when measurable under a qualified environment.

No single metric may erase the risk/coverage/cost trade-off.

## Stratification

At minimum, preserve the existing TDI-11.4 stratification discipline across:

- model;
- task family;
- hallucination / unsupported-claim type when labels permit;
- response-length stratum;
- source-evidence availability stratum.

Aggregate metrics must not replace stratum-level reporting.

## Baseline fairness

B1 and B2 must receive identical admissible source material and the same retrieval budget unless a separately reported ablation intentionally changes that factor. Any claim that decomposition itself adds value requires holding source access constant.

If B2 uses more inference calls or source tokens, the additional cost must be reported and the result must not be presented as a free improvement.

## Analysis plan

1. Verify all provenance, split and immutable-identity guards before scoring.
2. Measure B1 and B2 on Development using frozen metric code.
3. Freeze any operating thresholds permitted by the protocol.
4. Evaluate once on Validation under the frozen configuration.
5. Report paired differences and uncertainty intervals using a method frozen before Validation scoring.
6. Report all declared strata, including negative strata.
7. Test B2 vs B3 only after B3 is independently frozen.
8. Test B4 only as a secondary analysis; it cannot redefine the primary H11-DE outcome.

## Falsification / stop conditions

Stop and record a negative or inconclusive result if any of the following occurs:

- decomposition cannot be made deterministic/reproducible under the declared contract;
- leakage or source-budget mismatch is detected;
- B2 does not improve the frozen primary discrimination criterion over B1 on Validation;
- an apparent improvement is attributable to additional source access or unaccounted compute;
- calibration degrades beyond a preregistered tolerance without a compensating and explicitly accepted risk/coverage benefit;
- the result is driven by a post-hoc subset;
- required provenance is incomplete.

Do not rescue a failed hypothesis by changing the detector, threshold or aggregation rule under the same hypothesis id.

## Transferability gate

Only if a non-final result survives the above controls may reusable pieces be considered for promotion:

- generic claim decomposition/evidence schema -> SciRust only if it becomes domain-independent infrastructure;
- external evidence records -> ADA/Forge only through versioned artifact contracts, never runtime coupling to TDI internals;
- selective verification policy -> ElasticXxx only after independent runtime invariant validation;
- formal claim verification -> ProofLab only for propositions that can be translated to a formal kernel-checked statement; NLI entailment must never be treated as formal proof.

## Non-claims

This preregistration does not claim that:

- decomposed entailment is novel within Memorithm;
- entailment scores are truth;
- a black-box detector universally dominates model-internal uncertainty;
- source-grounded factuality detection solves open-domain hallucination;
- any result transfers to final or deployment conditions without separate evidence.
