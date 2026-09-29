#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
grep -Fq 'tdi25-t6-evaluator-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-evaluator-envelope-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-readout-budget-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn is_canonical_torsor_case' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn is_canonical_mixed_case' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub mod tdi25_eval;' tdi-ai/src/experimental.rs
grep -Fq '| 20 | Stage-B leakage/balance audit | **landed** in #660;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 21 | T6 evaluator | **current candidate** in #661;' docs/TDI-25-CAMPAIGN-50.md
