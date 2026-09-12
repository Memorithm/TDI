#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_uniform_subunit_decay
cargo +1.97.1 test -p tdi-operator

grep -Fq '0 < alpha_k <= rho' docs/tdi10/TDI-10.6-UNIFORM-SUBUNIT-DECAY.md
grep -Fq 'product_{k=1}^n alpha_k <= rho^n' docs/tdi10/TDI-10.6-UNIFORM-SUBUNIT-DECAY.md
grep -Fq 'EXACT algebraic companion' docs/tdi10/TDI-10.6-UNIFORM-SUBUNIT-DECAY.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves decay for the TDI-10\.5' \
    docs/tdi10/TDI-10.6-UNIFORM-SUBUNIT-DECAY.md \
    tdi-operator/tests/cavity_uniform_subunit_decay.rs; then
  echo 'TDI-10.6 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
