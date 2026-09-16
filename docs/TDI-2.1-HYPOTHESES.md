# TDI-2.1 — Frozen hypotheses

Date frozen: 2026-09-16

## H1 — Experience increment

With model class, observation budget and training budget matched, valid prior experience improves held-out decision quality versus an otherwise identical arm with no reusable experience.

Primary contrast: `I1 - B0`.

## H2 — Experience specificity

The improvement in H1 is reduced when the same-capacity memory is populated with shuffled or structurally unrelated experience.

Primary contrast: `I1 - B1`.

## H3 — Transfer

A pattern representation learned on Development retains positive incremental value on at least one preregistered structural shift without adapting on protected labels.

This is not equivalent to claiming absolute out-of-distribution validity.

## H4 — Selective reliability

A frozen confidence score can order cases by empirical risk well enough that abstaining or escalating low-confidence cases reduces error on the retained fast subset.

## H5 — Hybrid utility

Under a preregistered utility function that prices errors and System-2 use separately, the frozen hybrid policy `H` outperforms both `always-I1` and `always-S2` on the primary holdout.

## H6 — Consolidation

Permitted post-decision feedback improves future Development/Validation performance without materially degrading previously mastered preregistered strata.

## H7 — Component necessity

At least one of context gating, hierarchical representation, associative retrieval or calibrated arbitration contributes measurable incremental value under controlled ablation.

## H8 — No latency hypothesis

Latency is not part of H1–H7. Runtime measurements are secondary engineering observations. A faster candidate that fails the scientific criteria is a failed scientific candidate.

## Multiplicity policy

H1 is the primary hypothesis. H2–H7 are secondary and must be reported individually. Exploratory findings outside these hypotheses may generate later TDI-2.xx series but cannot be relabeled as preregistered confirmation.
