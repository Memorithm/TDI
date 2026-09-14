# TDI-21.x — Boolean Relational Architecture Research Programme

Status: **active Stage 0 bootstrap; not frozen; no confirmatory execution authorised**.

## Research question

Can a deterministic Boolean relational system reproduce useful functions normally supplied by token attention without using Q/K/V projections, `QK^T`, softmax normalization, pairwise similarity ranking, Hamming/POPCOUNT as an all-pairs attention-score substitute, or a dense `N x N` token-score matrix?

TDI-21 studies an architectural substitution rather than an attention-kernel optimization: Boolean state, Boolean relations, explicit routing, and bounded memory **instead of attention scoring**.

## Separation from existing programmes

TDI-21 is distinct from:

- TDI-7.x attention / memory recovery;
- TDI-8.x bounded recurrent-associative architectures;
- TDI-9.3 Boolean policy synthesis for adaptive inference;
- TDI-16.x event-triggered attention, which still permits an explicit attention action.

The TDI-21 candidate family begins only where attention primitives are absent.

## Candidate prohibition set

Candidate Boolean arms B2 and later MUST NOT use:

- Q, K, or V projections as architectural primitives;
- `QK^T`, dot-product, cosine, or learned pairwise similarity scores;
- softmax or another normalized token-score distribution serving the same role;
- Hamming distance or POPCOUNT as an all-pairs attention-score substitute;
- a materialized dense `N x N` token relation or score matrix;
- a hidden fallback to an attention kernel;
- address lookup implemented by scanning all prior tokens.

These prohibitions apply to candidate mechanisms, not to external baselines used for comparison.

## Architecture ladder

- **B0 — conventional attention reference.** Contextual baseline only.
- **B1 — binary/Hamming attention reference.** Control for attention binarization; still attention and therefore **not** a TDI-21 candidate architecture.
- **B2 — Boolean direct routing.** Boolean predicates plus bounded direct-address memory, no attention score.
- **B3 — Boolean associative routing.** B2 plus bounded associative memory with explicit collision and replacement semantics.
- **B4 — Boolean algebraic routing.** B3 plus explicit `F2` / algebraic-normal-form (Zhegalkin) relation composition.
- **B5 — Boolean relational architecture.** Boolean state evolution, routing, bounded memory, and Boolean relation composition without an attention primitive.

## Stage map

| Stage | Purpose | Status |
| --- | --- | --- |
| **TDI-21.0** | Bootstrap semantics, prohibition set, deterministic Boolean reference primitives | active bootstrap; not frozen |
| **TDI-21.1** | Freeze bounded deterministic tasks, budgets, metrics, development split, and rejection rules | planned |
| **TDI-21.2** | Compare B0/B1 references against B2/B3 candidates under declared matched budgets | planned |
| **TDI-21.3** | Evaluate explicit `F2` / ANF (Zhegalkin) relational composition | conditional |
| **TDI-21.4+** | Robustness, scaling, transfer, learned routing, and downstream promotion studies | conditional on evidence |

## Stage-0 falsifiable objectives

The bootstrap must establish only that:

1. a typed candidate architecture can process deterministic sequence tasks without a prohibited attention primitive;
2. Boolean predicates can deterministically activate routes and direct memory access;
3. memory writes, reads, collisions, misses, and replacements are explicit and reproducible;
4. exact semantic resource counters distinguish Boolean work from baseline attention work;
5. candidate correctness can be checked on small tasks with complete deterministic oracles;
6. structural counters expose any accidental reintroduction of pairwise token scoring.

Stage 0 does **not** test or claim general language-model quality.

## Initial deterministic task family

Development tasks may include:

- delayed bit recall;
- keyed Boolean fact recall;
- copy-after-marker;
- two-fact conjunction retrieval;
- distractor rejection.

Every task instance must expose a complete oracle answer. No stochastic language-model judge is permitted for Stage-0 correctness.

## Boolean state and routing model

An event is encoded as a bounded Boolean state `B_t`. Candidate mechanisms may derive clauses

`C_t = Phi(B_t, H_{t-1})`

and a bounded set of routes

`R_t = Gamma(C_t)`.

Routes may read or update bounded memory. A route MUST NOT be selected by ranking token pairs.

Initial auditable operators are NOT, AND, OR, XOR, and fixed-width masks. Explicit `F2` transforms and algebraic normal form are staged separately so their contribution can be measured rather than assumed.

## Required measurements

For every candidate run, capture at minimum:

- task correctness;
- Boolean primitive evaluations;
- route activations;
- memory reads and writes;
- misses, collisions, and replacements;
- peak semantic dynamic-memory bits;
- sequence length;
- pairwise-comparison counter, which MUST remain zero for B2-B5;
- deterministic provenance: configuration, development seed when applicable, and exact git SHA for evidence artefacts.

B0/B1 baselines retain their own native operation and resource counters. TDI-21 must not invent a universal FLOP-equivalent that conflates unlike operations without an explicit cost model.

## Stage-0 rejection conditions

Reject a candidate configuration if any of the following occurs:

- a prohibited attention primitive is used;
- all-pairs token comparisons are performed, explicitly or indirectly;
- addressing cost is implemented by scanning all prior tokens;
- correctness depends on nondeterministic iteration order;
- memory accounting omits semantic metadata required by the mechanism;
- the evaluator cannot distinguish a miss from a wrong retrieved value;
- the pairwise-comparison counter is nonzero in a B2-B5 run.

## Future FLAT-ATTENTION boundary

TDI-21 owns the scientific semantics of the Boolean candidate family. FLAT-ATTENTION remains the owner of its current softmax-attention engine.

No hybrid integration is authorised by TDI-21.0. If a Boolean mechanism later produces reproducible evidence, a separate downstream programme may evaluate an elastic hybrid router with modes such as:

- Boolean-only;
- softmax-only;
- Boolean pre-routing followed by restricted softmax.

That downstream work must preserve FLAT-ATTENTION's correctness and real-device evidence requirements.

## Downstream sequence

1. Establish deterministic correctness and resource semantics in TDI-21.
2. Freeze bounded tasks, controls, budgets, and rejection rules before confirmatory evidence.
3. Promote stable reusable Boolean / `F2` / ANF primitives to SciRust only when justified.
4. Evaluate hardware realization separately in NNIS or another systems repository.
5. Consider FLAT-ATTENTION hybridization only after a Boolean mechanism has reproducible evidence.

## Scientific interpretation rule

A positive bounded result would show only that the tested Boolean relational candidate performs the declared task under the declared budget. It would not by itself establish novelty, language-model quality, universal replacement of attention, asymptotic superiority, or real-hardware speedup.
