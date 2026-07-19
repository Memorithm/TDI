# TDI-5.4 — Nonlinear Conditional Sufficiency Preregistration

Status: preregistered design; no TDI-5.4 evaluator exists at the time this document is frozen, no TDI-5.4 test labels have been generated or inspected, and no full TDI-5.4 scientific run has occurred.

Date: 2026-07-19

## 1. Governance and immutable ancestry

TDI-5.4 is an additive scientific revision. It must not modify, regenerate, reformat, replace, or recommit any frozen TDI-5.3 or earlier artifact.

The immediate frozen ancestor is:

- commit: `989a4c772024aa9ee1219d771d477357c42484f6`;
- tag: `tdi-5.3-preregistered-results`;
- result SHA-256: `fbe3feb2883d52de018e61e2ee5771a808fa334d756bd5de87c91d772be034ef`;
- evaluator SHA-256: `93181fb75d4882be2ca0b26c1babe9db747583d6e08e6992617ec12b7f65460f`;
- preregistration SHA-256: `7223128dcfd751ebeb6488c01c3512d0a10b35937ec170504984295eb421682e`;
- scientific-manifest SHA-256: `2659e0afae239074262b1900ff2d5f6754df5247a31a7e0c729fc3fda759e7c6`.

The scientific motivation and validity analysis are frozen separately in `docs/research/TDI-5.4-SCIENTIFIC-GAP-ANALYSIS.md`.

The planned additive artifacts are:

- evaluator: `tdi-bench/src/bin/tdi-nonlinear-conditional-sufficiency-v54.rs`;
- reproduction script: `scripts/reproduce-tdi5.4.sh`;
- CI workflow: `.github/workflows/tdi54-ci.yml`;
- preregistration hash: `docs/TDI-5.4-NONLINEAR-CONDITIONAL-SUFFICIENCY-PREREGISTRATION.sha256`;
- evaluator hash: `docs/TDI-5.4-NONLINEAR-CONDITIONAL-SUFFICIENCY-EVALUATOR.sha256`;
- scientific manifest: `docs/TDI-5.4-SCIENTIFIC-CODE.sha256`.

Creating the evaluator is prohibited until this preregistration and its content hash have been committed and pushed. A full scientific run, merge to `main`, result tag, and release each require separate human authorization.

## 2. Scientific question

### 2.1 Primary question

Does O1 remain conditionally redundant once O2 is known when prediction is performed by preregistered deterministic nonlinear model families rather than only the frozen additive ridge family?

This asks whether the TDI-5.3 equivalence classification is robust to smooth interactions and bounded discontinuous/threshold structure, or is an artifact of linear model misspecification.

### 2.2 Primary estimand

For model family `f` and feature layout `S`, let `L_f(S)` be aggregate held-out mean squared error for block-standardized U6, using the complete preregistered fitting and validation pipeline for that family.

The conditional contribution of O1 given O2 is

\[
\Delta_f = \frac{L_f(B2)-L_f(B12)}{L_f(B2)}.
\]

Positive `Delta_f` favors adding O1. The practical-equivalence margin is frozen at:

\[
-0.02 \leq \Delta_f \leq 0.02.
\]

The margin is inherited from TDI-5.3 so that the nonlinear result answers the same practical question. It must not be changed after any TDI-5.4 final-test label is generated or inspected.

### 2.3 Hypotheses

- Absence-of-practical-benefit null: no eligible preregistered nonlinear family shows a replicable O1 benefit of at least 2% after O2.
- Nonlinear-benefit alternative: at least one eligible nonlinear family shows an aggregate relative U6-MSE improvement of at least 2%, positive simultaneous uncertainty evidence, and the frozen block-replication conditions.
- Equivalence hypothesis: every eligible confirmatory family has its aggregate simultaneous interval contained in `[-0.02, 0.02]`, satisfies the frozen block-equivalence conditions, and has no material model-class interaction relative to the linear control.
- Harm alternative: at least one eligible family shows a replicable relative worsening of at least 2% after O1 is added to B2.
- Model-class-interaction hypothesis: the O1 contribution in a nonlinear family differs practically from the contribution in the linear control.

A nonsignificant difference is not equivalence. A family that cannot meet the frozen predictive-competence and sensitivity floor is uninformative for conditional sufficiency; its apparent O1 equivalence cannot support the global equivalence verdict. The joint-signal and independent-O2 controls are reported separately so a richer baseline that absorbs some O2 information is not automatically mistaken for model failure.

### 2.4 Scope

The confirmatory scope is restricted to:

- the frozen TDI random nonempty-successor-mask generator;
- widths 3 and 4 for training, validation, and in-distribution testing;
- widths 5 and 6 for secondary distributional transfer;
- reference state zero;
- a flip of node `width - 1`;
- `Noop` future actions;
- observation horizons 1 and 2;
- target horizons 3, 4, 5, 6, and 8;
- the three model families and four feature layouts frozen below.

### 2.5 Prohibited interpretations

TDI-5.4 cannot establish:

- causal effects of O1 or O2;
- universal or mathematical sufficiency of O2 across all measurable predictors;
- validity for untested model families, generators, actions, perturbations, widths, or empirical systems;
- accurate width-6 calibration merely from comparative improvement;
- equivalence under every target scale or loss;
- bit-identical floating-point output on every architecture.

