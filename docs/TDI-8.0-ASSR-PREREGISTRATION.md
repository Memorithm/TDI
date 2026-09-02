# TDI-8.0 — Alternative Recurrent Associative Architecture Preregistration

Status: **FROZEN DESIGN CANDIDATE — NO TDI-8.2 HOLDOUT RUNNER EXISTS**

Date: 2026-09-02

## 1. Scientific boundary

TDI-8.x is a new scientific series. It does not reinterpret TDI-1 through
TDI-6.x and it does not modify, consume, authorize, execute, or inspect the
TDI-7.2 final holdout.

The working labels **ASSR** (associative state-space/recurrent architecture) and
**ASSR-H** (ASSR with a VSA/holographic workspace) are research labels, not
claims of novelty or superiority.

This preregistration tests bounded reference mechanisms. It does **not** claim:

- replacement of Transformers in language modelling;
- universal O(N) runtime in every implementation;
- constant total memory for an ever-growing external memory;
- tokenizer elimination;
- elimination of probabilistic decoding;
- dimension-level interpretability of VSA representations;
- Jetson, GPU, energy, bandwidth, or end-to-end LLM performance.

Those claims require separate evidence.

## 2. Primary scientific questions

### H8-A — associative memory incremental value

Under the same declared dynamic-memory budget, does an explicit associative
memory improve late retrieval/recovery relative to a recurrent-state-only
control on the frozen mechanistic task battery?

Primary contrast:

`A2 recurrent + associative memory` versus `A1 recurrent state only`.

### H8-B — VSA workspace incremental value

Under the same declared dynamic-memory budget, does adding a VSA/holographic
workspace improve late retrieval/recovery beyond associative memory alone?

Primary contrast:

`A3 recurrent + associative memory + VSA workspace` versus
`A2 recurrent + associative memory`.

A0 is a competent attention-like full-history reference used to establish task
solvability and contextualize failure. It is not the primary matched-budget
contrast.

## 3. Architecture ladder

All four arms are deterministic reference implementations with explicit state
and memory accounting.

### A0 — attention-like full-history reference

A0 retains the complete accessible history and computes a deterministic
content-based read over that history. It is the competent full-history control.
Its history storage is reported separately and is allowed to grow with sequence
length.

### A1 — recurrent-state-only reference

A1 processes one item at a time and retains only a fixed recurrent state. It has
no external associative table and no full-history store.

### A2 — ASSR candidate

A2 contains:

- a fixed recurrent state;
- an explicit bounded associative memory;
- deterministic address projection;
- deterministic collision handling;
- deterministic read, write and replacement semantics;
- a deterministic gate that fuses retrieved memory with recurrent state.

The associative memory is bounded in TDI-8.0. Claims about an unbounded external
memory hierarchy are outside this experiment.

### A3 — ASSR-H candidate

A3 contains all A2 components plus a bounded VSA workspace. The VSA component
may use only preregistered deterministic operations from the following set:

- binding;
- bundling/superposition;
- permutation when structurally required;
- unbinding/retrieval;
- similarity for cleanup/readout.

A3 receives no additional total dynamic-memory budget over A2. Any workspace
storage must be paid for by reducing another A3 state/memory component.

## 4. Budget discipline

The primary A1/A2/A3 comparison uses an identical total dynamic-memory budget
measured in bits.

For each arm, the evaluator records separately:

- recurrent-state bits;
- associative-memory payload bits;
- associative metadata/addressing bits;
- VSA workspace bits;
- temporary working storage required by the reference step;
- cumulative history storage where applicable;
- static parameter/constant-table bits.

A1 may use its full dynamic budget for recurrent state. A2 partitions the same
budget between recurrent state and associative memory. A3 partitions the same
budget among recurrent state, associative memory and VSA workspace.

No compression ratio or memory advantage may be reported without including the
metadata required by the implemented representation.

Operation counts are reported separately from memory and are not converted into
runtime claims.

## 5. Numerical policy

TDI-8.0/8.1 reference semantics use deterministic IEEE-754 binary64 with:

- single-threaded reference execution;
- fixed iteration and reduction order;
- fixed seeded projections where a projection is required;
- no nondeterministic hash implementation;
- explicit finite-value checks;
- declared comparison tolerances for non-exact floating operations.

Integer indexing, seed selection, collision bookkeeping and memory accounting
must be exact.

A later optimized implementation may use different numeric formats only after
reference equivalence rules are frozen.

## 6. Frozen mechanistic task families

TDI-8.0 freezes three task families for the bounded reference study.

### T1 — associative recall

A sequence introduces key/value associations followed by one or more delayed
queries. Distractor associations are present. The queried key and target value
are generated before execution and are identical across arms.

### T2 — copy with delayed retrieval

A bounded payload must be reproduced after a variable delay containing
irrelevant inputs. This measures retention independent of associative key
selection.

### T3 — interference recall

Multiple associations intentionally compete for representational capacity.
Queries target both recent and older associations. The generator includes
controlled key similarity/collision pressure so that table capacity and
recurrent-state compression can fail observably.

