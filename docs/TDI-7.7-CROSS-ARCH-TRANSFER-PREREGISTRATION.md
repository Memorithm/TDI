# TDI-7.7 — Cross-architecture transfer experiments

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-6:

> On deterministic associative-recall and copy tasks, do TDI recovery descriptors and ablation findings from TDI-7.3 through TDI-7.6 transfer across deterministic attention architectures, and is the transfer bidirectional or asymmetric?

The confirmatory object is cross-architecture transfer stability, not raw performance. Transfer failure is a valid result.

## 2. Scope

TDI-7.7 freezes the scientific protocol only. It contains no confirmatory evaluator result, no final holdout output, no language-model training, and no neural architecture claim.

Transfer is tested only between deterministic architectures with frozen mathematical specifications. No neural network weights are used.

## 3. Confirmatory task families

Exactly two task families are confirmatory, inherited from TDI-7.0: associative recall and copy.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds. Four disjoint seed ranges: training, development, validation, final holdout.

## 5. Reference architectures

The evaluator compares at least two deterministic attention architectures. Each must have:
- a frozen mathematical specification;
- a scalar/reference oracle with hand-deriveable fixtures;
- documented recovery dynamics from TDI-7.3 through TDI-7.6.

Candidate architectures include the TDI-7.5 FLAT semantics and additional deterministic mixing patterns (identity, strided, block-diagonal).

## 6. Transfer taxonomy

### 6.1 Forward transfer
Train recovery model on architecture A, evaluate on architecture B without retraining. Measures whether recovery descriptors learned on one architecture apply to another.

### 6.2 Reverse transfer
Train on B, evaluate on A. Tests whether transfer is bidirectional or asymmetric.

### 6.3 Joint training
Train on both A and B simultaneously. Measures whether joint training improves transfer over single-architecture training.

### 6.4 Architecture distance
Quantifies the distance between architectures using their recovery profiles. Tests whether architectural similarity predicts transfer success.

## 7. Transfer diagnostics

- transfer efficiency ratio (target performance / source performance);
- transfer asymmetry index (forward - reverse transfer);
- joint training benefit (joint - max(single));
- architecture distance correlation;
- transfer verdict: full, partial, asymmetric, or failed.

## 8. Null and negative results

Failed transfer, asymmetric transfer, or no joint training benefit are preserved and never renumbered away. Negative transfer is a valid H-AI-6 outcome.

## 9. Information-theoretic interpretation limits

Transfer results are not interpreted as evidence about MMI, causation, or human cognitive mechanisms. They are mechanistic transfer tests between deterministic reference architectures.

## 10. Relationship to TDI-7.0 through TDI-7.6

TDI-7.7 inherits the frozen protocols of prior TDI-7.x stages. It applies the ablation findings from TDI-7.6 to test cross-architecture transfer.

## 11. Evidence handoff

Must follow the evidence handoff format in `docs/TDI-7-EVIDENCE-HANDOFF.md`.

## 12. Frozen decision records

- `docs/TDI-7.7-POPULATION-DECISION.toml`
- `docs/TDI-7.7-SELECTION-DECISION.toml`
- `docs/TDI-7.7-REJECTION-POLICY.toml`

## 13. Non-claims

Does not establish that TDI recovery transfers to neural architectures, that any architecture is superior, or that synthetic transfer implies real-world transfer. Neural transfer claims require separate experiments with learned models.

## 14. Determinism and provenance

Repeated execution with same inputs must reproduce populations. Bit-exact cross-platform identity not assumed.

## 15. Final-holdout lock

Explicit final-run confirmation guard required. Token must not be supplied by CI or tests.

## 16. Required reporting

Per transfer direction: source architecture, target architecture, transfer efficiency ratio, transfer asymmetry index, joint training benefit, architecture distance, verdict. Aggregate: cross-architecture transfer stability verdict, most transferable factor, provenance.

## 17. Relationship to ITD Simulator

Transfer ladder extends the TDI-7.6 ladder: static -> spectral -> ITD -> TDI -> per-architecture TDI -> transfer. Each architecture is a separate arm. Transfer is a separate diagnostic layer.

## 18. Relationship to FLAT-ATTENTION

Does not itself select a FLAT semantic. Each architecture arm uses a frozen semantic. New architectures require separate preregistration with mathematical specification and oracle.

## 19. TDI-7.x programme map

- TDI-7.0: H-AI-1 protocol
- TDI-7.1: bounded evaluator
- TDI-7.2: H-AI-1 holdout executed
- TDI-7.3: H-AI-2 heterogeneity/coupling
- TDI-7.4: H-AI-3 joint information/synergy
- TDI-7.5: H-AI-4 FLAT semantic discrimination
- TDI-7.6: H-AI-5 evidence-justified ablations
- TDI-7.7: H-AI-6 cross-architecture transfer
- TDI-7.8+: evidence-justified extensions

## 20. Non-claims

Does not establish reality, neural transferability, or usefulness outside the synthetic TDI protocol. Transfer findings are conditional on the frozen deterministic architectures.
