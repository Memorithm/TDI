# TDI-2.1 — Frozen task families

Date frozen: 2026-09-16

TDI-2.1 begins with synthetic and replayable task families so causal controls, exact split provenance and counterfactual ablations remain possible before any application-specific claim.

## F1 — Prototype interpolation

Observation vectors are generated around latent prototypes with controlled noise. The target depends on prototype identity plus a bounded context variable. This family tests whether experience retrieval helps when repeated structural motifs exist.

## F2 — Context reversal

The same observation motif maps to different targets under distinct context values. This family tests whether context gating is necessary rather than decorative.

## F3 — Novel composition

Holdout cases combine previously seen subpatterns in unseen combinations. This family tests compositional transfer without permitting exact memorization.

## F4 — Distractor dimensions

Irrelevant dimensions are added with controlled variance and distribution shift. This family tests robustness of normalization and contextual feature selection.

## F5 — Ambiguous-neighbour cases

Two or more stored motifs are intentionally near-equidistant while their targets differ. This family is designed to stress confidence and abstention.

## F6 — Experience conflict

Development experience contains controlled minority contradictions or stale mappings. This family tests whether consolidation and retrieval amplify outdated evidence.

## F7 — Width transfer bridge

A later bridge family will connect TDI-2.1 back to the original TDI-2 concern: representations trained on one structural width are evaluated on a preregistered wider system without holdout adaptation.

## Generator requirements

Each generator must provide:

- deterministic seed semantics;
- a canonical configuration record;
- a structural-equivalence key for split isolation;
- exact target generation independent of candidate code;
- explicit difficulty/ambiguity parameters;
- a rejection-free or fully accounted rejection process.

No holdout distribution parameter may be changed after first protected evaluation.
