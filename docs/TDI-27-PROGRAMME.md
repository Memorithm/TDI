# TDI-27.x — Latent Orthogonal Innovation / Concept Geometry

**Status:** active Stage-0 / Development bootstrap.

```yaml
series: TDI-27.x
stage: 0
development_execution_authorized: true
confirmatory_execution_authorized: false
final_execution_authorized: false
```

TDI-27 turns a simple pair of latent-space operations into a falsifiable research
programme:

[
v^{(\ell)} =
\frac{1}{|P|}\sum_{s\in P}h_s^{(\ell)}
-
\frac{1}{|C|}\sum_{s\in C}h_s^{(\ell)}
]

and, after removing a declared known/nuisance subspace (U),

[
r^{(\ell)}=(I-P_U)v^{(\ell)},\qquad
\hat v^{(\ell)}=r^{(\ell)}/\|r^{(\ell)}\|.
]

The mean contrast and orthogonal projection are established linear-algebra
operations. **TDI-27 does not claim novelty for those primitives.** The research
question is whether a controlled combination of residual energy, stability,
causal intervention, interaction and cross-layer transport exposes useful
structure that simpler concept-vector measurements miss.

## Scientific questions

TDI-27 starts with five falsifiable Development questions.

1. **H27-A — geometric innovation.** After removal of a declared subspace, is
   the remaining contrast energy reproducibly greater than matched null and
   finite-sample controls?
2. **H27-B — stability.** Is the residual direction stable under resampling,
   sample-size changes and equivalent numerical projection methods?
3. **H27-C — causal specificity.** Under equal-norm interventions, does the
   residual direction change the declared target more than matched random
   orthogonal controls?
4. **H27-D — functional interaction.** Can two geometrically orthogonal
   directions still interact non-additively downstream?
5. **H27-E — effective latent rank.** Across ordered layers/tasks, how many
   sequentially independent residual directions are needed before additional
   contrasts become redundant?

All five can fail independently. A large geometric residual alone is not a
causal or mechanistic result.

## Primary Development metrics

For raw contrast (v) and residual (r=(I-P_U)v):

[
I = \frac{\|r\|^2}{\|v\|^2}
]

is the **innovation energy ratio**. It is descriptive, not a novelty score.

For a declared intervention effect (E(d)) and matched orthogonal controls
(z_j):

[
G = E(\hat v)-\frac{1}{m}\sum_j E(z_j)
]

is the **causal novelty gap**.

For two directions (a,b), with effects measured from the same baseline:

[
J(a,b)=E(a+b)-E(a)-E(b)
]

is the **interaction residual**. (J\neq0) is evidence that geometric
orthogonality does not imply functional independence under the declared
response function.

Sequential innovation uses the accepted unit residuals
(Q_{k-1}=[q_1,\ldots,q_{k-1}]):

[
r_k=(I-Q_{k-1}Q_{k-1}^{T})d_k.
]

The accepted rank is the count of residuals above the declared numerical and
statistical thresholds.

## Controls required before any scientific promotion

- explicit (P/C) provenance and a fixed sign convention;
- shuffled-label and no-contrast nulls;
- matched equal-norm orthogonal controls;
- resampling / bootstrap stability intervals;
- sample-size sensitivity;
- declared nuisance subspaces;
- numerical agreement checks between two-pass modified Gram-Schmidt and a
  later independent QR/SVD or pseudoinverse implementation;
- fail-closed handling of non-finite values and zero residuals;
- no use of final or protected TDI surfaces;
- no FLAT-ATTENTION, NNIS, SciRust or ElasticXxx performance claim from TDI
  geometry alone.

## Ordered research programme

### TDI-27.0 — bootstrap and synthetic falsification

Define the semantics, deterministic geometry kernel and synthetic controls.
The first runner must demonstrate all of the following without a trained model:

- a known + novel contrast whose innovation ratio is analytically known;
- a fully explained contrast whose residual direction is undefined rather than
  silently normalised;
- sequential extraction of independent directions;
- a causal-control example in which the target direction beats a matched
  orthogonal control;
- a nonlinear example in which orthogonal directions have non-zero interaction.

This stage is Development only.

### TDI-27.1 — resampling and null calibration

Add deterministic bootstrap/resampling, label-shuffle nulls, confidence
intervals for (I), cosine stability and accepted rank, and sample-size
sensitivity.

