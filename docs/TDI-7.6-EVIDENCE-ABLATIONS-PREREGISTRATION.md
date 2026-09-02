# TDI-7.6 — Evidence-justified ablations in attention recovery

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-5:

> On deterministic associative-recall and copy tasks, which experimental factors most strongly modulate the discriminative value of TDI recovery descriptors, and are those factors stable across the frozen TDI-7.5 semantic comparison?

The confirmatory object is ablation stability, not discovery of a new descriptor. Only ablations justified by TDI-7.3 through TDI-7.5 evidence are confirmatory.

## 2. Scope

TDI-7.6 freezes the scientific protocol only. It contains no confirmatory evaluator result, no final holdout output, no language-model training, and no FLAT-ATTENTION performance claim.

An ablation is confirmatory only if it is:
1. explicitly motivated by a frozen result from TDI-7.3, TDI-7.4, or TDI-7.5;
2. preregistered with a frozen protocol before holdout access;
3. evaluated on the same confirmatory task families.

## 3. Confirmatory task families

Exactly two task families are confirmatory, inherited from TDI-7.0: associative recall and copy.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds. Four disjoint seed ranges: training, development, validation, final holdout.

## 5. Reference semantics

The evaluator uses the same deterministic attention semantics as TDI-7.5. Each ablation arm uses a frozen semantic with a scalar/reference oracle.

## 6. Justified ablation taxonomy

The following ablation categories are evidence-justified by prior TDI-7.x stages:

### 6.1 Observation horizon ablation
Motivated by: TDI-7.4 (longer horizons), TDI-7.5 (discrimination at multiple depths).
Varies the early observation horizon while keeping the target depth fixed. Tests whether discriminative value is horizon-stable or horizon-dependent.

### 6.2 Intervention amplitude ablation
Motivated by: TDI-7.3 (heterogeneity across sites), TDI-7.4 (joint information).
Varies the intervention magnitude while keeping the location fixed. Tests whether recovery descriptors are amplitude-sensitive at fixed sites.

### 6.3 Static spectral depth ablation
Motivated by: TDI-7.5 (insufficiency indicator).
Varies the set of static spectral controls to test which static descriptors are necessary before recovery adds value. Maps the boundary of static sufficiency.

### 6.4 Cross-semantic transfer ablation
Motivated by: TDI-7.5 (semantic discrimination).
Tests whether a recovery model trained on one FLAT semantic transfers discrimination to another without retraining. Measures semantic-conditional transfer.

### 6.5 Feature combination ablation
Motivated by: TDI-7.4 (joint information diagnostics).
Systematically removes feature groups (static-only, recovery-only, joint) to quantify the marginal contribution of each block.

## 7. Ablation stability diagnostics

TDI-7.6 adds diagnostics that test whether ablation effects are stable:

- per-ablation recovery profile difference;
- cross-ablation rank correlation of semantic discrimination;
- ablation interaction summary (does ablation A change the effect of ablation B?);
- stability verdict: robust, sensitive, or unstable.

## 8. Null and negative results

Negative, null, or unstable ablation results are preserved and never renumbered away. A result showing that an ablation effect is not stable across conditions is a valid H-AI-5 outcome.

## 9. Information-theoretic interpretation limits

Ablation results are not interpreted as evidence about MMI, causation, or human cognitive mechanisms. They are mechanistic stability tests for an experimental method.

## 10. Relationship to TDI-7.0 through TDI-7.5

TDI-7.6 inherits the frozen protocols of prior TDI-7.x stages. It adds systematic ablation on top of the semantic discrimination from TDI-7.5. Each ablation arm must cite its motivating evidence from a prior stage.

## 11. Evidence handoff

Must follow the evidence handoff format in `docs/TDI-7-EVIDENCE-HANDOFF.md`.

## 12. Frozen decision records

- `docs/TDI-7.6-POPULATION-DECISION.toml`
- `docs/TDI-7.6-SELECTION-DECISION.toml`
- `docs/TDI-7.6-REJECTION-POLICY.toml`

## 13. Non-claims

Does not establish that any ablation finding transfers to neural architectures, that TDI recovery is universally useful, or that any FLAT semantic is superior. Transfer claims require separate cross-architecture experiments.

## 14. Determinism and provenance

Repeated execution with same inputs must reproduce populations. Bit-exact cross-platform identity not assumed.

## 15. Final-holdout lock

Explicit final-run confirmation guard required. Token must not be supplied by CI or tests.

## 16. Required reporting

Per ablation arm: motivating evidence reference, B0/B1 MSE, relative reduction, 95% interval, verdict, recovery profile, ablation effect size, stability summary. Aggregate: cross-ablation stability verdict, most-influential factor, provenance.

## 17. Relationship to ITD Simulator

Ablation ladder extends the TDI-7.5 ladder: static -> spectral -> ITD -> TDI -> per-semantic TDI -> ablated TDI. Each ablation is a separate arm. ITD and TDI remain separately measurable.

## 18. Relationship to FLAT-ATTENTION

Does not itself select a FLAT semantic. Each ablation arm uses a frozen semantic from TDI-7.5. New semantics require separate preregistration.

## 19. TDI-7.x programme map

- TDI-7.0: H-AI-1 protocol
- TDI-7.1: bounded evaluator
- TDI-7.2: H-AI-1 holdout executed
- TDI-7.3: H-AI-2 heterogeneity/coupling
- TDI-7.4: H-AI-3 joint information/synergy
- TDI-7.5: H-AI-4 FLAT semantic discrimination
- TDI-7.6: H-AI-5 evidence-justified ablations
- TDI-7.7+: cross-architecture transfer experiments

## 20. Non-claims

Does not establish reality, transferability, or usefulness outside the synthetic TDI protocol. Ablation findings are conditional on the frozen semantics and task families.
