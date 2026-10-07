#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_chiral_parity_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_chiral_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_direct_vs_factorized_bridge
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_reduction_point_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_c_preflight
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_parity_shuffle_control
grep -Fq 'tdi25-chiral-parity-shuffle-control-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn parity_shuffled_chiral_score' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_chiral_parity_shuffle_control' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn chiral_parity_shuffle_registered_seed' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_chiral_parity_shuffle_control(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_chiral_parity_shuffle_control_for_label' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_chiral_parity_shuffle_control_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_parity_shuffle_invalid("six_values_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq '"direct_product_multiset_drift"' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_parity_shuffle_invalid("seed_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_parity_shuffle_invalid("shuffle_not_reproducible")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_parity_shuffle_invalid("shuffle_contract_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_parity_shuffle_invalid("capacity_mismatch")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_parity_shuffle_invalid("protected_or_final_access")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_parity_shuffle_invalid("scientific_claim")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_parity_shuffle_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn reference_side_reproduces_the_matched_c6_primary_exactly' tdi-ai/tests/tdi25_chiral_parity_shuffle_control.rs
grep -Fq 'fn block_preserving_or_swapping_shuffles_are_rejected' tdi-ai/tests/tdi25_chiral_parity_shuffle_control.rs
grep -Fq 'fn control_reuses_the_tdi24_shuffle_and_an_already_registered_seed' tdi-ai/tests/tdi25_chiral_parity_shuffle_control.rs
grep -Fq 'tdi25-chiral-parity-shuffle-control-v1' docs/TDI-25-CHIRAL-PARITY-SHUFFLE-CONTROL-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-CHIRAL-PARITY-SHUFFLE-CONTROL-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_chiral_parity_shuffle_control.rs
grep -Fq '| 33 | Chiral `gamma=0` ablation | **landed** in #714, exact head `da2a98ae9ef596b2dc0fb5dbd79764e829609e5c`, merge `87b5f02fa053d415b3c9f5b2d5041d4fe9c58b41`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 34 \| Chiral parity-shuffle control \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
# Monotone: later qualified slices must be able to advance the merged count.
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 33
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
