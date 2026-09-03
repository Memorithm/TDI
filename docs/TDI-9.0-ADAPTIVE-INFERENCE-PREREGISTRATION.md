# TDI-9.0 — Autonomous Adaptive Inference Dynamics Preregistration

Status: **FROZEN DESIGN CANDIDATE — NO TDI-9.2 FINAL EVALUATION SURFACE EXISTS**

Date: 2026-09-03

## 1. Scientific boundary

TDI-9.x is a new scientific series. It does not reinterpret TDI-1 through
TDI-8.x and it does not consume, authorize, execute, or inspect TDI-7.2 or
TDI-8.2 protected/final material.

TDI-9 studies independently defined adaptive inference mechanisms. Motivation
from observable behavior of modern reasoning systems does not establish that any
named proprietary model implements the mechanisms tested here.

This preregistration does not claim:

- knowledge of Anthropic/Claude proprietary architecture;
- language-model quality or transfer from the bounded mechanistic tasks;
- optimality of adaptive compute;
- universal savings in FLOPs, latency, energy or memory;
- that verification is equivalent to truth;
- that backtracking is equivalent to human reasoning;
- that tool-use orchestration is solved;
- GPU, Jetson, NNIS or ElasticXxx performance.

Those require separate evidence.

## 2. Primary scientific questions

### H9-A — adaptive stopping incremental value

Under the same declared maximum-compute and maximum-memory envelopes, can an
observation-conditioned inference policy preserve acceptable solution quality
while consuming materially less deterministic reference computation than a
fixed-compute policy?

Primary contrast:

`C2 observation-conditioned CONTINUE/STOP` versus `C0 fixed compute`.

### H9-B — verification/recovery incremental value

Under the same declared maximum-compute and maximum-memory envelopes, can adding
explicit verification and backtracking/recovery preserve acceptable solution
quality while reducing deterministic reference computation relative to adaptive
stopping alone?

Primary contrast:

`C3 CONTINUE/VERIFY/BACKTRACK/STOP` versus
`C2 CONTINUE/STOP`.

C1 is a competent static-preallocation control. It is not a primary hypothesis
arm, but C2 versus C1 is reported as a secondary contrast to distinguish true
trajectory adaptation from task-class preallocation.

## 3. Policy ladder

All arms are deterministic bounded reference policies over the same solver/task
semantics.

### C0 — fixed compute

C0 receives the common maximum compute allowance and follows a fixed
preregistered stopping schedule. It does not inspect trajectory observables to
change its compute allocation.

### C1 — static preallocation

C1 may select a preregistered compute allowance from information available before
inference begins, such as task-family identity. It may not use hidden difficulty,
targets, future generator state, validation labels, or trajectory observations.

### C2 — adaptive stopping

C2 receives only the allowed current/past trajectory observation vector and may
choose:

- `CONTINUE` — execute the next solver transition;
- `STOP` — emit the current candidate and terminate.

C2 cannot call the explicit verifier and cannot backtrack.

### C3 — adaptive verification/recovery

C3 receives the same base observation channel as C2 and may choose:

- `CONTINUE` — execute the next solver transition;
- `VERIFY` — execute the frozen independent verifier and expose only its allowed result to subsequent policy decisions;
- `BACKTRACK` — restore an eligible prior checkpoint and continue from it;
- `STOP` — emit the current candidate and terminate.

Verification, checkpoint creation/restoration and replay consume measured
resources. They are never free side channels.

## 4. Allowed and forbidden policy information

TDI-9.1 freezes the exact observation vector before final evaluation. It may be
constructed only from the following classes:

- current step index and remaining declared resource envelope;
- current solver-state summaries computed from information already observed;
- deterministic state-change/residual summaries;
- current candidate-score/confidence-margin summaries produced by the solver;
- action/history counters;
- for C3 only, outputs of an explicitly invoked frozen verifier;
- for C3 only, checkpoint availability and locally generated checkpoint metadata.

The policy must never receive:

- exact task target or answer label before `STOP`;
- future task events or future generator state;
- generator-only difficulty labels or hidden stress annotations;
- final-evaluation seed material unavailable to the solver;
- evaluator success/failure labels from the current instance;
- source provenance annotations that are not part of the task input;
- results from alternative arms on the same instance.

## 5. Resource discipline

All C0/C1/C2/C3 arms share one preregistered maximum-compute envelope and one
preregistered maximum-memory envelope for each primary cell.

TDI-9.1 must freeze concrete envelopes before TDI-9.2 can exist.

The evaluator records separately:

