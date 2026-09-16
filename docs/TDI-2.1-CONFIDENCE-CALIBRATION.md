# TDI-2.1 — Confidence and selective-prediction contract

Date frozen: 2026-09-16

## Candidate confidence signals

Development may compare the following predeclared signals:

- normalized entropy of retrieval weights;
- top-1/top-2 score margin;
- distance to the selected prototype or attractor;
- ensemble/disagreement score if the ensemble is capacity-accounted;
- calibrated transformation of the above fitted on Validation only.

No signal is called a probability of correctness before calibration is evaluated.

## Primary calibration measurements

For bounded classification-like tasks report:

- Brier score;
- expected calibration error with frozen bin edges;
- maximum calibration error;
- reliability table;
- risk-coverage curve and area under that curve.

For continuous targets, replace correctness calibration with preregistered error quantiles or conformal-style residual coverage only if fitted without holdout leakage.

## Selective prediction

For coverage levels `100%, 90%, 75%, 50%`, report empirical error on retained `I1` cases after sorting by the frozen confidence score. Ties use a deterministic case-id order.

## Threshold selection

The primary arbitration threshold is chosen on Validation and frozen before either holdout is opened. No threshold may be optimized on PrimaryHoldout or TransferHoldout.

## Required negative control

A randomly permuted confidence ranking must be evaluated with the same coverage analysis. Confidence is useful for routing only if it outperforms this control and exhibits monotone or near-monotone risk reduction under the preregistered tolerance.

## Failure conditions

The confidence mechanism is rejected for routing if:

- higher confidence is not associated with lower empirical risk;
- calibration degrades materially under the primary holdout;
- apparent calibration depends on post-hoc holdout thresholding;
- ambiguity cases systematically receive high confidence while failing.
