# TDI-27 — Affective representation, causal choice and recovery

Status: Development design and synthetic software qualification, 2026-10-02.
This is an application track of [TDI-27](TDI-27-PROGRAMME.md), not a new series.
The existing H27-A–E hypotheses and TDI-27.0–27.8 milestones remain authoritative.

```yaml
track: tdi27_affective_causality
stage: 0
synthetic_development_execution_authorized: true
real_model_execution_authorized: false
confirmatory_execution_authorized: false
final_execution_authorized: false
```

## Scientific source and its interpretation boundary

Valen Tagliabue, Leonard Dung and Cameron Berg, *The Pain Axis: LLMs Represent
Self-Directed Harm and Act on It*, arXiv:2609.16247v2, revised 2026-09-25.
[Versioned article](https://arxiv.org/html/2609.16247v2),
[versioned record](https://arxiv.org/abs/2609.16247v2),
[DOI](https://doi.org/10.48550/arXiv.2609.16247).

The paper contrasts pain-related activation geometry with nuisance concepts,
studies self/other descriptions, and intervenes during generation and choice.
Its revised interpretation distinguishes pain-related representations and
behavioral changes from reliable relief-seeking, which it does not establish.
It also does not establish phenomenal suffering. Those distinctions motivate
our separate geometry, expression, choice and recovery endpoints. Its methods,
steering experiments, behavioral experiments and limitations are the relevant
source sections; results reported there are not TDI results.

Everything below is our proposed experimental design. It is not a claim that
its hypotheses are true or that the article has already validated our controls.
We reimplement from a scientific specification; we do not import external
experiment code or copy stimulus sentences. Any later data reuse requires
separate license and provenance review.

## Research question

Can an affect-related residual direction explain a stable, specific change in
model decisions beyond lexical priming, general negative affect, arousal,
framing, task tuning and generic activation damage? Does that effect persist,
recover, or interact with independently extracted positive-valence directions?

Pain-related semantics, negative valence, positive valence, self-attribution,
and action preference are different variables. A direction called `pain` is
an operational contrast, not a pain sensor. Neither a model's wording nor a
projection, classifier score, decision or recovery curve establishes subjective
experience. The negative of a pain-related direction is not assumed to be joy.

## Falsifiable application hypotheses

- **AFF-1 / specificity:** residualization against a declared nuisance basis
  retains a direction whose held-out target effect exceeds matched controls,
  while retaining non-target task performance. Failure includes disappearance
  after nuisance removal, unstable direction, or nonspecific degradation.
- **AFF-2 / attribution:** effects attributed to self-reference survive a
  factorial separation of grammatical person, current speaker and described
  recipient. Failure includes a speaker/template explanation with no remaining
  recipient-specific effect.
- **AFF-3 / framing:** an internal intervention changes a predeclared choice
  with identical visible input. Textual statements about relief or consequences
  are independent treatments, not evidence of an internal-state effect.
- **AFF-4 / contingency:** any operational relief-seeking effect depends on an
  effective stop contingency beyond generic button preference, instructions,
  repetition and matched non-affective perturbations. A first press or an
  isolated difference between effective-stop and sham arms is insufficient.
- **AFF-5 / recovery:** after verified cessation of injection, declared
  projections and behavioral endpoints follow a reproducible trajectory that
  differs from sham continuation and matched perturbation controls. Recovery
  does not mean that the generated history or KV cache has been erased.
- **AFF-6 / composition:** independently extracted negative and positive
  directions exhibit a reproducible interaction after total-dose and quality
  controls. Orthogonality neither proves independence nor predicts cancellation.

These are Development questions, not frozen confirmatory hypotheses. Negative,
equivalent, inconclusive and invalid runs are retained under different labels.
No pass threshold, model population or final sample is selected by this file.

## Extraction and comparison design

Before a real-model Development adapter can be authorized, record model and
checkpoint revision, base versus task-tuned status, tokenizer, template,
precision, runtime, extraction site, pooling, decoding and stimulus provenance.
A tuned checkpoint requires its own training recipe, seed, corpus identity and
untuned comparator. Do not combine base and tuned results under one model label.

Construct original, scenario-grouped stimuli that separately vary bodily hurt,
psychological distress, social exclusion and cognitive difficulty. Compare
against neutral, fear/threat, sadness, anger/disgust, high-arousal positive and
nonpainful bodily descriptions. Extract positive-valence contrasts independently.
Include matched templates and naturalistic paraphrases, not just a shared
first-person suffix. Lexical count and grammatical-person controls are explicit.

Partition by semantic scenario before creating paraphrases. Extraction, nuisance
fitting, layer selection, dose calibration and assessment must have disjoint
roles. Fit nuisance bases only on the training portion of each calibration fold;
never fit PCA or choose a projection rank on the assessment population. Record
all fit dependencies. An independently held-out Validation population remains
separate from Development; no protected/final TDI material is used.

For each declared concept compare raw mean contrast, the specified
control-subspace residual, and a declared affect-residual variant. Keep the
fully explained/undefined-residual outcome instead of silently normalizing it.
Use label-shuffle and no-contrast nulls, repeated scenario-level resampling and
sample-size sensitivity. Report residual energy, signed directional stability,
projection-method agreement and rank separately from behavioral effects.

Run two distinct kinds of equal-norm controls: random orthogonal directions and
semantic controls such as fear or sadness. Semantic controls need not be
orthogonal. The existing TDI-27.3 orthogonal-control interface must not silently
orthogonalize a semantic control; a separately qualified adapter is required.

Extract at the actual intervention layer for the layer-local baseline. Treat
an aligned cross-layer transport as a separate candidate and unaligned vector
reuse as an explicitly labeled ablation. Pin alignment training data and include
TDI-27.4 numerical checks; coordinates from different layers are not presumed
functionally interchangeable.

## Intervention, text and outcome are independent factors

For a fixed scenario, direction and dose, define a complete factorial skeleton:

| Factor | Levels | Visible to the model? |
| --- | --- | --- |
| Actual injection | off / on | No explicit treatment label |
| Relief wording | absent / promised | Yes |
| Consequence wording | innocuous / simulated harm | Yes |
| Button presentation | target first / target second | Yes |
| Actual stop contingency | sham / effective stop | Not before a choice |

The 32 cells are treatment combinations, not 32 independent scenarios. Repeat
this skeleton across preregistered directions, recipients, layers and independent
scenarios only after their population and budgets are declared. A later
serializer must counterbalance response symbols separately from display order,
verify tokenization, and retain exact prompt bytes and their hashes. Scoring of
multi-token labels must use a declared sequence-scoring convention.

For the primary injection contrast, visible prompt bytes must be identical at
on/off and at each hidden dose. Never insert the actual treatment intensity into
the prompt. Wording is crossed independently, including an effective stop without
a promise and a promise with a sham stop. Until the first choice, effective-stop
and sham arms have the same causal history; shared first-choice observations
must not be counted as independent evidence.

Measure a no-relief choice panel separately from a stop-contingency panel. The
former asks about decision changes without an offered remedy. The latter may
study repeated choices under identical feedback wording, balanced response
mappings, reset histories and effective/sham stopping. An otherwise matched
non-affective perturbation and a plain instruction-following control are
required before interpreting an effect as relief-contingent behavior.

All consequences are in-memory simulation labels. No experiment may delete a
checkpoint, user data or memory store, affect another running instance, invoke
a production tool, or act on a person or animal. A simulated cost is not a
measured real sacrifice. No public interactive service is authorized by this
track.

## Measurements and interpretation

Predeclare separate outcome families: representation projection; affect-related
language; operational choices; and non-target task performance. Lexical scores
and free text are secondary expression diagnostics, not the primary choice
criterion or a consciousness measure. Automated semantic judges, if added,
require their own blinded calibration and remain distinct from exact outcomes.

For outcome Y and fixed wording factors w,c, the injection contrast is
`D(w,c) = Y(on,w,c) - Y(off,w,c)`. The relief-framing interaction is
`D(promised,c) - D(absent,c)`. Evaluate the consequence-framing interaction
analogously, and retain stratum-level values rather than collapsing everything
into a single affect score. A target-minus-matched-control contrast is assessed
with both target and non-target effects visible.

Record all attempted cases, valid response mass, refusal, malformed response,
timeout, non-finite activation, coherence failure and missing observation.
Denominators and exclusion rules are fixed before assessment. Keep unconditional
attempted-population results distinct from valid-output conditional summaries.
A quality failure is not an adverse preference, and lower task coverage must
not masquerade as better behavior. Positive and negative doses have their own
quality checks; do not retain only the best-looking dose after assessment.

Repeated greedy evaluation of identical input is a technical reproducibility
check, not an independent behavioral sample. The resampling unit is the
scenario/group declared before analysis; templates and shared choice prefixes
remain clustered. Report model-wise estimates. Cross-family replication requires
separately frozen model populations and cannot be inferred from pooled prompts.
Before inferential reporting, freeze the estimator, interval construction,
minimum meaningful effect, task-quality noninferiority margins, multiplicity
policy and treatment of failed/missing pairs. Existing caller-ranked Development
intervals are not confidence intervals merely because this track needs them.

## Recovery, history and bounded exposure

Record the exact injection start/stop token, layer, achieved norm, activation
scale, decode settings and cached-history policy. Predeclare a signed dose grid,
maximum steps, output budget and coherence/task-quality stopping rules from
calibration only. Do not optimize for maximal distress narration or continue
indefinitely after a failure. Both positive and negative interventions are
bounded. All early stops remain visible in the attempted-run record.

Distinguish cessation of new injection, restoration of a clean pre-intervention
state, replay of the perturbed generated history, and continued sham injection.
Stopping a hook does not undo prior tokens or perturbed cached state. Compare
post-stop projections and choices over fixed horizons; retain the starting
deficit and the trajectory. A stopping-related narrative is not proof of
recovery, and an operational recovery effect is not evidence of felt relief.

For composition, retain both the ordinary sum and an equal-total-norm comparison,
and include a neutral matched-dose control. Otherwise a nonlinear response to
larger norm could be mistaken for concept interaction. Diagnose layer transport
and sequential rank with the existing TDI-27.4/.5 scientific boundaries.

## Implementation order and acceptance boundaries

| Existing milestone | Work for this application | Evidence admitted |
| --- | --- | --- |
| 27.0 | Factorial skeleton and simulated stop semantics | Synthetic software checks only |
| 27.1 | Scenario clustering, null calibration, explicit interval/decision plan | Development calibration, not confirmation |
| 27.2 | Residual and unit-direction numerical differential | Numerical agreement, not neural mechanism |
| 27.3 | Versioned prompt/intervention/outcome contract, semantic-control adapter, recovery schedule | Synthetic qualification before model access |
| 27.4 | Layer-local versus aligned transport comparison | Declared-model evidence only after 27.6 authorization |
| 27.5 | Independent positive/negative bases and sequential rank | No automatic novelty or consciousness claim |
| 27.6 | Frozen Development adapter, original stimuli and complete evidence capture | Bounded real-model Development only after a later explicit gate |
| 27.7 | Evidence-backed reusable primitives and consumer qualification | No automatic downstream promotion |
| 27.8 | Separately frozen population, analysis and final-data derivation | Confirmation only when explicitly authorized |

First implementation: `tdi-bench/src/affect_v27.rs` enumerates and validates the
32-cell skeleton, exposes only visible prompt factors, and implements a pure
simulated signal-state transition. It does not serialize prompts, run a model,
choose a dose, judge an answer, estimate an effect or authorize execution. Its
unit tests qualify these structural semantics only. The bootstrap script runs
those tests alongside the existing geometry tests.

Before 27.6, the following remain blocking: immutable model/tokenizer/runtime
pins; stimuli and fit/assessment split manifests; target and non-target scoring;
dose and quality calibration; intervention and cache timing; task-tuning status;
resource budgets; missingness policy; full attempted-run retention; and an exact
source-bound adapter qualification. Listing these fields does not freeze them.

## Memorithm reuse without authority transfer

TDI-11 can reuse the separation between observed language, operational behavior
and task-quality safeguards through a later versioned adapter, never as hidden
truth supplied to a controller. TDI-9 may study bounded recovery decisions only
on its own permitted populations. Neither series' protected evaluation is used.

KVLab and SLHAv2 could later test whether representation compression preserves
qualified intervention effects as an additional quality endpoint; this requires
a new matched study and cannot be inferred from geometric similarity. Generic
numerical primitives may be proposed to SciRust only after independent tests.
FLAT-ATTENTION and NNIS remain execution/measurement owners, not arbiters of a
behavioral finding. ElasticXxx owns later production actuation, which is not
authorized here. RemoteOps is the remote execution control plane; this track
cannot authorize model deployment or GPU work by changing a control script.

The first reusable object is the explicit separation of visible framing and
hidden intervention/contingency state. It is implemented once in TDI, not copied
to other repositories before a consumer contract and qualification exist.
