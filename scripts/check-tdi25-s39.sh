#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_data_volume_scaling
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_sequence_length_scaling
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_position_geometry_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_g6_orthogonal_basis_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_torsor_structure_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_chiral_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_direct_vs_factorized_bridge
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_reduction_point_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_c_preflight
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_parity_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_learned_basis_prototype
grep -Fq 'tdi25-data-volume-scaling-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const DATA_VOLUME_COUNTS: [u64; 3] = [8, 16, 32];' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_data_volume_scaling(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_data_volume_scaling_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'data_volume_invalid("nesting_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'data_volume_invalid("allocation_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'data_volume_invalid("count_set_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'data_volume_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn both_arms_receive_identical_allocation_at_every_count' tdi-ai/tests/tdi25_data_volume_scaling.rs
grep -Fq 'tdi25-data-volume-scaling-v1' docs/TDI-25-DATA-VOLUME-SCALING-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-DATA-VOLUME-SCALING-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_data_volume_scaling.rs
grep -Fq '| 38 | Sequence-length scaling | **landed** in #731, exact head `a718c117a8bef8b6229823510f9150d43cf761eb`, merge `4cb438183cd8b0d4ad0e39f6b450c4fbb837419f`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 39 \| Data-volume scaling \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 38
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
