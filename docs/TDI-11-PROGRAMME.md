# TDI-11.x — Hallucination Dynamics and Control Research Programme

TDI-11.x is the TDI scientific line dedicated to studying hallucination as an observable inference phenomenon and to testing whether measurable precursors can support bounded interventions that reduce unsupported generation without collapsing useful answer coverage.

The programme does **not** assume that hallucination has a single cause, that uncertainty alone is sufficient, or that complete elimination is possible. It separates detection, localization, verification, recovery and abstention under explicit evidence and compute budgets.

## Research objective

The programme asks whether an inference trajectory contains reproducible, intervention-relevant signals before or during unsupported generation, and whether those signals can drive a controller that improves the quality/risk frontier relative to fixed inference.

Working controller label: **HAC — Hallucination Adaptive Controller**.

`HAC` is a working engineering/research label only. TDI-11 makes no novelty, universality or superiority claim unless later evidence supports a narrower statement.

## Stage map

| Stage | Purpose | Status |
| --- | --- | --- |
| **TDI-11.0** | Freeze scope, taxonomy, evidence boundary and implementation gate | ✅ Frozen by preregistration once merged/blob-verified |
| **TDI-11.1** | Build deterministic fully specified worlds and reference evaluator | 🟠 Authorized after TDI-11.0 bootstrap passes on `main` |
| **TDI-11.2** | Instrument prospective trajectory precursors on local/open models | ⏳ Future; requires model/observation freeze |
| **TDI-11.3** | Run controlled causal perturbation experiments | ⏳ Future |
| **TDI-11.4** | Compare single-signal and multisignal hallucination-risk estimators | ⏳ Future |
| **TDI-11.5** | Evaluate bounded adaptive control: emit, continue, verify, recover, backtrack or abstain | ⏳ Future |
| **TDI-11.6+** | Transfer, robustness, architecture comparison and evidence-qualified promotion | ⏳ Conditional |

## Frozen TDI-11.0 contract

The TDI-11.0 preregistration fixes the first controlled research boundary before model evidence exists:

- primary phenomenon: unsupported generation under declared model-visible evidence;
- exact evaluator-owned controlled-world truth;
- structured primary response `ASSERT <subject_id> <relation_id> <object_id>` or `ABSTAIN`;
- no LLM judge in primary controlled-world scoring;
- exact atomic support/error labels;
- F1 explicit support, F2 derived support, F3 hidden-truth insufficiency, F4 contradiction/override stress;
- Shallow/Intermediate/Deep strata, producing 12 primary cells;
- B0 fixed inference, B1 static verification, B2 risk gating, B3 adaptive recovery;
- prospective timing requirement for precursor evidence;
- coverage and task-success protection before unsupported-risk reduction can count as beneficial;
- explicit resource accounting, typed rejection and provenance;
- strict hidden-truth leakage boundary;
- development/validation/final separation.

Concrete model adapters, generator sizes, exact observation vectors, numerical decision margins and final-confirmation derivation remain later freeze items.

## Core decomposition

TDI-11 separates five problems that must not be silently conflated:

1. **Detection** — is the current or candidate output unsupported under the declared evidence model?
2. **Localization** — where does the unsupported claim or transition first appear?
3. **Diagnosis** — what observable state distinguishes the failing trajectory from matched controls?
4. **Intervention** — can a bounded action change the outcome?
5. **Decision** — should the runtime emit, continue, verify, recover, backtrack or abstain?

A detector that only identifies an error after completion is useful evidence but is not equivalent to prevention. A verifier that corrects an answer is not evidence that its score was predictive before the error. These contrasts remain explicit.

## TDI-11.1 controlled-world principle

The first evaluator line uses fully specified deterministic synthetic worlds where the experiment owns the complete reference truth and can label:

- facts explicitly provided to the model;
- facts deterministically derivable from provided facts;
- hidden but true facts;
- contradictions introduced by intervention;
- nonexistent entities or relations;
- unsupported assertions.

This avoids depending on unknown training-set membership for the first mechanistic experiments. External factuality benchmarks may be added later as transfer evidence, not as the sole scientific oracle.

The first implementation slice is:

