# TDI-21 Pascal P3 Validation and paired campaign evidence

Status: frozen non-final Development/Validation evidence. No confirmatory,
protected, or final execution is authorized or represented here.

## Bound protocol and implementation

- Development evidence freeze commit: `6f6676cb4c184c708bf9cd10702551d3edc1fc88`
- Validation execution checkout commit: `6f6676cb4c184c708bf9cd10702551d3edc1fc88`
- frozen implementation source commit: `d698178d70544a4fef5c95f824f859c23b3b6871`
- P3 preregistration SHA-256: `4a65cc6eda2433122b178fe5774b482f1ca28c1fcc49de932efd57b2257b1f03`
- implementation-manifest SHA-256: `2c4aeda8705609717a2e923f106e5ccf8edb40fe90c2e232dd7548b56d6e6571`
- Development canonical result SHA-256: `7416e8546a0a62cd5f209b904405c65a84d14cf9082d752fec745d179bfa2449`
- Validation canonical result SHA-256: `1d998261760e603c740bc30f2007b62c1c48742df9d38d5b1e6e55681b9d5f02`
- complete 216-cell campaign TSV SHA-256: `066e875dbdece2b14e8cab601cf6cf95eb616ae7380f16ecaafd72f0d428140b`

Before Validation, every implementation path listed in
`results/tdi21_pascal_p3/implementation.sha256` was reverified with
`sha256sum -c`; all identities matched the frozen implementation. Development
evidence had already been committed before Validation was executed.

## Validation qualification and execution

The following qualification passed on the exact Validation checkout:

```text
sha256sum -c results/tdi21_pascal_p3/implementation.sha256
cargo fmt --all -- --check
cargo test -p tdi-ai --test tdi21_pascal_p3_preflight
cargo clippy -p tdi-ai --all-targets -- -D warnings
```

The preflight remained 4/4 passing. Validation then executed only the
preregistered non-final executable:

```text
cargo run -q -p tdi-ai --release --example tdi21_pascal_p3_validation
```

The already-built exact executable was run a second time. The two canonical byte
streams were identical and both had SHA-256
`1d998261760e603c740bc30f2007b62c1c48742df9d38d5b1e6e55681b9d5f02`.

## Exact Validation results

- cells: 108
- Pascal mismatch sum: 0
- generic-control mismatch sum: 0
- equal Pascal/direct/generic checksums: 108/108
- pairwise-token-comparison sum: 0
- `delta_direct`: 60 positive, 0 null, 48 negative
- `delta_generic`: 108 positive, 0 null, 0 negative
- `delta_direct` range: -114192 to 268189696
- `delta_generic` range: 208896 to 33439744

All 48 negative direct-control Validation cells are retained. They are balanced
across the three schedules: 16 affine, 16 low-weight-first, and 16
high-weight-first.

The negative Validation parameter groups are exactly:

```text
n=13 density=4   query_load=16 reuse=1 count=3
n=13 density=4   query_load=16 reuse=8 count=3
n=13 density=4   query_load=64 reuse=1 count=3
n=13 density=4   query_load=64 reuse=8 count=3
n=13 density=32  query_load=16 reuse=1 count=3
n=13 density=32  query_load=16 reuse=8 count=3
n=13 density=32  query_load=64 reuse=1 count=3
n=13 density=256 query_load=16 reuse=1 count=3
n=14 density=4   query_load=16 reuse=1 count=3
n=14 density=4   query_load=16 reuse=8 count=3
n=14 density=4   query_load=64 reuse=1 count=3
n=14 density=4   query_load=64 reuse=8 count=3
n=14 density=32  query_load=16 reuse=1 count=3
n=14 density=32  query_load=16 reuse=8 count=3
n=14 density=32  query_load=64 reuse=1 count=3
n=14 density=256 query_load=16 reuse=1 count=3
```

## Complete paired campaign

Across all 216 preregistered Development/Validation cells:

- zero Pascal or generic-control mismatches;
- all 216 matched-arm output checksums agree;
- zero pairwise-token comparisons;
- `delta_direct`: 150 positive, 0 null, 66 negative;
- `delta_generic`: 216 positive, 0 null, 0 negative.

All per-cell signed contrasts are retained in
`results/tdi21_pascal_p3/campaign.tsv`, while the Development and Validation
source artifacts remain separately retained. No positive, null, or negative
cell has been filtered.

## Gate interpretation

G1-G6 are satisfied for this non-final P3 campaign under the declared primitive
work proxy. This establishes a paired boundary map, not a wall-clock,
throughput, energy, latency, or model-quality result.

The direct-ANF control remains superior in 66/216 cells under the frozen work
proxy, so Pascal is not uniformly advantageous. The generic full-materialize
control is more expensive in all 216 cells under that same proxy.

No protected/final population was generated, opened, inspected, or executed.
SML-GENIUS remains the owner of model-side Pascal primitives. This evidence does
not authorize promotion into SBG, MOR, Delta-KV, context memory, or model
architecture.