The evaluator must not add a new task family after validation results are seen.
A materially new task requires a new preregistered TDI-8 follow-up.

## 7. Sequence-length and stress strata

Each task is evaluated at preregistered short, medium and long horizons. TDI-8.1
must freeze the concrete horizon values before any final-holdout implementation
exists.

The validation battery must include at least:

- delay growth;
- distractor growth;
- associative-memory occupancy growth;
- collision/interference growth;
- a capacity boundary where at least one bounded architecture can fail.

TDI-8.1 may determine safe concrete values using training/development data only.
Those values become frozen before TDI-8.2 is armed.

## 8. Intervention and TDI recovery measurements

Each arm exposes a framework-independent state snapshot through `tdi-ai`.
Permitted intervention sites are arm-specific but must be declared before
validation evaluation.

Candidate sites are:

- recurrent-state coordinate/block;
- associative slot/address entry for A2/A3;
- VSA workspace component for A3;
- history item/read coefficient for A0.

An intervention is one-shot and must not change the task label or target.
Reference and perturbed trajectories then evolve under identical downstream
semantics.

Early recovery descriptors must use only observations strictly earlier than the
late retrieval target.

## 9. Primary outcomes

For each task, horizon stratum and architecture, record:

- exact task success where the task has an exact discrete target;
- late retrieval deficit, oriented as `0 = no degradation`, larger = worse;
- early intervention-conditioned recovery descriptors;
- dynamic-memory bits;
- static parameter/constant bits;
- deterministic operation counts per processed item;
- rejected-record count and typed rejection reason.

Runtime, bandwidth, energy and GPU occupancy are **not** TDI-8.0 primary
outcomes.

## 10. Primary statistical contrasts and frozen decision rule

### H8-A

Compare A2 against A1 using paired generator-level observations under the same
memory budget.

### H8-B

Compare A3 against A2 using paired generator-level observations under the same
memory budget.

Each hypothesis has exactly nine primary cells: the Cartesian product of the
three frozen task families and the three frozen horizon strata. Additional
stress slices may be reported as secondary analyses but cannot be substituted
for or added to these nine primary cells after validation results are observed.

For a primary cell with mean baseline deficit `B > 0` and mean candidate deficit
`C`, the primary effect is relative mean-deficit reduction:

`R = (B - C) / B`.

Deficit is non-negative by definition. A non-finite or negative deficit is an
invalid numeric result and follows the typed fail-closed rejection policy rather
than receiving a scientific verdict.

### Zero-baseline rule

The relative statistic is never evaluated with a zero denominator.

If `B == 0` exactly:

- if `C == 0` exactly, the cell verdict is `Equivalent` and `R` is recorded as
  undefined because both arms have zero measured deficit;
- if `C > 0`, the cell verdict is `Harmful`, `R` is recorded as undefined, and
  the absolute degradation `C - B` is reported;
- no zero-baseline cell can produce a `Beneficial` verdict.

This branch is part of the primary classifier, not a post-hoc fallback.

### Paired uncertainty family

For cells with `B > 0`, TDI-8.1 must implement and software-validate a paired
uncertainty procedure capable of producing a two-sided interval `[L, U]` for
`R`. The resampling/replicate implementation, replicate count and deterministic
resampling seed must be frozen using non-final data before TDI-8.2 exists.

The inferential coverage is frozen here: within each hypothesis, the nine
primary cell intervals must provide at least 95% family-wise coverage using a
Bonferroni allocation of `alpha = 0.05 / 9` to each two-sided cell interval.
TDI-8.1 may improve numerical implementation of that interval construction but
must not weaken this family-wise coverage rule after validation results are
seen.

The equivalence/decision margin is frozen at `delta = 0.02`, i.e. two percentage
points of relative mean-deficit reduction. Changing `delta`, the classifier, or
the hypothesis-level aggregation rule requires a separately reviewed
preregistration amendment before any TDI-8.2 runner, seed range, or result
surface exists.

### Four-way primary-cell classifier

For a non-zero-baseline cell, classify the simultaneous interval `[L, U]` in
this exact precedence:

1. `Beneficial` iff `L > +delta`;
2. `Harmful` iff `U < -delta`;
3. `Equivalent` iff `L >= -delta` and `U <= +delta`;
4. `Inconclusive` otherwise.

Thus an interval that crosses zero but extends outside the equivalence band is
`Inconclusive`; an interval touching either `-delta` or `+delta` without
crossing outside the band remains `Equivalent`. Invalid or missing intervals do
not get coerced into any favorable category.

### Frozen hypothesis-level aggregation

The nine primary-cell verdicts map to the H8-A or H8-B verdict by the following
closed rule:

- `Beneficial` iff all nine cells are in `{Beneficial, Equivalent}` and at least
  one cell is `Beneficial`;
- `Harmful` iff all nine cells are in `{Harmful, Equivalent}` and at least one
  cell is `Harmful`;
- `Equivalent` iff all nine cells are `Equivalent`;
- `Inconclusive` in every other case, including any `Inconclusive` cell, any
  mixture containing both `Beneficial` and `Harmful`, or any missing/rejected
  primary cell.