- solver scalar/integer operation counts;
- verifier operation counts;
- policy-decision operation counts;
- checkpoint copy/store/restore bytes and operation counts;
- replayed solver operations after backtracking;
- persistent solver-state bits;
- policy-state bits;
- checkpoint-memory bits;
- temporary working-storage peak;
- total actually consumed reference operations;
- maximum allowed reference operations.

Primary compute evidence uses deterministic reference operation counts, not wall
clock time. Wall-clock, device utilization and energy are separate downstream
evidence.

A stopped arm consumes no unexecuted future solver operations.

## 6. Numerical and determinism policy

Reference semantics use deterministic IEEE-754 binary64 and exact integers with:

- fixed single-threaded reference execution;
- fixed reduction/iteration order;
- deterministic seeded generation on non-final domains;
- explicit finite-value checks;
- exact action and operation accounting;
- deterministic checkpoint semantics;
- no nondeterministic hash map iteration or scheduler-dependent policy state.

A later optimized implementation is not TDI-9 reference evidence unless it
passes separately frozen equivalence rules.

## 7. Frozen mechanistic task families

TDI-9.0 freezes three task families. Concrete sizes and difficulty parameters are
TDI-9.1 decisions chosen only on non-final development/validation surfaces.

### P1 — staged evidence accumulation

Evidence relevant to a deterministic target arrives sequentially. Some instances
become decisively solvable early; others require additional evidence. The task
measures whether C2/C3 can stop early when additional computation is unnecessary
without receiving a hidden difficulty label.

### P2 — verification-sensitive inference

The solver can produce a provisional candidate before all deterministic
constraints are guaranteed satisfied. An independent frozen verifier can test a
candidate without receiving the target label. The task measures whether explicit
verification is used selectively rather than blindly on every step.

### P3 — recoverable deceptive fork

An early branch is locally plausible under partial evidence, later evidence can
contradict it, and a bounded checkpoint permits recovery to an earlier valid
state. The task measures whether explicit backtracking can repair a wrong
trajectory within the same maximum resource envelope.

A new primary task family cannot be added after validation results are observed.
A materially new family requires a separately preregistered follow-up.

## 8. Difficulty strata and primary cells

Each task family is evaluated at frozen `Shallow`, `Intermediate`, and `Deep`
compute-demand strata. TDI-9.1 freezes the concrete generator parameters before
TDI-9.2 exists.

Each primary hypothesis therefore has exactly nine primary cells:

`3 task families × 3 difficulty strata = 9 cells`.

The stratum label is evaluator metadata and is not exposed to C2/C3 unless the
same information is part of the ordinary task input. C1 may use task-family
identity but not hidden instance difficulty.

## 9. Primary quality and compute outcomes

For each paired generator instance and arm, record:

- exact success/failure under the task's deterministic target;
- primary deficit `D`, defined as `0` for exact success and `1` for failure;
- actual deterministic reference compute `K > 0`;
- maximum compute envelope;
- memory accounting from Section 5;
- action counts (`CONTINUE`, `VERIFY`, `BACKTRACK`, `STOP`);
- verification outcomes where applicable;
- backtrack depth/replay counts where applicable;
- typed invalid/rejection reason;
- complete provenance.

The exact target remains evaluator-owned and is used only after an arm stops to
score its emitted candidate.

## 10. Frozen primary two-stage cell rule

TDI-9 separates correctness protection from compute efficiency. It does not
collapse them into an arbitrary weighted utility.

For one primary cell, let:

- `B_D` be baseline mean deficit;
- `C_D` be candidate mean deficit;
- `Q = C_D - B_D`, where positive values mean candidate quality is worse;
- `B_K` be baseline mean actual compute;
- `C_K` be candidate mean actual compute;
- `G = (B_K - C_K) / B_K`, where positive values mean candidate compute savings.

`B_K` must be strictly positive. Non-finite/negative deficits, non-positive
compute, accounting overflow, contract drift or incomplete paired observations
are typed fail-closed invalid results.

The frozen quality non-inferiority margin is:

`delta_q = 0.02`

(two absolute percentage points of failure probability).

The frozen material compute margin is:

`delta_k = 0.05`

(five percent relative mean deterministic reference compute).

Let `[L_Q, U_Q]` and `[L_G, U_G]` be the frozen paired uncertainty intervals.
The primary cell verdict is evaluated in this order:

