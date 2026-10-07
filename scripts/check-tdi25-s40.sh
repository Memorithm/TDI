#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
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
grep -Fq 'tdi25-stage-d-attribution-audit-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const ATTRIBUTION_AUDITED_SLICES: [(u8, &str); 10]' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_stage_d_attribution_audit(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_stage_d_attribution_audit_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'attribution_invalid("admissibility_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'attribution_invalid("withheld_entry")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'attribution_invalid("scientific_attribution_admissible")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'attribution_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn every_slice_supports_software_semantics_only' tdi-ai/tests/tdi25_stage_d_attribution_audit.rs
grep -Fq 'tdi25-stage-d-attribution-audit-v1' docs/TDI-25-STAGE-D-ATTRIBUTION-AUDIT-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-STAGE-D-ATTRIBUTION-AUDIT-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_stage_d_attribution_audit.rs
grep -Fq '| 39 | Data-volume scaling | **landed** in #733, exact head `beacb91291326106c0483a5abad582ef7f2f30ea`, merge `b4280a341cd8771a17879d5c0e0fa8e46b046ec8`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 40 \| Stage-D attribution audit \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 39
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
