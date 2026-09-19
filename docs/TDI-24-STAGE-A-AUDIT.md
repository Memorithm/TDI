# TDI-24 Stage-A audit/freeze candidate

Status: **candidate pending exact-head qualification, material review, and merge**.

This slice audits the merged Stage-A surfaces (campaign slices 01–09) as one
bounded contract. It does not execute training, task populations, Development,
Validation, protected/final data, or a performance benchmark.

The executable audit requires:

- exact V6/C6 width and logical carrier-storage equality;
- direct-only C6 to equal V6 before and after the shared full/causal normalizer;
- reflection to swap the R/L branches while versioned provenance remains exact;
- contract-version pins for algebra, channels, enantiomorphic scoring,
  recombination, masking, normalization and source accounting;
- fail-closed rejection of non-finite carrier inputs, derived overflow, and
  non-finite logits even when the affected position is masked.

The candidate manifest is `docs/tdi24-stage-a-freeze.yaml`. It is deliberately
non-authorizing. Stage A is not recorded as qualified merely because this file
exists: qualification requires this candidate to land on `main` after all
applicable exact-head CI and material review findings are green/resolved.

No scientific outcome, novelty, speedup, hardware behavior, trained-model
quality, or downstream FLAT-ATTENTION readiness follows from this audit.