## 3. Frozen dynamics, features, and targets

### 3.1 Generator

For width `w`, each of the `2^w` source states receives a deterministic SplitMix64-derived nonempty successor mask in `[1, 2^(2^w)-1]`. TDI-5.4 intentionally preserves the TDI-5.3 modulo mapping, including its negligible one-preimage bias toward mask 1, so that generator changes do not confound the model-class question.

The evaluator must use:

- fixed SplitMix64 constants identical to the frozen evaluator;
- one independently derived system for every candidate seed;
- sorted, deduplicated successors;
- stable ordered maps and fixed reduction order;
- no wall-clock, environment, network, or thread-scheduling input.

### 3.2 Dynamic quantities

Let `P_h` and `Q_h` be the exact rational state distributions at horizon `h` from the reference and perturbed initial states. Define:

\[
O_h = \sum_s \min(P_h(s),Q_h(s))
\]

and, when `O_h < 1`,

\[
U_h = -\log_2(1-O_h).
\]

State-distribution propagation and overlap must remain exact until the explicitly documented conversion to `f64` for feature construction and modeling.

### 3.3 Baseline features

`B0` contains exactly 13 values, in this order:

1. reference path entropy at depth 1, normalized by `ln(2^w)`;
2. reference path entropy at depth 2, normalized by `ln(2^w)`;
3. perturbed path entropy at depth 1, normalized by `ln(2^w)`;
4. perturbed path entropy at depth 2, normalized by `ln(2^w)`;
5. reference reachable fraction at depth 1;
6. reference reachable fraction at depth 2;
7. `ln(1 + reference path count)` at depth 1;
8. `ln(1 + reference path count)` at depth 2;
9. perturbed reachable fraction at depth 1;
10. perturbed reachable fraction at depth 2;
11. `ln(1 + perturbed path count)` at depth 1;
12. `ln(1 + perturbed path count)` at depth 2;
13. width as `f64`.

### 3.4 Feature layouts

The four confirmatory layouts are:

- `B0`: 13 baseline features;
- `B1`: B0 followed by O1;
- `B2`: B0 followed by O2;
- `B12`: B0 followed by O1 and O2, in that order.

No O2-O1 feature, target-derived feature, future-horizon summary, or post-hoc interaction may enter a confirmatory layout. Nonlinear families transform only the values already present in the selected layout.

### 3.5 Targets

Targets are U3, U4, U5, U6, and U8. U6 is primary. The exact corresponding overlaps O3, O4, O5, O6, and O8 are retained for reconstruction and secondary metrics.

No final-test target may be loaded by model-selection code. The evaluator must represent split identity at the type/API boundary or enforce an equivalent tested runtime contract.

## 4. Populations and blind seed namespaces

### 4.1 Accepted-record counts per seed block

Each of three blocks A, B, and C contains:

| Population | Width | Accepted records |
|---|---:|---:|
| Training width 3 | 3 | 12,000 |
| Validation width 3 | 3 | 3,000 |
| Final test width 3 | 3 | 5,000 |
| Training width 4 | 4 | 12,000 |
| Validation width 4 | 4 | 3,000 |
| Final test width 4 | 4 | 5,000 |
| OOD test width 5 | 5 | 10,000 |
| OOD test width 6 | 6 | 5,000 |

Each block therefore requests 55,000 accepted records; the full experiment requests exactly 165,000. Training and validation are combined only after hyperparameter selection, giving 30,000 final fitting records per block. The primary in-distribution test contains 10,000 records per block and 30,000 aggregate records.

### 4.2 Candidate seed starts

Every range below is new and disjoint from all TDI-5.3 and earlier scientific populations.

| Block | train w3 | validation w3 | test w3 | train w4 | validation w4 | test w4 | OOD w5 | OOD w6 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| A | 1,000,000,000 | 1,010,000,000 | 1,020,000,000 | 1,030,000,000 | 1,040,000,000 | 1,050,000,000 | 1,060,000,000 | 1,070,000,000 |
| B | 1,200,000,000 | 1,210,000,000 | 1,220,000,000 | 1,230,000,000 | 1,240,000,000 | 1,250,000,000 | 1,260,000,000 | 1,270,000,000 |
| C | 1,400,000,000 | 1,410,000,000 | 1,420,000,000 | 1,430,000,000 | 1,440,000,000 | 1,450,000,000 | 1,460,000,000 | 1,470,000,000 |

Candidate seeds advance by exactly one per attempted system, including excluded candidates. A population's final exclusive seed and every rejection reason must be reported.

### 4.3 Bootstrap and non-scientific seeds

- block A paired bootstrap: `0x5444_4935_3441_0001`;
- block B paired bootstrap: `0x5444_4935_3442_0002`;
- block C paired bootstrap: `0x5444_4935_3443_0003`;
- aggregate stratified bootstrap: `0x5444_4935_3441_4747`;
- deterministic smoke namespace: `0x5444_4935_3453_4D4B`.

Bootstrap population-scope domains are:

- in-distribution width-3/4 test: `0x4944_0000_0000_0001`;
- OOD width 5: `0x4F4F_4435_0000_0002`;
- OOD width 6: `0x4F4F_4436_0000_0003`.

