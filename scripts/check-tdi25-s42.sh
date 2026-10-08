#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_input_noise_robustness
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_multi_seed_replication
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_d_attribution_audit
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_data_volume_scaling
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_sequence_length_scaling
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_position_geometry_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
grep -Fq 'tdi25-input-noise-robustness-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const INPUT_NOISE_AMPLITUDES: [f64; 3] = [1e-3, 1e-2, 1e-1];' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn perturb_matched_input(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_input_noise_robustness(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_input_noise_robustness(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_input_noise_robustness_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reason: "clean_reference_drift"' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'input_noise_invalid("seed_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'input_noise_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn perturbations_are_bounded_masked_and_deterministic' tdi-ai/tests/tdi25_input_noise_robustness.rs
grep -Fq 'tdi25-input-noise-robustness-v1' docs/TDI-25-INPUT-NOISE-ROBUSTNESS-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-INPUT-NOISE-ROBUSTNESS-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_input_noise_robustness.rs
grep -Fq '| 41 | Multi-seed replication | **landed** in #739, exact head `c0d31dcb233bfaa8d9cab636f69dc52b308ef16c`, merge `325766691cd0e1ecfde954f28b3f811e5a15005b`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 42 \| Input/noise robustness \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 41
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
