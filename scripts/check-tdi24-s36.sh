#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_chiral
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_learned_basis_prototype
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_fixed_m_sensitivity
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_parity_shuffle_control
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-learned-basis-prototype-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const LEARNED_BASIS_PARAMETER_COUNT: usize = 15;' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const LEARNED_BASIS_PROBE_COUNT: usize = 4;' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn learned_basis_probes()' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_learned_basis_transform(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn score_c6_in_learned_basis_from_view' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_learned_basis_prototype(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_learned_basis_prototype_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'LearnedBasisPrototypeInvalid' tdi-ai/src/tdi24_eval.rs
grep -Fq 'learned_basis_invalid("orthogonality_failure")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'learned_basis_invalid("basis_algebra_failure")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'learned_basis_invalid("canonical_reference_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'learned_basis_invalid("gauge_invariance_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'learned_basis_invalid("protected_or_final_access")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'learned_basis_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn identity_probe_reproduces_the_stage_c_c6_evaluator_exactly' tdi-ai/tests/tdi24_learned_basis_prototype.rs
grep -Fq 'fn non_orthogonal_or_non_finite_transforms_are_rejected' tdi-ai/tests/tdi24_learned_basis_prototype.rs
grep -Fq 'tdi24-learned-basis-prototype-v1' docs/TDI-24-LEARNED-BASIS-PROTOTYPE-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-LEARNED-BASIS-PROTOTYPE-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_learned_basis_prototype.rs
grep -Fq '| 35 | fixed-M sensitivity | **landed** in #718, exact head `b1af49b0e831472baaa47f097781c76dc8851513`, merge `2fb389b27150d875cbac95f64df1a08e9158a32a`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 36 \| structure-preserving learned basis prototype \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 35
