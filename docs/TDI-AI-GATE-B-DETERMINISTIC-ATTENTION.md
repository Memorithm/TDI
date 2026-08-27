# TDI-AI Gate B — deterministic attention/mixing oracle

Status: **implementation fixture, not an AI efficacy result**.

This note specifies the first non-finite-state adapter used to exercise the
TDI-AI intervention/recovery contracts introduced by PR #58.

The purpose of Gate B is not to demonstrate that TDI transfers to neural
attention. It is to establish a tiny attention-like dynamical system whose
response to a controlled perturbation can be derived by hand and then recovered
by the generic `tdi-ai` execution protocol.

## 1. Why a deterministic toy comes first

Before attaching TDI-AI to FLAT-ATTENTION, a Transformer, or an expensive
language-model experiment, we need to know that the generic adapter itself does
not manufacture recovery structure.

A useful first fixture must therefore have:

1. deterministic state evolution;
2. no training;
3. no stochastic sampling;
4. no framework dependency;
5. an explicit sequence-mixing matrix;
6. a controlled one-shot intervention;
7. a hand-derived downstream trajectory;
8. a recovery metric whose interpretation is frozen in advance.

The fixture in `tdi-ai/src/toy_attention.rs` satisfies those conditions.

## 2. State and operator

The state is a three-token scalar vector

```text
x = [x0, x1, x2]^T.
```

One dynamics step applies the fixed row-stochastic operator

```text
      [ 1/2  1/2   0  ]
M  =  [ 1/4  1/2  1/4 ]
      [  0   1/2  1/2 ].
```

Every row is non-negative and sums to one.

This is intentionally **not** claimed to be standard softmax attention. It is
an analytically tractable attention-like sequence mixer that exercises the same
high-level concepts future adapters require:

- token-indexed state;
- weighted cross-token interaction;
- repeated layer/step evolution;
- intervention on token content;
- comparison of reference and perturbed futures.

`FixedAttentionMixer` validates at construction time that the matrix is square,
finite, non-negative and row-normalized within a fixed `1e-12` tolerance.

## 3. Intervention

The first intervention is a balanced redistribution:

```text
I_a(x) = x + a [1, 0, -1]^T.
```

For the frozen fixture, `a = 1` and the reference state is

```text
x0_ref = [0, 0, 0]^T.
```

Therefore

```text
x0_int = [1, 0, -1]^T.
```

The intervention adds and subtracts the same amount, so the global scalar sum
is preserved. This avoids a trivial experiment where the recovery metric merely
detects that the total content was changed.

`BalancedTokenShift` also preserves the untouched reference object and refuses
an intervention whose two token locations are identical.

## 4. Hand-derived spectral trajectory

The perturbation direction

```text
v = [1, 0, -1]^T
```

is an eigenvector of `M`:

```text
M v = (1/2) v.
```

Hence after `h` dynamics steps,

```text
Delta_h = M^h v = 2^(-h) v.
```

The first four perturbed states relative to the zero reference are therefore

```text
h=1 : [1/2,  0, -1/2]
h=2 : [1/4,  0, -1/4]
h=3 : [1/8,  0, -1/8]
h=4 : [1/16, 0, -1/16].
```

This decay is not estimated from output. It is fixed by the operator's exact
algebraic action on the chosen mode.

This is also the first explicit bridge between the TDI-AI programme and the
broader operator/spectral attention programme: the intervention probes a known
mode of a sequence operator and TDI-AI records how that mode survives downstream.

## 5. Recovery metric

Gate B uses the deliberately simple metric

```text
R(x,y) = 1 / (1 + ||x-y||_inf).
```

Properties relevant to this fixture:

- `R = 1` iff the two observables are identical;
- `0 < R <= 1` for finite inputs;
- larger maximum component-wise discrepancy means smaller recovery score;
- the metric is deterministic;
- no probability interpretation is assigned to it;
- it is **not** the historical exact TDI distribution overlap.

The implementation is named `ReciprocalLInfRecovery` to make its mathematical
content explicit and avoid overloading the word "information".

For the analytic trajectory above,

```text
||Delta_h||_inf = 2^(-h),
```

so

```text
R_h = 1 / (1 + 2^(-h)).
```

The first four expected values are

```text
R_1 = 2/3
R_2 = 4/5
R_3 = 8/9
R_4 = 16/17.
```

The unit test compares the generic TDI-AI profile against these independently
specified values.

## 6. What the test proves

If the Gate B tests pass, we have evidence that:

1. the generic intervention is applied once at depth zero;
2. the same operator advances reference and perturbed trajectories;
3. token redistribution is preserved as specified;
4. the known eigenmode decays by its expected factor;
5. the generic `analyze_intervention_recovery` pipeline produces the hand-derived
   recovery sequence;
6. the first non-finite-state adapter can be expressed without changing
   `tdi-core` or any frozen TDI experiment.

These are **software/mathematical adapter claims only**.

## 7. What the test does not prove

Gate B does not establish that:

- TDI recovery descriptors improve prediction in an AI model;
- this toy mixer is a useful attention semantic;
- standard softmax attention has the same dynamics;
- the recovery metric is optimal;
- the observed spectral mode exists in trained Transformers;
- recovery causes task success;
- any finite-state TDI effect transfers to neural systems.

No such claim should be inferred from a green unit test.

## 8. Why this fixture is useful for FLAT-ATTENTION later

FLAT-ATTENTION is being evolved toward multiple attention/mixing semantics.
Before a semantic is GPU-optimized, its deterministic reference path should be
intervention-observable.

Gate B establishes the expected research interface:

```text
semantic state
    + controlled intervention
    + deterministic reference advance
    + declared observable
    + declared recovery metric
          -> RecoveryProfile
```

A future FLAT adapter can replace `FixedAttentionMixer` while keeping this
experimental structure.

This makes it possible to compare, under a common intervention protocol:

- StandardSoftmax;
- differential/signed mechanisms;
- Toeplitz-conditioned mixers;
- prolate/spectral mixers;
- Green-kernel / ground-state operators;
- recurrent or delta-memory semantics;
- hybrids.

The experiment harness must still use matched task baselines and static
operator diagnostics; TDI recovery is only interesting when it adds
non-redundant information.

## 9. Next gate

After Gate B is merged, the next scientific step is **not** immediate LLM
training.

Gate C should define a frozen mechanistic task, initially associative recall,
copy, or overwrite/interference, and preregister the first falsifiable question:

> Do early intervention-conditioned recovery measurements predict later task
> failure beyond competent static attention/operator diagnostics?

That experiment should include an untouched holdout, fixed seeds, explicit
margins for Beneficial / Equivalent / Harmful / Inconclusive, and a negative
result path before any confirmatory run is executed.
