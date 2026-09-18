# TDI-25 Slice 02 — Source-contract provenance pin

Status: bounded Stage-A provenance contract; non-confirmatory.

TDI-25 compares two independently defined representation hypotheses. The comparison is admissible only while the compiled upstream semantic contracts match this explicit pin.

## Pin identity

- TDI-25 source pin: `tdi25-source-contract-pin-v1`
- TDI-22 torsor source: `tdi22-torsor-dual-pairing-v1`
- TDI-24 chiral source: `tdi24-mirror-coupled-chiral-v1`

The executable pin lives in `tdi-ai/src/tdi25_torsor_chiral.rs` as `PINNED_SOURCE_CONTRACTS`. `validate_source_contracts()` compares the compiled TDI-22/TDI-24 contract constants against this pin and returns a typed `SourceContractMismatch` on drift.

## Source boundaries

The torsor identity is imported from `tdi22_torsor::TORSOR_CONTRACT`. The chiral identity is imported from `tdi24_chiral::CHIRAL_CONTRACT`. TDI-25 does not duplicate either algebra.

A future upstream semantic revision must receive a new upstream contract identity and a dedicated TDI-25 pin update before later comparison slices may consume it. Silent acceptance of a changed contract is forbidden.

This pin records semantic contract identities. Run-level code commit, configuration, data, seed and toolchain provenance remain separate later-campaign responsibilities.

## Scientific boundary

This tranche does not train or evaluate a model, access protected/final populations, change either upstream algebra, select a winner, or create a torsor-plus-chiral hybrid.
