#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_operator_families
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — named families realize the trichotomy' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md
grep -Fq 'Family `F_S` — Type S' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md
grep -Fq 'Family `F_U` — Type U' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md
grep -Fq 'Family `F_D` — Type D' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md
grep -Fq 'FrozenToeplitzCavity::contraction' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md
grep -Fq 'REFUTED: every cavity family forces product decay' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md
grep -Fq 'FORMAL pointer — future slowly-varying Jacobi hypothesis' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md
grep -Fq 'EXACT claim 2 — classification, not slowly-varying asymptotics' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md
grep -Fq 'Operator-family interface (EXACT bookkeeping)' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves slowly-varying' \
    docs/tdi10/TDI-10.13-OPERATOR-FAMILY-WITNESSES.md \
    tdi-operator/tests/cavity_operator_families.rs; then
  echo 'TDI-10.13 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
