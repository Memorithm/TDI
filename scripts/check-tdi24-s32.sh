#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_beta_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-beta-zero-ablation-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const fn beta_zero_weights' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn score_c6_beta_zero_from_view' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_beta_zero_ablation(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_beta_zero_ablation_for_label' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_beta_zero_ablation_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub reference_beta_already_zero: bool' tdi-ai/src/tdi24_eval.rs
grep -Fq 'BetaZeroAblationInvalid' tdi-ai/src/tdi24_eval.rs
grep -Fq 'beta_zero_invalid("mirror_even_residual")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'beta_zero_invalid("degenerate_identity")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'beta_zero_invalid("degeneracy_flag_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'beta_zero_invalid("parity_odd_channel_lost")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'beta_zero_invalid("capacity_mismatch")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'beta_zero_invalid("protected_or_final_access")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn reference_side_reproduces_the_stage_c_c6_evaluator_exactly' tdi-ai/tests/tdi24_beta_zero_ablation.rs
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_beta_zero_ablation.rs
grep -Fq '| 31 | `gamma=0` ablation | **landed** in #702, exact head `56ff53b183e10f052c1d62b97c03f4991252b147`, merge `2d59303a73c39100c52b872b7f6ad61a7a1e2504`;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 32 | `beta=0` ablation | **current stacked slice**;' docs/TDI-24-CAMPAIGN-50.md
# Do not pin the global merged count here: later qualified slices must advance it.
