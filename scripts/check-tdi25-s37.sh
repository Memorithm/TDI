#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_position_geometry_ablation
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
grep -Fq 'tdi25-position-geometry-ablation-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const POSITION_GEOMETRY_ABLATION_ARMS' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_position_geometry_ablation' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_position_geometry_ablation(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_position_geometry_ablation_report' tdi-ai/src/tdi25_eval.rs
grep -Fq '"matched_reference_drift"' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'position_geometry_invalid("arm_set_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq '"protected_or_final_access"' tdi-ai/src/tdi25_eval.rs
grep -Fq 'position_geometry_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn matched_arm_reproduces_the_matched_t6_primary_exactly' tdi-ai/tests/tdi25_position_geometry_ablation.rs
grep -Fq 'tdi25-position-geometry-ablation-v1' docs/TDI-25-POSITION-GEOMETRY-ABLATION-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-POSITION-GEOMETRY-ABLATION-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_position_geometry_ablation.rs
grep -Fq '| 36 | G6 orthogonal-basis control | **landed** in #725, exact head `ffa6cae574eaec88a2711363178ff131683cfbf1`, merge `790b888a203733ee963865162ee7d382924652fb`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 37 \| Position-geometry ablation \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 36
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
