#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_chiral
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_width_scaling
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_head_sharing_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_learned_basis_prototype
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_fixed_m_sensitivity
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_parity_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-width-scaling-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const WIDTH_SCALING_WIDTHS: [usize; WIDTH_SCALING_WIDTH_COUNT] = [2, 4, 6];' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn restrict_carrier_to_width' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_width_scaling(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_width_scaling_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'width_scaling_invalid("full_width_reference_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'width_scaling_invalid("width_set_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'width_scaling_invalid("protected_or_final_access")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'width_scaling_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn full_width_reproduces_the_stage_c_c6_evaluator_exactly' tdi-ai/tests/tdi24_width_scaling.rs
grep -Fq 'tdi24-width-scaling-v1' docs/TDI-24-WIDTH-SCALING-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-WIDTH-SCALING-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_width_scaling.rs
grep -Fq '| 37 | head-sharing ablation | **landed** in #724, exact head `e13f41eb85fd6a6d37fb5e43c37ce6576cd76233`, merge `9895cd9ee7cec4adff0dfdde862eda7d985e8731`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 38 \| width scaling \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 37