1. `Harmful` iff `L_Q > +delta_q`;
2. `Inconclusive` iff `U_Q > +delta_q` (quality non-inferiority not established);
3. after quality non-inferiority is established (`U_Q <= +delta_q`):
   - `Beneficial` iff `L_G > +delta_k`;
   - `Harmful` iff `U_G < -delta_k`;
   - `Equivalent` iff `L_G >= -delta_k` and `U_G <= +delta_k`;
   - `Inconclusive` otherwise.

A candidate with materially better quality but no compute saving may be reported
as a secondary quality result, but it is not relabeled `Beneficial` for the
primary adaptive-efficiency claim.

## 11. Paired uncertainty and family-wise coverage

Primary observations are paired at generator-instance level. TDI-9.1 must freeze
one deterministic paired interval implementation, replicate count if resampling
is used, non-final resampling seed policy, and degenerate-case handling before
TDI-9.2.

For each hypothesis there are nine cells and two primary intervals per cell
(`Q` and `G`). The confirmatory family therefore contains 18 intervals.

The required family-wise error rate is at most 0.05. If Bonferroni intervals are
used, the per-interval allocation is frozen as:

`alpha = 0.05 / 18`.

A different deterministic family-wise method may be selected during TDI-9.1
only on non-final evidence and must be frozen before final entropy is knowable.
It must still guarantee at least 95% family-wise coverage for the 18 primary
intervals.

## 12. Frozen nine-cell hypothesis aggregation

For H9-A and H9-B separately:

- `Beneficial` iff all nine cells are in `{Beneficial, Equivalent}` and at least one is `Beneficial`;
- `Harmful` iff all nine cells are in `{Harmful, Equivalent}` and at least one is `Harmful`;
- `Equivalent` iff all nine cells are `Equivalent`;
- `Inconclusive` otherwise, including any missing/rejected primary cell.

Mixed beneficial/harmful evidence is therefore `Inconclusive`, not averaged into
a favorable aggregate claim.

## 13. Secondary analyses

Secondary, non-substitutable analyses may include:

- C2 versus C1 static preallocation;
- quality superiority intervals;
- compute distribution tails;
- verifier precision/recall against deterministic constraints;
- rescue rate after a verifier-detected contradiction;
- backtrack depth and replay overhead;
- action-frequency trajectories;
- policy calibration on development/validation domains;
- mechanistic failure taxonomy.

Secondary results cannot replace a failed or inconclusive primary cell.

## 14. Autonomous agent research policy

TDI-9 is explicitly agent-first.

Before final entropy is revealed, autonomous agents may:

- generate and mutate candidate C2/C3 policies;
- propose allowed observation subsets;
- construct software oracles and adversarial fixtures;
- run bounded train/development/validation experiments;
- perform ablations and falsification;
- reject weak candidates;
- open, repair and merge CI-green non-final PRs under repository policy;
- use ADA for semantic candidate generation and Forge only after a leak-safe search contract exists.

Agents must preserve all negative/null results relevant to promotion decisions and
must not weaken the frozen TDI-9.0 primary rules in response to observed results.

## 15. Autonomous sealed TDI-9.2 confirmation

TDI-9 uses no human confirmation token.

TDI-9.1 must freeze all of the following before the final entropy value exists:

1. the identity and verification rule of a public randomness beacon or equivalent immutable external entropy source;
2. a specific future event/round/block/checkpoint whose value is not yet knowable;
3. canonical byte encoding of that future value;
4. deterministic domain-separated derivation from those bytes to final generator seeds and any final resampling randomness;
5. final population size and rejection policy;
6. exact evaluator commit/manifests;
7. exact result/provenance schema;
8. a no-retry rule.

After the future entropy value is revealed, the first valid value satisfying the
frozen source/event rule is binding. Agents and CI must not select another value,
retry with a different entropy event, or change the derivation because of
results.

No concrete TDI-9.2 seed list, dataset, result payload or executable final runner
may exist during TDI-9.0.

## 16. Stage gate

TDI-9.1 may begin only after this TDI-9.0 preregistration is merged, its Git blob
identity is pinned, bootstrap integrity passes, and repository agent contracts
encode the TDI-9 autonomous-confirmation exception without weakening TDI-7.2 or
TDI-8.2 protections.

TDI-9.2 remains nonexistent until TDI-9.1 freezes its complete evaluator,
resource/task parameters, uncertainty method, rejection taxonomy, final entropy
derivation contract and provenance schema.

## 17. Interpretation

Positive bounded evidence would support only the stated adaptive-efficiency
claim on the tested mechanistic battery. It would not establish proprietary
architecture equivalence, language-model transfer, autonomous truth discovery,
or real-device acceleration.