The Development resampling layer must keep the replicate count and RNG seed
caller-supplied: there is no implicit default that could later masquerade as a
frozen statistical choice. Index bootstrap uses sampling with replacement and
unbiased bounded integer draws; every requested draw must be accounted for.
The first slice establishes only this deterministic sampling substrate and does
not define an interval, p-value, acceptance threshold, or confirmatory rule.

Positive and control groups are resampled separately at their original
cardinalities. Their deterministic pseudo-random streams are domain-separated,
so changing the cardinality of one group does not perturb the other group's
draw sequence. This is a reproducibility and coupling-control property, not a
claim that a deterministic PRNG provides physical randomness.

The next Development layer converts each paired bootstrap index replicate into
explicit positive/control means and the signed mean contrast
`mean(P*) - mean(C*)`. The implementation validates group dimensions before
sampling, preserves the declared P-minus-C sign, and fails closed if a derived
sum or mean becomes non-finite.

Each bootstrap contrast can then be residualised against one caller-declared
known/nuisance subspace using the same tolerance for every replicate. This
produces a Development distribution of residual geometry and innovation energy
without silently dropping zero-contrast replicates. Fully explained non-zero
contrasts remain explicit zero-residual cases with no unit residual direction.

Directional stability is measured against the signed full-sample residual
direction using cosine similarity. The P-minus-C sign convention is preserved.
Bootstrap replicates with an undefined residual direction remain explicit
undefined entries; they are not silently removed. If the full-sample residual
direction itself is undefined, directional-stability analysis fails closed.

A generic deterministic order-statistic primitive is also available for later
Development summaries. It accepts zero-based ranks supplied directly by the
caller, sorts finite scalar values deterministically, and returns either one
order statistic or a closed rank interval. It deliberately does not choose a
probability-to-rank conversion, interpolation convention, confidence level,
coverage interpretation, or decision threshold.

Innovation energy can now be summarized without discarding the underlying
bootstrap evidence: the Development report keeps the full-sample
`I = ||r||^2 / ||v||^2`, every bootstrap replicate value, and one closed
order-statistic interval at caller-supplied ranks. The interval remains an
ordered-value summary only and has no confidence interpretation at this stage.

The null-control substrate now includes deterministic label permutations over
the pooled P/C samples. Every replicate preserves the original group
cardinalities and forms a complete partition of the pooled indices. The
permutation stream is domain-separated from the bootstrap streams.

Each shuffled partition is then mapped to an explicit signed null contrast
`mean(P_shuffle) - mean(C_shuffle)`. The report retains the exact pooled
indices assigned to both groups so every null contrast is reproducible and
auditable.

Null contrasts are residualised against the same caller-declared basis used by
the observed geometry. A shuffled contrast whose raw norm is at or below the
declared tolerance is retained explicitly with undefined residual geometry; it
is neither dropped nor assigned an artificial innovation-energy value.

A descriptive null comparison now reports the observed innovation energy, all
defined shuffled-label innovation energies, the count of undefined null
replicates, the total replicate count, and the count of defined null values
greater than or equal to the observed value. No normalization or finite-sample
correction is applied, so this is explicitly not a p-value and has no rejection
threshold or verdict.

Sample-size sensitivity starts from a separate deterministic subsampling
primitive. P and C are sampled without replacement, with caller-supplied sample
sizes and domain-separated RNG streams.

A sensitivity grid can then be evaluated at an explicit ordered list of
caller-supplied `(n_P, n_C)` points. Every point retains all Development
replicate innovation-energy values, including explicit undefined entries for
zero raw contrasts. The code does not generate grid points, optimize sample
size, interpolate between points, or define an adequacy threshold.

Sequential latent rank now has a non-pinning Development summary. Given the
full-sample direction sequence and caller-provided replicate direction
sequences, TDI reports the exact accepted rank for the full sample, every
replicate rank, and integer lower/upper values at caller-supplied order ranks.
The primitive does not generate replicate direction sets or attach confidence
semantics to those integer bounds.

The first 15-slice TDI-27.1 Development batch is now represented end to end:
deterministic resampling plans, independent P/C bootstrap streams, bootstrap
contrasts, residual geometry, signed direction stability, caller-ranked order
statistics, innovation-energy summaries, label-shuffle null partitions and
contrasts, explicit undefined null geometry, descriptive null exceedance
counts, without-replacement sample-size sensitivity, and sequential accepted
rank summaries. The synthetic runner exercises these surfaces together and
emits machine-readable non-pinning/non-confirmatory markers.

