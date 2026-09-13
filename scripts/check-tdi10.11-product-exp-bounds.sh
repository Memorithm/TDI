#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_product_exp_bounds
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — product upper bound' \
    docs/tdi10/TDI-10.11-PRODUCT-EXP-BOUNDS.md
grep -Fq 'product_{k=1}^n alpha_k <= exp( - sum_{k=1}^n x_k )' \
    docs/tdi10/TDI-10.11-PRODUCT-EXP-BOUNDS.md
grep -Fq 'Declared cutoff' \
    docs/tdi10/TDI-10.11-PRODUCT-EXP-BOUNDS.md
grep -Fq 'exp( - C_K - 2 sum_{k=K}^n x_k )' \
    docs/tdi10/TDI-10.11-PRODUCT-EXP-BOUNDS.md
grep -Fq 'EXACT claim 3 — Type D sandwich identities' \
    docs/tdi10/TDI-10.11-PRODUCT-EXP-BOUNDS.md
grep -Fq 'exp( - 2 (H_{n+1} - 1) ) <= 1/(n+1) <= exp( -(H_{n+1} - 1) )' \
    docs/tdi10/TDI-10.11-PRODUCT-EXP-BOUNDS.md
grep -Fq 'REFUTED: product upper bound is sharp as equality for all sequences' \
    docs/tdi10/TDI-10.11-PRODUCT-EXP-BOUNDS.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|weakens TDI-10.10' \
    docs/tdi10/TDI-10.11-PRODUCT-EXP-BOUNDS.md \
    tdi-operator/tests/cavity_product_exp_bounds.rs; then
  echo 'TDI-10.11 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
