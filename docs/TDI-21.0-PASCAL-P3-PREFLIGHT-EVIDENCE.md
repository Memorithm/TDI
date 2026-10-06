# TDI-21 Pascal P3 protocol preflight evidence

Status: non-final protocol/implementation preflight only. No Pascal P3 scientific Development or Validation cell was executed.

## Bound inputs

Execution source commit:

`c6fda5ec3828578d194411b16643457d081fcf2e`

P3 preregistration SHA-256:

`4a65cc6eda2433122b178fe5774b482f1ca28c1fcc49de932efd57b2257b1f03`

P3 preflight test source SHA-256:

`84b85ff670734e81320aed08c5e78da415e5d8db36d4a6e4a634f0750d311f2a`

Commands:

```text
cargo fmt --all -- --check
cargo test --manifest-path tdi-ai/Cargo.toml --locked --test tdi21_pascal_p3_preflight -- --nocapture
```

## Exact preflight result

The formatting qualification passed with no diff. Four preflight tests passed:

```text
p3_frozen_geometry_is_exactly_216_nonfinal_cells ... ok
p3_work_accounting_preserves_positive_null_and_negative_deltas ... ok
p3_work_sums_fail_closed_on_overflow ... ok
p3_generator_is_coprime_and_unique_for_every_declared_cell_shape ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

The preflight establishes only that the prospectively frozen P3 generator step is coprime with every declared `2^n-1`, generates the declared number of unique masks for all frozen width/density/gate shapes, that the frozen grid remains 216 non-final cells, and that the declared signed work-accounting arithmetic preserves positive, null, and negative values without unsigned-wrap semantics.

The source was requalified after a rustfmt-only normalization. This qualification is not a scientific replication and does not change the preregistered design.

It does not establish any Pascal utility result. It does not execute the Development or Validation scientific matrix. No protected/final material was opened, generated, inspected, or executed.

SML-GENIUS remains the owner of model-side Pascal primitives. This preflight authorizes no SBG, MOR, Delta-KV, context-memory, or model-architecture promotion.
