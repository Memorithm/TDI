# TDI-13.0 — Boolean Relational Architecture

Status: active bootstrap; not frozen; no confirmatory execution authorized.

## Research question

Can a deterministic Boolean relational system reproduce useful functions normally provided by token attention without using Q/K/V projections, QK^T products, softmax normalization, cosine or Hamming similarity as token-to-token attention scores, or a dense N x N score matrix?

This line is intentionally separate from TDI-7 attention/memory work and TDI-8 recurrent-associative work. TDI-13 studies a stricter architectural substitution: Boolean state, Boolean relations, explicit routing, and bounded memory instead of attention scoring.

## Non-negotiable candidate constraints

Candidate Boolean arms B2 and later MUST NOT use:

- Q, K, or V projections as architectural primitives;
- QK^T, dot-product, cosine, or learned pairwise similarity scores;
- softmax or another normalized token-score distribution serving the same role;
- Hamming distance or POPCOUNT as an all-pairs attention-score substitute;
- a materialized dense N x N token relation matrix;
- a hidden fallback to an attention kernel.

These prohibitions apply to the candidate mechanism, not to external baselines used for comparison.

## Initial architecture ladder

- B0 — conventional attention reference. Contextual baseline only.
- B1 — binary/Hamming attention reference. Control for simple attention binarization; still attention and therefore NOT a TDI-13 candidate architecture.
- B2 — Boolean routing with bounded direct-address memory and no attention score.
- B3 — B2 plus bounded associative memory with explicit collision/replacement semantics.
- B4 — B3 plus learned or searched Boolean relations expressed through an explicit F2 / algebraic-normal-form representation.
- B5 — complete Boolean relational candidate combining Boolean state evolution, routing, bounded memory, and Boolean relation composition without any attention primitive.

## Stage-0 falsifiable objectives

The first implementation slice does not claim language-model quality, novelty, or performance superiority. It must establish only that:

1. a typed candidate architecture can process a deterministic sequence without any prohibited attention primitive;
2. Boolean predicates can deterministically select routes and memory addresses;
3. memory writes, reads, collisions, misses, and replacement are explicit and reproducible;
4. exact resource counters can distinguish Boolean work from baseline attention work;
5. candidate correctness can be tested on small retrieval/copy tasks with an exhaustive oracle where feasible.

## Minimal deterministic task family

Stage 0 uses synthetic bounded tasks only:

- delayed bit recall;
- keyed Boolean fact recall;
- copy-after-marker;
- two-fact conjunction retrieval;
- distractor rejection.

Every task instance must expose a complete oracle answer. No stochastic language-model evaluator is permitted in Stage 0.

## Boolean state model

A token/event is encoded as a fixed-width Boolean state B_t. The candidate mechanism may derive Boolean clauses C_t = Phi(B_t, H_{t-1}) and a bounded set of route identifiers R_t = Gamma(C_t). Routes may read or update bounded memory. No route may be selected by ranking token pairs.

The initial implementation should favor auditable operators:

- NOT;
- AND;
- OR;
- XOR;
- fixed-width masks;
- F2 linear transforms;
- algebraic normal form (Zhegalkin) terms in later stages.

## Required measurements

For every candidate run, capture at minimum:

- task correctness;
- number of Boolean primitive evaluations;
- number of memory reads/writes;
- number of route activations;
- number of collisions/replacements/misses;
- peak dynamic memory bits;
- sequence length;
- deterministic provenance (configuration + seed where applicable + exact git SHA when producing evidence).

For B0/B1 baselines capture their own native operation/resource counters separately. Do not convert unlike operations into an invented universal FLOP-equivalent without an explicit model.

## Stage-0 rejection conditions

Reject a candidate configuration if any of the following holds:

- it uses a prohibited attention primitive;
- it silently performs all-pairs token comparisons;
- addressing cost grows by scanning all prior tokens;
- correctness depends on nondeterministic iteration order;
- memory accounting omits metadata needed by the mechanism;
- the evaluator cannot distinguish a miss from a wrong retrieved value.

## Relation to future FLAT-ATTENTION work

TDI-13 is the research owner for the Boolean candidate semantics. FLAT-ATTENTION remains the owner of its current softmax attention engine. No hybrid integration is authorized by this bootstrap.

Only after a TDI-13 mechanism has reproducible evidence should downstream work evaluate an elastic hybrid router with possible modes such as Boolean-only, softmax-only, or Boolean pre-routing followed by restricted softmax. Any such promotion requires a separate engineering contract and real-device measurements.

## Evidence discipline

TDI-13 follows TDI's fail-closed research policy:

- bootstrap before freeze;
- freeze hypotheses, tasks, metrics, budgets, and rejection rules before confirmatory use;
- keep development data distinct from future holdouts;
- retain negative and equivalent outcomes;
- do not promote benchmark observations into universal architecture or hardware claims.

## Immediate implementation target

Implement a small Rust reference module providing:

1. typed architecture identities B0-B5;
2. a prohibition-aware candidate configuration validator;
3. fixed-width Boolean state;
4. deterministic Boolean route evaluation;
5. bounded direct-address memory with explicit miss/collision/replacement counters;
6. exact resource accounting;
7. unit tests for delayed recall and distractor rejection.

This is a bootstrap contract, not a frozen preregistration.