For a block or aggregate base bootstrap seed and a population scope, the effective seed is `splitmix64(base_seed XOR scope_domain)`. One index stream for that scope is reused across all horizons, layouts, families, and paired interaction calculations.

The smoke namespace must never contribute a scientific record. The frozen models use no stochastic initialization; there is therefore no model-training RNG. Adding a stochastic training step is a protocol change and is forbidden in TDI-5.4.

### 4.4 Split isolation

- Training data may fit candidate models and training-only normalizers.
- Validation data may select one family configuration per block and may not set scientific thresholds.
- Final test and OOD labels may be evaluated only after configuration selection and final refitting are complete.
- The full evaluator must not print final-test metrics until every model in every block has been frozen in memory and its selected configuration printed.
- TDI-5.3 test seeds are public and may not be reused for TDI-5.4 confirmatory inference.

## 5. Exclusions and failure separation

The only scientific candidate exclusions are:

1. exact full recovery at O2, making the conditioning population outside scope and U at later horizons structurally problematic;
2. exact full recovery at any target horizon, making that U target undefined.

The first applicable exclusion reason in the order above is recorded. No record is excluded for model residual, leverage, target magnitude, feature rarity, graph density, or perceived outlier status.

The following are fatal evaluator failures, not exclusions:

- malformed or empty successor sets;
- overlap outside its exact mathematical range;
- missing distribution or exploration layer;
- arithmetic overflow;
- nonfinite feature, target, normalization, prediction, loss, or interval;
- inconsistent feature length or population count;
- duplicate or overlapping seed reservation;
- singular/invalid solver state or tree invariant violation;
- checkpoint incompatibility or corruption.

Fatal failures produce no scientific verdict. They must preserve a machine-readable incomplete record and nonzero exit status.

## 6. Termination bounds

Generation uses the same width-specific hard bounds as TDI-5.3:

| Width | Maximum-attempt multiplier | Consecutive no-progress limit |
|---|---:|---:|
| 3 | 64 | 25,000 |
| 4 | 96 | 50,000 |
| 5 | 128 | 75,000 |
| 6 | 256 | 100,000 |

For each population:

- `max_attempts = accepted_target * multiplier`;
- the evaluator stops with a typed failure before an attempt beyond that bound;
- the evaluator stops after the frozen number of consecutive exclusions without an accepted record;
- seed arithmetic is checked before generation;
- no rejection or retry loop is unbounded.

Across all 24 populations, the absolute maximum is 17,280,000 candidate attempts. Model-fitting, validation, bootstrap, tree depth, split candidates, checkpoint count, and output cardinality are also finitely bounded below.

## 7. Confirmatory model families

Exactly three families are confirmatory.

### 7.1 L — linear ridge continuity control

Purpose: determine whether the new blind split reproduces the qualitative TDI-5.3 ridge behavior and provide the reference for model-class interaction.

- Input: standardized raw layout features only.
- Objective: sum of squared standardized-target residuals plus `lambda * sum(beta_j^2)`.
- Fixed `lambda`: 1.0.
- Intercept: fitted and unpenalized.
- Hyperparameter selection: none.

This is a new additive implementation and does not modify or call a frozen evaluator. Mathematical equivalence, not byte identity with TDI-5.3 coefficients, is the test oracle.

### 7.2 Q — complete degree-2 polynomial ridge

Purpose: detect smooth nonlinear effects, including O1-by-O2, O1-by-baseline, O2-by-baseline, and nonlinear baseline adjustment.

For `p` standardized raw features, the expansion contains:

- all `p` linear terms;
- every product `z_i * z_j` for `0 <= i <= j < p`.

The feature count is `p + p(p+1)/2`; B12 therefore has exactly 135 expanded terms. Expanded columns are centered and scaled on the fit subset after expansion.

The frozen lambda grid, in simplicity-preferred tie order, is:

`[1000.0, 100.0, 10.0, 1.0, 0.1]`.

The objective and intercept treatment match family L.

### 7.3 T — bounded quantile-grid regression tree

Purpose: detect threshold, saturation, and discontinuous interaction structure not represented by family Q.

The frozen configuration grid, in simplicity-preferred tie order, is:

1. maximum depth 3, minimum leaf size 512;
2. maximum depth 3, minimum leaf size 128;
3. maximum depth 5, minimum leaf size 512;
4. maximum depth 5, minimum leaf size 128.

Additional frozen rules:

- root depth is 0; depth 5 permits at most 32 leaves and 63 total nodes;
- leaf prediction is the arithmetic mean standardized target in stable row order;
- each node examines each feature in layout order;
- rows are sorted by `(feature total order, original row index)`;
- candidate left counts are `floor(q * n / 32)` for `q = 1..31`;
- duplicate candidate counts are evaluated once;
- a candidate is valid only when both children meet minimum leaf size and the adjacent feature values differ;
- threshold is `lower + (upper - lower) / 2` and must be finite and strictly separating;
- gain is parent SSE minus child SSE values, accumulated in frozen order;
- splitting requires gain greater than `1e-12 * max(parent_SSE, 1.0)`;
- gain ties within that same tolerance prefer lower feature index, then lower candidate count;
- no pruning, surrogate split, missing-value path, random feature subset, or bagging is permitted.

