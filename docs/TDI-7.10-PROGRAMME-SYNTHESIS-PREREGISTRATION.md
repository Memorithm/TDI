# TDI-7.10 — Programme synthesis and frozen archive

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-9:

> Can the accumulated findings from TDI-7.3 through TDI-7.9 be synthesized into a coherent frozen archive that preserves the full experimental lineage, and does the synthesis reveal any systematic patterns or contradictions across the programme?

The confirmatory object is programme-level synthesis, not new experiments. This stage produces the frozen archive.

## 2. Scope

TDI-7.10 freezes the scientific synthesis protocol only. It contains no new confirmatory evaluator result, no final holdout output, no language-model training, and no FLAT-ATTENTION performance claim.

The synthesis is a meta-analysis of frozen TDI-7.3 through TDI-7.9 results. No new data is generated.

## 3. Confirmatory task families

Exactly two task families are confirmatory, inherited from TDI-7.0: associative recall and copy.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds. Four disjoint seed ranges: training, development, validation, final holdout.

## 5. Reference semantics

The synthesis uses the same deterministic attention semantics as TDI-7.5 through TDI-7.9. Each synthesis arm references frozen semantics with scalar/reference oracles.

## 6. Synthesis taxonomy

### 6.1 Cross-stage pattern synthesis
Synthesizes findings across all TDI-7.3 through TDI-7.9 stages. Identifies systematic patterns in recovery dynamics, discrimination, transfer, and robustness.

### 6.2 Contradiction detection
Tests whether any findings from different stages contradict each other. Documents resolved and unresolved contradictions.

### 6.3 Stability trajectory
Tracks how stability verdicts evolve from TDI-7.6 (ablations) through TDI-7.9 (robustness). Produces a stability trajectory summary.

### 6.4 Archive completeness verification
Verifies that the frozen archive contains all required artifacts: preregistrations, decision records, validation scripts, bounded examples, and test results.

## 7. Synthesis diagnostics

TDI-7.10 adds programme-level diagnostics:

- cross-stage pattern consistency score;
- contradiction index (number of unresolved contradictions);
- stability trajectory summary;
- archive completeness checklist;
- synthesis verdict: coherent, coherent-with-caveats, or fragmented.

## 8. Null and negative results

Synthesis failures, unresolved contradictions, and fragmented findings are preserved and never renumbered away. A finding that the programme is fragmented is a valid H-AI-9 outcome.

## 9. Information-theoretic interpretation limits

Synthesis results are not interpreted as evidence about MMI, causation, or human cognitive mechanisms. They are programme-level quality assessments.

## 10. Relationship to TDI-7.0 through TDI-7.9

TDI-7.10 inherits the frozen protocols of all prior TDI-7.x stages. It produces the frozen archive that preserves the full experimental lineage.

## 11. Evidence handoff

Must follow the evidence handoff format in `docs/TDI-7-EVIDENCE-HANDOFF.md`.

## 12. Frozen decision records

- `docs/TDI-7.10-POPULATION-DECISION.toml`
- `docs/TDI-7.10-SELECTION-DECISION.toml`
- `docs/TDI-7.10-REJECTION-POLICY.toml`

## 13. Non-claims

Does not establish neural transfer, production readiness, or that the synthesis applies beyond the frozen TDI-7.x protocol boundaries.

## 14. Determinism and provenance

Repeated execution with same inputs must reproduce populations. Bit-exact cross-platform identity not assumed.

## 15. Final-holdout lock

Explicit final-run confirmation guard required. Token must not be supplied by CI or tests.

## 16. Required reporting

Cross-stage: pattern summary, contradiction index, stability trajectory, archive completeness. Aggregate: synthesis verdict, programme quality assessment, provenance.

## 17. Relationship to ITD Simulator

Synthesis ladder: static -> spectral -> ITD -> TDI -> per-architecture TDI -> transfer -> extensions -> robustness -> synthesis. The synthesis produces the evidence package for ITD Simulator consumption.

## 18. Relationship to FLAT-ATTENTION

Does not itself select a FLAT semantic. References frozen semantics from prior stages. The synthesis documents which semantics passed mechanistic experiments.

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
- TDI-7.10: H-AI-9 programme synthesis and frozen archive

## 20. Non-claims

Does not establish reality, neural transferability, or production readiness. The synthesis is a quality assessment of the frozen TDI-7.x programme.
