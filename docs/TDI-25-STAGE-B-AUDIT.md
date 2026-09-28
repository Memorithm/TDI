# TDI-25 Stage-B leakage/balance audit candidate

Status: **candidate pending exact-head qualification, material review, and merge**.

This slice audits the merged Stage-B surfaces (campaign slices 11–19) as one
bounded leakage, balance and provenance contract. It does not execute training,
model evaluation, protected/final data, or a performance benchmark.

The executable audit requires:

- inference callbacks to receive only sealed inference inputs with oracles
  retained outside the callback;
- Development/Validation seed domains to remain mixed-seed disjoint for the
  declared registry block;
- case canonical records/digests to be stable and free of oracle/target fields;
- typed split manifests to reject protected/final labels fail-closed;
- contract-version pins for every Phase-B generator and leakage-discipline
  surface.

The candidate manifest is `docs/tdi25-stage-b-freeze.yaml`. It is deliberately
non-authorizing. Stage B is not recorded as qualified merely because this file
exists: qualification requires this candidate to land on `main` after all
applicable exact-head CI and material review findings are green/resolved.

No scientific outcome, novelty, speedup, hardware behavior, trained-model
quality, or downstream FLAT-ATTENTION readiness follows from this audit.
