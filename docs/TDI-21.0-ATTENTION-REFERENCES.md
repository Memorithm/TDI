# TDI-21.0 — Isolated identity/recency attention references

Status: **development-only software controls, not trained Transformers, matched-budget evidence or a scientific freeze**.

Source inspected: TDI `dd27e6acf4ed8c864e16815aafa5720a9e6964b4`, after #240. The existing B2/B3 causal references, independent prefix oracle, exact-dictionary/no-memory controls and expected-query scorer remain unchanged.

## What B0/B1 mean in this increment

B0 now has an actual floating-point dot-product/softmax/weighted-value reference. B1 computes the equivalent bipolar Q/K dot product with XOR and POPCOUNT, then uses the SAME floating-point values, softmax and readout as B0. They are not labels for trained Transformer models or implementations of a published optimized binary-attention kernel.

The task adapter deliberately encodes public 64-bit identities losslessly and supplies a hand-constructed recency bias. This makes an auditable reference for the current exact-identity tasks. It does not model learned semantic relevance, latent language representations or generalization. Success is a competence control, like the ordinary dictionary, not evidence that the architectural research is complete.

Candidate code in `tdi21_stream.rs` and `tdi21_boolean_relational.rs` does not import the reference module. Baseline work and types remain outside the strict B2-B5 candidate path. A direct-import regression helps detect accidental coupling but is not a general execution attestation.

## Explicit, reproducible mathematics

Write history is append-only: every admitted write, including an override, gets an index i starting at zero. Rejected markers and delay events add no history entry. Let C be declared write capacity, m the current write count with m <= C <= 256, q the encoded current identity, and k_i the encoded stored identity. Each identity bit maps to -1 or +1 in 64 dimensions.

B0 computes the 64 scalar-product terms explicitly. B1 computes exactly

`d_i = 64 - 2 * popcount(query_identity XOR stored_identity)`.

These two constructions yield the same integer dot product, exactly representable in f64. Neither reference filters candidates using identifier equality. Every stored write is scored on every lookup, including old overrides.

For the reference-only identity/recency construction, the logit is

`z_i = 16 * ((d_i - 64) * (C + 1) + (i + 1))`.

An additional null lane has logit zero, presence zero and zero-valued payload coordinates. A stored lane has presence one and a bipolar f64 payload. The attention row is a max-subtracted softmax over ALL m + 1 lanes. The reference computes weighted sums of the stored values and their presence. Presence <= 0.5 yields `Miss`; otherwise positive payload coordinates decode to one and nonpositive coordinates to zero. A stored zero therefore remains different from absence. Conjunction performs two such lookups, then the already declared Boolean payload conjunction if both operands are present.

This is scaled dot-product attention with an explicit positional/null bias and a task-specific readout, not the default Transformer parameterization. The scale, bias, null policy and readout are fixed public development semantics, not parameters fitted to withheld data.

### Bounded exact-arithmetic interpretation

If a matching identity exists, its latest admitted occurrence has the largest logit. Earlier exact matches differ by at least 16. Every nonmatching identity has d_i <= 62, hence logit at most `-16 * (C + 2)`; null has logit zero. If there is no match, null wins. The winning lane therefore has a margin of at least 16 over each competitor.

There are at most 256 competitors. In real arithmetic its weight is at least

`1 / (1 + 256 * exp(-16)) > 0.99997`.

That mass exceeds one half, so both presence and each bipolar payload coordinate have the winner's decoded answer. This is a constructive bound for this finite adapter, NOT a theorem about trained language models. Floating-point implementation behavior is separately tested. Max subtraction avoids exponential overflow; underflow of negligible weights is permitted. Nonfinite arithmetic rejects the step instead of becoming an apparently valid value.

## Causality, rejection and resource boundaries

`AttentionReference::step(Event)` receives exactly the present event; it receives no oracle, expected answer, future stream or scoring callback. The same public Write/Recall/Conjunction/Ignore vocabulary and marker semantics used by B2/B3 apply here.

Construction rejects capacities outside 1..256, payload widths outside 1..64 and event budgets outside 1..16,384, before fallible allocation. History capacity includes overrides: a full history rejects another admitted write, even for an existing identity. It does not silently evict or bypass attention with a dictionary update. Malformed input, full-history and exhausted-event errors preserve state. A numerical error rejects execution; its diagnostic row must be discarded and the reference reset. No error may remove an expected query from evaluation.

