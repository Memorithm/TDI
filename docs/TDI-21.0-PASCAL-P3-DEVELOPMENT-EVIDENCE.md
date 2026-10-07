# TDI-21 Pascal P3 Development evidence

Status: frozen non-final Development evidence only. Validation, confirmatory, protected, and final execution are not part of this dossier.

## Bound source and protocol

- execution source commit: `d698178d70544a4fef5c95f824f859c23b3b6871`
- execution source tree: `13b807f5295ee7fff018deeedfa348cf1e8e3ab4`
- P3 preregistration SHA-256: `4a65cc6eda2433122b178fe5774b482f1ca28c1fcc49de932efd57b2257b1f03`
- implementation-manifest SHA-256: `2c4aeda8705609717a2e923f106e5ccf8edb40fe90c2e232dd7548b56d6e6571`
- canonical Development result SHA-256: `7416e8546a0a62cd5f209b904405c65a84d14cf9082d752fec745d179bfa2449`

The complete per-file implementation identities are retained in
`results/tdi21_pascal_p3/implementation.sha256`. The complete canonical
Development bytes are retained in `results/tdi21_pascal_p3/development.tsv`.

## Qualification before execution

On the exact implementation tree used for Development:

```text
cargo fmt --all -- --check
cargo test -p tdi-ai --test tdi21_pascal_p3_preflight
cargo clippy -p tdi-ai --all-targets -- -D warnings
```

All commands completed successfully. The P3 preflight remained 4/4 passing.

## Development execution

Only the preregistered Development executable was run:

```text
cargo run -p tdi-ai --release --example tdi21_pascal_p3_development
```

The canonical output contains one header plus 108 Development cells. It was
executed a second time with the already-built exact executable. The two byte
streams were identical and both had SHA-256
`7416e8546a0a62cd5f209b904405c65a84d14cf9082d752fec745d179bfa2449`.

No Validation executable was run.

## Exact Development results

- cells: 108
- Pascal mismatch sum: 0
- generic-control mismatch sum: 0
- cells with equal Pascal/direct/generic checksums: 108/108
- pairwise-token-comparison sum: 0
- `delta_direct`: 90 positive, 0 null, 18 negative
- `delta_generic`: 108 positive, 0 null, 0 negative
- `delta_direct` range: -10768 to 33526784
- `delta_generic` range: 14080 to 4183040

The 18 negative direct-control cells are retained rather than filtered. Their
parameter groups, with all three schedules represented in each listed group,
are:

```text
n=9  density=4  query_load=16 reuse=1  count=3
n=9  density=4  query_load=64 reuse=1  count=3
n=11 density=4  query_load=16 reuse=1  count=3
n=11 density=4  query_load=16 reuse=8  count=3
n=11 density=4  query_load=64 reuse=1  count=3
n=11 density=32 query_load=16 reuse=1  count=3
```

Schedule distribution among negative direct-control cells is exactly six
`affine`, six `low_weight_first`, and six `high_weight_first` cases.

## Development gate interpretation

The Development evidence supports the preregistered exactness, accounting, and
TDI-21 prohibition checks for the executed non-final cells. It also preserves a
material negative boundary against the direct ANF control. It does not establish
Validation generalization, model utility, wall-clock speed, GPU throughput,
energy, or latency.

Development evidence is now frozen before any Validation execution. Any later
Validation step must retain the same P3 protocol, generator, accounting,
implementation identities, and 108-cell frozen Validation geometry.

SML-GENIUS remains the owner of model-side Pascal primitives. Nothing in this
dossier authorizes promotion into SBG, MOR, Delta-KV, context memory, or model
architecture.
