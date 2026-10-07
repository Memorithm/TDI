#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_chiral
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_fixed_m_sensitivity
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_parity_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_direct_only_collapse
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_beta_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_gamma_zero_ablation
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-fixed-m-sensitivity-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const FIXED_MIRROR_BASIS_COUNT: usize = 20;' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn fixed_mirror_bases()' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_fixed_mirror_basis(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn score_c6_in_basis_from_view' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_fixed_m_sensitivity(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_fixed_m_sensitivity_for_label' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_fixed_m_sensitivity_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'FixedMSensitivityInvalid' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fixed_m_invalid("basis_algebra_failure")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fixed_m_invalid("basis_order_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fixed_m_invalid("basis_family_incomplete")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fixed_m_invalid("canonical_reference_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fixed_m_invalid("complement_antisymmetry")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fixed_m_invalid("capacity_mismatch")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fixed_m_invalid("protected_or_final_access")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fixed_m_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn canonical_basis_reproduces_the_stage_c_c6_evaluator_exactly' tdi-ai/tests/tdi24_fixed_m_sensitivity.rs
grep -Fq 'fn frozen_family_is_complete_ordered_and_algebraic' tdi-ai/tests/tdi24_fixed_m_sensitivity.rs
grep -Fq 'tdi24-fixed-m-sensitivity-v1' docs/TDI-24-FIXED-M-SENSITIVITY-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-FIXED-M-SENSITIVITY-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_fixed_m_sensitivity.rs
grep -Fq '| 34 | parity-shuffle control | **landed** in #713, exact head `17407c30e4d132b8d68af72798d9a15193400d10`, merge `8aef82dfda23baa9104178b1ff8e7dcd7652e2fa`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 35 \| fixed-M sensitivity \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
# Monotone: later qualified slices must be able to advance the merged count.
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 34
