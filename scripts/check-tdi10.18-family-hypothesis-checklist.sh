#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_family_hypothesis_checklist
cargo +1.97.1 test -p tdi-operator

grep -Fq 'Item D — harmonic remainder lower bound (cites TDI-10.9)' \
    docs/tdi10/TDI-10.18-FAMILY-HYPOTHESIS-CHECKLIST.md
grep -Fq 'EXACT claim 1 — Item D classifies via TDI-10.9' \
    docs/tdi10/TDI-10.18-FAMILY-HYPOTHESIS-CHECKLIST.md
grep -Fq 'EXACT claim 2 — TDI-10.2 one-step identity on family-realized steps' \
    docs/tdi10/TDI-10.18-FAMILY-HYPOTHESIS-CHECKLIST.md
grep -Fq 'EXACT claim 3 — checklist is finite; FORMAL pointer remains open' \
    docs/tdi10/TDI-10.18-FAMILY-HYPOTHESIS-CHECKLIST.md
grep -Fq 'REFUTED: informal `alpha_k → 1` / “slow variation” alone meets Item D' \
    docs/tdi10/TDI-10.18-FAMILY-HYPOTHESIS-CHECKLIST.md
grep -Fq 'REFUTED: finite-window harmonic lower bound alone forces product → 0' \
    docs/tdi10/TDI-10.18-FAMILY-HYPOTHESIS-CHECKLIST.md
grep -Fq '1 - alpha_k ≥ L / k' \
    docs/tdi10/TDI-10.18-FAMILY-HYPOTHESIS-CHECKLIST.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves slowly-varying' \
    docs/tdi10/TDI-10.18-FAMILY-HYPOTHESIS-CHECKLIST.md \
    tdi-operator/tests/cavity_family_hypothesis_checklist.rs; then
  echo 'TDI-10.18 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi
