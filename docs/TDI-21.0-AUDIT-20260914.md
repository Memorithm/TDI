# TDI-21.0 scientific and implementation audit — 2026-09-14

Status: **development audit and corrective engineering, not confirmatory evidence**.

## Scope and source of truth

Inspected TDI `main` at `41a71e22a6bde76870d44f952f8d0b5767541e54`, after bootstrap #233 and homepage #234. Scope: the four `tdi21_*` source modules, `experimental.rs`, TDI-21 programme/status, README, workspace/package manifests, applicable AGENTS/coordination instructions and experimental validation script. This is a full review of the TDI-21 bootstrap, not a full audit of every historical TDI series or downstream repository.

The user objective is to test a Boolean relational replacement for attention, not merely binarize Q/K or put a Boolean mask before an unchanged attention engine. Preserve that distinction. The candidate path has no Q/K/V projections, pairwise similarity scores, token softmax, all-pairs attention matrix or hidden attention fallback. Exact checking of a bounded number of memory identifiers is admissible and must be counted separately from token-pair scoring.

## Findings and corrections

### A1 — high: the route transform aliases different identifiers

The original transform repeatedly applied `x XOR rotate(x)`. For any rotation, the all-ones word is a nonzero kernel element of `I + R` over GF(2). Consequently a key and its bitwise complement have the same output for any fixed salt. This is not just a finite-capacity slot collision: storing only that transformed tag can make a different key appear to be an exact hit.

For salt `0x5444493231000001`, both zero and `u64::MAX` mapped to `0x58c0e1c840dbb76d`. Exact Gaussian elimination on the 64 basis images gives rank 61: the transform has only 61 independent output bits, hence eight preimages per output in its image. The old counterexample is retained in the regression suite.

Correction: `tdi21-xorshift64-v2` uses salt XOR followed by XOR-shifts left 13, right 7, left 17. For a shift operator L, L is nilpotent on fixed-width words, so `(I + L)^-1 = I + L + L^2 + ...` is finite over GF(2). Each step and their composition are invertible for fixed salt. Tests independently invert the transform and verify rank 64. Slot reduction can still collide; full-tag checks and miss/replacement accounting remain necessary. This is not a cryptographic hash or a learned semantic router.

### A2 — high: invalid negated literals can activate routes

Previously `test(64)` returned false and `NotBit(64)` negated that to true. Indices 64 through 255 could therefore make malformed rules succeed. Correction: invalid negated literals evaluate false; `Clause::validate(width)` rejects every out-of-range literal, including unreachable short-circuit tails. Width-aware operational constructors must call it before execution.

### A3 — medium: evidence consistency did not validate dimensions

The old evidence guard accepted zero sequence length, zero memory or widths outside 1..64 whenever the arm label was Boolean and its pairwise counter was zero. Correction: validate these manifest dimensions as part of the consistency predicate. This still does not attest arbitrary code, prove that an arm is implemented, or validate an empirical claim.

### A4 — medium: addressing depended on pointer width

`(tag as usize) % slots` discards high bits before modulo on 32-bit hosts. Correction: perform full-width modulo before the narrowing conversion. A regression uses identifiers 1 and 2^32 with three slots, where both full-width remainders must be one.

### A5 — medium: accounting omitted work and could wrap

Routing, full-tag equality checks and conjunction composition were not represented in exported work counters. Plain unsigned increments could panic in debug but wrap in optimized builds; slot-bit multiplication could also overflow.

Correction: checked arithmetic and separately named categories for literal evaluation, explicitly instrumented packed-word operations, address derivations, tag equality, ANF terms and memory activity. Export these in evidence v2. Count the conjunction word operation in its fixture. Raw convenience helpers remain explicitly uninstrumented. There is deliberately no universal conversion to FLOPs, energy or CPU cycles. Checked-counter exhaustion aborts a run instead of emitting a saturated or wrapped value as exact.

### A6 — medium: no-marker correctness confused failure with expected absence

The copy fixture expected a payload even when the marker was false. It marked correct non-copying as failure. Correction: the oracle expects `Miss` for no marker and the payload for a genuine marker. The expectation is derived from the input, not the actual routing decision, so failure to honor a genuine marker remains detectable.

### A7 — medium: text provenance allowed field injection and noncanonical SHA case

An unrestricted task identifier could contain semicolons, equals signs or newlines inside the canonical record. Correction: hex-encode its UTF-8 bytes and lowercase SHA text in `tdi21-evidence-v2`. Preserve negative and invalid observations explicitly. A valid-looking 40-hex string is only syntax: a future runner must bind actual source/binary/input/configuration identities. No authenticity claim follows from this formatter.

### A8 — medium: freeze structure admitted duplicate grids and blank rule names

