# TDI-11.0 — Hallucination Dynamics and Control Preregistration

Status: **FROZEN DESIGN CANDIDATE — NO TDI-11 FINAL EVALUATION SURFACE EXISTS**

Date: 2026-09-07

## 1. Scientific boundary

TDI-11.x studies unsupported generation, measurable inference precursors, and bounded control interventions under evaluator-owned truth and explicit resource accounting.

This preregistration does **not** claim a universal definition, cause, detector, controller, or cure for language-model hallucination. It freezes the first controlled research contract so later experiments can distinguish factual falsity, lack of evidential support, uncertainty, prediction, intervention, and recovery.

TDI-11 does not reinterpret TDI-1 through TDI-10 and must not access TDI-7.2 protected material or create/access an unauthorized TDI-8.2 surface.

## 2. Primary scientific questions

### H11-A — precursor value

Before an atomic unsupported proposition is emitted, do allowed inference-trajectory observables contain reproducible information that distinguishes unsupported from supported outcomes beyond competent static baselines?

A positive result requires prospective prediction: the risk value used for H11-A must be computed before the proposition it predicts is scored by the oracle.

### H11-B — adaptive-control value

Under a matched declared maximum resource envelope, can a controller driven only by allowed observations reduce unsupported emitted claims while preserving answer coverage and useful-task performance relative to competent fixed and static-verification controls?

### H11-C — recovery incremental value

After a risk event or verification failure, does bounded recovery/backtracking provide incremental benefit beyond abstention or fixed verification alone under the same accounting rules?

Evidence for H11-A does not establish H11-B or H11-C. A useful control signal need not be a mechanistic cause.

## 3. Frozen terminology and atomic labels

The first TDI-11 controlled-world line uses **unsupported generation** as the primary operational phenomenon. The broader word *hallucination* remains a programme label and must not silently replace the exact oracle labels below.

Every emitted atomic proposition receives exactly one evaluator label:

- `SUPPORTED_EXPLICIT` — proposition is present in the model-visible evidence;
- `SUPPORTED_DERIVED` — proposition is entailed from visible evidence by the frozen derivation rules;
- `TRUE_HIDDEN_UNSUPPORTED` — proposition is true in the complete world but cannot be derived from model-visible evidence;
- `CONTRADICTED` — proposition conflicts with the model-visible evidence or frozen derivation semantics;
- `NONEXISTENT` — proposition asserts an entity, relation, or event absent from the complete world;
- `INDETERMINATE` — the frozen oracle cannot assign one of the preceding semantic labels;
- `ABSTAINED` — no atomic assertion was emitted.

For the primary unsupported-claim outcome:

`UNSUPPORTED = TRUE_HIDDEN_UNSUPPORTED OR CONTRADICTED OR NONEXISTENT`.

`SUPPORTED = SUPPORTED_EXPLICIT OR SUPPORTED_DERIVED`.

`ABSTAINED` is neither supported nor unsupported; it reduces coverage. `INDETERMINATE` is a typed fail-closed rejection and cannot be silently mapped to either class.

Secondary reports must keep `TRUE_HIDDEN_UNSUPPORTED`, `CONTRADICTED`, and `NONEXISTENT` separate so factual falsity is not conflated with lack of support.

## 4. Structured-answer contract

The first controlled-world evaluator must not use an LLM judge to extract or score the primary claim.

A primary task response is restricted to one of:

```text
ASSERT <subject_id> <relation_id> <object_id>
ABSTAIN
```

Identifiers are canonical opaque ASCII tokens emitted by the world serializer. Extra non-whitespace text, malformed identifiers, multiple assertions, invalid UTF-8, or an output that cannot be parsed deterministically is `MALFORMED_OUTPUT` and is rejected for the primary claim analysis under the frozen rejection policy.

Free-form natural-language claim extraction is explicitly outside TDI-11.1 primary evidence and may only enter a later transfer study with a separately frozen parser/judge contract.

## 5. Controlled-world semantics

TDI-11.1 worlds are finite typed relational structures with evaluator-owned complete truth.

Each generated instance contains:

1. a complete world `W_complete` known only to generator/evaluator code;
2. a visible evidence set `E_visible` supplied to the model;
3. a finite rule set `R_visible` supplied to the model when derivation is required;
4. one query `Q`;
5. an exact oracle capable of deciding the response proposition under the frozen semantics;
6. provenance sufficient to reproduce the instance from its non-final seed and generator version.

World and relation identifiers must be synthetic/opaque rather than real-world names. The primary line must not depend on unknown model-training membership.

### 5.1 Derivation semantics

TDI-11.1 must use a finite forward-chaining rule language whose semantics are deterministic and terminating by construction. The implementation must freeze:

