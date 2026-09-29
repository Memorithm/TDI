#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
grep -Fq 'tdi25-g6-evaluator-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-c6-evaluator-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-t6-evaluator-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-evaluator-envelope-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-readout-budget-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-generic6-control-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn score_neutral_control' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn is_canonical_neutral_case' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct G6EvaluatorRun' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn evaluate_neutral_control' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub mod tdi25_eval;' tdi-ai/src/experimental.rs
grep -Fq '| 22 | C6 evaluator | **landed** in #663, exact head `25ff8e020a0c784d3a52d021fb13d9fcc986702f`, merge `ae501e9b3941a293c3cb18f3450c3c077b25f48f`;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 23 | G6 evaluator | **landed** in #666, exact head `3403b8768d24ac1543b75120d15a09dd5b58c50e`, merge `d6c8226025de1dbe4848f83d8092c316066f97b1`;' docs/TDI-25-CAMPAIGN-50.md
