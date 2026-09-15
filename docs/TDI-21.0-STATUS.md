# TDI-21.0 Status

Status: **active development; not frozen; no confirmatory execution**.

## Lineage and audit

Bootstrap #233, homepage #234, audit #235, causal stream #237, evaluation controls #240, isolated attention references #241 and first B4 ANF substitution #243 are merged. The [2026-09-14 audit](TDI-21.0-AUDIT-20260914.md) reviews the initial TDI-21 source at `41a71e22a6bde76870d44f952f8d0b5767541e54`, documents eight defects/limitations and supplies corrective regressions. A merge or passing software test is not scientific confirmation.

Corrective engineering includes invertible versioned routing, invalid-literal rejection, declared-width clause validation, full-width address reduction, checked arithmetic, separate resource categories, evidence-dimension validation, correct absent-marker scoring, delimiter-safe evidence v2 and duplicate/blank freeze-field rejection. A recorded zero pairwise count is not execution attestation; valid SHA syntax is not proof of executed code.

## Implemented candidate and reference surfaces

The opt-in `tdi-ai::experimental` facade contains Boolean primitives, literal clauses, ANF evaluation, bounded memory, evidence serialization and a structural freeze-template validator. The historical `delayed_bit_recall` function remains a two-event smoke fixture, not a long-delay experiment.

[The causal stream increment](TDI-21.0-CAUSAL-STREAM.md) adds a closed one-event-at-a-time B2/B3 reference. B2 addresses one slot; B3 probes at most two slots in an addressed bucket and records deterministic evictions. The candidate receives no oracle labels or future sequence. It distinguishes absent from stored-zero values, rejects malformed input atomically, bounds event/storage resources and resets between episodes.

Independent-oracle tests enumerate every length-four stream over seven events for both modes. Additional fixtures cover 4,096-event delays, updates, gated writes, capacity collisions, full-width identities, probe bounds and reset. The development example emits both successful retrievals and deliberate capacity failures, actual reference-object memory footprints and separate work counters. Debug/release tests and exact repeated output are wired into read-only CI.

[Shared development scoring and competence controls](TDI-21.0-EVALUATION-CONTROLS.md) add an independent prefix-history oracle, exact expected-query denominators, disjoint error categories, bounded episode/oracle admission, common Boolean memory-substrate ceilings, and separately labelled no-memory/exact-dictionary controls. Ten contract tests include 387 tiny query-containing episode strings. Validation outcomes belong to exact-head CI logs, not this status description.

[Isolated B0/B1 references](TDI-21.0-ATTENTION-REFERENCES.md) implement actual numerical versus packed-binary Q/K scoring, shared floating-point values, stable softmax and weighted readout on the same causal event vocabulary. Their explicit identity/recency/null adapter is hand-constructed, not a trained Transformer. Both are cross-checked against the independent oracle on 2,145 query-containing length-four strings, with full-width identities, overrides, delays, native work, numerical stability, capacity rejection and reset regressions. The six-mechanism example keeps all Boolean capacity failures visible and explicitly labels the unequal total budgets.

[The first B4 development surface](TDI-21.0-B4-ANF-SYNTHESIS.md) adds an exact bounded truth-table-to-ANF Möbius transform and a B3 + synthesized-Zhegalkin admission control. The initial polynomial is intentionally semantically identical to the existing marker rule: `x0 XOR x0*x1`. This isolates and accounts the algebraic composition layer rather than claiming an improvement. Contract tests reconstruct every three-variable Boolean function and require B4/B3 parity over all 4,096 four-event streams in the declared alphabet. B4 remains pairwise-score free.

[The bounded B4 search scaffold](TDI-21.0-B4-ANF-SEARCH.md) adds exhaustive sparse-ANF selection over a typed `DevelopmentSet`, while `ValidationSet` is accepted only after selection. Search is bounded to six variables, degree three, four monomials, 65,536 candidates and at most 64 unique labelled assignments — the complete largest domain possible under six variables. Candidate-space size is checked before enumeration rather than silently truncated. Selection and search-cost accounting are deterministic. `SearchResult` retains the complete selection envelope — variable count, degree limit, term limit and candidate budget — and Validation retains the fitted arity, preventing caller-driven reinterpretation of a selected program under a different search identity. The exact-function set rejects duplicate assignments by design; empirical policy learning with conflicting objectives requires a separate weighted/count-preserving surface rather than abusing this type. Software fixtures recover `x0 XOR (x1*x2)` and three-bit parity and separately exercise post-selection Validation behavior. This is not yet a sequence-task learning result.

B5 learned relational state evolution, event-to-predicate learning, trained attention-model comparisons and matched total-resource experiments are **not implemented**. Hand-written B0/B1/B2/B3 references, the equivalent B4 substitution, sparse truth-table search fixtures and successful dictionary controls do not demonstrate language modeling, learned semantic addressing or attention replacement at model scale.

## Validation and reproduction

```bash
cargo fmt --all -- --check
cargo test --locked -p tdi-ai --all-features
cargo clippy --locked -p tdi-ai --all-targets --all-features -- -D warnings
cargo test --locked --workspace
bash scripts/check-tdi-ai-experiments.sh
bash scripts/check-tdi21-development.sh
```

Use exact-head CI logs for current outcomes. The development script requires a clean checkout, records its real SHA and source hashes, and executes no protected/final population. Equal entry count does not imply equal memory budget. The B2/B3 common ceiling covers only the declared memory substrate; B4 additionally reports its ANF representation bits and term evaluations; sparse search reports search candidate/case/monomial work separately; the attention references separately reserve bounded write history and score buffers. Dictionary payload bits are a lower bound. None of these is matched total compute/training/allocator evidence.

## Next scientific milestones

1. Define a versioned event-to-predicate adapter for a nontrivial B4 routing decision using only permitted present/past state, then connect the already bounded exact-function search surface without exposing Validation labels to selection.
2. Keep exact Boolean-function synthesis distinct from the future empirical weighted policy search required when identical causal observations carry conflicting hindsight objectives.
3. Freeze Development/Validation task-family generation, search envelopes, metrics, rejection rules and stop conditions before comparative evidence.
4. Extend resource accounting to explicit total memory/compute/training/tuning envelopes, then compare competent trained models where the task requires learning.
5. Build B5 only after B4 search and event-feature semantics are falsifiable and reproducible rather than promoting a toy truth-table fit, then qualify behavior before generic SciRust promotion or an elastic FLAT-ATTENTION hybrid.

The structural freeze template remains insufficient to authorize a confirmatory experiment. All inherited holdout protections remain intact.