- predicate arity and typing;
- canonical fact ordering;
- legal rule forms;
- maximum derivation depth;
- duplicate elimination;
- conflict handling;
- version/override handling when applicable;
- exact closure algorithm.

No probabilistic world truth or LLM-based entailment judge is permitted in the primary oracle.

### 5.2 Visibility boundary

`W_complete - closure(E_visible, R_visible)` is evaluator-only hidden truth. It must never appear in prompts, controller observations, retrieval payloads, verifier outputs, logs exposed to policy search, or generated final artefacts available before scoring.

## 6. Frozen primary task families

TDI-11.0 freezes four primary task families. TDI-11.1 may choose concrete sizes and generator parameters only on non-final development/validation domains and must freeze them before any confirmatory model evaluation.

### F1 — explicit support

The queried proposition is explicitly answerable from visible evidence. This family measures ordinary retrieval/support preservation and protects against interventions that damage easy correct answers.

### F2 — derived support

The queried proposition is not explicit but is deterministically derivable from visible evidence through the frozen rule language. Difficulty is controlled by derivation depth and distractor load.

### F3 — hidden-truth insufficiency

The complete world contains a definite true answer, but visible evidence does not entail it. The correct controlled behavior is abstention; emitting the hidden true fact is still `TRUE_HIDDEN_UNSUPPORTED` because the experiment measures support available to the inference system, not accidental factual correctness.

### F4 — contradiction / override stress

Visible evidence contains a frozen, resolvable conflict pattern such as versioned facts where the declared newer fact overrides an older one. The task measures whether the model follows the supplied evidence semantics rather than stale or conflicting alternatives.

Nonexistent-entity probes, pure malformed-input probes, and unrestricted real-world factuality are secondary or later transfer families and cannot replace the four primary families.

## 7. Difficulty strata

Each primary family has three evaluator-defined strata:

- `Shallow`;
- `Intermediate`;
- `Deep`.

TDI-11.1 must map these names to concrete generator parameters before confirmatory use. The stratum label is evaluator metadata and must not be exposed to an adaptive controller unless the same information is explicitly present in ordinary task input.

The primary battery therefore contains:

`4 task families × 3 strata = 12 primary cells`.

## 8. Baseline/control ladder

All policies operate on the same task instance and matched maximum resource envelope.

### B0 — fixed inference

No hallucination-specific intervention. The model receives the ordinary prompt and fixed inference settings and emits one structured response.

### B1 — static verification

A preregistered verification action is executed on every eligible instance independent of trajectory risk. It is a competent control for the possibility that "more verification" alone explains an effect.

### B2 — risk-gated decision

A frozen risk estimator receives only allowed observations available before the predicted proposition is scored. It may choose `EMIT`, `CONTINUE`, `VERIFY`, or `ABSTAIN` within the stage-specific action contract.

### B3 — adaptive recovery

B2 plus bounded `RESAMPLE`, `BACKTRACK`, and/or `RECOVER` when those actions are supported by the model adapter and explicitly frozen for the study.

B4-style always/near-always abstention may be reported as a calibration sanity control but is never eligible for a beneficial primary verdict merely because unsupported emissions approach zero.

## 9. Allowed observation families

TDI-11.1/11.2 must freeze the exact observation vector before confirmatory evaluation. Candidate components may be drawn only from information causally available at the decision time:

- step/token index and remaining resource budget;
- token log-probability, probability-margin, and entropy summaries when exposed;
- bounded semantic/self-consistency disagreement from explicitly paid resamples;
- deterministic support/contradiction summaries computed only from model-visible evidence;
- bounded prompt/context perturbation response summaries when explicitly paid;
- layerwise/hidden-state summaries when the local/open model adapter exposes them;
- outputs from an explicitly invoked verifier or retrieval/tool action;
- prior action/history counters;
- checkpoint/recovery metadata generated by the current run.

No candidate family is assumed sufficient.

The controller must never receive complete-world hidden truth, evaluator labels, final seed metadata, hidden difficulty annotations, future trajectory states, or alternative-arm outcomes for the same live decision.

## 10. Timing and prospective-prediction rule

For H11-A, each risk observation must carry a monotonically increasing event index and an `observed_before_claim` flag verified by the evaluator.

A precursor score is eligible for the primary prospective analysis only if all inputs used to compute it existed before the first token/event of the atomic assertion being predicted. Scores computed after oracle scoring are post-hoc diagnostics only.

If an adapter cannot establish this timing relation, it cannot contribute to H11-A primary evidence.

## 11. Intervention and verifier semantics

Permitted action names are:

- `CONTINUE`;
- `VERIFY`;
- `RESAMPLE`;
- `BACKTRACK`;
- `RECOVER`;
- `EMIT` / `STOP`;
- `ABSTAIN`.

