#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_multi_seed_replication
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_d_attribution_audit
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
grep -Fq 'tdi25-multi-seed-replication-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const REPLICATION_SEED_BLOCKS: [u64; 8] = [0, 1, 2, 3, 4, 5, 6, 7];' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_multi_seed_replication(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_multi_seed_replication_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'replication_invalid("pairing_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'replication_invalid("block_overlap")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'replication_invalid("paired_count_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'replication_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn cells_are_paired_and_match_the_matched_primary_runs' tdi-ai/tests/tdi25_multi_seed_replication.rs
grep -Fq 'tdi25-multi-seed-replication-v1' docs/TDI-25-MULTI-SEED-REPLICATION-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-MULTI-SEED-REPLICATION-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_multi_seed_replication.rs
grep -Fq '| 40 | Stage-D attribution audit | **landed** in #736, exact head `6077f5d7412878717a1503e1db21376fbc7db389`, merge `15de338b190359004e8afa5b8aa57612caae0ef5`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 41 \| Multi-seed replication \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 40
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
