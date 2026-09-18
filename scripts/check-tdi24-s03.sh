#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_vector

grep -Fq 'pub const VECTOR6_CONTRACT: &str = "tdi24-matched-vector6-v1";' tdi-ai/src/tdi24_vector.rs
grep -Fq 'pub const VECTOR6_WIDTH: usize = 6;' tdi-ai/src/tdi24_vector.rs
grep -Fq 'pub mod tdi24_vector;' tdi-ai/src/experimental.rs
grep -Fq '| 02 | Arithmetic hardening | **landed** in #471;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 03 | Matched V6 vector reference | **current slice**;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '**2/50 merged** (#406, #471).' docs/TDI-24-CAMPAIGN-50.md
