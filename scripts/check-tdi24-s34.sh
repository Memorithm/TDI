#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_chiral
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_parity_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_direct_only_collapse
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_beta_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-parity-shuffle-control-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const MAX_PARITY_SHUFFLE_DRAWS: u32 = 64;' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn parity_shuffle_from_seed' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_parity_shuffle(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn score_c6_parity_shuffled_from_view' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_parity_shuffle_control(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_parity_shuffle_control_for_label' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_parity_shuffle_control_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'ParityShuffleControlInvalid' tdi-ai/src/tdi24_eval.rs
grep -Fq 'parity_shuffle_invalid("sector_blocks_preserved")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'parity_shuffle_invalid("mirror_structure_preserved")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'parity_shuffle_invalid("complex_structure_preserved")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'parity_shuffle_invalid("shuffle_not_reproducible")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'parity_shuffle_invalid("seed_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'parity_shuffle_invalid("direct_product_multiset_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'parity_shuffle_invalid("capacity_mismatch")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'parity_shuffle_invalid("protected_or_final_access")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn reference_side_reproduces_the_stage_c_c6_evaluator_exactly' tdi-ai/tests/tdi24_parity_shuffle_control.rs
grep -Fq 'fn block_preserving_or_swapping_shuffles_are_rejected' tdi-ai/tests/tdi24_parity_shuffle_control.rs
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_parity_shuffle_control.rs
grep -Fq '| 33 | direct-only collapse | **landed** in #707, exact head `e775f1fbfc7dde86124886a8acbe6383e7ded303`, merge `3f6fd02aa68222110384405cf230e6c049fb8767`;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 34 | parity-shuffle control | **current stacked slice**;' docs/TDI-24-CAMPAIGN-50.md
# Do not pin the global merged count here: later qualified slices must advance it.
