#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi25_torsor_chiral
grep -Fq 'tdi25-chiral-invariant-bridge-v1' tdi-ai/src/tdi25_torsor_chiral.rs
grep -Fq '| 07 | Chiral invariant bridge tests |' docs/TDI-25-CAMPAIGN-50.md