## 8. Normalization, fitting, and selection

### 8.1 Raw feature normalization

For each block, family, layout, fitting stage, and applicable model:

- means and population standard deviations are computed in row order from the allowed fit subset only;
- values are centered and divided by that standard deviation;
- if a standard deviation is finite but at most `1e-12`, the column is recorded as degenerate, its scale is set to 1, and its centered values are therefore zero;
- nonfinite moments are fatal.

Family Q performs a second center/scale pass over its expanded columns. Family T receives standardized raw features. Family L receives standardized raw features.

### 8.2 Target normalization

For each block and horizon, target mean and population standard deviation are fitted from the allowed fit subset. A finite target standard deviation at most `1e-12` is a fatal degenerate-target failure; it is not replaced by 1.

During selection, training target moments standardize training and validation targets. After selection, target moments are refitted on combined training + validation and are used for final fitting, in-distribution test, and OOD evaluation.

### 8.3 Linear solver

Families L and Q use a deterministic Cholesky solution of the ridge normal equations, with:

- fixed row, feature, and triangular-loop order;
- an unpenalized intercept;
- checked finite accumulation;
- a strictly positive Cholesky pivot requirement;
- a reported diagonal condition proxy `(max diagonal / min diagonal)^2`;
- fatal failure if the proxy is nonfinite or greater than `1e14`.

No iterative convergence tolerance, BLAS, FFI, or parallel reduction is permitted.

### 8.4 Nested validation rule

Configuration selection is performed separately for each seed block and family Q/T. Family L has its single fixed configuration.

For every candidate configuration:

1. fit four U6 models on combined width-3/4 training data, one for each B0/B1/B2/B12 layout;
2. calculate U6 MSE on the combined width-3/4 validation data using training-only moments;
3. define selection score as the arithmetic mean of the four validation MSE values.

The configuration with the lowest score is selected. If scores differ by no more than

`1e-12 * max(1, abs(current_best_score))`,

the earlier configuration in the frozen simplicity order wins. Selection scores, component MSE values, and the chosen configuration must be printed before test evaluation.

After selection, the evaluator refits every layout at every target horizon on combined training + validation data. U6-selected family complexity is reused unchanged at U3, U4, U5, and U8. No horizon-specific or layout-specific complexity selection is permitted.

## 9. Predictions and metrics

### 9.1 Primary prediction space

Models predict standardized U. Predictions must be finite and are not clipped in U space. They are unstandardized with the final fit moments.

Reconstructed overlap is:

`O_hat = clamp(1 - 2^(-U_hat), 0, 1)`.

Every clamp is counted. A nonfinite pre-clamp or reconstructed value is fatal; fallback substitution is forbidden.

### 9.2 Required metrics

For every block, population, horizon, family, and layout, report:

- MSE and MAE;
- absolute and relative paired MSE changes for the frozen comparisons;
- R2;
- Spearman rank correlation with deterministic average ties;
- bias, observed mean, and predicted mean;
- calibration intercept and slope;
- fraction clamped at 0 and 1;
- target standard deviation;
- deterministic 10-bin equal-count calibration RMSE;
- normalized calibration RMSE, equal to calibration RMSE divided by target standard deviation.

Equal-count bins are formed after stable sorting by `(prediction total order, original row index)` and splitting the ordered rows into ten groups whose sizes differ by at most one, with earlier bins receiving any remainder. Calibration RMSE weights bins by record count.

If target variance or prediction variance is at most `1e-15`, R2, correlation, or calibration slope is reported as an explicit `DEGENERATE` status. Zero must not be substituted for an undefined statistic.

### 9.3 Overfitting diagnostics

For every selected family/configuration/layout at U6, report:

- selection-stage training MSE;
- selection-stage validation MSE;
- validation minus training MSE;
- final combined-fit in-sample MSE;
- final test MSE;
- test minus combined-fit MSE;
- coefficient count for L/Q or node/leaf/depth counts for T;
- degenerate-column count and condition proxy where applicable.

These diagnostics do not change the primary classification after the test is observed. They distinguish predictive benefit from possible overfitting and are mandatory context for every verdict.

### 9.4 Calibration guardrail label

For B2 -> B12, calibration is reported independently from predictive equivalence. Define normalized absolute bias as `abs(bias) / target_standard_deviation`. In standardized U and reconstructed O, B12 is `CALIBRATION_NOT_MATERIALLY_WORSE` when both:

- normalized absolute bias increases by no more than 0.02;
- normalized calibration RMSE increases by no more than 0.02.

Otherwise it is `CALIBRATION_WORSE`; mixed U/O labels remain mixed and do not get collapsed. This label never converts a failed predictive criterion into success or equivalence.

## 10. Bootstrap and uncertainty contract

### 10.1 Resampling unit

The unit is one accepted generated system. Predictions are paired across layouts and families. Models, selected configurations, normalizers, and scalers are held fixed.

This is explicitly conditional test-set uncertainty. It does not include training-sample or model-selection uncertainty. Three blind seed blocks, validation/test gaps, and family agreement are separate robustness evidence; the final report must not call the intervals unconditional.

### 10.2 Replicates and ordering

- 4,000 replicates per seed block;
- 4,000 block-stratified aggregate replicates;
- resampling with replacement within each block;
- block sizes preserved exactly in aggregate replicates;
- for each frozen population scope, one shared resample-index stream is used across every family, layout, horizon, and paired interaction within a block/aggregate replicate;
- no model refitting in the bootstrap;
- percentile intervals with deterministic linear interpolation.

### 10.3 Confidence levels

- ordinary block and secondary intervals: 95%, quantiles 0.025 and 0.975;
- simultaneous aggregate intervals across the three confirmatory families: Bonferroni familywise 95%, individual confidence 98.333333%, quantiles `1/120` and `119/120`;
- simultaneous aggregate model-class-interaction intervals across Q-vs-L and T-vs-L: Bonferroni familywise 95%, individual confidence 97.5%, quantiles 0.0125 and 0.9875.

Each interval reports lower, median, and upper values. Absolute MSE improvement, relative MSE improvement, reconstructed-O MSE improvement, reconstructed-O MAE improvement, calibration change, and model-class interaction are all bootstrapped from the shared draws.

## 11. Confirmatory criteria

### 11.1 A — joint-signal positive control, B0 -> B12

For family `f`, A passes only if all are true:

1. B12 standardized-U6 MSE is lower in every block;
2. every block's ordinary 95% absolute-improvement interval has lower bound greater than zero;
3. the median block relative reduction is at least 15%;
4. aggregate relative reduction is at least 15%;
5. the family's simultaneous aggregate absolute-improvement interval has lower bound greater than zero;
6. B12 Spearman is greater than B0 Spearman in every block;
7. aggregate absolute bias is no more than B0 absolute bias plus 0.02 standardized units.

### 11.2 B — independent-O2 positive control, B1 -> B12

For family `f`, B passes only if all are true:

1. B12 standardized-U6 MSE is lower in every block;
2. every block's ordinary 95% absolute-improvement interval has lower bound greater than zero;
3. the median block relative reduction is at least 10%;
4. aggregate relative reduction is at least 10%;
5. the family's simultaneous aggregate absolute-improvement interval has lower bound greater than zero;
6. B12 Spearman is not lower than B1 Spearman in any block;
7. aggregate absolute bias is no more than B1 absolute bias plus 0.02 standardized units.

### 11.3 Family eligibility

A family is `ELIGIBLE` for the conditional-sufficiency conclusion only if:

- all generation, fitting, prediction, and metric checks complete finitely;
- its selection contract completes exactly;
- B2 and B12 predictions are nondegenerate in every block;
- aggregate B2 standardized-U6 R2 is greater than zero;
- B2 standardized-U6 Spearman is greater than zero in every block.

Family L must additionally pass A and B, providing the blind continuity control. Each nonlinear family Q/T must instead have aggregate B2 standardized-U6 MSE no more than 10% above family L's aggregate B2 MSE. This frozen sensitivity floor prevents a severely underperforming nonlinear model from supporting equivalence merely because it cannot exploit either overlap. A/B outcomes for Q/T remain mandatory confirmatory context but do not gate their eligibility, because a nonlinear baseline may legitimately absorb information that was incremental under the linear baseline.

Otherwise it is `UNINFORMATIVE_FOR_SUFFICIENCY`. An uninformative family is preserved and reported; it cannot be silently dropped or counted toward equivalence.

### 11.4 C — O1 conditional classification, B2 -> B12

For every eligible family, calculate block and aggregate `Delta_f`.

`BENEFICIAL` requires:

- at least two blocks have point `Delta_f >= 0.02` and ordinary block absolute-improvement interval lower bound greater than zero;
- aggregate `Delta_f >= 0.02`;
- simultaneous aggregate absolute-improvement interval lower bound greater than zero.

`EQUIVALENT` requires:

- every block point estimate is inside `[-0.02, 0.02]`;
- at least two ordinary block relative intervals are wholly inside `[-0.02, 0.02]`;
- the simultaneous aggregate relative interval is wholly inside `[-0.02, 0.02]`.

`HARMFUL` requires:

- at least two blocks have point `Delta_f <= -0.02` and ordinary block absolute-improvement interval upper bound less than zero;
- aggregate `Delta_f <= -0.02`;
- simultaneous aggregate absolute-improvement interval upper bound less than zero.

Classification precedence is `BENEFICIAL`, then `EQUIVALENT`, then `HARMFUL`, then `INCONCLUSIVE`. This preserves the TDI-5.3 exact-boundary convention. Ineligible families are labeled `UNINFORMATIVE`, not inconclusive.

### 11.5 D — model-class interaction

For each nonlinear family `f` in `{Q,T}`, when both `f` and L are eligible, define:

\[
\Theta_f = \Delta_f - \Delta_L.
\]

`POSITIVE_INTERACTION` requires aggregate `Theta_f >= 0.02`, its simultaneous interval lower bound greater than zero, and at least two block point interactions at least 0.02.

`INTERACTION_EQUIVALENT` requires all block point interactions inside `[-0.02,0.02]` and the simultaneous aggregate interval wholly inside that range.

`NEGATIVE_INTERACTION` is the sign-reversed analogue of positive interaction. Otherwise the interaction is `INCONCLUSIVE`.

### 11.6 Global primary classification

Apply the following rules in order:

1. `MIXED_DIRECTIONAL_EVIDENCE` if at least one eligible family is beneficial and at least one eligible family is harmful.
2. `NONLINEAR_MODEL_CLASS_BENEFIT` if L and the relevant nonlinear family are eligible, Q or T is beneficial, L is not beneficial, and the corresponding nonlinear-vs-L interaction is positive.
3. `O1_BENEFIT_NOT_MODEL_CLASS_SPECIFIC` if any eligible family is beneficial but rule 2 does not apply.
4. `ROBUST_EQUIVALENCE_ACROSS_TESTED_FAMILIES` if L, Q, and T are all eligible and equivalent, and both nonlinear-vs-L interactions are interaction-equivalent.
5. `O1_HARM_IN_AT_LEAST_ONE_FAMILY` if at least one eligible family is harmful and no family is beneficial.
6. `INCONCLUSIVE_OR_UNINFORMATIVE` otherwise.

No global category is called a universal pass. Negative, mixed, harmful, equivalent, and inconclusive classifications are valid completed scientific outcomes.

## 12. Preregistered secondary analyses

Secondary analyses cannot change the primary classification.

### 12.1 Distributional transfer

Using final width-3/4-fitted models and scalers without refitting or reselection, evaluate B2 -> B12 separately on width 5 and width 6 for each family. Report the complete metric set, ordinary 95% intervals, calibration labels, and C-style classification. These are labeled `SECONDARY_OOD` and receive no claim beyond the frozen generator.

### 12.2 Multi-horizon conditional sufficiency

Using U6-selected family configurations, evaluate B2 -> B12 on in-distribution U3, U4, U5, and U8. Report block/aggregate effects and ordinary 95% intervals. State the count of horizons beneficial, equivalent, harmful, and inconclusive per family. No multiplicity-adjusted confirmatory verdict is assigned.

### 12.3 Target-scale agreement

For U-trained predictions reconstructed into O space, report B2 -> B12 MSE, MAE, rank correlation, bias, calibration, and bootstrap differences. Label each family/horizon/population as:

- `SAME_DIRECTION` when U-MSE and O-MSE point changes have the same sign;
- `TARGET_SCALE_DISAGREEMENT` otherwise.

No direct-O model is selected or promoted to confirmatory status.

### 12.4 Exploratory outputs

The following are exploratory only:

- ridge/polynomial coefficients;
- tree split/importance summaries;
- residual summaries by width, target decile, O1 decile, and O2 decile;
- model learning curves on deterministic prefixes;
- any post-run hypothesis not named above.

Exploratory results must appear after all confirmatory verdicts and carry an `EXPLORATORY` label on every machine-readable row.

## 13. Checkpoint and resume protocol

### 13.1 Immutable generation chunks

Scientific generation is checkpointed in fixed chunks of 1,000 accepted records, with a smaller final chunk when required. Each population therefore has a predetermined accepted-count chunk schedule.

Each canonical chunk records:

- schema/resume version `TDI54_CHECKPOINT_V1`;
- experiment identifier;
- Git commit;
- evaluator SHA-256;
- preregistration SHA-256;
- scientific-manifest SHA-256;
- seed block and population identifier;
- width and target accepted count;
- chunk index and accepted-index range;
- first candidate seed and final exclusive seed;
- attempted, accepted, and excluded counts;
- rejection counts by frozen reason;
- every record's candidate seed plus its scientific values in canonical field order, using lowercase 16-digit hexadecimal `f64::to_bits` values for floats;
- completion status.

Chunks are written to a same-filesystem temporary path, flushed, closed, and atomically renamed. A separate immutable `.sha256` sidecar records the SHA-256 of the completed chunk; the population checkpoint ledger references the ordered `(chunk path, chunk SHA-256)` pairs. The chunk and sidecar are then read-only. No file contains its own hash.

### 13.2 Resume validation

On resume, the evaluator must:

1. verify experiment, resume version, Git commit, and all three scientific hashes;
2. verify population identity and frozen chunk schedule;
3. verify every chunk hash and canonical record count;
4. require chunk indices to be contiguous from zero with no duplicate, gap, or overlap;
5. reconstruct the exact next accepted index, attempt index, seed, and rejection accounting;
6. refuse any incompatible, malformed, nonfinite, writable-completed, or unexpected file;
7. continue only from the first missing chunk.

It must never silently repair, skip, replace, or truncate a checkpoint.

### 13.3 Semantic equivalence

For the same miniature population specs, uninterrupted and interrupted/resumed execution must produce byte-identical canonical scientific records, selected configurations, predictions, metrics, intervals, criteria, and final scientific payload. Operational metadata may additionally report interruption/resume events and is excluded from the scientific-payload equality hash.

Checkpoint data are not final results. A partial-output aggregate SHA-256 is updated only by the reproduction script from the ordered list of immutable chunk hashes and is recorded in the incomplete metadata.

Checkpoint records, especially final-test and OOD chunks, are blinded operational artifacts. They must not be inspected for scientific patterns, used to alter code or thresholds, or summarized outside the frozen resume path. Resume may execute only the same hash-identified evaluator and protocol. If an incompatible scientific correction is required after checkpoint labels exist, TDI-5.4 remains incomplete and the correction requires an incident report and a new numbered revision with new test seeds.

## 14. Determinism and numerical reproducibility

