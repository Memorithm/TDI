#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-v6-evaluator-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'tdi24-evaluator-envelope-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'tdi24-readout-budget-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub mod tdi24_eval;' tdi-ai/src/experimental.rs
grep -Fq '| 20 | Stage-B data audit/freeze | **landed** in #610; leakage/adversarial audit green before training/evaluation |' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 21 | V6 evaluator | **current stacked slice**; deterministic non-final Development/Validation path |' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '**20/50 merged**' docs/TDI-24-CAMPAIGN-50.md
