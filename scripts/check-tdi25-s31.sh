#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_reduction_point_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_c_preflight
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
grep -Fq 'tdi25-torsor-reduction-point-ablation-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const TRANSPORT_IDENTITY_RELATIVE_TOLERANCE: f64 = 1e-12;' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn untransported_torsor_score' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn reduction_point_transport_term' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_reduction_point_ablation' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_torsor_reduction_point_ablation(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_torsor_reduction_point_ablation_for_label' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_torsor_reduction_point_ablation_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reduction_point_invalid("transport_residual")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reduction_point_invalid("coincident_point_transport")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reduction_point_invalid("capacity_mismatch")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reduction_point_invalid("protected_or_final_access")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reduction_point_invalid("scientific_claim")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn reference_side_reproduces_the_matched_t6_primary_exactly' tdi-ai/tests/tdi25_reduction_point_ablation.rs
grep -Fq 'tdi25-torsor-reduction-point-ablation-v1' docs/TDI-25-REDUCTION-POINT-ABLATION-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_reduction_point_ablation.rs
grep -Fq '| 31 | Torsor reduction-point ablation | **current candidate**' docs/TDI-25-CAMPAIGN-50.md
# Monotone: later qualified slices must be able to advance the merged count.
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 29
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