No task, horizon, favorable cell, or secondary analysis may override this
hypothesis-level gate. No negative, null, harmful, rejected, or inconclusive
result may be discarded or renumbered away.

## 11. Secondary analyses

Secondary analyses may describe:

- task/horizon heterogeneity;
- memory occupancy and collision behavior;
- retrieval degradation versus delay;
- recovery-profile differences;
- exact accounting of state versus external-memory growth;
- A0 contextual upper/reference behavior.

Secondary analyses cannot override a failed primary contrast.

## 12. Split discipline

TDI-8 uses seed ranges that are structurally disjoint from TDI-7.

The exact TDI-8 train/development/validation/final-holdout ranges and population
counts must be frozen by TDI-8.1 before any final-holdout runner is created.

TDI-8.0 deliberately does not invent those counts solely by analogy with TDI-7.
TDI-8.1 must justify them from bounded evaluator cost and variance using only
non-holdout data.

After the final ranges are frozen:

- no final seed may be used in tests, CI, smoke runs or tuning;
- no final result may be inspected before human authorization;
- selection from the frozen range must itself be explicit and machine-validated.

## 13. Leakage and anti-tuning rules

Before TDI-8.2:

- candidate architecture semantics may use train/development information;
- validation is used only under the TDI-8.1 protocol to freeze bounded evaluator
  choices;
- final-holdout information is inaccessible;
- Forge must not receive final-holdout tasks, targets, results, seeds or oracles;
- NNIS/FLAT optimization must not influence the reference scientific verdict;
- TDI-7.2 authorization state and final seeds are irrelevant and inaccessible to
  TDI-8.

Once TDI-8.2 final-holdout access occurs, no architecture, feature, task,
intervention, metric, budget or decision-rule change is permitted for that
confirmatory result.

## 14. Rejection policy

TDI-8.1 must define a closed typed rejection taxonomy before final-holdout
arming. At minimum, malformed task, invalid numeric state, impossible memory
address, invalid intervention and non-finite target/result conditions must fail
closed.

Result-dependent exclusion is forbidden.

Every rejected final record must be counted and reported by reason.

## 15. TDI-8 staged programme

### TDI-8.0 — preregistration

Freeze the scientific question, architecture ladder, task families, matched
budget principle, primary contrasts, non-claims and holdout discipline.

### TDI-8.1 — bounded deterministic evaluator

Implement and validate A0/A1/A2/A3 reference semantics, tasks, interventions,
metrics, memory accounting, operation accounting, paired statistics, provenance
and fail-closed holdout guards using only non-final data.

TDI-8.1 must also freeze the concrete dimensions, budgets, horizon strata, seed
ranges, sample counts, paired interval implementation/replicate count and final
rejection taxonomy before TDI-8.2 can be armed. It must not alter the frozen
`delta`, four-way classifier, nine-cell primary set, family-wise coverage rule,
or hypothesis-level aggregation gate.

### TDI-8.2 — confirmatory holdout

Human-only, separately armed and separately authorized. Autonomous agents must
never supply the confirmation token or initiate this run.

### TDI-8.3+ — evidence-justified follow-ups only

Candidate follow-ups include:

- memory-capacity/collision ablations;
- write/forget-gate ablations;
- episodic versus semantic memory separation;
- VSA incremental-value and binding-operator controls;
- cross-length and cross-task transfer;
- bounded hot-memory plus external cold-memory experiments;
- Forge search over evidence-qualified parameter spaces;
- NNIS real-device implementation after reference evidence exists.

None is authorized merely by numbering.

## 16. Ecosystem ownership

- **TDI** owns the preregistered scientific reference study and frozen evidence.
- **Forge** may later search candidate parameters/implementations after TDI
  publishes a bounded verify/measure domain contract; Forge scores are not TDI
  scientific results.
- **SciRust** may receive general reusable VSA/statistical primitives only after
  they are sufficiently general and independently tested.
- **NNIS** is the downstream NVIDIA execution target for real device evidence.
- **FLAT-ATTENTION** remains the optimized attention execution target and may be
  used as a comparator/integration target, not as the owner of ASSR semantics.
- **SLHAv2** remains the primary owner of DA-LUC-like KV representation work;
  TDI may later evaluate its dynamic recovery effects.

## 17. Promotion criteria

TDI-8 reference evidence is necessary but not sufficient to create a dedicated
ASSR product repository.

A repository split should be considered only if TDI-8 establishes a stable
reference semantic worth owning independently. Productization, training-scale
claims and hardware acceleration remain separate phases.

## 18. Frozen non-claims

A positive TDI-8 result would establish only the stated bounded mechanistic
contrast under the frozen tasks, budgets, implementations and seed population.
It would not by itself establish:

- language-model quality;
- superiority to Mamba or any named external architecture;
- asymptotic superiority in every workload;
- constant total memory with unlimited history;
- practical GPU speedup;
- reduced energy use;
- cognitive transparency;
- tokenizer-free general language modelling;
- multi-agent reasoning by vector addition.

These remain separate hypotheses requiring separate experiments.