TDI-5.4 requires:

- pure Rust scientific computation with zero FFI;
- fixed stable iteration and sort tie-breaking;
- sequential floating-point accumulation in declared order;
- no parallel scientific reductions;
- fixed seed namespaces and RNG algorithm;
- no external network access during evaluation;
- no CPU-feature-dependent approximation selected at runtime;
- explicit finite checks at every scientific boundary.

Within one architecture/toolchain matching the frozen provenance, repeated smoke and miniature runs must be byte-identical.

Across supported architectures, the contract is numerical rather than bit-exact:

- exact integer/rational fields and seed/accounting fields must be identical;
- finite scalar metrics must agree to `abs(a-b) <= 1e-12 + 1e-10 * max(abs(a),abs(b))`;
- selected configurations, classifications, and verdicts must be identical;
- every decision-boundary distance must exceed the observed cross-architecture numeric discrepancy by at least a factor of 10, otherwise the result is `NUMERICALLY_UNSTABLE` and receives no scientific verdict.

## 15. Output schema

The canonical result is UTF-8 TSV with schema `TDI54_RESULT_V1`. Every row has exactly these columns:

`record_type, scope, block, population, family, layout, horizon, metric, estimate, ci_level, ci_lower, ci_median, ci_upper, status, detail`

Missing dimensions use the literal `NA`; numeric fields use a frozen scientific decimal formatter or `NA`; NaN and infinity are forbidden. Text fields may not contain a tab, carriage return, or newline; structured details use semicolon-separated `key=value` pairs in frozen key order.

Rows occur in this order:

1. schema and provenance;
2. hashes and compiler/Cargo versions;
3. frozen constants and seed definitions;
4. checkpoint/resume summary;
5. population generation and exclusion accounting;
6. normalization and configuration selection;
7. final model diagnostics;
8. primary metrics and bootstrap intervals;
9. A/B family eligibility;
10. C family classifications;
11. D model-class interactions;
12. calibration/overfitting labels;
13. secondary OOD and multi-horizon analyses;
14. global primary classification;
15. terminal record.

The scientific-payload SHA-256 covers every canonical row before the terminal row. The terminal row must be unique and last. For a completed scientific execution it contains:

- `record_type = COMPLETE`;
- `scope = CONFIRMATORY`;
- `status` equal to the global classification;
- `detail` containing accepted total, attempted total, bootstrap replicate count, and that preterminal scientific-payload SHA-256.

The evaluator must provide `--validate-result <path>` to parse the canonical file, verify schema, row order, required cardinalities, uniqueness, finite fields, criteria consistency, terminal record, and scientific-payload hash. Presence-only `grep` validation is forbidden.

Human-readable stdout/log output is generated from the same in-memory report but is not the canonical statistical source.

## 16. Reproduction-script contract

`scripts/reproduce-tdi5.4.sh` must:

- use defensive Bash (`set -Eeuo pipefail`, safe `IFS`, restrictive `umask`);
- resolve and enter the repository root without recording it in scientific output;
- refuse any dirty tracked repository state;
- permit untracked files only under explicitly allowlisted, hash-verified frozen prior-result directories or the selected TDI-5.4 result/checkpoint directory, and refuse every other untracked path;
- verify all frozen TDI-5.1, TDI-5.2, TDI-5.3, and TDI-5.4 hashes;
- require exact confirmation `TDI54_CONFIRM_FULL_RUN=I_AUTHORIZE_TDI54_PREREGISTERED_RUN` before expensive work;
- acquire an exclusive lock atomically;
- refuse completed outputs and refuse conflicting/incompatible incomplete outputs;
- use only the frozen result and checkpoint paths;
- build release and offline;
- preserve evaluator exit status through `tee`;
- write initial metadata and an incomplete marker before scientific generation;
- preserve checkpoints and mark interruption/failure without a completion marker;
- validate the canonical result through evaluator `--validate-result`;
- write canonical/log SHA-256 files;
- create the completion marker only after all validation and hash checks pass;
- make completed results, hashes, metadata, chunks, and marker read-only;
- never delete or overwrite a result.

The only optional untracked prior-result allowlist is `results/tdi5.3-independent-overlap-activation/`, and, when present, it must contain exactly the four frozen release files with these hashes:

- completion marker: `c5f63d6030364ea1287c14c1efd058d83162d75e9336140c1c80fa3d747329ef`;
- result log: `fbe3feb2883d52de018e61e2ee5771a808fa334d756bd5de87c91d772be034ef`;
- result-log hash file: `b0ffcd11962f02d8c7328a56a2b383416c67ab4bb8313ae1d68d0e2d78f53095`;
- metadata: `1168bf0181a26b0d580b4bf3b336e5fc9c044552a05edd6da20b48d2ee4d649b`.

Absence of that directory is valid. Any extra, missing, changed, symlinked, or non-regular entry makes it invalid when present.

Expected paths are under:

`results/tdi5.4-nonlinear-conditional-sufficiency/`

with canonical result `tdi-nonlinear-conditional-sufficiency-v54.tsv`, human log `tdi-nonlinear-conditional-sufficiency-v54.log`, metadata, hash files, incomplete/complete marker, lock, and `checkpoints/` subtree.

## 17. Computational budget and planning estimate

