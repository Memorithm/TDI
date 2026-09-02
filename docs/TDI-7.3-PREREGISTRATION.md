# TDI-7.3 — Intervention heterogeneity and coupling stability preregistration

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-2:

> On deterministic associative-recall and copy tasks, does intervention-location heterogeneity produce systematically different recovery dynamics, and are those dynamics stably coupled across the frozen TDI-7.1 evaluator configuration?

The confirmatory object is interaction and stability, not overall effectiveness. Location-specific effects are primary; any overall signal is secondary.

## 2. Scope

TDI-7.3 freezes the scientific protocol only. It contains no confirmatory evaluator result, no final holdout output, no language-model training, and no FLAT-ATTENTION performance claim.

## 3. Confirmatory task families

Exactly two task families are confirmatory, inherited from TDI-7.0: associative recall and copy.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds. Four disjoint seed ranges: training, development, validation, final holdout.

## 5. Reference semantics

The evaluator uses the same deterministic attention semantic as TDI-7.0 and TDI-7.1.

## 6. Intervention taxonomy

Single-site state perturbations that leave the task target unchanged. At least two intervention locations.

## 7. Early TDI recovery descriptors

TDI features computed strictly before the target evaluation depth. Minimum descriptor set from TDI-7.1 is retained.

## 8. Coupling stability diagnostics

TDI-7.3 adds diagnostics that compare recovery trajectories across intervention locations: per-site recovery overlap, absolute/relative difference, site-pair stability summary.

## 9. Null and negative results

Negative, null, inconclusive, or unstable coupling results are preserved and never renumbered away.

## 10. Heterogeneity interpretation limits

Not interpreted as evidence about Transformer architectures or human cognitive mechanisms.

## 11. Relationship to TDI-7.0 and TDI-7.1

TDI-7.3 inherits the frozen TDI-7.0 protocol and bounded TDI-7.1 evaluator.

## 12. Evidence handoff

Must follow the evidence handoff format in `docs/TDI-7-EVIDENCE-HANDOFF.md`.

## 13. Frozen decision records

- `docs/TDI-7.3-POPULATION-DECISION.toml`
- `docs/TDI-7.3-SELECTION-DECISION.toml`
- `docs/TDI-7.3-REJECTION-POLICY.toml`

## 14. Non-claims

Does not establish transferability, select a FLAT semantic, or provide GPU performance gains.

## 15. Determinism and provenance

Repeated execution with same inputs must reproduce populations. Bit-exact cross-platform identity not assumed.

## 16. Final-holdout lock

Explicit final-run confirmation guard required. Token must not be supplied by CI or tests.

## 17. Required reporting

B0/B1 MSE, relative reduction, 95% interval, verdict, rejection counts, coupling stability summary, provenance.

## 18. Relationship to ITD Simulator

Ablation ladder: static -> ITD -> TDI -> ITD+TDI. TDI and ITD remain separately measurable.

## 19. Relationship to FLAT-ATTENTION

Does not itself select a FLAT semantic. FLAT promotion requires frozen specification and oracle.

## 20. TDI-7.x programme map

- TDI-7.0: H-AI-1 protocol
- TDI-7.1: bounded evaluator
- TDI-7.2: H-AI-1 holdout executed
- TDI-7.3: H-AI-2 heterogeneity/coupling
- TDI-7.4: H-AI-3 joint-information
- TDI-7.5+: FLAT semantic comparisons

## 21. Non-claims

Does not establish reality, transferability, or usefulness outside the synthetic TDI protocol.