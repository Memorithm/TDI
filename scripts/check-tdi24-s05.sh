#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_chiral
grep -Fq "tdi24-enantiomorphic-score-pair-v1" tdi-ai/src/tdi24_chiral.rs
grep -Fq "| 04 | Channel decomposition contract | **landed** in #492;" docs/TDI-24-CAMPAIGN-50.md
grep -Fq "| 05 | R/L enantiomorphic score pair | **current slice**;" docs/TDI-24-CAMPAIGN-50.md
