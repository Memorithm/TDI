# TDI-7 attention evidence handoff

This document defines the minimum evidence packet that TDI may hand to ITD Simulator or FLAT-ATTENTION research tooling after a TDI-7.x stage has produced a frozen result.

It does not authorize TDI-7.2 and does not promote any FLAT semantic.

## Required packet fields

The packet contains exactly the two confirmatory task families frozen by TDI-7.0:

- `associative_recall`;
- `copy`.

A task-level evidence record contains:

- task family identifier;
- evaluated generator count;
- evaluated intervention-pair count;
- rejected record count plus an explicit rejection ledger whose reason counts sum exactly to the rejected count;
- B0 MSE;
- B1 MSE;
- relative MSE reduction;
- paired 95% interval lower/upper bounds;
- frozen task verdict;
- intervention-location summaries.

For the current frozen evaluator, the minimum intervention-location summary contains the site identifier and record count. Both `early-token` and `late-token` must be present exactly once and each must contain one record per evaluated generator. This freezes the already-implemented equal record weighting of the two sites without inventing a post-hoc location-specific effect statistic. Any later per-site effect measure remains secondary and requires its own explicit definition before use.

The packet also contains the aggregate two-task TDI-7 verdict, recomputed from the two task verdicts using the TDI-7.0 multi-task gate rather than trusted from serialized input.

The packet-level provenance contains:

- TDI commit SHA;
- protocol identifier;
- evaluator specification identifier;
- semantic identifier;
- generator version;
- seed-range identity;
- intervention definition;
- intervention aggregation policy;
- observation depths;
- feature schema;
- model class and hyperparameter grid;
- bootstrap seed and replicate count;
- numerical policy;
- classifier margins and classifier precedence policy;
- final-holdout access status.

## Validation rules

A consumer must reject a packet when:

- the confirmatory task set is not exactly associative recall plus copy;
- either frozen intervention site is missing, duplicated or not balanced one-record-per-generator;
- rejection-reason counts do not reconcile exactly with the rejected count or a reason code is duplicated;
- any required floating-point value is non-finite;
- `B0 MSE <= 0` for a claimed primary comparison;
- `B1 MSE < 0`;
- interval lower bound exceeds upper bound;
- the supplied relative reduction disagrees with `(B0-B1)/B0` beyond the declared tolerance;
- a task verdict is inconsistent with the frozen TDI-7.0 decision rule and its explicit implementation precedence;
- the supplied aggregate verdict disagrees with the frozen two-task gate;
- provenance is incomplete;
- the packet claims a final-holdout result while its provenance says the holdout was not accessed.

At the exact ±2% relevance boundaries, the implementation precedence is frozen as: Beneficial, then Harmful, then Equivalent, then Inconclusive. This records the behavior already implemented by the TDI-7.1 classifier and removes ambiguity at overlapping textual boundaries without changing the classifier outputs.

## ITD Simulator use

ITD Simulator may consume the packet as one evidence source in its explicit ablation ladder:

`static/task -> ITD -> TDI -> ITD+TDI`.

ITD must not rewrite a TDI task verdict. Any joint ITD+TDI conclusion is a separate comparative result.

## FLAT-ATTENTION use

FLAT research tooling may attach a valid TDI evidence packet to a frozen candidate semantic as mechanistic evidence. It must not interpret a positive TDI result as sufficient evidence for semantic promotion.

A FLAT candidate still requires its own mathematical specification, scalar/reference oracle, invariant/adversarial tests, quality evidence, cost evidence and failure-mode record.

## Stability policy

The first implementation remains an internal TDI-7 schema fixture. Promotion to a reusable public Rust API occurs only after the first frozen TDI-7.2 packet demonstrates which fields are actually required in practice.
