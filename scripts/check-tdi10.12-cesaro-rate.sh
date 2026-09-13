#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_cesaro_mean_remainder_rate
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — positive Cesàro liminf implies exponential rate' \
    docs/tdi10/TDI-10.12-CESARO-MEAN-REMAINDER-RATE.md
grep -Fq 'product_{k=1}^n alpha_k <= exp( - (lambda - epsilon) n )' \
    docs/tdi10/TDI-10.12-CESARO-MEAN-REMAINDER-RATE.md
grep -Fq 'Alternating blocks (liminf strictly positive, not constant)' \
    docs/tdi10/TDI-10.12-CESARO-MEAN-REMAINDER-RATE.md
grep -Fq 'REFUTED: positive Cesàro limit is necessary for product → 0' \
    docs/tdi10/TDI-10.12-CESARO-MEAN-REMAINDER-RATE.md
grep -Fq '(1/n) sum_{k=1}^n x_k -> 0' \
    docs/tdi10/TDI-10.12-CESARO-MEAN-REMAINDER-RATE.md
grep -Fq 'EXACT claim 2 — sufficiency for exponential rate (stronger than → 0)' \
    docs/tdi10/TDI-10.12-CESARO-MEAN-REMAINDER-RATE.md
grep -Fq 'elementary subunit-product arc' \
    docs/tdi10/TDI-10.12-CESARO-MEAN-REMAINDER-RATE.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|weakens TDI-10.10' \
    docs/tdi10/TDI-10.12-CESARO-MEAN-REMAINDER-RATE.md \
    tdi-operator/tests/cavity_cesaro_mean_remainder_rate.rs; then
  echo 'TDI-10.12 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
