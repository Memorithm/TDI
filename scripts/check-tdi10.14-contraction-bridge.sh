#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_contraction_type_u_bridge
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — contraction coefficient is Type-U ρ' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md
grep -Fq 'EXACT claim 2 — dual-path finite-n product identity' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md
grep -Fq 'FrozenToeplitzCavity::contraction' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md
grep -Fq 'REFUTED: every frozen Toeplitz symbol yields Type D' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md
grep -Fq 'Equality saturation' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md
grep -Fq 'Inequality envelope' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md
grep -Fq 'must agree on `ρ^n`' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md
grep -Fq 'Boundary case' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves slowly-varying' \
    docs/tdi10/TDI-10.14-CONTRACTION-TYPE-U-BRIDGE.md \
    tdi-operator/tests/cavity_contraction_type_u_bridge.rs; then
  echo 'TDI-10.14 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
