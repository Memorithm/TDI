#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
grep -Fq 'tdi25-optimizer-update-budget-matcher-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn match_optimizer_update_budgets' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct OptimizerUpdateBudget' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct MatchedOptimizerUpdateBudget' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub enum StoppingRule' tdi-ai/src/tdi25_eval.rs
grep -Fq 'OptimizerUpdateBudgetMismatch' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-parameter-readout-matcher-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub mod tdi25_eval;' tdi-ai/src/experimental.rs
grep -Fq '| 24 | Parameter/readout matcher | **landed** in #669, exact head `69f5ec9c27ceeadd047fd196193131964b7a0023`, merge `32c02cb7625dae788dc5d44d4449447a408f0589`;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 25 | Optimizer/update-budget matcher | **current candidate** in #' docs/TDI-25-CAMPAIGN-50.md
