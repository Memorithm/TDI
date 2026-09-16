# TDI-2.1 — Falsification ledger

Date frozen: 2026-09-16

This document lists observations that would count against the programme's working claims. Negative results are first-class outputs.

## Falsifiers for H1

- mean paired improvement `B0 - I1 <= 0` on PrimaryHoldout;
- 95% paired-bootstrap lower bound `<= 0`;
- the gain disappears when parameter/storage budgets are matched more closely.

## Falsifiers for H2

- shuffled/unrelated experience performs as well as valid experience within the preregistered equivalence tolerance;
- the gain is explained by additional parameters rather than information content.

## Falsifiers for H3

- the candidate loses incremental value under every preregistered structural shift;
- absolute transfer error becomes unusable despite a small relative gain.

## Falsifiers for H4

- confidence does not rank risk better than the permuted control;
- high-confidence errors concentrate in the ambiguity strata;
- calibration fails materially on PrimaryHoldout.

## Falsifiers for H5

- hybrid utility fails to exceed either always-fast or always-slow;
- any advantage depends on post-hoc utility penalties or thresholds.

## Falsifiers for H6

- consolidation fails to improve forward performance;
- improvement is accompanied by forgetting beyond the frozen tolerance;
- gains require target access not allowed by the protocol.

## Falsifiers for H7

- no controlled ablation changes performance in the preregistered direction;
- apparent component gains vanish under capacity-matched controls.

## Global invalidators

- holdout leakage;
- generator/target code shared with candidate logic in a way that exposes answers;
- non-reproducible split or seed identity;
- result artifact not bound to the executed commit;
- selective omission of failed runs or rejected cases.

A falsified hypothesis remains documented. It is not silently removed, renamed or replaced by a favorable exploratory observation.
