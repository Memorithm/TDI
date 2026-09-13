#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_family_reverse_cross_composition
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — F_D then F_U product identity' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'EXACT claim 2 — F_S then F_U product identity' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'EXACT claim 3 — F_D then F_S product identity' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'EXACT claim 4 — F_S then F_D product identity' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'P_{D→U}(m,n;ρ) = 1/(m+1) · ρ^n' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'P_{S→U}(m,n;ρ) = (m+2)/(2(m+1)) · ρ^n' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'P_{D→S}(m,n) = 1/(m+1) · (n+2)/(2(n+1))' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'P_{S→D}(m,n) = (m+2)/(2(m+1)) · 1/(n+1)' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'FrozenToeplitzCavity::contraction' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'REFUTED: Type-S prefix blocks Type-D suffix decay' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'REFUTED: finite Type-D prefix forces composed liminf 0 under Type-S suffix' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'REFUTED: F_D / F_S block order is immaterial' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md
grep -Fq 'indices restart at 1' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves slowly-varying' \
    docs/tdi10/TDI-10.19-FAMILY-REVERSE-CROSS-COMPOSITION.md \
    tdi-operator/tests/cavity_family_reverse_cross_composition.rs; then
  echo 'TDI-10.19 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi

printf 'TDI-10.19 reverse/cross family composition: PASS\n'
