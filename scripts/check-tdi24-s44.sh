#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_numerical_precision
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_reflection_adversarial_set
grep -Fq 'tdi24-numerical-precision-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const PRECISION_BOUND_FACTOR: f64 = 32.0;' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn chiral_score_f32(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_numerical_precision(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_numerical_precision_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'precision_invalid("tolerance_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'precision_invalid("failure_accounting_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'precision_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn smoke_study_accounts_every_case_within_the_declared_bound' tdi-ai/tests/tdi24_numerical_precision.rs
grep -Fq 'tdi24-numerical-precision-v1' docs/TDI-24-NUMERICAL-PRECISION-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-NUMERICAL-PRECISION-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_numerical_precision.rs
grep -Eq '\| 43 \| Reflection adversarial set \| \*\*landed\*\* in #[0-9]+, exact head `[0-9a-f]{40}`, merge `[0-9a-f]{40}`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 44 \| Numerical precision study \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 43
