# TDI-9.3.2 — Complete C3 search-carrier calibration

Scope: **EXACT REPRESENTATION CALIBRATION ONLY**. No trajectory data, observation/threshold freeze, concrete model, final material or execution authorization is introduced.

## Reproduce

```bash
cargo test --locked -p tdi-ai --features experimental --example boolean_c3_calibration
cargo run --locked -p tdi-ai --features experimental --example boolean_c3_calibration
bash scripts/check-tdi9.3-representation-calibration.sh
```

The example is gated by the existing `experimental` feature. Its tests and report are executed by the existing representation-calibration gate, which is already called by the TDI AI experimental-contract CI workflow. The executable accepts no arguments or external datasets. Output is commented TSV with schema `tdi9.3.c3-calibration.v1`.

## Complete abstract carrier, not observed trajectories

The nine existing C3 predicates define 512 bit assignments. This calibration uses the existing verifier-encoding validity helper **before** calling the reference action oracle. Exactly one of the four verifier-state bits must be set.

The complete carrier is partitioned into:

- **384 invalid verifier encodings**: zero or multiple verifier-state bits;
- **8 well-formed unrecoverable violations**: verifier violated, no checkpoint and no remaining work; the reference oracle returns no action;
- **120 action-bearing rows**: 48 CONTINUE, 16 VERIFY, 16 BACKTRACK and 40 STOP.

All 512 addresses appear exactly once in these categories. Rejections are not dropped silently, relabeled CONTINUE or included as successful actions.

This partition is enforced by the calibration harness. The raw `BooleanPolicy` action-only IR is not claimed to reject malformed or unrecoverable inputs. A future runtime adapter needs its own validated observation carrier and explicit rejection boundary. The 120 abstract rows are not a claim that every combination is reachable in a concrete solver trajectory.

## Candidate and oracle comparison

The existing C3 baseline generator contributes 27 single-predicate controls, in its original order. The full existing ordered C3 reference policy is appended as a separately identified 28th control. Every candidate is evaluated through the shared `evaluate_policy_candidates` API on exactly the same 120 rows.

The full policy reproduces all oracle actions. Its regression counts over the 120 rows are 784 predicate reads and 328 logical operations under the existing eager-expression/ordered-rule accounting. Its worst-case structural vector remains 13 reads, 6 logical operations and depth 2 per decision. These are reference-IR counts, not measured hardware time.

Among the 27 single-rule controls, the unique best action-mismatch count is 40/120, obtained by `VERIFIER_SATISFIED -> STOP`, with CONTINUE fallback. This is agreement with the reference oracle, not task accuracy or evidence that this policy is useful for adaptive inference.

## Confusion matrix and Pareto evidence

Matrix rows are expected actions and columns are actual actions. Both use the order CONTINUE, VERIFY, BACKTRACK, STOP. Every candidate reports all 16 cells, including zeros.

For the best single-rule control, the matrix is:

```text
48  0  0   0
16  0  0   0
16  0  0   0
 8  0  0  32
```

This exposes all 16 missed VERIFY actions, all 16 missed BACKTRACK actions and eight missed STOP actions. Each matrix's off-diagonal count is independently checked against the shared search evaluator's mismatch count.

The existing multi-objective Pareto function retains the cheaper best single-rule control and the exact but more expensive full reference policy. It does not collapse correctness and cost into a scalar score or promote the simplified controller.

## Scientific boundary and reuse

This extends TDI-9.3 representation/search calibration only. It does not freeze TDI-9.1 fields, choose real observation thresholds, change C2/C3 production semantics, authorize TDI-9.2, or access other series' protected/final material.

BooleanLab BL-14.5 and TDI-9.3 use the same experimental discipline: match the question and carrier, retain ambiguity and failed cases, account for verification work, and avoid converting representation agreement into a model-quality claim. Implementations remain bench-owned; no unqualified cross-repository runtime dependency or copied production control semantics are introduced.