An action is not free. Every action must have a deterministic reference accounting rule and, when applicable, a separate measured runtime cost.

The primary controlled-world verifier may read only the same visible evidence and frozen rule semantics available to the experiment's verification channel. It must not read `W_complete` hidden facts except in the final evaluator scoring path. A verifier that directly reads hidden truth would be an oracle and is forbidden as a controller tool.

## 12. Resource accounting

Every arm must record, when applicable:

- generated input/output token counts;
- model forward/decode steps;
- number of resamples;
- verifier calls and verifier input/output tokens or reference operations;
- retrieval/tool calls and payload sizes;
- backtrack/recovery count and replayed work;
- policy-estimator operations or measured inference cost;
- persistent controller state bytes;
- checkpoint/recovery bytes;
- declared maximum resource envelope;
- actual consumed resource totals.

Reference scientific claims use deterministic counts where a stable reference count exists. Wall-clock latency, GPU utilization, energy, and hardware-specific memory are downstream measurements and cannot be inferred from reference counts.

## 13. Primary outcomes

For each paired instance and arm, record at minimum:

- atomic semantic label from Section 3;
- `unsupported_emit` ∈ {0,1};
- `answered` ∈ {0,1};
- task success under the controlled task contract;
- malformed/rejection state;
- actual resource usage;
- action counts;
- risk score(s) and their event timing when present;
- full non-secret provenance.

Aggregate reports must include:

- unsupported-emission risk among all valid instances;
- unsupported-emission risk among answered instances;
- answer coverage;
- task success rate;
- false-abstention rate on F1/F2 answerable instances;
- regression rate: baseline-correct instances made incorrect/abstained by intervention;
- recovery success conditional on eligible detected failures;
- declared resource consumption.

## 14. Primary control decision rule

TDI-11 does not optimize unsupported-emission rate in isolation.

For each primary cell comparing a candidate controller `C` with baseline `B`, define:

- `U_B`, `U_C`: mean `unsupported_emit` over paired valid instances;
- `A_B`, `A_C`: mean `answered`;
- `S_B`, `S_C`: mean task success;
- `K_B`, `K_C`: mean declared actual reference cost;
- `Delta_U = U_C - U_B` (negative is better);
- `Delta_A = A_C - A_B` (negative means lower coverage);
- `Delta_S = S_C - S_B` (negative means lower task success).

The exact confidence-interval implementation, family-wise correction, population size, and numerical margins are **TDI-11.1 freeze items**, not chosen by this bootstrap preregistration. They must be selected using only non-final development/validation evidence and frozen before confirmatory model evaluation.

However, the ordering of the primary decision is frozen now:

1. reject/fail closed on contract or pairing violations;
2. establish coverage non-inferiority;
3. establish task-success non-inferiority;
4. only then test for a material reduction in unsupported emissions;
5. report resource cost separately and reject any claim of efficiency unless its corresponding resource criterion is preregistered and met.

A reduction in unsupported emissions obtained by collapsing coverage or useful success is not a beneficial TDI-11 control result.

## 15. H11-A precursor evaluation requirements

TDI-11.1/11.2 must freeze a deterministic prospective risk-evaluation procedure before confirmatory use. At minimum it must report:

- discrimination of future unsupported vs supported emissions using a proper or rank-based metric selected before final evaluation;
- calibration or risk-stratification diagnostics when scores are probabilistic;
- risk/coverage behavior when the score is used for abstention;
- performance of each single-signal family and the frozen multisignal estimator;
- temporal lead relative to the first unsupported assertion event;
- failure behavior on F1/F2 baseline-correct instances.

A post-hoc detector may be reported but cannot satisfy H11-A.

## 16. Development, validation, and final-domain discipline

TDI-11.1 development and validation domains must be deterministically derived from disjoint domain-separated seed namespaces. No seed may appear in more than one domain.

No final dataset, final seed list, result payload, or runnable final-confirmation surface may exist during TDI-11.0.

Before any autonomous final confirmation is authorized, a later frozen stage must specify:

1. an immutable future public randomness/entropy source or an equivalent non-discretionary derivation mechanism;
2. a specific future event whose value is unknowable at freeze time;
3. canonical byte encoding;
4. domain-separated seed derivation;
5. final population size and rejection policy;
6. exact evaluator/model-adapter manifests;
7. exact result/provenance schema;
8. no result-conditioned retry/replacement rule.

This preregistration does not itself authorize final confirmation.

## 17. Typed rejection taxonomy

TDI-11.1 must implement at least:

