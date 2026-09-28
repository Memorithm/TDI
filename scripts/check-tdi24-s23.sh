#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-parameter-count-matcher-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn match_parameter_counts' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct TrainableCapacity' tdi-ai/src/tdi24_eval.rs
grep -Fq '| 22 | C6 evaluator | **landed** in #615;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 23 | Parameter-count matcher | **current stacked slice**;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '**22/50 merged**' docs/TDI-24-CAMPAIGN-50.md
