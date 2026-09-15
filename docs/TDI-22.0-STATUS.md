# TDI-22.0 — Status

Status: **active Stage-0 bootstrap; not frozen; no confirmatory execution authorised**.

## Implemented on the current development branch

- `docs/TDI-22-PROGRAMME.md` defines the research question, exact sign convention, candidate/control ladder and FLAT boundary.
- `docs/TDI-22.0-SCOPE.md` limits Stage 0 to deterministic algebraic scaffolding.
- `tdi-ai/src/tdi22_torsor.rs` implements the finite 3D torsor/twist contract behind the existing `experimental` feature.
- Unit tests cover reduction-point transport, declared invariants, factorized-key independence, direct/factorized pairing equivalence, global-origin translation consistency and non-finite rejection.
- `scripts/check-tdi22-bootstrap.sh` is the local bootstrap gate.

## Not implemented / not authorised

- no T0/T1/T2/T3/T4 attention evaluator;
- no learned geometry;
- no model-quality experiment;
- no final or confirmatory population;
- no paged/hierarchical torsor memory;
- no Boolean × torsor experiment;
- no FLAT-ATTENTION integration;
- no hardware-performance claim.

## Next gate

The immediate acceptance target is green CI for the Stage-0 algebra and bootstrap integrity. A later reviewed change must explicitly freeze TDI-22.0 before TDI-22.1 can preregister the first matched attention experiment.