`World schema -> visible/hidden partition -> deterministic closure -> ASSERT/ABSTAIN parser -> exact support label -> rejection/provenance record`.

## Candidate observables

Later TDI-11 stages may select from candidate observables, but the exact vector must be frozen before confirmatory evaluation. Candidate families include:

- token probability, margin and entropy summaries;
- semantic disagreement across bounded resamples;
- contradiction/support scores against declared context or evidence;
- sensitivity to bounded prompt/context/decoding perturbations;
- layerwise or hidden-state summaries where the model/runtime exposes them;
- retrieval/verifier disagreement;
- trajectory changes induced by extra compute or recovery actions.

No observable is presumed sufficient. Internal-state observables are architecture/runtime dependent and must not be treated as universally available.

## Candidate actions

TDI-11 aligns with the existing TDI-9 adaptive-inference vocabulary where possible. Candidate bounded actions are:

- `EMIT` / `STOP`;
- `CONTINUE`;
- `VERIFY`;
- `BACKTRACK` / `RECOVER`;
- bounded `RESAMPLE`;
- evidence retrieval or tool verification when a study explicitly permits it;
- `ABSTAIN`.

The controller must not receive hidden evaluator labels, complete-world hidden truth, final seeds, future events or alternative-arm outcomes as policy features.

## Primary evaluation axes

A TDI-11 controller is not judged by hallucination rate alone. At minimum, experiments report jointly:

- unsupported-claim risk under the declared oracle;
- answer coverage;
- task accuracy or utility;
- calibration/risk stratification where meaningful;
- false-abstention rate;
- regression rate on answers that were correct without intervention;
- compute and memory cost under the declared accounting model;
- verifier/retrieval/tool cost when used.

A trivial always-abstain policy is therefore not a successful controller.

## Relationship to TDI-9

TDI-11 owns the hallucination-specific scientific questions, labels, precursor studies and risk-estimation experiments.

TDI-9 remains the generic adaptive-inference policy line. Evidence-qualified TDI-11 risk signals may later become allowed TDI-9 trajectory observables or may be evaluated through an explicit TDI-9-compatible adapter. This relationship does not make a TDI-11 result automatically a generic adaptive-inference result.

## Ecosystem interfaces

- **ITD Simulator** may contribute versioned structural trajectory diagnostics as candidate information sources, never as assumed truth.
- **SciRust** is a promotion target for reusable statistical, calibration or control primitives after qualification.
- **Forge** may search bounded detector/controller candidates only after TDI-11 defines a leak-safe search contract.
- **NNIS** may later expose lower-level model/runtime signals and measure NVIDIA execution cost.
- **ElasticXxx** may later consume evidence-qualified risk/compute policies for adaptive resource actuation.
- **TDI-8** may later provide an architecture-transfer target for recurrent/associative systems, without importing TDI-8 protected final material.

## Evidence discipline

TDI-11 inherits the repository-wide rules:

1. preregister before confirmatory execution;
2. keep development, validation and final confirmation surfaces disjoint;
3. preserve negative, harmful, equivalent and inconclusive results;
4. freeze metrics, decision rules, observation schema and intervention budget before final evaluation;
5. never expose final-evaluation labels or hidden truth to candidate generation;
6. classify claims by what the experiment actually demonstrates;
7. require separate evidence for architecture, runtime, hardware or cross-model transfer claims.

## Non-claims

TDI-11 does not currently claim:

- a universal definition of hallucination;
- a universal hallucination detector;
- that uncertainty is a complete explanation;
- that hallucinations can be eliminated;
- that hidden-state signals transfer across architectures;
- that HAC is scientifically novel;
- that any controller improves every model or task;
- that reduced unsupported generation implies improved factuality in every setting;
- that a research controller is production-safe.

## Current gate

TDI-11.1 non-final reference implementation may begin after the TDI-11.0 preregistration is merged, its Git blob identity is verified, and `scripts/check-tdi11-bootstrap.sh` passes on `main`.

That authorization does not create or authorize final/confirmatory model evaluation. Before such evaluation, a later gate must freeze concrete model/adaptor identities, observation timing/vector, generator parameters, statistical margins, resource envelopes, final population, provenance and non-discretionary final-seed derivation.
