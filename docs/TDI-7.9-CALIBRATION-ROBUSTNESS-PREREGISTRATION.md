# TDI-7.9 — Calibration and robustness replication

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-8:

> On deterministic associative-recall and copy tasks, are the accumulated findings from TDI-7.3 through TDI-7.8 robust to calibration variations in numerical precision, seed perturbation, and protocol re-execution, and can they be replicated under independently varied conditions?

The confirmatory object is robustness replication, not discovery. A robustness failure is a valid result that must be reported.

## 2. Scope

TDI-7.9 freezes the scientific protocol only. It contains no confirmatory evaluator result, no final holdout output, no language-model training, and no FLAT-ATTENTION performance claim.

Robustness is tested only within the frozen TDI-7.x protocol boundaries. No neural network calibration is performed.

## 3. Confirmatory task families

Exactly two task families are confirmatory, inherited from TDI-7.0: associative recall and copy.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds. Four disjoint seed ranges: training, development, validation, final holdout.

## 5. Reference semantics

The evaluator uses the same deterministic attention semantics as TDI-7.5 through TDI-7.8. Each robustness arm uses a frozen semantic with a scalar/reference oracle.

## 6. Calibration and robustness taxonomy

### 6.1 Numerical precision robustness
Motivated by: TDI-7.5 (semantic discrimination), TDI-7.8 (extended horizons).
Tests whether findings are robust to variations in floating-point accumulation order and tolerance. Measures precision sensitivity of recovery saturation.

### 6.2 Seed perturbation robustness
Motivated by: TDI-7.3 (heterogeneity), TDI-7.7 (cross-architecture transfer).
Tests whether findings hold under small perturbations to generator seeds. Measures seed sensitivity of ablation and transfer verdicts.

### 6.3 Protocol re-execution robustness
Motivated by: TDI-7.6 (ablation stability), TDI-7.8 (extension stability).
Re-runs the full TDI-7.3 through TDI-7.8 pipeline with identical inputs and verifies bit-exact or tolerance-bound reproducibility.

### 6.4 Cross-parameter robustness
Motivated by: TDI-7.4 (joint information), TDI-7.5 (semantic discrimination).
Tests whether findings hold when intervention amplitudes, observation depths, or other protocol parameters are varied within preregistered bounds.

## 7. Robustness diagnostics

TDI-7.9 adds diagnostics that test calibration and replication:

- numerical precision sensitivity score;
- seed perturbation stability index;
- re-execution reproducibility verdict (bit-exact, tolerance-bound, divergent);
- cross-parameter robustness summary;
- overall robustness verdict: robust, bounded-robust, or fragile.

## 8. Null and negative results

Robustness failures, calibration divergences, and non-replicable results are preserved and never renumbered away. A finding that a prior result is fragile or non-replicable is a valid H-AI-8 outcome.

## 9. Information-theoretic interpretation limits

Robustness results are not interpreted as evidence about MMI, causation, or human cognitive mechanisms. They are calibration tests for an experimental method.

## 10. Relationship to TDI-7.0 through TDI-7.8

TDI-7.9 inherits the frozen protocols of all prior TDI-7.x stages. It tests whether the accumulated evidence from TDI-7.3 through TDI-7.8 is robust and replicable.

## 11. Evidence handoff

Must follow the evidence handoff format in `docs/TDI-7-EVIDENCE-HANDOFF.md`.

## 12. Frozen decision records

- `docs/TDI-7.9-POPULATION-DECISION.toml`
- `docs/TDI-7.9-SELECTION-DECISION.toml`
- `docs/TDI-7.9-REJECTION-POLICY.toml`

## 13. Non-claims

Does not establish neural calibration, real-world robustness, or that synthetic robustness implies production robustness. Each robustness finding is conditional on the frozen protocol boundaries.

## 14. Determinism and provenance

Repeated execution with same inputs must reproduce populations. Bit-exact cross-platform identity not assumed. Tolerance bounds are preregistered.

## 15. Final-holdout lock

Explicit final-run confirmation guard required. Token must not be supplied by CI or tests.

## 16. Required reporting

Per robustness arm: arm label, prior finding tested, robustness metric, tolerance bound, verdict. Aggregate: overall robustness verdict, most fragile finding, most robust finding, replication summary, provenance.

## 17. Relationship to ITD Simulator

Robustness ladder extends the TDI-7.8 ladder: static -> spectral -> ITD -> TDI -> per-architecture TDI -> transfer -> extensions -> robustness. Each robustness check is a separate diagnostic layer.

## 18. Relationship to FLAT-ATTENTION

Does not itself select a FLAT semantic. Each robustness arm uses a frozen semantic from prior stages. New semantics require separate preregistration.

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
- TDI-7.9: H-AI-8 calibration and robustness replication
- TDI-7.10+: programme synthesis and frozen archive

## 20. Non-claims

Does not establish reality, neural robustness, or production readiness. Robustness findings are conditional on the frozen deterministic protocol and its explicitly bounded calibration variations.
