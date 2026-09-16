# TDI-2.1 — Operational definitions

Date frozen: 2026-09-16

## Experience

Information derived only from Development observations available before the tested decision. Experience may be compressed into parameters, prototypes, associative memories, counters or sufficient statistics, but the construction procedure must be reproducible.

## Pattern

A reproducible representation element used to match or retrieve prior experience. A pattern is not assumed to correspond to a human concept.

## Immediate proposal

A single bounded forward pass that returns a candidate output without tree search, iterative planning, external tool use or repeated self-refinement. Complexity may still depend on fixed model dimensions and memory-bank size; therefore TDI-2.1 does not call the total computation `O(1)` with respect to those quantities.

## Confidence

A scalar used for ranking or routing cases. It is not called a probability of correctness unless calibration tests support that interpretation.

## System 1

The frozen immediate-proposal arm `I1`.

## System 2

A separately specified reference procedure allowed to use more computation but not protected labels or forbidden future information.

## Arbitration

A frozen policy that chooses `I1`, `S2` or abstention from information available at routing time.

## Consolidation

A bounded update to reusable experience after permitted feedback. Consolidation is evaluated separately from initial training.

## Structural transfer

Evaluation on a preregistered distribution or generator variant not used for fitting the tested representation. Transfer does not imply universal out-of-distribution validity.

## Leakage

Any use, direct or indirect, of protected target labels, future observables, holdout-derived thresholds or holdout-derived feature design before the holdout is formally opened.
