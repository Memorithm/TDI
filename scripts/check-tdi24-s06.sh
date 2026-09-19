#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_chiral
grep -Fq "tdi24-parity-recombination-v1" tdi-ai/src/tdi24_chiral.rs
grep -Fq "| 05 | R/L enantiomorphic score pair | **landed** in #548;" docs/TDI-24-CAMPAIGN-50.md
grep -Fq "| 06 | Even/odd attention recombination contract | **current slice**;" docs/TDI-24-CAMPAIGN-50.md
