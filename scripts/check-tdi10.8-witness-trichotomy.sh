#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_witness_trichotomy
cargo +1.97.1 test -p tdi-operator

grep -Fq 'Type S — summable remainder' \
    docs/tdi10/TDI-10.8-WITNESS-TRICHOTOMY.md
grep -Fq 'Type U — uniform geometric bound' \
    docs/tdi10/TDI-10.8-WITNESS-TRICHOTOMY.md
grep -Fq 'Type D — divergent remainder' \
    docs/tdi10/TDI-10.8-WITNESS-TRICHOTOMY.md
grep -Fq 'EXACT classification of witness families' \
    docs/tdi10/TDI-10.8-WITNESS-TRICHOTOMY.md
grep -Fq 'REFUTED: alpha_k -> 1 implies Type S' \
    docs/tdi10/TDI-10.8-WITNESS-TRICHOTOMY.md
grep -Fq 'REFUTED: divergent remainder is automatic for every cavity family' \
    docs/tdi10/TDI-10.8-WITNESS-TRICHOTOMY.md
grep -Fq 'Toeplitz / constant-coefficient cavity' \
    docs/tdi10/TDI-10.8-WITNESS-TRICHOTOMY.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|introduces a new sufficient decay criterion' \
    docs/tdi10/TDI-10.8-WITNESS-TRICHOTOMY.md \
    tdi-operator/tests/cavity_witness_trichotomy.rs; then
  echo 'TDI-10.8 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
