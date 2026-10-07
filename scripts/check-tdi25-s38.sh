#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
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
grep -Fq 'tdi25-sequence-length-scaling-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const SEQUENCE_SCALING_LENGTHS: [usize; 3] = [2, 4, 8];' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_sequence_length_scaling' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq '"diagonal_reference_drift"' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_sequence_length_scaling(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_sequence_length_scaling_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'sequence_length_invalid("length_set_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'sequence_length_invalid("cost_accounting_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'sequence_length_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn smoke_study_has_exact_matched_resource_accounting' tdi-ai/tests/tdi25_sequence_length_scaling.rs
grep -Fq 'tdi25-sequence-length-scaling-v1' docs/TDI-25-SEQUENCE-LENGTH-SCALING-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-SEQUENCE-LENGTH-SCALING-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_sequence_length_scaling.rs
grep -Fq '| 37 | Position-geometry ablation | **landed** in #728, exact head `f1b89404af364e0cd74ed3edb19db8103a951b8c`, merge `cb4c98d7f88c6ffaefcca687b8fe2d58ae4d8125`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 38 \| Sequence-length scaling \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 37
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
