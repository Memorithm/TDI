# TDI-9.0 status

- Scientific series: TDI-9.x
- Stage: TDI-9.0 adaptive inference dynamics preregistration/bootstrap
- Status: active candidate; not yet merged/frozen
- Tracking issue: #92
- Bootstrap branch: `research/tdi-9.0-adaptive-inference-preregistration`
- Primary H9-A: C2 observation-conditioned adaptive stopping versus C0 fixed compute
- Primary H9-B: C3 explicit verification/backtracking versus C2 adaptive stopping
- Primary cells: 3 task families × 3 difficulty strata = 9 per hypothesis
- Quality non-inferiority margin: `delta_q = 0.02`
- Material compute margin: `delta_k = 0.05`
- TDI-9.2 final runner: does not exist
- TDI-9.2 final seed list: does not exist
- TDI-9.2 final dataset: does not exist
- TDI-9.2 result payload: does not exist
- Human confirmation token: intentionally not part of TDI-9
- Final confirmation model: future-derived public entropy, contract to be frozen during TDI-9.1 before reveal
- TDI-7.2 interaction: forbidden
- TDI-8.2 interaction: forbidden

## Current deliverables

TDI-9.0 establishes only the research contract and integrity boundary. It does not implement C0/C1/C2/C3 evaluators or produce adaptive-inference evidence.

Current bootstrap surfaces:

- `docs/TDI-9-PROGRAMME.md`;
- `docs/TDI-9.0-ADAPTIVE-INFERENCE-PREREGISTRATION.md`;
- `docs/TDI-9.0-ADAPTIVE-INFERENCE-PREREGISTRATION.gitblob`;
- `docs/TDI-9.0-IMPLEMENTATION-GATE.md`;
- `docs/TDI-9.0-STATUS.md`;
- repository agent-policy updates;
- `scripts/check-tdi9-bootstrap.sh` once added;
- CI invocation of the bootstrap checker once added.

TDI-9.1 remains blocked until this bootstrap is merged, blob-pinned and CI-green on the exact PR head.
