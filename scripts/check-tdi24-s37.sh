#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_chiral
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_head_sharing_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_learned_basis_prototype
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_fixed_m_sensitivity
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_parity_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-head-sharing-ablation-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const HEAD_SHARING_HEAD_COUNT: usize = 2;' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const PER_HEAD_BASIS_RANKS: [usize; HEAD_SHARING_HEAD_COUNT] = [0, 1];' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn shared_head_c6_score_from_view' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn per_head_c6_score_from_view' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_head_sharing_ablation(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_head_sharing_ablation_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'head_sharing_invalid("shared_reference_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'head_sharing_invalid("head_mean_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'head_sharing_invalid("capacity_mismatch")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'head_sharing_invalid("protected_or_final_access")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'head_sharing_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn shared_arm_reproduces_the_single_head_stage_c_c6_evaluator_exactly' tdi-ai/tests/tdi24_head_sharing_ablation.rs
grep -Fq 'tdi24-head-sharing-ablation-v1' docs/TDI-24-HEAD-SHARING-ABLATION-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-HEAD-SHARING-ABLATION-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_head_sharing_ablation.rs
grep -Fq '| 36 | structure-preserving learned basis prototype | **landed** in #721, exact head `b40da215d4a39e5cd843713af2cb5b3bf0e81511`, merge `ee49a5e63b6402fd8169b10da543640f9b2e17c2`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 37 \| head-sharing ablation \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 36
