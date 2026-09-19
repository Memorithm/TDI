# TDI-2.2 — Autonomous Experience Abstraction & Template Induction

Status: Stage A / Development-only. No PrimaryHoldout access is authorized.
Tracker: GitHub issue #522.

## Research question

TDI-2.1 established a bounded reference engine able to apply Boolean, contextual,
temporal and relational experiential templates once those templates and, for
relational transfer, their role mapping are supplied. TDI-2.2 asks the earlier
question:

> Can transferable templates and their role mappings be induced from declared
> experience observations without access to the expected evaluation label?

The target pipeline is:

```
experience
  -> admissible predicate candidates
  -> invariant / contingent structure discovery
  -> template induction
  -> role induction
  -> structural mapping
  -> transfer
  -> observed outcome
  -> evidence-gated consolidation
```

Runtime latency is not part of this algorithm. It remains an externally measured
consequence only.

## Operational hypotheses

### H2.2-A — Template induction

Given multiple positive episodes sharing latent structure but differing in
surface identities and nuisance observations, a deterministic induction procedure
can recover a template that admits held-out structural instances while rejecting
frozen counterexamples.

### H2.2-B — Mapping induction

Given an induced relational template and a novel target observation graph, the
role/entity mapping can be recovered from declared structure without supplying
the correct mapping or entity names as privileged features.

### H2.2-C — Evidence-conditioned consolidation

Positive and negative experience can support reviewable CREATE, GENERALIZE,
SPECIALIZE, SPLIT and MERGE proposals without silently mutating inference-time
memory and without worsening frozen negative controls.

### H2.2-D — Surface-disjoint transfer

A template induced in one surface vocabulary can transfer to a disjoint target
vocabulary when the relation system is preserved. Shared identifiers must not be
required for success.

## Null explanations and mandatory controls

The programme must preserve the following explanations until experimentally
excluded on a frozen comparison:

- exact memorization;
- nearest Boolean overlap / prototype retrieval;
- direct expected-label leakage;
- hand-authored predicate leakage;
- identity/name matching masquerading as role mapping;
- ordinary decision-tree/rule learning;
- first-order anti-unification / least-general-generalization;
- ILP-style relational rule induction;
- structure-mapping style relation matching.

A negative, equivalent or inconclusive result remains a valid TDI result.

## Literature boundary

TDI-2.2 does not claim invention of relational analogy, rule induction or
anti-unification. The programme explicitly treats the following as antecedents
and comparison families:

- Gentner (1983), *Structure-Mapping: A Theoretical Framework for Analogy*,
  Cognitive Science 7(2), 155–170, DOI 10.1207/s15516709cog0702_3. The key
  comparison is relation-preserving base-to-target mapping rather than surface
  attribute similarity.
- Falkenhainer, Forbus & Gentner (1989), *The Structure-Mapping Engine:
  Algorithm and Examples*, Artificial Intelligence 41(1), 1–63,
  DOI 10.1016/0004-3702(89)90077-5. SME is a structural mapping baseline family,
  not evidence for TDI-2.2 performance.
- Plotkin/Reynolds anti-unification / least-general-generalization: derive a
  most-specific common generalization by replacing structural disagreements with
  variables. TDI-2.2 will implement a bounded first-order baseline before claiming
  value for its own template induction.
- Muggleton (1991), *Inductive Logic Programming*, New Generation Computing
  8, 295–318, DOI 10.1007/BF03037089. ILP is a required symbolic induction
  comparison family.
- Evans & Grefenstette (2018), *Learning Explanatory Rules from Noisy Data*,
  JAIR 61, 1–64, DOI 10.1613/jair.5714. Differentiable ILP motivates a noise-aware
  comparison, not a TDI-2.2 component by default.

## Information boundary

An induction input may contain only declared observations and provenance needed
to identify the episode. It must not contain:

- expected template id;
- expected role mapping;
- expected evaluation label;
- final/PrimaryHoldout identity;
- post-hoc evaluator annotations;
- runtime latency used as a feature.

Outcome information may enter only after an action/prediction has been evaluated,
and then only through the explicit consolidation interface.

## Evaluation stages

1. Development: construct algorithms and bounded deterministic task families.
2. Validation: frozen disjoint populations; no retuning after observation.
3. Protected/final: not created or opened by this 50-PR campaign.

## Primary measurements

Report separately:

- template coverage;
- positive recall and negative false-admission rate;
- exact / partial role-mapping accuracy;
- transfer correctness;
- abstention / ambiguity rate;
- calibration of empirical reliability;
- logical operation and memory accounting;
- external latency only as a diagnostic.

Do not collapse these into one score before individual denominators are reported.

## Stop conditions

Record the corresponding hypothesis as negative/inconclusive rather than changing
it post hoc if any of the following occurs:

- induction requires expected labels or protected metadata;
- role mapping requires shared entity names/ids;
- a claimed gain vanishes against a competent structural baseline;
- Validation requires parameter retuning;
- candidate explosion violates the frozen bound;
- cross-domain transfer is explained by shared surface predicates.

## Current protected boundary

TDI-2.1 PrimaryHoldout remains closed. This programme creates no final seeds,
protected examples, final runner authorization or confirmatory result payload.
