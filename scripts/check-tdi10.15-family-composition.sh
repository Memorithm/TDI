#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_family_composition
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — F_U then F_D product identity' \
    docs/tdi10/TDI-10.15-FAMILY-COMPOSITION.md
grep -Fq 'EXACT claim 2 — F_U then F_S product identity' \
    docs/tdi10/TDI-10.15-FAMILY-COMPOSITION.md
grep -Fq 'P_{U→D}(m,n;ρ) = ρ^m · 1/(n+1)' \
    docs/tdi10/TDI-10.15-FAMILY-COMPOSITION.md
grep -Fq 'P_{U→S}(m,n;ρ) = ρ^m · (n+2)/(2(n+1))' \
    docs/tdi10/TDI-10.15-FAMILY-COMPOSITION.md
grep -Fq 'FrozenToeplitzCavity::contraction' \
    docs/tdi10/TDI-10.15-FAMILY-COMPOSITION.md
grep -Fq 'REFUTED: Type-S suffix erases Type-U prefix decay uniformly in m' \
    docs/tdi10/TDI-10.15-FAMILY-COMPOSITION.md
grep -Fq 'indices restart at 1' \
    docs/tdi10/TDI-10.15-FAMILY-COMPOSITION.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves slowly-varying' \
    docs/tdi10/TDI-10.15-FAMILY-COMPOSITION.md \
    tdi-operator/tests/cavity_family_composition.rs; then
  echo 'TDI-10.15 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
