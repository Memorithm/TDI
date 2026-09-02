# TDI-7.8 — Evidence-justified extensions of the TDI-7.x programme

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-7:

> On deterministic associative-recall and copy tasks, do the accumulated findings from TDI-7.3 through TDI-7.7 support preregistered extensions to longer horizons, richer intervention taxonomies, and multi-site perturbations, and do these extensions preserve the stability verdicts from TDI-7.6?

The confirmatory object is evidence-justified extension stability. Only extensions motivated by frozen TDI-7.3 through TDI-7.7 findings are confirmatory.

## 2. Scope

TDI-7.8 freezes the scientific protocol only. It contains no confirmatory evaluator result, no final holdout output, no language-model training, and no FLAT-ATTENTION performance claim.

An extension is confirmatory only if:
1. it is explicitly motivated by a frozen ablation or transfer finding from TDI-7.3 through TDI-7.7;
2. the motivating evidence is cited in the extension protocol;
3. the extension is preregistered before holdout access.

## 3. Confirmatory task families

Exactly two task families are confirmatory, inherited from TDI-7.0: associative recall and copy.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds. Four disjoint seed ranges: training, development, validation, final holdout.

## 5. Reference semantics

The evaluator uses the same deterministic attention semantics as TDI-7.5 through TDI-7.7. Each extension arm uses a frozen semantic with a scalar/reference oracle.

## 6. Evidence-justified extension taxonomy

### 6.1 Extended horizon extension
Motivated by: TDI-7.4 (longer horizons), TDI-7.6 (horizon ablation stability verdict).
Extends observation horizons beyond the TDI-7.4/TDI-7.5 maximum of 4 depths. Tests whether joint information findings stabilize or destabilize at longer horizons. Maximum horizon is preregistered per architecture.

### 6.2 Multi-site intervention extension
Motivated by: TDI-7.3 (heterogeneity across sites), TDI-7.6 (amplitude ablation findings).
Applies interventions at multiple sites simultaneously. Tests whether single-site findings from TDI-7.3 compose predictably or produce interaction effects.

### 6.3 Cross-length transfer extension
Motivated by: TDI-7.7 (cross-architecture transfer verdict).
Tests whether recovery models trained on one sequence length transfer to different lengths. Measures length-conditional transfer efficiency.

### 6.4 Composite feature extension
Motivated by: TDI-7.4 (joint information), TDI-7.6 (feature combination ablation findings).
Combines feature groups from multiple prior stages into a single diagnostic. Tests whether the composite descriptor improves discrimination beyond any single-stage descriptor.

## 7. Extension stability diagnostics

TDI-7.8 adds diagnostics that test whether extensions preserve prior stability:

- extension vs. baseline stability comparison;
- horizon-length sensitivity of joint information;
- multi-site interaction summary;
- cross-length transfer efficiency;
- composite feature marginal contribution;
- extension verdict: stable, extended-stable, or unstable.

## 8. Null and negative results

Extensions that do not improve over baseline, produce unstable results, or fail to transfer are preserved and never renumbered away. A result showing that an evidence-justified extension does not work is a valid H-AI-7 outcome.

## 9. Information-theoretic interpretation limits

Extension results are not interpreted as evidence about MMI, causation, or human cognitive mechanisms. They are mechanistic tests of whether prior findings support further extension.

## 10. Relationship to TDI-7.0 through TDI-7.7

TDI-7.8 inherits the frozen protocols of all prior TDI-7.x stages. It tests whether the accumulated evidence supports further extension. Each extension arm must cite its motivating evidence.

## 11. Evidence handoff

Must follow the evidence handoff format in `docs/TDI-7-EVIDENCE-HANDOFF.md`.

## 12. Frozen decision records

- `docs/TDI-7.8-POPULATION-DECISION.toml`
- `docs/TDI-7.8-SELECTION-DECISION.toml`
- `docs/TDI-7.8-REJECTION-POLICY.toml`

## 13. Non-claims

Does not establish that extensions transfer to neural architectures, that longer horizons are always better, or that multi-site interventions are superior. Each extension finding is conditional on the frozen motivating evidence.

## 14. Determinism and provenance

Repeated execution with same inputs must reproduce populations. Bit-exact cross-platform identity not assumed.

## 15. Final-holdout lock

Explicit final-run confirmation guard required. Token must not be supplied by CI or tests.

## 16. Required reporting

Per extension arm: motivating evidence reference, baseline comparison, extension effect size, stability verdict, horizon sensitivity. Aggregate: cross-extension stability verdict, most promising extension direction, provenance.

## 17. Relationship to ITD Simulator

Extension ladder extends the TDI-7.7 ladder: static -> spectral -> ITD -> TDI -> per-architecture TDI -> transfer -> extensions. Each extension is a separate arm. ITD and TDI remain separately measurable.

## 18. Relationship to FLAT-ATTENTION

Does not itself select a FLAT semantic. Each extension arm uses a frozen semantic from prior stages. New semantics require separate preregistration.

## 19. TDI-7.x programme map

- TDI-7.0: H-AI-1 protocol
- TDI-7.1: bounded evaluator
- TDI-7.2: H-AI-1 holdout executed
- TDI-7.3: H-AI-2 heterogeneity/coupling
- TDI-7.4: H-AI-3 joint information/synergy
- TDI-7.5: H-AI-4 FLAT semantic discrimination
- TDI-7.6: H-AI-5 evidence-justified ablations
- TDI-7.7: H-AI-6 cross-architecture transfer
- TDI-7.8: H-AI-7 evidence-justified extensions
- TDI-7.9+: calibration and robustness replication

## 20. Non-claims

Does not establish reality, transferability, or usefulness outside the synthetic TDI protocol. Extension findings are conditional on the frozen architectures, task families, and motivating evidence.
