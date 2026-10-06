#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_direct_vs_factorized_bridge
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_reduction_point_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_c_preflight
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
grep -Fq 'tdi25-direct-vs-factorized-bridge-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE: f64 = TRANSPORT_IDENTITY_RELATIVE_TOLERANCE;' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn direct_torsor_bridge_score' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_torsor_bridge_equivalence' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_direct_vs_factorized_bridge(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_direct_vs_factorized_bridge_for_label' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_direct_vs_factorized_bridge_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'bridge_invalid("bridge_residual")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'bridge_invalid("residual_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'bridge_invalid("tolerance_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'bridge_invalid("capacity_mismatch")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'bridge_invalid("protected_or_final_access")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'bridge_invalid("scientific_claim")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn factorized_side_reproduces_the_matched_t6_primary_exactly' tdi-ai/tests/tdi25_direct_vs_factorized_bridge.rs
grep -Fq 'fn matched_population_bridge_is_bit_exact_and_records_the_degeneracy' tdi-ai/tests/tdi25_direct_vs_factorized_bridge.rs
grep -Fq 'tdi25-direct-vs-factorized-bridge-v1' docs/TDI-25-DIRECT-VS-FACTORIZED-BRIDGE-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-DIRECT-VS-FACTORIZED-BRIDGE-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_direct_vs_factorized_bridge.rs
grep -Fq '| 32 | Direct vs factorized torsor bridge | **landed** in #708, exact head `9a899209a16ba2ac3cd745fa1e601901e66c2caa`, merge `c73f98a9ed69f842ee2d49cdc8b24b5fafc42f7a`;' docs/TDI-25-CAMPAIGN-50.md
# Monotone: later qualified slices must be able to advance the merged count.
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 32
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
