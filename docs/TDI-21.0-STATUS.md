# TDI-21.0 Status

Status: **active development; not frozen; no confirmatory execution**.

## Lineage and audit

Bootstrap #233, homepage #234, audit #235, causal stream #237, evaluation controls #240, isolated attention references #241, first B4 ANF substitution #243, bounded exact-function search #244, local-predicate boundary #245, identifiability audit #246, conflict-preserving objective #247, distributional ANF search #256 and causal sequence materializer #257 are merged. The [2026-09-14 audit](TDI-21.0-AUDIT-20260914.md) reviews the initial TDI-21 source at `41a71e22a6bde76870d44f952f8d0b5767541e54`, documents eight defects/limitations and supplies corrective regressions. A merge or passing software test is not scientific confirmation.

Corrective engineering includes invertible versioned routing, invalid-literal rejection, declared-width clause validation, full-width address reduction, checked arithmetic, separate resource categories, evidence-dimension validation, correct absent-marker scoring, delimiter-safe evidence v2 and duplicate/blank freeze-field rejection. A recorded zero pairwise count is not execution attestation; valid SHA syntax is not proof of executed code.

## Implemented candidate and reference surfaces

The opt-in `tdi-ai::experimental` facade contains Boolean primitives, literal clauses, ANF evaluation, bounded memory, evidence serialization and a structural freeze-template validator. The historical `delayed_bit_recall` function remains a two-event smoke fixture, not a long-delay experiment.

[The causal stream increment](TDI-21.0-CAUSAL-STREAM.md) adds a closed one-event-at-a-time B2/B3 reference. B2 addresses one slot; B3 probes at most two slots in an addressed bucket and records deterministic evictions. The candidate receives no oracle labels or future sequence. It distinguishes absent from stored-zero values, rejects malformed input atomically, bounds event/storage resources and resets between episodes.

Independent-oracle tests enumerate every length-four stream over seven events for both modes. Additional fixtures cover 4,096-event delays, updates, gated writes, capacity collisions, full-width identities, probe bounds and reset. The development example emits both successful retrievals and deliberate capacity failures, actual reference-object memory footprints and separate work counters. Debug/release tests and exact repeated output are wired into read-only CI.

[Shared development scoring and competence controls](TDI-21.0-EVALUATION-CONTROLS.md) add an independent prefix-history oracle, exact expected-query denominators, disjoint error categories, bounded episode/oracle admission, common Boolean memory-substrate ceilings, and separately labelled no-memory/exact-dictionary controls. Ten contract tests include 387 tiny query-containing episode strings. Validation outcomes belong to exact-head CI logs, not this status description.

[Isolated B0/B1 references](TDI-21.0-ATTENTION-REFERENCES.md) implement actual numerical versus packed-binary Q/K scoring, shared floating-point values, stable softmax and weighted readout on the same causal event vocabulary. Their explicit identity/recency/null adapter is hand-constructed, not a trained Transformer. Both are cross-checked against the independent oracle on 2,145 query-containing length-four strings, with full-width identities, overrides, delays, native work, numerical stability, capacity rejection and reset regressions. The six-mechanism example keeps all Boolean capacity failures visible and explicitly labels the unequal total budgets.

[The first B4 development surface](TDI-21.0-B4-ANF-SYNTHESIS.md) adds an exact bounded truth-table-to-ANF Möbius transform and a B3 + synthesized-Zhegalkin admission control. The initial polynomial is intentionally semantically identical to the existing marker rule: `x0 XOR x0*x1`. This isolates and accounts the algebraic composition layer rather than claiming an improvement. Contract tests reconstruct every three-variable Boolean function and require B4/B3 parity over all 4,096 four-event streams in the declared alphabet. B4 remains pairwise-score free.

[The bounded B4 search scaffold](TDI-21.0-B4-ANF-SEARCH.md) adds exhaustive sparse-ANF selection over a typed `DevelopmentSet`, while `ValidationSet` is accepted only after selection. Search is bounded to six variables, degree three, four monomials, 65,536 candidates and at most 64 unique labelled assignments — the complete largest domain possible under six variables. Candidate-space size is checked before enumeration rather than silently truncated. Selection and search-cost accounting are deterministic. `SearchResult` retains the complete selection envelope — variable count, degree limit, term limit and candidate budget — and Validation retains the fitted arity, preventing caller-driven reinterpretation of a selected program under a different search identity. The exact-function set rejects duplicate assignments by design; empirical policy learning with conflicting objectives requires a separate weighted/count-preserving surface rather than abusing this type. Software fixtures recover `x0 XOR (x1*x2)` and three-bit parity and separately exercise post-selection Validation behavior. This is not yet a sequence-task learning result.

