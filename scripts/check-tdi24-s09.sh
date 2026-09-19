#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_accounting
grep -Fq 'tdi24-reference-accounting-v3' tdi-ai/src/tdi24_accounting.rs
grep -Fq '| 09 | Operation + storage accounting |' docs/TDI-24-CAMPAIGN-50.md