Correction: reject duplicate sequence/state/memory/seed grid values and whitespace-only identifiers. Passing the template still does not specify executable task/oracle/decision semantics or demonstrate Development/Validation separation. Do not fill scientific fields merely to make the validator pass.

## Scientific assessment

The bootstrap supplies useful engineering primitives, but its small fixtures are not yet an architecture comparison. The delayed-recall function performs one write and one read with sequence length two. Other fixtures use hand-written rules and explicit addresses. They do not demonstrate learned addressing, semantic generalization, long-context performance, matched-budget superiority, or a replacement for a trained Transformer. Enum variants B0/B1/B3/B4/B5 do not implement those architectures.

The strict no-attention hypothesis remains worth testing. However:

- A Boolean word can mathematically be an element of a vector space over GF(2); the excluded object is numerical similarity-based attention, not the word 'vector'.
- Zhegalkin/ANF is a polynomial representation of Boolean functions over GF(2), not an unrelated fourth scalar algebra. Classical OR and XOR have different semantics and must never be interchanged silently.
- An arbitrary Boolean function can require exponentially large representations. Restrict and measure term count, degree, fan-in, depth and state/memory size rather than assume logic is compact.
- A fixed b-bit internal state has at most 2^b states. Perfectly remembering every possible sequence of N independent bits requires at least N state bits unless additional storage or task restrictions are declared. Therefore bounded internal memory cannot imply unlimited exact recall. External storage, tags and replacement state must be included.
- FlashAttention avoids materializing the dense score matrix; merely avoiding that allocation is not a novel architectural result. Distinguish quadratic full-sequence pair work from the per-step history traversal of cached decode.
- Physical Boolean instructions do not imply an end-to-end speedup. Memory traffic, bit packing, irregular access, control divergence and lost accelerator throughput belong in later hardware experiments.
- A hybrid sum that computes both branches pays both costs. A future elastic dispatcher must include routing, branch-state synchronization, switching, calibration and fallback costs. A late softmax fallback cannot recover numerical KV state that was never retained or reconstructed.

## Next bounded experiment

Implement a causal step interface that receives only the present event, never the evaluator's answer. Use one event vocabulary across candidates, with independent evaluator-owned expected answers. Include delay/no-op events, overwritten facts, genuine absent facts, irrelevant stores, deterministic capacity collisions, reset and rejection cases. Keep `Missing`, a stored false bit and a wrong value distinct. Report both successes and expected failures.

A B3 extension should use a declared constant number of bucket probes, exact full-identifier checks and deterministic replacement. Include its replacement metadata and physical object footprint. Such a memory remains a hand-written reference, not learned associative cognition. Compare B2/B3 at explicitly accounted budgets, and only then add separately isolated B0/B1 references. An ordinary exact dictionary and a no-memory predictor are necessary task-competence controls so a trivial key-value task cannot be presented as a breakthrough in attention.

This development fixture work does not freeze TDI-21.1 or open a final holdout. Later comparative preregistration must define training/tuning budgets, independent data splits, generalization tasks, decision rules and stop conditions.

## Cross-project reuse

TDI retains sequential task semantics and scientific verdicts. BooleanLab can characterize route functions and circuits; SciRust is the eventual owner of qualified generic primitives; Forge can later search under a bounded, non-leaking contract; ElasticXxx owns resource actuation; FLAT-ATTENTION owns later hybrid execution. Do not duplicate this unqualified prototype into production repositories. A GitHub code search for `route_tag` in SciRust, BooleanLab, ElasticXxx and FLAT returned no indexed matches during this audit; this is not an exhaustive proof that no similar implementation exists.

## Sources and limits

Repository findings above are derived from the named immutable TDI revision and reproduced by added software regressions. Independent Python algebra checks reproduced the old rank 61, new rank 64 and inverse round trips; those checks are not Rust CI results.

External context, consulted 2026-09-14:

1. Dao et al., *FlashAttention: Fast and Memory-Efficient Exact Attention with IO-Awareness*, 2022, https://arxiv.org/abs/2205.14135 — IO-aware exact attention; operation count alone does not determine wall-clock performance.
2. Petersen et al., *Deep Differentiable Logic Gate Networks*, NeurIPS 2022, https://research.ibm.com/publications/deep-differentiable-logic-gate-networks — learned relaxed gates can be discretized; this does not establish a Boolean language-model replacement.
3. Xiao, Zhang and Zhang, *BinaryAttention: One-Bit QK-Attention for Vision and Diffusion Transformers*, CVPR 2026, https://openaccess.thecvf.com/content/CVPR2026/html/Xiao_BinaryAttention_One-Bit_QK-Attention_for_Vision_and_Diffusion_Transformers_CVPR_2026_paper.html — binarized Q/K attention is a baseline, not our no-attention candidate.

No speedup, quality improvement, novel function or confirmatory result is claimed by this audit.
