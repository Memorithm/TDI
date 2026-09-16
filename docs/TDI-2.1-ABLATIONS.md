# TDI-2.1 — Frozen ablation matrix

Date frozen: 2026-09-16

The full candidate is decomposed into independently removable components. Ablations are evaluated on Development/Validation before the confirmatory holdout and then frozen.

| Arm | Experience retrieval | Context gating | Hierarchy | Confidence routing | Consolidation |
|---|---|---|---|---|---|
| A0 | no | no | no | no | no |
| A1 | yes | no | no | no | no |
| A2 | yes | yes | no | no | no |
| A3 | yes | no | yes | no | no |
| A4 | yes | yes | yes | no | no |
| A5 | yes | yes | yes | yes | no |
| A6 | yes | yes | yes | yes | yes, separate stream |

## Additional targeted ablations

- replace learned/derived patterns with random normalized vectors;
- shuffle pattern-output associations;
- set context gate to all ones;
- replace hierarchical representation with a single linear projection of matched width;
- replace soft retrieval with hard nearest-pattern selection;
- permute confidence scores before routing;
- freeze all online updates.

## Analysis rule

A component receives credit only from a paired comparison that differs in that component while keeping other frozen inputs and budgets as close as possible.

## H7 criterion

H7 is supported if at least one preregistered component ablation produces a reproducible degradation on Validation and the same direction of degradation on PrimaryHoldout. The component is not called *necessary* in a universal sense; the conclusion is restricted to the tested task families and candidate configuration.
