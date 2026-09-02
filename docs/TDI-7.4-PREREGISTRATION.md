# TDI-7.4 — Long-horizon joint information and synergy preregistration

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-3:

> On deterministic associative-recall and copy tasks, do early intervention-conditioned recovery descriptors and static attention diagnostics jointly encode more information about later retrieval deficit than either source alone?

The confirmatory object is joint information and synergy, not raw improvement.

## 2. Scope

TDI-7.4 freezes the scientific protocol only. It contains no confirmatory evaluator result, no final holdout output, no language-model training, and no FLAT-ATTENTION performance claim.

The first implementation remains a cheap deterministic mechanistic gate.

## 3. Confirmatory task families

Exactly two task families are confirmatory, inherited from TDI-7.0: associative recall and copy.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds. Four disjoint seed ranges: training, development, validation, final holdout.

## 5. Reference semantics

The evaluator uses the same deterministic attention semantic as TDI-7.0 and TDI-7.1.

## 6. Intervention taxonomy

Single-site state perturbations that leave the task target unchanged. At least two intervention locations.

## 7. Joint information diagnostics

TDI-7.4 adds joint information diagnostics between static attention features and early TDI recovery features. At minimum:

- mutual information between static and recovery features;
- synergy and redundancy diagnostics;
- joint predictive value above either source alone.

## 8. Long-horizon protocol

The protocol extends observation horizons compared to TDI-7.1, but stays strictly before the target evaluation depth.

## 9. Null and negative results

Negative, null, inconclusive, or low-synergy results are preserved and never renumbered away. A null synergy result is a valid H-AI-3 outcome.

## 10. Information-theoretic interpretation limits

Joint information diagnostics are not interpreted as evidence about MMI, causation, or human cognitive mechanisms.

## 11. Relationship to TDI-7.0, TDI-7.1, TDI-7.3

TDI-7.4 inherits the frozen protocols of prior TDI-7.x stages.

## 12. Evidence handoff

Must follow the evidence handoff format in `docs/TDI-7-EVIDENCE-HANDOFF.md`.

## 13. Frozen decision records

- `docs/TDI-7.4-POPULATION-DECISION.toml`
- `docs/TDI-7.4-SELECTION-DECISION.toml`
- `docs/TDI-7.4-REJECTION-POLICY.toml`

## 14. Non-claims

Does not establish that joint information exists, transfers, or is useful outside the synthetic TDI protocol.

## 15. Determinism and provenance

Repeated execution with same inputs must reproduce populations. Bit-exact cross-platform identity not assumed.

## 16. Final-holdout lock

Explicit final-run confirmation guard required. Token must not be supplied by CI or tests.

## 17. Required reporting

Static-only baseline, recovery-only baseline, joint diagnostic, B0/B1 MSE, relative reduction, 95% interval, verdict, provenance.

## 18. Relationship to ITD Simulator

Ablation ladder: static -> ITD -> TDI -> ITD+TDI -> joint. TDI and ITD remain separately measurable.

## 19. Relationship to FLAT-ATTENTION

Does not itself select a FLAT semantic. FLAT promotion requires frozen specification and oracle.

## 20. TDI-7.x programme map

- TDI-7.0: H-AI-1 protocol
- TDI-7.1: bounded evaluator
- TDI-7.2: H-AI-1 holdout executed
- TDI-7.3: H-AI-2 heterogeneity/coupling
- TDI-7.4: H-AI-3 joint information/synergy
- TDI-7.5+: FLAT semantic comparisons

## 21. Non-claims

Does not establish reality, transferability, or usefulness outside the synthetic TDI protocol.