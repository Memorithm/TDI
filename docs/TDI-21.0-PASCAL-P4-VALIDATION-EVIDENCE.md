# TDI-21 Pascal P4 Validation and complete crossover evidence

Status: frozen non-final Development/Validation evidence. No confirmatory,
protected, or final execution is authorized or represented here.

## Bound identities

- frozen implementation/case-plan commit:
  `4b6a2675c7e34ca60e673920f96feec803e08b30`
- committed Development evidence checkout used for Validation:
  `56c1bbaf4fb4051af077c6236114c14c7e4bd031`
- P4 preregistration SHA-256:
  `22fc406f4c81488062cb6bfc7c088f50df82b7821cde002d2cda0b2e46ba7df1`
- frozen case-plan SHA-256:
  `adf6564d2851112ce418a5366bacf3a0804ac62eceebc7b1a94a5f0f31637680`
- Development canonical TSV SHA-256:
  `274d8c4f226986ac0e5136c4bcc8e2a81809c279b78c9b20fb5ba279d82820b3`
- Validation canonical TSV SHA-256:
  `1ff561cb278850c7aeff8d900eff74ba77904789a6b63e51afab500c18145e3e`
- complete 162-cell campaign TSV SHA-256:
  `64baab03421de71ac0635a91d02f5bbe219b21d69477de3ebb4db59464eda712`

Before Validation, every path in
`results/tdi21_pascal_p4/implementation.sha256` was reverified successfully,
the case-plan and committed Development result hashes matched exactly,
`cargo fmt --all -- --check` passed, the P4 preflight remained 4/4 passing,
the focused harness remained 2/2 passing, and clippy passed with warnings denied.

## Validation execution

Only the preregistered non-final Validation executable was run:

```text
cargo run -q -p tdi-ai --release --example tdi21_pascal_p4_validation
```

The exact frozen executable was run twice. The canonical byte streams were
identical and both had SHA-256
`1ff561cb278850c7aeff8d900eff74ba77904789a6b63e51afab500c18145e3e`.

## Exact Validation results

- cells: 54
- Pascal mismatch sum: 0
- generic-control mismatch sum: 0
- equal Pascal/direct/generic checksums: 54/54
- pairwise-token-comparison sum: 0
- `delta_direct`: 30 positive, 0 null, 24 negative
- `delta_generic`: 54 positive, 0 null, 0 negative
- `delta_direct` range: -123120 to 245984
- `delta_generic` range: 802816 to 16531456

The direct-control boundary remains visible in Validation:

- LOW region: 18 negative, 0 null, 0 positive
- CROSS region: 6 negative, 0 null, 12 positive
- HIGH region: 0 negative, 0 null, 18 positive
- density 4: 6 negative, 0 null, 12 positive
- density 16: 9 negative, 0 null, 9 positive
- density 64: 9 negative, 0 null, 9 positive
- reuse=1: 12 negative, 0 null, 15 positive
- reuse=8: 12 negative, 0 null, 15 positive

All signed contrasts are retained in
`results/tdi21_pascal_p4/validation.tsv`.

## Complete paired campaign

Across all 162 preregistered Development/Validation cells:

- zero Pascal or generic-control mismatches;
- all 162 matched-arm output checksums agree;
- zero pairwise-token comparisons;
- `delta_direct`: 75 positive, 0 null, 87 negative;
- `delta_generic`: 162 positive, 0 null, 0 negative;
- `delta_direct` range: -123120 to 245984;
- `delta_generic` range: 27648 to 16531456.

The complete signed dossier is retained in
`results/tdi21_pascal_p4/campaign.tsv`; no positive, null, or negative cell was
filtered.

## Interpretation boundary

G1-G6 are satisfied for this non-final P4 crossover campaign under the frozen
primitive work proxy. The preregistered crossover pattern is directional rather
than uniformly favorable: the direct ANF control remains cheaper in 87/162
cells, while Pascal is cheaper in 75/162 cells. The generic full-materialize
control is more expensive in all 162 cells under the same accounting proxy.

This is not wall-clock, throughput, energy, memory-efficiency, or model-quality
evidence. No protected/final population was generated, opened, inspected, or
executed. SML-GENIUS remains the owner of model-side Pascal primitives. Nothing
in P4 authorizes promotion into SBG, MOR, Delta-KV, context memory, or model
architecture.
