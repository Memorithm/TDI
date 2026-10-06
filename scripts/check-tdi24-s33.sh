#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_vector
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_direct_only_collapse
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_beta_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-direct-only-collapse-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const DIRECT_ONLY_V6_MATCH_TOLERANCE: f64 = 0.0;' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const fn direct_only_weights' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn score_c6_direct_only_from_view' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_direct_only_collapse(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_direct_only_collapse_for_label' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_direct_only_collapse_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub collapse_coincides_with_gamma_zero: bool' tdi-ai/src/tdi24_eval.rs
grep -Fq 'DirectOnlyCollapseInvalid' tdi-ai/src/tdi24_eval.rs
grep -Fq 'direct_only_invalid("removed_channel_residual")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'direct_only_invalid("v6_score_mismatch")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'direct_only_invalid("tolerance_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'direct_only_invalid("degeneracy_flag_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'direct_only_invalid("enantiomorphic_split")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'direct_only_invalid("capacity_mismatch")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'direct_only_invalid("protected_or_final_access")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn both_sides_reproduce_the_stage_c_evaluators_exactly' tdi-ai/tests/tdi24_direct_only_collapse.rs
grep -Fq 'fn collapse_coincides_with_the_gamma_zero_ablation_bit_for_bit' tdi-ai/tests/tdi24_direct_only_collapse.rs
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_direct_only_collapse.rs
grep -Fq '| 32 | `beta=0` ablation | **landed** in #705, exact head `a3b4a8e986baf4b37bc6421ab14c55d28556af41`, merge `dcbd11751aebcdc4de3882e84ea7504d81abe936`;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 33 | direct-only collapse | **landed** in #707, exact head `e775f1fbfc7dde86124886a8acbe6383e7ded303`, merge `3f6fd02aa68222110384405cf230e6c049fb8767`;' docs/TDI-24-CAMPAIGN-50.md
# Do not pin the global merged count here: later qualified slices must advance it.
