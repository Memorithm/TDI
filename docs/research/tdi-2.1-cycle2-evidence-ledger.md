# TDI-2.1 cycle 2 — non-final evidence ledger

This ledger records the bounded Development/Validation evidence accumulated by the second TDI-2.1 autonomous cycle. It is not a PrimaryHoldout report and does not authorize protected/final execution.

## Scientific boundary

The working definition remains experience-conditioned template use: accumulated validated experience changes the available internal template/reliability state; inference applies that consolidated state to a current situation; latency is measured only after the decision as an external consequence. No timing measurement is an algorithm input.

## Reusable-template evidence

- Motif retrieval: frozen Validation selected 34/34 and was correct on 34/34. B2 nearest-Boolean also achieved 34/34 on this simple family, so motif retrieval alone does not discriminate experiential weighting from ordinary Boolean overlap.
- No-experience and invalid-experience controls: each lost all 34 paired motif cases against the reference engine.
- Support scaling: support 0 produced 34 abstentions; support 1, 2, 4 and 8 each produced 34/34 correct selections on the frozen motif population. This establishes the presence-vs-absence boundary, not a monotonic accuracy benefit from larger support.
- Irrelevant-predicate stress: 0, 1, 8, 64 and 256 distractors all preserved 34/34 motif correctness on the bounded fixture.
- OOD: 34/34 uncovered motif states were diagnosed as uncovered and the engine abstained on all 34.
- Exact ambiguity: two equally supported applicable experiences produced two candidates with zero top-two margin.

## Context and temporal evidence

- Reusable context Development: 52/52 correct.
- Context ablation: 0/52 correct on the paired control.
- Reusable context Validation: 52/52 correct without retuning.
- Temporal Development: 33/33 correct.
- Temporal order reversal: original ordering 33/33 correct; reversed ordering 0/33 for the original target.
- Temporal Validation: 33/33 correct without retuning.

## Analogical transfer evidence

- Analogical Development: role-preserving transfer was exact on 32/32 novel-identity cases; literal no-transfer was exact on 0/32.
- Relation ablation: full relational transfer was exact on 32/32; relation-ablated transfer was exact on 0/32.
- Analogical Validation: transfer remained exact on 32/32; no-transfer remained 0/32.

These bounded fixtures support the narrower claim that the implementation can reuse frozen Boolean/relational/temporal templates across novel nuisance predicates, contexts, temporal observations and entity identities. They do not establish human-like intuition or general-domain competence.

## Experience quality, consolidation and abstention

- Equal total support with 8 successes / 2 failures yields greater experience weight than 2 successes / 8 failures.
- External minimum-weight thresholds can trade coverage for abstention without making latency or novelty a decision input.
- Generalization proposals are evaluated against positive and negative states before mutation; a tested unsafe generalization was rejected because it admitted the counterexample.
- A tested specialization preserved both positive states and rejected the negative state, making it admissible for review.
- Consolidation records update reliability only after explicit confirmed/refuted outcomes.

## Calibration, cost and latency diagnostics

- Calibration probability is derived from empirical success/failure evidence rather than ranking weight.
- Logical operation accounting is explicit. On one bounded reference query, motif memory accounted for 17 templates, 17 clauses, one weight and one candidate; context memory accounted for 26 templates, 52 clauses, one weight and one candidate.
- Logical memory accounting reports structural units, not physical bytes: motif memory has 17 templates / 17 clauses / 34 reliability counters; context memory has 26 templates / 52 clauses / 52 reliability counters.
- One local external timing observation over 2,000 repetitions measured median 287 ns at support 1 and 315 ns at support 64. This is environment-specific development evidence only, and the ordering is not interpreted causally or as a performance claim.
- Deterministic replay reproduced identical motif and context canonical artifacts in the bounded replay check.

## Explicit non-claims

This ledger makes no claim of superiority over learned neural baselines, no hardware/cache/SIMD claim, no human-equivalence claim, and no confirmatory/final result. PrimaryHoldout remains closed. Any future protected opening requires a separately frozen protocol and one-time authorization boundary.