These slices still define no statistical confidence interval, p-value,
acceptance threshold, or scientific verdict.

### TDI-27.2 — projection-method differential

Compare the current two-pass modified Gram-Schmidt reference against an
independent QR/SVD or Moore-Penrose implementation. Near-degenerate subspaces
must be included. Numerical disagreement is a blocker, not a parameter-tuning
opportunity.

The first Development slice adds a dependency-free, column-pivoted Householder
QR oracle. It retains the caller-supplied numerical tolerance, compares rank
and residual coordinates against the existing two-pass modified Gram-Schmidt
reference, and fails closed when either the retained rank differs or the
maximum absolute residual-coordinate difference exceeds a separately supplied
agreement bound. The slice includes analytic and near-degenerate controls. It
does not choose either tolerance, define a scientific threshold, or authorize
real-model, confirmatory, or final execution.

The second Development slice applies the same fail-closed differential to every
replicate of an explicit bootstrap plan. Replicate population indices and mean
contrasts remain attached to each comparison, and any rank or residual mismatch
blocks the complete batch instead of silently excluding a replicate. The
synthetic runner exposes only the replicate count and maximum observed method
difference under caller-declared numerical bounds. It does not convert those
observations into a calibrated tolerance, statistical interval, performance
claim, or scientific decision.

The third Development slice extends the comparison to normalized-direction
semantics. The two methods must agree on whether a residual direction is
defined at the caller-declared numerical tolerance; when both are defined,
their direction coordinates must satisfy a separate caller-declared absolute
bound. The bound is recorded independently from the residual-coordinate bound.
This prevents small absolute residual differences near zero from hiding
different downstream direction semantics. It does not calibrate either bound,
select a tolerance from observed data, or authorize a scientific decision.

### TDI-27.3 — causal intervention harness

Introduce a model-agnostic intervention adapter with fixed intervention norms,
matched controls, target and non-target outcomes, and signed dose-response
curves over predeclared alpha values.

The first Development slice is intentionally limited to deterministic state
construction. It requires a non-empty, finite, strictly increasing signed
alpha grid; normalizes the target and every caller-declared control direction;
rejects controls that are not orthogonal to the target within a separate
caller-declared bound; and constructs target/control displacements with equal
norm `abs(alpha)`. The achieved state-minus-baseline norm is checked against
the requested norm under a dimension-scaled machine-roundoff bound, so a dose
lost against a baseline coordinate's ULP fails closed rather than silently
breaking target/control matching. It does not run a model, inspect outcomes,
choose controls or doses, authorize actuation, or infer a causal or scientific
result.

### TDI-27.4 — cross-layer transport

Measure whether residual directions persist after a declared alignment
operator, including orthogonal Procrustes-style alignment where justified.
Raw cosine across incompatible representation bases is not sufficient.

### TDI-27.5 — sequential latent-rank programme

Build ordered innovation bases across layers, tasks and concept families;
estimate effective rank and identify where later contrasts collapse into
previously discovered subspaces.

### TDI-27.6 — real-model Development adapters

Consume Development-only activation traces through versioned adapters. TDI
owns the scientific protocol. FLAT-ATTENTION is an execution/comparison target;
NNIS may later provide device execution evidence. Neither becomes scientific
ground truth.

### TDI-27.7 — promotion gates

Only general, independently tested numerical/statistical primitives may be
proposed for SciRust. FLAT-ATTENTION integration requires reproducible causal
and structural evidence. Hardware and runtime claims require their owning
repositories and direct measurements.

### TDI-27.8 — possible confirmation

A confirmatory stage may be designed only after the Development line has fixed
the model population, activation extraction, controls, metrics, thresholds,
resampling protocol and final-data derivation. Stage 0 does not authorise it.

## Initial implementation

The Stage-0 reference implementation lives in
`tdi-bench/src/concept_geometry_v27.rs`.

The executable Development smoke is
`tdi-bench/src/bin/tdi27_concept_geometry.rs`.

Run:

```bash
bash scripts/check-tdi27-bootstrap.sh
cargo run --locked -p tdi-bench --bin tdi27_concept_geometry
```

The executable output is synthetic Development evidence only. It is intended to
falsify the implementation before real-model experiments, not to establish a
neural-model result.
