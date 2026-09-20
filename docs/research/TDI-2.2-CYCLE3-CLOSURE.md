# TDI-2.2 autonomous template-induction campaign — closure

Tracker: #522  
Status: non-final Development/Validation closure candidate  
Protected boundary: **TDI-2.1 PrimaryHoldout remains closed**

## Question

TDI-2.2 moved the intuition programme one stage earlier than TDI-2.1:

```
experience
  -> admissible predicate candidates
  -> invariant / contingent discovery
  -> template induction
  -> role induction
  -> automatic structural mapping
  -> transfer
  -> observed outcome
  -> evidence-gated consolidation
```

Speed remains outside the algorithm. No latency value is used to induce a
predicate, template, role, mapping or consolidation decision.

## Stage A — information boundary and representation

The campaign established label-free episode and relational-observation IRs,
candidate provenance, disjoint Development/Validation membership, leakage
guards, canonical induction provenance and a paired baseline contract.

Expected template ids, expected role mappings, evaluator labels, protected
identities and post-hoc latency are absent from the induction input surface.

## Stage B — bounded predicate discovery

The campaign implemented deterministic bounded candidates for:

- scalar numeric thresholds;
- pairwise numeric relations;
- temporal deltas;
- canonical candidate identity/deduplication;
- positive support and counterexample accounting;
- separate evidence and description-complexity measures;
- a frozen selection policy;
- explicit predicate-discovery controls/ablations.

These are candidate-generation mechanisms, not evidence that any one predicate
family is universally useful.

## Stage C — template induction and consolidation review

The campaign added:

- bounded first-order structural terms derived from observations;
- exact pairwise anti-unification as a baseline;
- replayable incremental multi-example anti-unification;
- positive-only template candidates with exact source provenance;
- exact first-order candidate matching with consistent repeated variables;
- explicit positive/negative constraint accounting;
- fixed-vs-contingent structural partition;
- role induction only when a contingent variable binds to observable entity
  constants across all supplied positives;
- descriptive relation-system systematicity;
- separate fit and complexity fields rather than one hidden weighted score;
- frozen held-out lexicographic selection;
- alpha-equivalence;
- evidence-bound split and alpha-equivalent merge proposals;
- a common review gate for CREATE / GENERALIZE / SPECIALIZE / SPLIT / MERGE.

Inference-time memory is not silently structurally mutated by these review
surfaces.

## Stage D — automatic analogical mapping

TDI-2.2 no longer requires the evaluator to supply the correct RoleMap for the
bounded mapping family.

The implementation now provides:

- complete role/entity correspondence candidates;
- complete injective role maps;
- typed directional relation-preservation accounting;
- shared-role systematicity;
- bounded exact mapping search for small graphs;
- deterministic structural degree mapping as a larger-domain approximation;
- explicit Selected / Ambiguous / InsufficientStructure outcomes;
- surface-identity and rotated-mapping controls;
- disjoint Development and Validation campaigns.

The executable acceptance tests require all 32 Development and all 32
Validation typed-relation cases to recover the unique exact mapping without
retuning. These are bounded synthetic mapping results, not a claim about
arbitrary graph isomorphism or human analogy.

## Stage E — surface-disjoint cross-domain transfer

The final task family makes the following surface vocabularies disjoint between
domains:

- concrete entity identifiers;
- relation identifiers;
- Boolean surface-predicate identifiers.

Only constructor-level graph topology is shared. Two domains are supplied to
induction; a third domain is evaluation-only.

The executable Development/Validation contracts require simultaneous reporting
of:

- structural transfer success;
- raw Boolean surface overlap;
- equality to the direct anti-unification baseline;
- false admission by the bounded conjunctive-rule baseline.

### Important null result encoded by the campaign

The current TDI-2.2 candidate generalization is built on incremental
anti-unification. Therefore the cross-domain campaign explicitly expects:

```
candidate generalization == direct anti-unification generalization
```

on this bounded family.

This is retained as a **null/equivalence result**, not hidden or reinterpreted as
an advantage. TDI-2.2 demonstrates an integrated intuition pipeline around that
baseline; it does **not** yet demonstrate a superior template-induction
algorithm versus anti-unification.

The Boolean conjunctive control is also intentionally diagnostic: with fully
disjoint surface predicates its positive intersection is empty and therefore
admits unrelated surface states. That result localizes the weakness of a naive
surface rule; it is not evidence against competent ILP systems in general.

## Noise and changing experience

The campaign also includes:

- deterministic dropped-predicate and novel-noise corruption with exact
  accounting;
- non-mutating Stable / Refuted / SplitSuggested concept-drift diagnosis.

These fixtures are foundations for later robustness work, not a universal
robustness claim.

## Reproducibility and accounting

`scripts/check-tdi2-2-cycle3-summary.sh` executes the three TDI-2.2 module
families:

- template learning;
- automatic analogical mapping;
- cross-domain transfer.

The dedicated TDI-2.x workflow is widened to `tdi2_*.rs`, preserves exact-head
checkout, immutable action pins, formatting, Clippy with warnings denied, full
tests and experimental MSRV qualification.

Cross-domain Validation is replayed twice and must produce identical summaries.
Logical accounting reports template nodes, induced variables, retained support,
source records and canonical bytes. Wall-clock latency is not an algorithm
input.

## Explicit non-claims

This campaign does not establish:

- human-like intuition;
- general intelligence;
- novelty of anti-unification, ILP or structure mapping;
- superiority to anti-unification on the tested cross-domain family;
- superiority to competent ILP, neural or graph-learning systems;
- universal cross-domain transfer;
- O(1) inference;
- cache, SIMD, GPU or other hardware superiority;
- protected/final confirmation.

## Protected boundary and next scientific gate

The TDI-2.1 PrimaryHoldout was not opened, generated, inspected or tuned against
by this campaign.

A later protected programme is admissible only after choosing a materially
stronger induction candidate and preregistering it against competent structural
baselines. In particular, the anti-unification equivalence observed by the
current design must not be presented as evidence of algorithmic superiority.

## Closure rule

This document becomes the cycle-3 closure record only when slice 50 and all
preceding TDI-2.2 slices are integrated and the exact-head repository checks
required by `main` complete successfully. Queued, skipped, cancelled or
older-head checks are not scientific qualification.
