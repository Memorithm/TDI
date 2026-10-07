# TDI-21 Pascal P4 Development evidence

Status: frozen non-final Development evidence. Validation has not been executed in
this evidence freeze. No confirmatory, protected, or final execution is
authorized.

## Frozen identities

- frozen implementation/case-plan commit:
  `4b6a2675c7e34ca60e673920f96feec803e08b30`
- P4 preregistration SHA-256:
  `22fc406f4c81488062cb6bfc7c088f50df82b7821cde002d2cda0b2e46ba7df1`
- frozen case-plan SHA-256:
  `adf6564d2851112ce418a5366bacf3a0804ac62eceebc7b1a94a5f0f31637680`
- canonical Development TSV SHA-256:
  `274d8c4f226986ac0e5136c4bcc8e2a81809c279b78c9b20fb5ba279d82820b3`

Before Development execution, every path in
`results/tdi21_pascal_p4/implementation.sha256` was reverified successfully on
the exact frozen checkout. The P4 preflight remained 4/4 passing, the focused
harness tests remained 2/2 passing, `cargo fmt --all -- --check` passed, and
`cargo clippy -p tdi-ai --all-targets -- -D warnings` passed.

## Execution

Only the preregistered Development executable was run:

```text
cargo run -q -p tdi-ai --release --example tdi21_pascal_p4_development
```

The exact frozen executable was run twice. Both canonical byte streams were
identical and both had SHA-256
`274d8c4f226986ac0e5136c4bcc8e2a81809c279b78c9b20fb5ba279d82820b3`.

## Exact Development results

- cells: 108
- Pascal mismatch sum: 0
- generic-control mismatch sum: 0
- equal Pascal/direct/generic checksums: 108/108
- pairwise-token-comparison sum: 0
- `delta_direct`: 45 positive, 0 null, 63 negative
- `delta_generic`: 108 positive, 0 null, 0 negative
- `delta_direct` range: -12384 to 24590
- `delta_generic` range: 27648 to 2072576

The direct-control boundary is preserved rather than filtered:

- LOW region: 36 negative, 0 null, 0 positive
- CROSS region: 27 negative, 0 null, 9 positive
- HIGH region: 0 negative, 0 null, 36 positive
- n=10: 33 negative, 0 null, 21 positive
- n=12: 30 negative, 0 null, 24 positive
- reuse=1: 30 negative, 0 null, 24 positive
- reuse=8: 33 negative, 0 null, 21 positive

All 108 signed contrasts and exact matched-arm outputs are retained in
`results/tdi21_pascal_p4/development.tsv`.

## Gate interpretation

Development satisfies the non-final software/evidence gates G1-G5 on the frozen
implementation: collision-free declared generators, exact matched outputs,
frozen work accounting, zero pairwise-token operations, and complete retention
of positive/null/negative signed evidence.

This evidence does not satisfy G6 by itself: Validation may start only after this
Development evidence is committed and the protocol, case geometry, and
implementation identities remain unchanged.

P4 is a crossover-boundary experiment under the frozen primitive work proxy. It
does not establish wall-clock latency, throughput, energy, memory efficiency, or
model quality. SML-GENIUS remains the owner of model-side Pascal primitives.
Nothing here authorizes promotion into SBG, MOR, Delta-KV, context memory, or
model architecture.
