#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_reference_cost
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_numerical_precision
grep -Fq 'tdi24-reference-cost-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const REFERENCE_COST_TIMING_STATUS: &str = "timing_not_measured";' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_reference_cost(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_reference_cost_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'cost_invalid("timing_status_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'cost_invalid("accounting_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'cost_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn smoke_study_executes_both_arms_with_exact_totals' tdi-ai/tests/tdi24_reference_cost.rs
grep -Fq 'tdi24-reference-cost-v1' docs/TDI-24-REFERENCE-COST-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-REFERENCE-COST-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_reference_cost.rs
grep -Eq '\| 44 \| Numerical precision study \| \*\*landed\*\* in #[0-9]+, exact head `[0-9a-f]{40}`, merge `[0-9a-f]{40}`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 46 \| CPU reference cost study \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 44
