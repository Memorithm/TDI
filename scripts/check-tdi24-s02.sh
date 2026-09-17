#!/usr/bin/env bash
set -euo pipefail
export CARGO_INCREMENTAL=0
cargo test --locked -p tdi-ai --features experimental tdi24_chiral::tests:: -- --nocapture
cargo clippy --locked -p tdi-ai --features experimental --all-targets -- -D warnings
