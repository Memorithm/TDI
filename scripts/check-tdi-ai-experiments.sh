#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all -- --check
cargo clippy -p tdi-ai --all-targets --all-features -- -D warnings
cargo test -p tdi-ai --all-features
bash scripts/check-tdi9.1-reference-rejections.sh
bash scripts/check-tdi11-bootstrap.sh
