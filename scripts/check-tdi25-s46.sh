#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_numerical_precision
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_translation_origin_stress
grep -Fq 'tdi25-numerical-precision-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn t6_score_f32(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn c6_score_f32(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_matched_numerical_precision(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_numerical_precision(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_numerical_precision_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'precision_invalid("tolerance_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'precision_invalid("failure_accounting_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'precision_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn f32_kernels_are_exact_on_the_dyadic_clean_population' tdi-ai/tests/tdi25_numerical_precision.rs
grep -Fq 'tdi25-numerical-precision-v1' docs/TDI-25-NUMERICAL-PRECISION-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-NUMERICAL-PRECISION-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_numerical_precision.rs
grep -Eq '\| 45 \| Mixed adversarial suite \| \*\*landed\*\* in #[0-9]+, exact head `[0-9a-f]{40}`, merge `[0-9a-f]{40}`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 46 \| Numerical precision study \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 45
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
# Slice 47 runs under this exact-head gate (no dedicated workflow file).
bash scripts/check-tdi25-s47.sh
