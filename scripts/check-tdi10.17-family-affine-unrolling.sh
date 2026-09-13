#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_family_affine_unrolling
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — zero-drift family chains collapse 10.4 onto the product' \
    docs/tdi10/TDI-10.17-FAMILY-AFFINE-UNROLLING.md
grep -Fq 'EXACT claim 2 — constant-drift F_U geometric unrolling' \
    docs/tdi10/TDI-10.17-FAMILY-AFFINE-UNROLLING.md
grep -Fq 'EXACT claim 3 — constant-drift F_D accumulated-drift closed form' \
    docs/tdi10/TDI-10.17-FAMILY-AFFINE-UNROLLING.md
grep -Fq 'B_n = δ · (1 - ρ^n) / (1 - ρ)' \
    docs/tdi10/TDI-10.17-FAMILY-AFFINE-UNROLLING.md
grep -Fq 'REFUTED: Type-U product decay alone forces cavity-error → 0 under drift' \
    docs/tdi10/TDI-10.17-FAMILY-AFFINE-UNROLLING.md
grep -Fq 'lim_{n→∞} E_n = δ / (1 - ρ) ≠ 0' \
    docs/tdi10/TDI-10.17-FAMILY-AFFINE-UNROLLING.md
grep -Fq 'E_n = A_n E_0 + B_n' \
    docs/tdi10/TDI-10.17-FAMILY-AFFINE-UNROLLING.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves slowly-varying' \
    docs/tdi10/TDI-10.17-FAMILY-AFFINE-UNROLLING.md \
    tdi-operator/tests/cavity_family_affine_unrolling.rs; then
  echo 'TDI-10.17 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
