#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_chain
cargo +1.97.1 test -p tdi-operator

grep -q 'EXACT finite algebra' docs/tdi10/TDI-10.4-CHAIN-UNROLLING.md
grep -q 'uniform contraction or boundary-error decay' docs/tdi10/TDI-10.4-CHAIN-UNROLLING.md
grep -q 'No factor, product, boundary weight, or' tdi-operator/src/chain.rs

if grep -Eiq 'uniform[_ -]?contraction[_ -]?(holds|proved|theorem)|guaranteed[_ -]?decay|soft[_ -]?edge[_ -]?(theorem|uniformity)[_ -]?(holds|proved)' \
    tdi-operator/src/chain.rs \
    tdi-operator/tests/cavity_chain.rs \
    docs/tdi10/TDI-10.4-CHAIN-UNROLLING.md; then
  echo 'TDI-10.4 scientific-boundary gate rejected an unsupported bound/theorem claim' >&2
  exit 1
fi
