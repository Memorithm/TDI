#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_contraction_monotonicity
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — domain calculus for κ(a,b)' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md
grep -Fq 'EXACT claim 2 — monotonicity in the radial coordinate' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md
grep -Fq 'EXACT claim 3 — TDI-10.3 factorization at the matched frozen point is Type-U ρ' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md
grep -Fq 'κ(a,b) = (1 - s)/(1 + s)' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md
grep -Fq 'reconstructed_transport_factor = step.transport_factor() = κ' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md
grep -Fq 'REFUTED: κ → 0 as the symbol approaches the positivity boundary' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md
grep -Fq 'REFUTED: increasing the diagonal alone forces smaller κ without fixing |b|' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md
grep -Fq 'r(a,b) := 2|b|/a' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves slowly-varying' \
    docs/tdi10/TDI-10.16-CONTRACTION-MONOTONICITY.md \
    tdi-operator/tests/cavity_contraction_monotonicity.rs; then
  echo 'TDI-10.16 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