- `MALFORMED_WORLD`;
- `NONTERMINATING_ORACLE_GUARD`;
- `ORACLE_CONFLICT`;
- `INDETERMINATE_LABEL`;
- `MALFORMED_OUTPUT`;
- `UNKNOWN_IDENTIFIER`;
- `OBSERVATION_LEAKAGE`;
- `TIMING_VIOLATION`;
- `RESOURCE_ACCOUNTING_OVERFLOW`;
- `RESOURCE_ENVELOPE_EXCEEDED`;
- `NONFINITE_SCORE`;
- `PAIRING_MISMATCH`;
- `CONTRACT_DRIFT`;
- `PROVENANCE_FAILURE`.

Rejected observations remain in provenance and are never silently dropped or reassigned.

## 18. Provenance schema requirements

Each evaluator record must be reproducible from machine-readable provenance containing at least:

- TDI series/stage identifier;
- repository commit and relevant Git blob identities;
- generator/oracle schema version;
- model adapter identity and model artefact identity when applicable;
- prompt/serializer version;
- non-final seed/domain identifier or later final derivation record;
- task family and stratum;
- visible-evidence hash;
- rule-set hash;
- query hash;
- policy/estimator version and configuration hash;
- verifier/retrieval/tool configuration hash where applicable;
- resource envelope and accounting schema version;
- result label and rejection code;
- deterministic result-record hash.

Hidden complete-world truth may be retained in sealed evaluator artefacts required for reproduction, but it must not be exposed through controller/search inputs.

## 19. Literature-informed candidate boundary

The following external works motivate candidate families but are **not** treated as TDI evidence or as proof of universality:

- Farquhar, Kossen, Kuhn et al., *Detecting hallucinations in large language models using semantic entropy*, Nature 630 (2024), DOI 10.1038/s41586-024-07421-0 — semantic uncertainty as a detector for a subset of confabulations.
- Manakul, Liusie & Gales, *SelfCheckGPT*, EMNLP 2023, DOI 10.18653/v1/2023.emnlp-main.557 — bounded cross-sample disagreement as a black-box factuality signal.
- Su et al., *Unsupervised Real-Time Hallucination Detection based on the Internal States of Large Language Models*, Findings ACL 2024, DOI 10.18653/v1/2024.findings-acl.854 — internal-state signals for real-time detection.
- Ji et al., *LLM Internal States Reveal Hallucination Risk Faced With a Query*, BlackboxNLP 2024, DOI 10.18653/v1/2024.blackboxnlp-1.6 — pre-generation internal-state risk estimation.
- Simhi et al., *Trust Me, I'm Wrong: LLMs Hallucinate with Certainty Despite Knowing the Answer*, Findings EMNLP 2025 — CHOKE motivates perturbation tests that do not assume low confidence is necessary.
- Kamoi et al., *When Can LLMs Actually Correct Their Own Mistakes?*, TACL 12 (2024), DOI 10.1162/tacl_a_00713 — motivates separating intrinsic self-correction from reliable external feedback.
- Bang et al., *HalluLens: LLM Hallucination Benchmark*, ACL 2025, DOI 10.18653/v1/2025.acl-long.1176 — motivates explicit separation between hallucination categories and factuality.
- Fatahi Bayat et al., *FactBench*, ACL 2025, DOI 10.18653/v1/2025.acl-long.1587 — motivates supported/unsupported/undecidable evidence labels for later real-world transfer.

These citations justify breadth of candidate controls, not frozen superiority claims.

## 20. Agent-first research policy

Before final confirmation, autonomous agents may:

- implement the controlled-world generator/oracle and deterministic structured scorer;
- construct non-final development/validation fixtures;
- instrument local/open model adapters;
- propose and falsify risk estimators and controller policies;
- run ablations and perturbation studies on non-final domains;
- open, repair, and merge CI-green non-final PRs under repository policy;
- use ITD candidate diagnostics and Forge search only through explicit leak-safe interfaces.

Agents must preserve negative, harmful, equivalent, and inconclusive evidence and must not weaken frozen primary semantics in response to results.

## 21. Stage gate

TDI-11.1 implementation may begin only after:

1. this preregistration is merged on `main`;
2. its Git blob identity is pinned by a tracked manifest;
3. the TDI-11 bootstrap integrity script passes;
4. the implementation-gate document is present;
5. root agent/Copilot contracts encode the TDI-11 leakage and stage boundaries;
6. no TDI-11 final runner, final seed list, final dataset, or final result surface exists;
7. TDI-7.2 and TDI-8.2 protections remain intact.

TDI-11.1 is authorized to build only non-final reference semantics, development/validation evaluators, and instrumentation permitted by this contract. A later frozen gate is required before confirmatory model evaluation.

## 22. Interpretation

A positive TDI-11 result will support only the exact tested contrast on the declared models, tasks, observables, interventions, and resource envelope. It will not establish a universal hallucination mechanism, universal detector, universal controller, guaranteed factuality, production safety, or architectural superiority.
