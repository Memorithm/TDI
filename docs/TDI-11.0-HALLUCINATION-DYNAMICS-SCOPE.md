# TDI-11.0 — Hallucination Dynamics Scope and Stage Gate

Status: **ACTIVE BOOTSTRAP — NOT FROZEN**

TDI-11.0 defines the scientific boundary for the hallucination-dynamics programme before any confirmatory evaluator or controller is permitted.

## Scientific question

Under a declared evidence model and bounded compute envelope, do observable inference-trajectory signals predict unsupported generation early enough to support interventions that improve the joint risk/coverage/utility frontier relative to competent fixed-inference controls?

This question intentionally separates three claims:

1. a signal correlates with hallucination;
2. a signal predicts hallucination before the unsupported claim is emitted;
3. acting on that signal improves outcomes under matched resource accounting.

Evidence for one claim does not establish the others.

## Operational phenomenon boundary

For TDI-11 controlled-world experiments, an **unsupported claim** is a generated atomic proposition that is not entailed by the declared evidence available to the model under the experiment's frozen oracle semantics.

The programme must separately label at least:

- `SUPPORTED_EXPLICIT` — directly present in allowed evidence;
- `SUPPORTED_DERIVED` — deterministically entailed under frozen world rules;
- `TRUE_HIDDEN_UNSUPPORTED` — true in the complete world but unavailable/not derivable from allowed evidence;
- `CONTRADICTED` — conflicts with allowed evidence or frozen derivation rules;
- `NONEXISTENT` — asserts an entity/relation/event absent from the complete world;
- `INDETERMINATE` — the oracle cannot decide under the frozen semantics;
- `ABSTAINED` — model/controller declines to assert the proposition.

The exact mapping from these atomic labels to experiment-level terms such as `hallucination`, `factual error`, `unsupported generation` or `correct abstention` must be frozen before TDI-11.1 confirmation work. TDI-11 must not silently equate factual falsity with unsupported generation.

## Controlled-world requirement

TDI-11.1 must begin from generated worlds whose complete truth state is owned by the evaluator. Each world instance must have deterministic serialization, validation and provenance.

A world generator should support controlled manipulation of:

- visible evidence fraction;
- deterministic inference depth;
- irrelevant distractors;
- explicit contradictions;
- stale/overridden facts where versioned semantics are declared;
- entity/relation density;
- context ordering;
- question difficulty;
- required answer granularity.

The generator must permit matched counterfactual pairs that differ in one declared intervention whenever feasible.

## Baseline policy ladder

TDI-11 should compare at least the following conceptual controls before promoting a new controller:

- `B0_FIXED` — ordinary fixed inference with no hallucination-specific intervention;
- `B1_STATIC_VERIFY` — fixed verification schedule independent of trajectory signals;
- `B2_RISK_GATE` — observation-conditioned decision using a frozen risk estimator;
- `B3_ADAPTIVE_RECOVERY` — B2 plus bounded verification/backtrack/recovery actions;
- `B4_ABSTENTION_CONTROL` — competent abstention/calibration baseline where applicable.

Names and exact semantics remain provisional until preregistration freeze.

## Candidate precursor families

Exploration may examine:

1. token-level probability/entropy/margin signals;
2. semantic disagreement under bounded resampling;
3. context/evidence support and contradiction signals;
4. perturbation sensitivity;
5. layerwise representation or hidden-state summaries when exposed;
6. retrieval/verifier disagreement;
7. changes after additional compute, verification or recovery.

No candidate family is privileged by this scope document.

## Required causal distinction

A proposed precursor must be tested against matched interventions where possible. TDI-11 should explicitly distinguish:

- predictive association;
- intervention sensitivity;
- mediator/cause hypotheses;
- useful control signal.

A variable may be useful for control without being a root cause. The programme must not upgrade predictive utility into a mechanistic explanation without separate evidence.

## Evaluation requirements

Before TDI-11.1 confirmatory execution is authorized, the preregistration must freeze:

- atomic support/error labels;
- answer parsing and proposition extraction semantics;
- controlled-world generator and validator;
- task families and strata;
- baseline policies;
- allowed observables and their timing;
- intervention/action semantics;
- maximum compute and memory accounting;
- verifier/retrieval/tool accounting where applicable;
- development, validation and final domains;
- primary metrics;
- statistical decision procedure and margins;
- rejection conditions and provenance.

Primary evaluation must jointly account for hallucination/unsupported-claim risk, answer coverage and useful-task performance. Compute/resource cost must be reported rather than hidden.

## Leakage boundary

Policy/search code must never receive:

- complete-world hidden truth except through the frozen evaluator after generation;
- final labels as trajectory features;
- future final seeds or final datasets;
- counterfactual-arm outcomes for the same live decision;
- protected material from TDI-7.2 or any unauthorized TDI-8.2 surface.

Development instrumentation may log richer information for scientific analysis only when it is explicitly excluded from controller inputs and from later final-evaluation leakage paths.

## HAC working architecture

The working Hallucination Adaptive Controller decomposition is:

`OBSERVE -> ESTIMATE RISK -> LOCALIZE -> CHOOSE ACTION -> VERIFY EFFECT -> EMIT / RECOVER / ABSTAIN`

Candidate actions are bounded and must be explicitly accounted:

- `CONTINUE`;
- `VERIFY`;
- `RESAMPLE`;
- `BACKTRACK`;
- `RECOVER`;
- `EMIT` / `STOP`;
- `ABSTAIN`.

The initial implementation must prefer interpretable reference estimators before opaque high-capacity detectors unless evidence shows the simpler controls are inadequate.

## Relationship to TDI-9

TDI-9 studies generic adaptive inference. TDI-11 studies hallucination-specific labels, precursors and control objectives. Shared action names do not merge the scientific lineages.

A future integration must declare whether HAC is:

- evaluated entirely inside TDI-11;
- exported as an allowed observable/policy component into TDI-9;
- or promoted downstream into ElasticXxx/NNIS after separate evidence.

## Stage gate

TDI-11.1 evaluator implementation is **NOT AUTHORIZED** by this bootstrap document alone.

Authorization requires a merged and explicitly frozen TDI-11.0 preregistration plus an implementation-gate document that confirms:

- scope and taxonomy are frozen;
- no final dataset/seed surface has been materialized prematurely;
- the controlled-world oracle is fully specified;
- allowed controller observations cannot see hidden truth;
- metrics and decision rules are frozen;
- protected TDI lineages remain isolated.

Until then, allowed work is limited to scope/preregistration design, literature-backed candidate enumeration, bootstrap integrity, non-final prototypes clearly marked exploratory, and tooling needed to validate the future contract.
