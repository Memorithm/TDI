# TDI-7.5 — Static spectral insufficiency and FLAT semantic discrimination

Status: **frozen protocol candidate** for TDI issue #57.

TDI-7.x is the attention/memory continuation of the TDI research programme. It does not revise, overwrite, or reinterpret TDI-1 through TDI-6.x. Historical finite-state results remain valid only in their tested domains.

## 1. Confirmatory question

Primary hypothesis H-AI-4:

> On deterministic associative-recall and copy tasks, do different FLAT attention semantics produce systematically different recovery dynamics, and can TDI recovery descriptors discriminate between semantics beyond static spectral diagnostics?

The confirmatory object is semantic discrimination via recovery dynamics, not raw performance. A semantic that performs equally on a task may still produce distinguishable recovery signatures.

## 2. Scope

TDI-7.5 freezes the scientific protocol only. It contains no confirmatory evaluator result, no final holdout output, no language-model training, and no FLAT-ATTENTION GPU performance claim.

Each candidate semantic requires its own frozen mathematical specification, scalar/reference oracle, quality evidence, cost evidence, and failure-mode record before it enters a confirmatory comparison.

## 3. Confirmatory task families

Exactly two task families are confirmatory, inherited from TDI-7.0: associative recall and copy.

## 4. Split discipline

All examples are synthetic and generated from explicit integer seeds. Four disjoint seed ranges: training, development, validation, final holdout.

## 5. Reference semantics

The evaluator compares at least two deterministic FLAT attention semantics. Each semantic must have:

- a frozen mathematical specification independently of the evaluator;
- a scalar/reference oracle with hand-deriveable fixtures;
- an explicit numerical policy (accumulation type, tolerance, rounding);
- documented invariances and failure modes.

Candidate semantic families include: standard softmax, differential/signed attention, Toeplitz/relative structured mixing, prolate/spectral concentration, ground-state/Green-kernel mixing, recurrent/delta-memory mechanisms, and hybrid mechanisms.

No semantic is promoted from toy-task success alone. A semantic must survive mechanistic experiments before GPU specialization is treated as a priority.

## 6. Static spectral controls

Before TDI recovery descriptors are compared across semantics, the evaluator must first establish whether static spectral/operator diagnostics alone can discriminate the semantics. This is the insufficiency gate: TDI recovery is only interesting if it adds discriminative value beyond cheaper static summaries.

Static controls include: operator norm, spectral concentration, effective support, entropy, and structure-specific invariants preregistered for each semantic.

## 7. Semantic discrimination diagnostics

TDI-7.5 adds diagnostics that compare recovery trajectories across FLAT semantics:

- per-semantic recovery profile at frozen early depths;
- inter-semantic recovery distance at each depth;
- joint static+recovery discrimination score;
- insufficiency indicator: whether static diagnostics alone separate the semantics.

## 8. Long-horizon protocol

The protocol extends observation horizons compared to TDI-7.4, but stays strictly before the target evaluation depth.

## 9. Null and negative results

Negative, null, inconclusive, or indistinguishable semantic results are preserved and never renumbered away. A result showing that two semantics are recovery-indistinguishable is a valid H-AI-4 outcome.

## 10. Information-theoretic interpretation limits

Semantic discrimination diagnostics are not interpreted as evidence about MMI, causation, or human cognitive mechanisms. They are mechanistic comparisons between deterministic reference semantics.

## 11. Relationship to TDI-7.0 through TDI-7.4

TDI-7.5 inherits the frozen protocols of prior TDI-7.x stages. It adds semantic comparison on top of the joint information diagnostics from TDI-7.4.

## 12. Evidence handoff

Must follow the evidence handoff format in `docs/TDI-7-EVIDENCE-HANDOFF.md`.

## 13. Frozen decision records

- `docs/TDI-7.5-POPULATION-DECISION.toml`
- `docs/TDI-7.5-SELECTION-DECISION.toml`
- `docs/TDI-7.5-REJECTION-POLICY.toml`

## 14. Non-claims

Does not establish that any FLAT semantic is superior, transfers to neural architectures, or provides GPU performance gains. FLAT promotion requires frozen specification and oracle for each semantic.

## 15. Determinism and provenance

Repeated execution with same inputs must reproduce populations. Bit-exact cross-platform identity not assumed.

## 16. Final-holdout lock

Explicit final-run confirmation guard required. Token must not be supplied by CI or tests.

## 17. Required reporting

Per-semantic: B0/B1 MSE, relative reduction, 95% interval, verdict, rejection counts, static spectral summary, recovery profile, inter-semantic discrimination summary, provenance. Aggregate: insufficiency indicator, best-discriminating depth, joint diagnostic verdict.

## 18. Relationship to ITD Simulator

Ablation ladder: static -> spectral -> ITD -> TDI -> per-semantic TDI -> joint. TDI and ITD remain separately measurable. Each semantic is a separate ablation arm.

## 19. Relationship to FLAT-ATTENTENTION

Each candidate semantic requires frozen math, scalar/reference oracle, invariant/adversarial tests, quality evidence, cost evidence, and failure-mode record. A positive TDI result for a semantic is not sufficient evidence for semantic promotion.

## 20. TDI-7.x programme map

- TDI-7.0: H-AI-1 protocol
- TDI-7.1: bounded evaluator
- TDI-7.2: H-AI-1 holdout executed
- TDI-7.3: H-AI-2 heterogeneity/coupling
- TDI-7.4: H-AI-3 joint information/synergy
- TDI-7.5: H-AI-4 FLAT semantic discrimination
- TDI-7.6+: evidence-justified ablations

## 21. Non-claims

Does not establish reality, transferability, or usefulness outside the synthetic TDI protocol.