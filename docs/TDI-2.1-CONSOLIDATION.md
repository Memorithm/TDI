# TDI-2.1 — Online consolidation contract

Date frozen: 2026-09-16

## Question

Can permitted feedback update reusable experience so later cases improve without erasing performance on previously mastered strata?

## Separation from the primary experiment

The primary H1–H5 confirmatory experiment uses a frozen experience store. Consolidation is a separate H6 experiment and cannot modify the already scored primary holdout.

## Permitted feedback

A consolidation run must state one feedback source before execution:

- true target revealed after the decision;
- bounded scalar reward supplied by the task generator;
- correction produced by a frozen `S2` procedure, evaluated separately from true-target supervision.

These sources may not be mixed without an explicit arm.

## Candidate updates

Development may compare:

- prototype exponential moving update;
- competitive Hebbian-style update with bounded learning rate;
- output-association delta update;
- bounded insertion plus deterministic eviction;
- no-update control.

The exact equations and learning rates are frozen before the consolidation holdout stream begins.

## Continual-learning measurements

After each preregistered block report:

- forward performance on the next block;
- retained performance on a fixed replay audit set;
- forgetting = best prior audit score minus current audit score;
- experience-store occupancy and replacements;
- update count and rejected updates.

## Contamination boundary

The consolidation stream is not reused as the untouched PrimaryHoldout. Any case whose target or feedback has been revealed becomes training history and must be labeled accordingly.

## H6 support rule

H6 requires improved forward performance relative to the no-update control and forgetting below a preregistered tolerance on the replay audit set. Improvement with uncontrolled forgetting does not support H6.
