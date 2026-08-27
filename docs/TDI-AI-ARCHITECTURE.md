# TDI-AI — intervention/recovery architecture

Status: **open research architecture**.

This document defines how TDI may be reused in AI-attention research without
rewriting the historical TDI experiments or claiming that finite-state results
already transfer to neural networks.

## 1. Why this layer exists

The validated TDI programme studies a specific question in small synthetic
finite-state systems: whether early, intervention-conditioned overlap between
accessible future distributions contains predictive information about later
recovery that is not exhausted by increasingly strong static descriptors.

The AI programme asks a new question:

> Can controlled perturbations of an attention, memory, or sequence-mixing
> mechanism produce recovery trajectories that reveal useful information about
> later retrieval, interference, forgetting, robustness, or failure beyond
> competent static diagnostics?

This is a transfer of **experimental method and abstractions**, not a transfer
of an empirical result.

## 2. Scientific boundary

The following are non-negotiable.

1. Existing TDI-1 through TDI-6.x reports, preregistrations, result logs and
   frozen scientific code remain the source of truth for their experiments.
2. `tdi-core` remains the exact finite-state oracle layer.
3. `tdi-ai` sits above `tdi-core`; the first AI adapter must not silently alter
   the meaning of the existing branching overlap profile.
4. No Transformer/LLM claim may cite a finite-state TDI result as evidence of
   transfer without a separate AI experiment.
5. Negative, equivalent, harmful, inconclusive and calibration-failure results
   remain first-class outcomes.
6. A recovery score is not called a probability, overlap, information measure,
   or causal quantity unless its selected metric actually justifies that name.
7. An intervention experiment is not automatically causal identification.

## 3. Core abstraction

The first implementation decomposes one recovery experiment into four roles.

### `ReferenceDynamics`

Defines one deterministic or explicitly reproducible advancement unit.
Depending on the adapter, one unit may mean:

- one attention/mixer application;
- one Transformer layer;
- one recurrent-memory update;
- one generated token;
- one step of a synthetic state machine.

The unit must be frozen by the experiment protocol. Results from different
units must not be compared as if they shared the same horizon.

### `Intervention<State>`

Applies one controlled perturbation at depth zero while preserving an untouched
reference state.

Future concrete interventions may target:

- one token representation;
- one attention head;
- one key/value slot;
- one recurrent memory slot;
- one structured spectral mode;
- one coefficient of a candidate Toeplitz/prolate/Green operator;
- one mask or routing decision.

The generic crate deliberately does not hard-code those domain labels yet.

### `FutureObservable<State>`

Extracts the object that will be compared between reference and perturbed
trajectories. This separation prevents the full model state from becoming the
metric accidentally.

Possible future observables include:

- task output distributions;
- hidden representations;
- memory state;
- attention/operator state;
- retrieval logits;
- a preregistered low-dimensional descriptor.

### `FutureOverlap<Observable>`

Compares reference and perturbed observables. Its score type is generic.

The exact finite-state oracle uses `ExactRatio`. Neural adapters will likely
require floating-point metrics, but each metric must define its range,
orientation, numerical policy, invariances and interpretation before use.

## 4. Recovery profile

For an initial state `x_0` and intervention `I`, define

```text
x_0^ref = x_0
x_0^int = I(x_0)
```

and evolve both with the same dynamics `F`:

```text
x_h^ref = F^h(x_0^ref)
x_h^int = F^h(x_0^int).
```

At every preregistered depth `h`, observe both trajectories and compare them:

```text
O_h^ref = observe(x_h^ref)
O_h^int = observe(x_h^int)
R_h     = overlap(O_h^ref, O_h^int).
```

The ordered sequence

```text
R = (R_1, R_2, ..., R_H)
```

is the generic `RecoveryProfile`.

This PR intentionally does **not** prescribe a monotonicity constraint. The
historical finite-state overlap can reconverge, separate or fluctuate; an AI
metric may also behave non-monotonically. Monotonic transforms or deficit
geometries belong in later, explicitly specified analysis layers.