Frozen computational quantities:

- accepted records: 165,000;
- maximum candidate attempts: 17,280,000;
- model families: 3;
- layouts: 4;
- horizons: 5;
- Q candidate configurations: 5;
- T candidate configurations: 4;
- bootstrap replicates: 4,000 per block and 4,000 aggregate;
- tree maximum depth: 5;
- tree split-grid candidates: at most 31 per feature/node;
- checkpoint accepted chunk size: 1,000;
- immutable generation chunks: 165.

Pre-implementation planning estimates, not scientific stopping rules:

- expected candidate attempts: approximately 165,300, using the frozen TDI-5.3 exclusion rate only as an operational estimate;
- expected reference ARM64 wall time: 6 to 10 hours;
- expected CPU consumption: 6 to 10 single-core CPU-hours, with no parallel scientific reduction;
- bootstrap/model-fitting portion: approximately 2 to 5 hours;
- peak resident memory: at most approximately 2 GiB under the intended streaming implementation;
- completed/checkpoint disk use: at most approximately 500 MiB;
- attempt-cap worst case: below approximately 450 CPU-hours under linear scaling from the frozen run, plus bounded model fitting.

Preflight and miniature benchmarks may refine reported operational estimates before authorization, but may not change populations, attempts, models, bootstrap count, criteria, or any final-test-facing contract.

## 18. Required tests before activation PR

The evaluator and reproduction layer must include and pass:

- hand-derived exact-distribution, overlap, U-transform, polynomial-expansion, ridge, tree-split, tree-leaf, calibration-bin, SHA-256, and metric oracles;
- tests that fail if an oracle is changed to buggy implementation output;
- exact population and chunk counts;
- all seed-domain and worst-case reservation disjointness checks, including frozen ancestors;
- train/validation/test label-access and split-leakage tests;
- fixed feature order/count tests for every layout/family;
- configuration-grid, selection-score, tolerance, and tie-break boundary tests;
- model determinism for L, Q, and T;
- bootstrap index, interval, familywise quantile, and paired-interaction determinism;
- nonfinite, degenerate-variance, solver-condition, and invalid-tree failure tests;
- equivalence, benefit, harm, eligibility, interaction, and global-classification boundary tests;
- exact output schema, cardinality, terminal-row, and validator negative tests;
- attempt-budget and no-progress termination tests;
- interrupted-run and incompatible-checkpoint tests;
- uninterrupted versus resumed miniature scientific-payload equality;
- reproduction-script tests for missing/wrong token, dirty repository, lock conflict, existing output, invalid checkpoint, evaluator failure through `tee`, truncated result, and completion-marker ordering;
- repeated deterministic smoke comparison;
- preflight proving zero scientific generation;
- no-argument and unsupported-argument refusal;
- release and offline build;
- formatting, Clippy with warnings denied, and all workspace/all-target/all-feature tests;
- clean tracked working-tree and no-unexpected-generated-file checks.

CI and local smoke tests must use miniature or synthetic populations only. They may not execute any frozen full population.

## 19. Outcome handling

### 19.1 Scientific completion

Any valid global category in Section 11.6 is a completed result and must be preserved, including negative, harmful, mixed, equivalent, or inconclusive evidence.

### 19.2 Numerical instability

Any nonfinite quantity, solver failure, unstable cross-architecture decision, invalid interval, or criterion inconsistency yields `NUMERICALLY_UNSTABLE`. No family is silently removed, no threshold is changed, and no scientific verdict is issued.

### 19.3 Incomplete execution

Interruption, resource exhaustion, attempt/no-progress termination, incompatible checkpoint, output validation failure, or provenance mismatch yields an incomplete run. Checkpoints, logs, metadata, and hashes are retained; no completion marker, results tag, or release may be created.

### 19.4 Success, failure, equivalence, and mixed evidence

- `ROBUST_EQUIVALENCE_ACROSS_TESTED_FAMILIES` supports conditional redundancy only across L/Q/T and the frozen distribution.
- `NONLINEAR_MODEL_CLASS_BENEFIT` falsifies the broad linear-derived redundancy interpretation while leaving the narrower TDI-5.3 ridge verdict intact.
- `O1_BENEFIT_NOT_MODEL_CLASS_SPECIFIC` indicates a new-data benefit not uniquely attributable to nonlinearity.
- harmful and mixed categories are preserved without reinterpretation.
- `INCONCLUSIVE_OR_UNINFORMATIVE` does not support equivalence.

No post-result protocol repair is permitted. A scientific defect discovered after the run requires a new incident report and a new numbered revision.

## 20. Activation and human authorization gate

The planned real-run command is:

```bash
TDI54_CONFIRM_FULL_RUN=I_AUTHORIZE_TDI54_PREREGISTERED_RUN \
  bash scripts/reproduce-tdi5.4.sh
```

The command must not be run until:

1. evaluator, script, CI, tests, and all hashes are frozen;
2. the activation PR is open and fully reviewed;
3. local and CI evidence is complete;
4. an execution-readiness report names the exact branch, PR, commit, hashes, resource estimate, output paths, interruption/recovery procedure, and all verified frozen files;
5. the human owner explicitly authorizes the full scientific execution.

Authorization to merge, tag results, or publish a release is separate and is not implied by authorization to run.
