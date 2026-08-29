# TDI-AI — static attention baselines

Status: **implementation baseline, not an H-AI-1 result**.

This document defines the first static attention controls required by issue #57
before the TDI-AI programme tests whether intervention-conditioned recovery adds
predictive information beyond cheaper descriptors.

The purpose is methodological separation:

```text
static description of one attention operator
            !=
intervention-conditioned future recovery trajectory
```

A future positive TDI-AI result is only interesting if its dynamic recovery
features improve a preregistered task model beyond competent static controls.

## 1. Input contract

`analyze_static_attention` accepts a finite non-empty row-stochastic matrix.
Rows may be rectangular so the same control layer can later describe
self-attention and cross-attention operators.

For each row `p_i`:

- all weights must be finite;
- all weights must be non-negative;
- the row must be non-empty;
- every row must have the same width;
- `|sum_j p_ij - 1| <= 1e-12`.

Malformed inputs are rejected rather than silently renormalized. This matters
for research provenance: a baseline must summarize the operator that was
actually declared, not a repaired operator created by the diagnostic code.

## 2. Frozen baseline descriptors

The first implementation reports the following aggregate descriptors.

### Mean row entropy

For row `i`:

```text
H_i = - sum_j p_ij ln(p_ij)
```

with `0 ln(0)` defined as zero. Entropy is reported in nats.

### Mean normalized row entropy

For more than one column:

```text
H_i / ln(number_of_columns)
```

is averaged across rows. A one-column row has normalized entropy zero by
definition.

### Mean row maximum weight

```text
mean_i max_j p_ij
```

This is a simple concentration/control statistic.

### Mean row L2 concentration

```text
mean_i sum_j p_ij^2
```

This increases as a row concentrates mass on fewer positions.

### Mean row effective support

```text
mean_i exp(H_i)
```

This quantity is an entropy-derived effective support size. It is deliberately
named `mean_row_effective_support`.

It is **not** matrix rank, effective rank, stable rank, singular-value entropy,
or a spectral invariant. Those quantities require their own definitions and
numerical policies before use in H-AI-1.

### Frobenius norm

```text
||A||_F = sqrt(sum_i sum_j p_ij^2)
```

This is retained as a simple whole-operator norm statistic. It must not be
reported as a spectral norm.

## 3. Analytic controls

The implementation includes fixtures whose expected values are known without
running an experiment.

For two-token identity attention:

```text
[1 0]
[0 1]
```

- mean row entropy = 0;
- normalized entropy = 0;
- mean max weight = 1;
- mean L2 concentration = 1;
- mean effective support = 1;
- Frobenius norm = sqrt(2).

For uniform two-token attention:

```text
[1/2 1/2]
[1/2 1/2]
```

- mean row entropy = ln(2);
- normalized entropy = 1;
- mean max weight = 1/2;
- mean L2 concentration = 1/2;
- mean effective support = 2;
- Frobenius norm = 1.

A rectangular fixture also verifies that the baseline layer does not
accidentally assume square self-attention.

## 4. What this baseline does not implement

This slice intentionally does **not** implement:

- H-AI-1 outcome estimation;
- associative-recall/copy/overwrite data generation;
- train/holdout splitting;
- regression or classification;
- confidence intervals;
- Beneficial / Equivalent / Harmful / Inconclusive decisions;
- matrix rank or effective rank;
- singular values or eigenspectra;
- TDI recovery features;
- FLAT-ATTENTION runtime dependencies;
- GPU code.

Those omissions are explicit. Issue #57 requires H-AI-1 to test incremental
value beyond static diagnostics, so static controls must exist and be validated
before the holdout experiment is executed.

## 5. Relationship to FLAT-ATTENTION

FLAT-ATTENTION already treats deterministic scalar/reference implementations as
correctness oracles for optimized kernels. Its current EPG reference path uses
an online softmax and deliberately does **not** materialize an `N x N`
score/probability matrix.

Therefore the production/reference FLAT path must not be changed merely to
satisfy this matrix-based research helper. A later TDI/FLAT bridge has two
legitimate implementation choices, which must be fixed by the Gate C protocol:

1. accumulate diagnostics equivalent to this module from streaming softmax
   sufficient statistics, without constructing a dense attention matrix; or
2. use a separate bounded research oracle that materializes the operator only
   when a preregistered experiment requires full operator-level diagnostics.

The intended research comparison remains:

```text
FLAT deterministic semantic
  -> static operator diagnostics -> static baseline features
  -> controlled intervention     -> TDI recovery features
  -> task outcome                -> preregistered incremental comparison
```

No optimized FLAT kernel should be selected because a TDI descriptor looks
promising. A candidate semantic should first survive the deterministic,
mechanistic experiment and only then become an optimization target.

The exact spectral controls from historical finite-state TDI experiments must
also not be transferred by name alone. There, exactness follows from rational
transition kernels. Neural attention uses floating-point scores and softmax, so
any spectral control used by H-AI-1 needs its own numerical definition,
precision and tolerance policy.

## 6. Next gate

After this baseline slice is merged, Gate C must preregister the complete H-AI-1
experiment **before touching its untouched holdout**. At minimum the frozen
protocol must specify:

- deterministic associative-recall/copy/overwrite task generator;
- train/development/holdout split policy;
- intervention taxonomy;
- early recovery depths and features;
- later retrieval-deficit target;
- task-local and static baseline feature set;
- any additional spectral baseline and its exact definition;
- model class and regularization;
- seeds;
- uncertainty method;
- fixed Beneficial / Equivalent / Harmful / Inconclusive margins;
- negative-result preservation policy.

This ordering follows issue #57 and Gate B: define competent controls, freeze the
falsifiable protocol, then execute the holdout experiment.
