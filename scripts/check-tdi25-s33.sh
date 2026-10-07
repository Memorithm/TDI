#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_chiral_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_direct_vs_factorized_bridge
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_reduction_point_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_c_preflight
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
grep -Fq 'tdi25-chiral-gamma-zero-ablation-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const fn chiral_gamma_zero_weights' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn gamma_zero_chiral_score' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn chiral_parity_odd_channel' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_chiral_gamma_zero_ablation' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_chiral_gamma_zero_ablation(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_chiral_gamma_zero_ablation_for_label' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_chiral_gamma_zero_ablation_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_gamma_zero_invalid("parity_odd_residual")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_gamma_zero_invalid("enantiomorphic_split")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_gamma_zero_invalid("inactive_channel_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_gamma_zero_invalid("ablated_weights_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_gamma_zero_invalid("capacity_mismatch")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_gamma_zero_invalid("protected_or_final_access")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'chiral_gamma_zero_invalid("scientific_claim")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn reference_side_reproduces_the_matched_c6_primary_exactly' tdi-ai/tests/tdi25_chiral_gamma_zero_ablation.rs
grep -Fq 'fn chiral_favorable_target_is_the_reference_score_and_records_the_degeneracy' tdi-ai/tests/tdi25_chiral_gamma_zero_ablation.rs
grep -Fq 'tdi25-chiral-gamma-zero-ablation-v1' docs/TDI-25-CHIRAL-GAMMA-ZERO-ABLATION-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-CHIRAL-GAMMA-ZERO-ABLATION-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_chiral_gamma_zero_ablation.rs
grep -Fq '| 32 | Direct vs factorized torsor bridge | **landed** in #708' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 33 | Chiral `gamma=0` ablation | **current candidate**' docs/TDI-25-CAMPAIGN-50.md
# Monotone: later qualified slices must be able to advance the merged count.
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 32
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
grep -Fq 'chiral_gamma_zero_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
