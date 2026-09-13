#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_remainder_rate
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT harmonic rate lemma' \
    docs/tdi10/TDI-10.9-HARMONIC-RATE-LEMMA.md
grep -Fq '1 - alpha_k >= c / k' \
    docs/tdi10/TDI-10.9-HARMONIC-RATE-LEMMA.md
grep -Fq 'Type D sufficient rate' \
    docs/tdi10/TDI-10.9-HARMONIC-RATE-LEMMA.md
grep -Fq 'REFUTED: superharmonic lower rate is sufficient for decay' \
    docs/tdi10/TDI-10.9-HARMONIC-RATE-LEMMA.md
grep -Fq 'Closed-form witness calculus' \
    docs/tdi10/TDI-10.9-HARMONIC-RATE-LEMMA.md
grep -Fq 'Sharpness of the harmonic window' \
    docs/tdi10/TDI-10.9-HARMONIC-RATE-LEMMA.md
grep -Fq 'product_{k=1}^n alpha_k = (n+2)/(2(n+1)) -> 1/2 != 0' \
    docs/tdi10/TDI-10.9-HARMONIC-RATE-LEMMA.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|replaces TDI-10.7' \
    docs/tdi10/TDI-10.9-HARMONIC-RATE-LEMMA.md \
    tdi-operator/tests/cavity_remainder_rate.rs; then
  echo 'TDI-10.9 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
