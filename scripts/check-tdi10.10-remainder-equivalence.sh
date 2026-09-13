#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_remainder_product_equivalence
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT equivalence theorem' \
    docs/tdi10/TDI-10.10-REMAINDER-PRODUCT-EQUIVALENCE.md
grep -Fq 'x <= -log(1 - x) <= x / (1 - x)' \
    docs/tdi10/TDI-10.10-REMAINDER-PRODUCT-EQUIVALENCE.md
grep -Fq 'sum_{k=1}^{infinity} (1 - alpha_k) = infinity' \
    docs/tdi10/TDI-10.10-REMAINDER-PRODUCT-EQUIVALENCE.md
grep -Fq 'sum_{k=1}^{infinity} (-log alpha_k) = infinity' \
    docs/tdi10/TDI-10.10-REMAINDER-PRODUCT-EQUIVALENCE.md
grep -Fq 'REFUTED: necessity without the hypothesis' \
    docs/tdi10/TDI-10.10-REMAINDER-PRODUCT-EQUIVALENCE.md
grep -Fq 'REFUTED: `alpha_k -> 1` is required for the equivalence' \
    docs/tdi10/TDI-10.10-REMAINDER-PRODUCT-EQUIVALENCE.md
grep -Fq 'Witness consistency (Types S / U / D)' \
    docs/tdi10/TDI-10.10-REMAINDER-PRODUCT-EQUIVALENCE.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|weakens TDI-10.7' \
    docs/tdi10/TDI-10.10-REMAINDER-PRODUCT-EQUIVALENCE.md \
    tdi-operator/tests/cavity_remainder_product_equivalence.rs; then
  echo 'TDI-10.10 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
