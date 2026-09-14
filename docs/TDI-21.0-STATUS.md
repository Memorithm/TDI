# TDI-21.0 Status

Status: **active development; not frozen; no confirmatory execution**.

## Lineage and audit

Bootstrap #233, homepage #234, audit #235, causal stream #237 and evaluation controls #240 are merged. The [2026-09-14 audit](TDI-21.0-AUDIT-20260914.md) reviews the initial TDI-21 source at `41a71e22a6bde76870d44f952f8d0b5767541e54`, documents eight defects/limitations and supplies corrective regressions. A merge or passing software test is not scientific confirmation.

Corrective engineering includes invertible versioned routing, invalid-literal rejection, declared-width clause validation, full-width address reduction, checked arithmetic, separate resource categories, evidence-dimension validation, correct absent-marker scoring, delimiter-safe evidence v2 and duplicate/blank freeze-field rejection. A recorded zero pairwise count is not execution attestation; valid SHA syntax is not proof of executed code.

## Implemented candidate and reference surfaces

The opt-in `tdi-ai::experimental` facade contains Boolean primitives, literal clauses, ANF evaluation, bounded memory, evidence serialization and a structural freeze-template validator. The historical `delayed_bit_recall` function remains a two-event smoke fixture, not a long-delay experiment.

[The causal stream increment](TDI-21.0-CAUSAL-STREAM.md) adds a closed one-event-at-a-time B2/B3 reference. B2 addresses one slot; B3 probes at most two slots in an addressed bucket and records deterministic evictions. The candidate receives no oracle labels or future sequence. It distinguishes absent from stored-zero values, rejects malformed input atomically, bounds event/storage resources and resets between episodes.

Independent-oracle tests enumerate every length-four stream over seven events for both modes. Additional fixtures cover 4,096-event delays, updates, gated writes, capacity collisions, full-width identities, probe bounds and reset. The development example emits both successful retrievals and deliberate capacity failures, actual reference-object memory footprints and separate work counters. Debug/release tests and exact repeated output are wired into read-only CI.

[Shared development scoring and competence controls](TDI-21.0-EVALUATION-CONTROLS.md) add an independent prefix-history oracle, exact expected-query denominators, disjoint error categories, bounded episode/oracle admission, common Boolean memory-substrate ceilings, and separately labelled no-memory/exact-dictionary controls. Ten contract tests include 387 tiny query-containing episode strings. Validation outcomes belong to exact-head CI logs, not this status description.

[Isolated B0/B1 references](TDI-21.0-ATTENTION-REFERENCES.md) now implement actual numerical versus packed-binary Q/K scoring, shared floating-point values, stable softmax and weighted readout on the same causal event vocabulary. Their explicit identity/recency/null adapter is hand-constructed, not a trained Transformer. Both are cross-checked against the independent oracle on 2,145 query-containing length-four strings, with full-width identities, overrides, delays, native work, numerical stability, capacity rejection and reset regressions. The six-mechanism example keeps all Boolean capacity failures visible and explicitly labels the unequal total budgets.

B4/B5 learned candidate systems and trained attention-model comparisons are **not implemented**. Hand-written B0/B1/B2/B3 references and successful dictionary controls do not demonstrate language modeling, learned semantic addressing or attention replacement at model scale.

## Validation and reproduction

```bash
cargo fmt --all -- --check
cargo test --locked -p tdi-ai --all-features
cargo clippy --locked -p tdi-ai --all-targets --all-features -- -D warnings
cargo test --locked --workspace
bash scripts/check-tdi-ai-experiments.sh
bash scripts/check-tdi21-development.sh
```

Use exact-head CI logs for current outcomes. The development script requires a clean checkout, records its real SHA and source hashes, and executes no protected/final population. Equal entry count does not imply equal memory budget. The B2/B3 common ceiling covers only the declared memory substrate; the attention references separately reserve bounded write history and score buffers. Dictionary payload bits are a lower bound. None of these is matched total compute/training/allocator evidence.

## Next scientific milestones

1. Implement and ablate learned or synthesized Boolean routing under a bounded search contract.
2. Extend resource accounting to explicit total memory/compute/training/tuning envelopes, then compare competent trained models where the task requires learning.
3. Preregister independent Development/Validation task families, metrics, decision rules and stop conditions before comparative evidence.
4. Qualify runtime and model behavior before generic SciRust promotion or an elastic FLAT-ATTENTION hybrid.

The structural freeze template remains insufficient to authorize a confirmatory experiment. All inherited holdout protections remain intact.
