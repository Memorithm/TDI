#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_attention
grep -Fq "tdi24-masked-softmax-reference-v1" tdi-ai/src/tdi24_attention.rs