The fixed example stops on reference rejection rather than publishing a partial score. Only a complete output trace enters the existing shared expected-query scorer. It does not invent a StreamError conversion for reference failures. The more general future experiment must publish typed rejected-run records in its population ledger before making statistical comparisons.

Step and reset allocate no new buffers. Reset clears the current episode but is not counted as free computation. Each lookup costs m pairwise scores and at least m + 1 normalization lanes; conjunction performs two lookups. B1 removes floating Q/K dot terms but does NOT remove pairwise traversal, floating value aggregation or softmax. No dense N-by-N matrix is materialized by this incremental implementation; that is not a novelty claim.

## Native accounting and memory

`AttentionWork` separately records accepted events, writes, inhibited writes, lookups, pairwise scores, scalar dot terms, XOR/POPCOUNT words, exponential/division operations, weighted-value terms, encoding components, presence accumulation, readout comparisons and conjunctions. A scalar dot/value term is a product accumulated into a sum, not one hardware instruction. Index arithmetic, logit maps, validation and allocator work are not implicitly included. No unlike categories are converted to a fabricated universal FLOP cost.

Both references store full 64-component f64 values, including when fewer payload bits are logically used. B0 additionally stores 64 f64 key components; B1 stores one 64-bit key word. Reserved entry-component costs are therefore 8192*C bits for B0 and 4160*C bits for B1. The reusable score buffer costs 64*(C+1) component bits in both. Actual Vec capacity and inline Rust object bytes are reported separately. Vector metadata, counters and configuration are in inline bytes, not hidden inside the component-bit numbers. Allocator metadata, query/readout temporaries, input/oracle/output traces, code and process memory are excluded. These are NOT total model/VRAM measurements.

The fixed six-mechanism example reserves C=16 writes for attention, while B2/B3 keep their existing common 518-bit memory-substrate ceiling. These are explicitly NOT matched total budgets. Do not compare a larger full history against a bounded candidate and call the result an architecture ranking.

## Software validation

The integration suite enumerates every query-containing length-four string over seven events: `7^4 - 4^4 = 2145` input strings. Both references are compared with the independent prefix oracle and their normalized rows are compared directly. Additional tests exercise all 64 identity bits, complements, zero-valued facts, 256 repeated overrides, stable normalization, null-lane absence, exact native work counts, invalid configuration, atomic capacity/input/budget errors, reset and 4096 delay events. Unit tests reject nonfinite logits and test large-logit stability. The module includes an executable rustdoc example.

The six-mechanism executable uses five public fixtures: delay, pair collision, saturated bucket, mixed absence/zero/override/inhibited-write semantics, and the existing B2-favouring replacement case. The expected B0/B1/dictionary result is exact retrieval on all five; B2/B3 capacity failures and the no-memory control remain visible. These are source-defined software regression targets, not a held-out benchmark score.

```bash
bash scripts/check-tdi21-development.sh
```

The existing gate retains all prior tests and adds B0/B1 integration/unit tests in debug and release. It fingerprints the new sources and executes each public example twice for byte-exact replay on the same environment. Source SHA, rustc metadata and clean-checkout checks remain mandatory. Actual outcomes must be read from final-head CI; writing assertions does not establish their execution.

## Research continuation and downstream boundary

The next missing mechanism is learned or boundedly synthesized Boolean routing/encoding, with independent task families and declared training/tuning/total-resource envelopes. These task-specific reference adapters cannot stand in for a trained attention baseline in a language-model comparison. TDI-21.1 and later comparative/final stages remain unfrozen and unauthorized here.

B0/B1 equivalence and native cost separation can later serve as reference-oracle ideas for SciRust, BooleanLab or FLAT-ATTENTION, but this TDI-specific identity/recency adapter is not copied into their production code. A hybrid still needs state synchronization, numerical KV retention/reconstruction, routing calibration and measured switch costs. This increment implements no production hybrid.

## External context versus this construction

Vaswani et al. (2017), *Attention Is All You Need*, https://arxiv.org/abs/1706.03762, supplies the conventional attention context. Official PyTorch `scaled_dot_product_attention` documentation, consulted 2026-09-14, https://docs.pytorch.org/docs/main/generated/torch.nn.functional.scaled_dot_product_attention.html, documents scaled dot products, additive bias, softmax and value aggregation. The identity/recency/null construction, bounded argument and software contracts above are this repository's explicit development design, not claims attributed to those sources or a claim of novelty.