[The local event-predicate boundary](TDI-21.0-B4-EVENT-PREDICATES.md) defines six versioned write-routing predicates from the current two-bit marker and the addressed B3 bucket only: exact-tag-present, any-entry, full-bucket and next-victim state in addition to the marker bits. The public adapter binds observation atomically to the current `Write` key; it exposes no stored payload, oracle answer, future event or Validation label. Observation work, local slot probes and tag checks remain separately accounted.

[The exact v1 identifiability audit](TDI-21.0-B4-PREDICATE-IDENTIFIABILITY.md) preserves a constructed counterexample: the same six-bit observable vector `0b011001` can require opposite hindsight-optimal admission actions under two different future recalls. This proves non-identifiability of the v1 representation for that bounded objective; it does not prove that Boolean routing is impossible or that attention is necessary.

[The conflict-preserving distributional objective](TDI-21.0-B4-DISTRIBUTIONAL-OBJECTIVE.md) therefore retains repeated observable states and all four counterfactual outcome classes instead of collapsing them to one Boolean label. It separates hindsight failure, deterministic feature-conflict floor and selected-policy excess. [The bounded distributional ANF search](TDI-21.0-B4-DISTRIBUTIONAL-SEARCH.md) selects from Development only, re-scores the selected policy independently, and evaluates Validation post-selection without refitting.

[The causal sequence materializer](TDI-21.0-B4-SEQUENCE-MATERIALIZER.md), merged as #257, derives those counterfactual samples by executing explicit causal episodes through the audited evaluator rather than hand-authoring labels. Future probes and branch outcomes stay evaluator-side; the candidate receives only its current causal observation.

[The relational binding prototype](TDI-21.0-RELATIONAL-BINDING.md) is the next development slice in this branch. It leaves cache/admission qualification and tests a broader attention function: exact subject–relation binding, bounded one-to-four-hop composition, renamed Development/Validation identifiers, an independent dictionary oracle, explicit address-work accounting and an ANF-identity substitution control. The ANF control must match direct symbolic addressing exactly; it is not yet learned semantic addressing or a trained-model result.

B5 learned relational state evolution, learned relational addressing beyond the identity control, trained attention-model comparisons and matched total-resource experiments are **not implemented**. Hand-written B0/B1/B2/B3 references, the B4 admission-policy machinery, the new relational harness and successful dictionary controls do not demonstrate language modeling or attention replacement at model scale.

## Validation and reproduction

```bash
cargo fmt --all -- --check
cargo test --locked -p tdi-ai --all-features
cargo clippy --locked -p tdi-ai --all-targets --all-features -- -D warnings
cargo test --locked --workspace
bash scripts/check-tdi-ai-experiments.sh
bash scripts/check-tdi21-development.sh
```

Use exact-head CI logs for current outcomes. The development script requires a clean checkout, records its real SHA and source hashes, and executes no protected/final population. Equal entry count does not imply equal memory budget. The B2/B3 common ceiling covers only the declared memory substrate; B4 additionally reports its ANF representation bits and term evaluations; sparse search reports search candidate/case/monomial work separately; local predicate observation reports bounded route-observation, slot-probe and tag-check work; the attention references separately reserve bounded write history and score buffers. Dictionary payload bits are a lower bound. None of these is matched total compute/training/allocator evidence.

## Next scientific milestones

1. Qualify the relational binding/composition harness on exact current-main CI, including direct-vs-ANF identity equivalence, renamed identifiers, distractors, bounded path length and failure accounting.
2. Add a separate Development-only learned/synthesized relational address surface and evaluate it unchanged on renamed Validation identifiers; preserve negative coverage counterexamples rather than hiding them.
3. Add matched B0/B1 relational controls and explicit total memory/search/inference envelopes before any comparative claim.
4. Freeze relational Development/Validation generation, search envelopes, metrics, rejection rules and stop conditions before confirmatory evidence.
5. Build B5 only after B4 relational learning is falsifiable and reproducible, then qualify behavior before generic SciRust promotion or an elastic FLAT-ATTENTION hybrid.

The structural freeze template remains insufficient to authorize a confirmatory experiment. All inherited holdout protections remain intact.
