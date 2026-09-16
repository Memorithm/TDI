# TDI-22.1 — Status

Status: **freeze candidate; bounded non-final TDI-22.2 authorized only after this freeze is merged and all gates are green on `main`**.

## Frozen by this stage

- T0/T1/T3/T4 exact arm semantics; T2 is explicitly deferred.
- T3 versus T4 remains the critical torsor-specific contrast.
- Primary cells P1–P5 and their F1/F2/F3 × G0/G1/G2/G3 assignments.
- Development/validation episode counts, query/candidate counts and structural index bounds.
- Binary64 numeric domain and derived-component bounds.
- Deterministic G1/G2 geometry and bounded G3 supplied-coordinate lattice.
- Exact SplitMix64 stream packing and scalar mappings.
- Family-specific draw ordering and F3 target-before-nuisance causality.
- T1 `AmbiguousT1Top` fail-closed tie policy.
- Full resource-accounting ledger including dynamic-state and static-parameter bits.
- Byte-canonical `tdi22-eval-record-v1` encoding.

## After merge

TDI-22.2 may implement and execute only bounded non-final development/validation surfaces listed in `docs/TDI-22.1-IMPLEMENTATION-GATE.md` after all TDI-22 integrity scripts pass on `main`.

## Still forbidden

- T2 execution;
- learned latent geometry;
- final/confirmatory runner, seeds, dataset or result payload;
- result-conditioned changes to frozen protocol constants;
- FLAT-ATTENTION runtime/kernel changes;
- production or hardware performance claims.
