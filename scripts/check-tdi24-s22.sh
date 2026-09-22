#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-c6-evaluator-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn c6(split: DataSplit)' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn evaluate_c6_binary' tdi-ai/src/tdi24_eval.rs
grep -Fq '| 21 | V6 evaluator | **landed** in #611;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 22 | C6 evaluator | **current stacked slice**;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '**21/50 merged**' docs/TDI-24-CAMPAIGN-50.md