## 5. Exact-oracle bridge

`from_exact_branching_analysis` converts the existing
`BranchingRecoveryAnalysis::overlap_profile()` directly into the generic
schema while preserving `ExactRatio` values.

This serves two purposes:

1. it proves the generic schema can represent the historical exact TDI profile
   without floating-point conversion;
2. it gives future AI adapters a regression fixture anchored to the existing
   finite-state semantics.

The bridge is deliberately one-way. `tdi-ai` consumes `tdi-core`; `tdi-core`
does not depend on AI concepts.

## 6. Relationship with ITD Simulator

ITD Simulator is the comparative experimental harness.

The intended first attention experiment compares at least:

- competent task/domain baselines;
- standard static attention diagnostics;
- spectral/operator diagnostics;
- ITD-AI structural descriptors;
- TDI-AI dynamic recovery descriptors;
- ITD + TDI jointly.

This matters because a positive TDI-AI result is only interesting if the
recovery profile adds information beyond simpler descriptors.

ITD and TDI must remain independently ablatable candidate information sources.

## 7. Relationship with FLAT-ATTENTION

FLAT-ATTENTION is the execution and optimization target for attention/mixing
semantics, not the place where a research metric becomes true by construction.

Research-mode FLAT adapters will eventually need to expose sufficient state to
perform controlled interventions under semantic families such as:

- standard softmax;
- differential/signed attention;
- Toeplitz/relative structured mixing;
- prolate/spectral concentration;
- ground-state/Green-kernel mixing;
- recurrent/delta-memory mechanisms;
- hybrid mechanisms.

A new semantic should first have deterministic reference semantics and survive
mechanistic experiments before GPU specialization is treated as a priority.

## 8. Relationship with RiemannBench and SciRust

`riemann_ndim_bench` can supply independently meaningful structured operators
and perturbation questions. TDI-AI asks how perturbations propagate through
those candidate dynamics; it does not import Riemann/zeta interpretations.

SciRust remains the preferred home for reusable mathematical/statistical
primitives once a TDI research method becomes general infrastructure. Frozen
TDI experiments remain in TDI even if supporting primitives later move or are
reimplemented in SciRust.

## 9. First experiment sequence

### Gate A — adapter correctness

- generic traits compile on the workspace MSRV;
- intervention is applied once at depth zero;
- reference and perturbed paths use identical dynamics afterward;
- observations are evaluated at identical depths;
- zero-horizon behavior is explicit;
- exact finite-state overlap maps without numerical conversion.

### Gate B — deterministic attention toy

Build a small deterministic attention/memory mechanism with an analytically or
exhaustively checkable task. Add interventions on inputs/state and verify the
recovery profile against hand-derived fixtures.

No learned language model is required at this gate.

### Gate C — first falsifiable AI hypothesis

On associative recall/copy/overwrite tasks, preregister whether early recovery
features improve prediction of later retrieval failure beyond static controls.
Use untouched holdout data and fixed decision margins.

### Gate D — intervention heterogeneity

Perturb multiple locations independently and test two quantities separately:

1. heterogeneity of failure magnitude;
2. stability of the early-recovery to late-failure relationship.

This is motivated by TDI-6.4 but does not assume its result transfers.

### Gate E — joint information

Test whether multiple early recovery observables become increasingly useful in
combination over longer horizons. Do not rely on a single PID definition and
do not reinterpret MMI-specific atoms as universal quantities.

## 10. What this first PR deliberately does not implement

- no neural-network framework dependency;
- no PyTorch/JAX bridge;
- no FLAT-ATTENTION dependency;
- no ITD dependency;
- no GPU code;
- no specific attention intervention taxonomy;
- no chosen neural overlap metric;
- no claim that recovery profiles improve AI prediction;
- no modification of frozen TDI experiments;
- no expensive confirmatory run.

The purpose is to create a narrow, testable boundary on which those later
experiments can be built without contaminating the existing scientific record.
