# TDI-21.0 Status

Status: **active bootstrap; not frozen; development-only**.

## Verified repository lineage

Bootstrap #233 and homepage #234 are merged. This status supersedes the bootstrap-era note that those PRs were still queued. Their merge does not constitute scientific confirmation.

See [the 2026-09-14 audit](TDI-21.0-AUDIT-20260914.md) for the inspected source revision, defects, corrections and remaining experimental gaps.

## Implemented surfaces

The opt-in `tdi-ai::experimental` facade contains Boolean state, literals, clauses, bounded direct-address memory, ANF evaluation, smoke task fixtures, evidence serialization and a freeze-template structure validator. B0/B1 and B3-B5 labels in an enum are not implemented comparative architectures.

The audit correction adds an invertible versioned route transform; rejects invalid negated literals; exposes declared-width clause validation; reduces full-width tags before pointer narrowing; checks counter and semantic-memory arithmetic; counts address derivations, packed-word operations, tag equality checks and ANF terms separately; validates evidence dimensions; correctly scores no-marker misses; and uses delimiter-safe, versioned evidence serialization. Freeze-template grids reject duplicates and whitespace-only rule names.

`pairwise_comparisons == 0` is a consistency check on recorded work, NOT proof that arbitrary caller code executed no attention. Syntactically valid SHA text is NOT proof of the executed revision. `129 * slots` counts only direct-memory semantic payload, not total process or architecture memory. Raw Boolean/ANF helper calls are uninstrumented unless their counted interface is used.

The historical `delayed_bit_recall` function is a two-event round-trip smoke fixture. It does not establish recall across long delays. A miss is not the Boolean value false; both must remain distinct.

## Required next implementation

1. A causal event-stream candidate interface with no oracle-answer argument, explicit reset and resource bounds.
2. Development-only episodes with genuine delays, multiple writes, misses, overrides, distractors and controlled collisions; an evaluator-owned oracle independent of candidate storage.
3. A bounded B3 memory with a declared lookup-probe bound and replacement metadata accounting.
4. Separate B0/B1 baseline adapters; shared task inputs, training/tuning budgets and explicit memory accounting before comparative claims.
5. Learned logical routing and systematic ablations before any language-model or hybrid-engine claim.

## Required before TDI-21.1 freeze

Resolve and preregister task definitions, sequence/state/memory grids, task counts, seeds, Development/Validation separation, resource envelopes and acceptance rules. A resolved template alone does not freeze these semantics. Preserve all existing protected holdout boundaries. Confirmatory execution remains unauthorized.

## Validation

Run on the exact candidate commit:

```bash
cargo fmt --all -- --check
cargo test --locked -p tdi-ai --all-features
cargo clippy --locked -p tdi-ai --all-targets --all-features -- -D warnings
cargo test --locked --workspace
bash scripts/check-tdi-ai-experiments.sh
```

The audit regressions include the old route's rank-61 counterexample and the corrected route's rank-64 test, an independently implemented inverse, invalid literals/dimensions, full-width modulo, checked overflow, accounting, marker semantics and provenance injection. Test definitions are not execution results; use exact-head CI logs for the outcome.

## Downstream boundary

TDI owns the sequence experiment; BooleanLab owns broader Boolean-function characterization. General primitives may be promoted to SciRust only with a versioned contract and qualification. FLAT-ATTENTION hybridization is a separate future question: Boolean-only, softmax-only and Boolean pre-routing followed by restricted softmax. No hybrid kernel or hardware speedup is established here.
