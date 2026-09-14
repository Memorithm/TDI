# TDI-21.0 Status

Status: **active development; not frozen; no confirmatory execution**.

## Lineage and audit

Bootstrap #233 and homepage #234 are merged. The [2026-09-14 audit](TDI-21.0-AUDIT-20260914.md) reviews their TDI-21 source at `41a71e22a6bde76870d44f952f8d0b5767541e54`, documents eight defects/limitations and supplies corrective regressions. A merge or passing software test is not scientific confirmation.

Corrective engineering includes invertible versioned routing, invalid-literal rejection, declared-width clause validation, full-width address reduction, checked arithmetic, separate resource categories, evidence-dimension validation, correct absent-marker scoring, delimiter-safe evidence v2 and duplicate/blank freeze-field rejection. A recorded zero pairwise count is not execution attestation; valid SHA syntax is not proof of executed code.

## Implemented candidate surfaces

The opt-in `tdi-ai::experimental` facade contains Boolean primitives, literal clauses, ANF evaluation, bounded memory, evidence serialization and a structural freeze-template validator. The historical `delayed_bit_recall` function remains a two-event smoke fixture, not a long-delay experiment.

[The causal stream increment](TDI-21.0-CAUSAL-STREAM.md) adds a closed one-event-at-a-time B2/B3 reference. B2 addresses one slot; B3 probes at most two slots in an addressed bucket and records deterministic evictions. The candidate receives no oracle labels or future sequence. It distinguishes absent from stored-zero values, rejects malformed input atomically, bounds event/storage resources and resets between episodes.

Independent-oracle tests enumerate every length-four stream over seven events for both modes. Additional fixtures cover 4,096-event delays, updates, gated writes, capacity collisions, full-width identities, probe bounds and reset. The development example emits both successful retrievals and deliberate capacity failures, actual reference-object memory footprints and separate work counters. Debug/release tests and exact repeated output are wired into read-only CI.

B0/B1 adapters and B4/B5 learned candidate systems are **not implemented**. Hand-written B2/B3 references do not demonstrate language modeling, learned semantic addressing or attention replacement at model scale.

## Validation and reproduction

```bash
cargo fmt --all -- --check
cargo test --locked -p tdi-ai --all-features
cargo clippy --locked -p tdi-ai --all-targets --all-features -- -D warnings
cargo test --locked --workspace
bash scripts/check-tdi-ai-experiments.sh
bash scripts/check-tdi21-development.sh
```

Use exact-head CI logs for current outcomes. The development script requires a clean checkout, records its real SHA and source hashes, and executes no protected/final population. Equal entry count does not imply equal memory budget; B3 replacement metadata and physical storage are reported separately.

## Next scientific milestones

1. Add isolated B0/B1 adapters and exact-dictionary/no-memory task-competence controls on common causal inputs.
2. Define total memory/compute/training/tuning envelopes rather than compare only slot counts.
3. Implement and ablate learned or synthesized Boolean routing under a bounded search contract.
4. Preregister independent Development/Validation task families, metrics, decision rules and stop conditions before comparative evidence.
5. Qualify runtime and model behavior before promotion to SciRust or an elastic FLAT-ATTENTION hybrid.

The structural freeze template remains insufficient to authorize a confirmatory experiment. All inherited holdout protections remain intact.
