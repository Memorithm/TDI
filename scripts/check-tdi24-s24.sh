#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-initialization-matcher-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn match_initialization' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct InitializationPolicy' tdi-ai/src/tdi24_eval.rs
grep -Fq '| 23 | Parameter-count matcher | **landed** in #655;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 24 | Initialization matcher | **landed** in #657;' docs/TDI-24-CAMPAIGN-50.md
