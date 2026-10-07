#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_g6_orthogonal_basis_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_torsor_structure_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_torsor_structure_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_chiral_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_direct_vs_factorized_bridge
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_reduction_point_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_c_preflight
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_parity_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_learned_basis_prototype
grep -Fq 'tdi25-g6-orthogonal-basis-control-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn rotated_generic_and_chiral_scores' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_g6_orthogonal_basis_control' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_g6_orthogonal_basis_control(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_g6_orthogonal_basis_control_report' tdi-ai/src/tdi25_eval.rs
grep -Fq '"g6_rotation_invariance_drift"' tdi-ai/src/tdi25_eval.rs
grep -Fq '"identity_probe_drift"' tdi-ai/src/tdi25_eval.rs
grep -Fq 'g6_orthogonal_basis_invalid("probe_set_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq '"protected_or_final_access"' tdi-ai/src/tdi25_eval.rs
grep -Fq 'g6_orthogonal_basis_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn reference_side_reproduces_the_matched_c6_primary_exactly' tdi-ai/tests/tdi25_g6_orthogonal_basis_control.rs
grep -Fq 'tdi25-g6-orthogonal-basis-control-v1' docs/TDI-25-G6-ORTHOGONAL-BASIS-CONTROL-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-G6-ORTHOGONAL-BASIS-CONTROL-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_g6_orthogonal_basis_control.rs
grep -Fq '| 35 | Torsor structure-shuffle control | **landed** in #722, exact head `0a94994806a3877ed9590f77d131e4901faa7502`, merge `e49e905f133f26824d03cc20e4089aba01d6c8a1`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 36 \| G6 orthogonal-basis control \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 35
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
