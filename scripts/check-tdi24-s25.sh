#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-optimizer-update-budget-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn match_optimizer_update_budgets' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct OptimizerUpdateBudget' tdi-ai/src/tdi24_eval.rs
grep -Fq '| 24 | Initialization matcher | **landed** in #657;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 25 | Optimizer/update-budget contract | **current stacked slice**;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '**24/50 merged**' docs/TDI-24-CAMPAIGN-50.md
