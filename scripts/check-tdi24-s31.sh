#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-gamma-zero-ablation-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const C6_REFERENCE_WEIGHTS: ChiralScoreWeights' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const fn gamma_zero_weights' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn score_c6_gamma_zero_from_view' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_gamma_zero_ablation(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_gamma_zero_ablation_for_label' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_gamma_zero_ablation_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'GammaZeroAblationInvalid' tdi-ai/src/tdi24_eval.rs
grep -Fq 'gamma_zero_invalid("parity_odd_residual")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'gamma_zero_invalid("enantiomorphic_split")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'gamma_zero_invalid("capacity_mismatch")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'gamma_zero_invalid("protected_or_final_access")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn reference_side_reproduces_the_stage_c_c6_evaluator_exactly' tdi-ai/tests/tdi24_gamma_zero_ablation.rs
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_gamma_zero_ablation.rs
grep -Fq '| 30 | Stage-C preflight | **landed** in #698, exact head `586de6338a486bb9cbef3183f2be8faf2450d75f`, merge `60bb25afa271079181df17ca6a4f1201b291c5b5`;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 31 | `gamma=0` ablation | **current stacked slice**;' docs/TDI-24-CAMPAIGN-50.md
# Do not pin the global merged count here: later qualified slices must advance it.
