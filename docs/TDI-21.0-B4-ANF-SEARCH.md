# TDI-21.0 — Bounded sparse-ANF search contract

Status: **development-only search scaffold; not frozen; not confirmatory; no final/holdout material**.

This increment follows the first B4 algebraic substitution. It does not change the B4 stream semantics or claim an improvement over B3. Its purpose is to make the next step — searching nontrivial Boolean relations — explicit, bounded and leakage-safe before connecting search to sequence behavior.

## Search boundary

The search API consumes only a typed `DevelopmentSet`. `ValidationSet` is a different Rust type and cannot be passed to `fit_sparse_anf`. Validation labels are accepted only by `evaluate_validation` after a `SearchResult` already exists.

This type boundary is not a proof against every possible information leak in a future experiment. It does prevent the ordinary search function from directly consuming Validation labels and makes later experiment-level leakage review auditable.

No final population, final seed, protected holdout, attention output, evaluator hidden state or future event is accepted by this module.

## Candidate family

A candidate is a sparse algebraic-normal-form Boolean function over `n` declared bits:

`f(x) = c XOR m_1(x) XOR ... XOR m_k(x)`

where each monomial is the product/AND of the variables selected by one bit mask. Search bounds are deliberately small:

- variables: at most 6;
- monomial degree: at most 3;
- monomial count per candidate: at most 4;
- candidates evaluated: at most 65,536;
- labelled cases per set: at most 256.

The implementation constructs the eligible monomial universe from the declared degree, computes the complete candidate-space size before enumeration, and refuses search when the required population exceeds the caller's candidate budget. It does not silently truncate a search space and call the truncated result optimal.

For `m` eligible monomials and term limit `K`, the candidate count is exactly

`2 * sum_{k=0..K} C(m,k)`

where the factor two represents the ANF constant bit.

## Deterministic selection and provenance

All admitted candidates are evaluated exhaustively on Development rows. Selection is lexicographic and fully declared:

1. fewer Development mismatches;
2. fewer monomials;
3. lower maximum monomial degree;
4. constant `false` before `true`;
5. lexicographically smaller canonical monomial masks.

`SearchResult` retains the complete `SearchEnvelope` — variable count, maximum degree, maximum term count and candidate budget — rather than only the selected polynomial. The fitted arity is therefore also used automatically for exact ANF conversion and Validation admission. A caller cannot reinterpret a three-variable selected candidate as a two-variable program, nor silently detach a result from the search space that selected it.

This envelope binding is provenance, not a scientific freeze. Development/Validation population identity, source revision and later task-generation contracts still require their own records.

`SearchWork` records candidates, case evaluations and monomial evaluations. Those categories are semantic search work, not CPU instructions, FLOPs, bandwidth, energy or wall-clock measurements.

## Validation discipline

`evaluate_validation` evaluates an already selected program without modifying it. Validation evidence records the fitted variable count, mismatches and monomial evaluations separately from Development search work. Validation is not used to choose among candidates, choose the envelope, change degree/term limits or retry the search.

A future experiment must additionally freeze how Development and Validation rows are generated, prove their intended separation, define stopping/rejection rules before reporting comparative evidence, and retain negative results. This Stage-0 scaffold does none of that automatically.

## Qualification fixtures

The contract tests currently include:

- exact recovery of `x0 XOR (x1*x2)` from all eight three-variable assignments under a degree-2/two-term envelope;
- exact candidate/work-count checks for that search (`44` candidates, `352` case evaluations, `576` monomial evaluations);
- retention of the exact selection envelope in the selected result;
- conversion of the selected sparse candidate through the independent exact truth-table-to-ANF synthesizer at its fitted arity;
- Validation evaluation with both matching and deliberately inverted labels, confirming that post-selection validation cannot mutate the selected result and retains the fitted arity;
- deterministic simplicity/canonical tie breaking;
- candidate-space rejection before an oversized degree-3/four-term six-variable enumeration;
- duplicate and out-of-range assignment rejection;
- variable-arity drift rejection;
- exact recovery of three-bit parity under a degree-1/three-term envelope.

These are software qualification cases. They do not show semantic generalization, language-model learning, attention replacement, novelty or hardware efficiency.

## Next integration step

After this search surface is qualified, a later development PR may define a versioned event-to-predicate adapter for a nontrivial B4 routing decision. That adapter must expose only present/past permitted state, not oracle answers or Validation labels. Search cost, selected-program representation bits and runtime ANF work must remain separate.

Only after Development/Validation task generation and decision rules are preregistered can a nontrivial B4 search result be interpreted as research evidence. B5 and FLAT-ATTENTION hybridization remain downstream.
