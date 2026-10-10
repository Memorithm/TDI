#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_reference_cost
grep -Fq 'tdi25-reference-cost-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const QUALIFIED_TIMING_ENVIRONMENT: Option<QualifiedTimingEnvironment> = None;' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const REFERENCE_TIMING_STATUS: &str = "timing_non_qualifie";' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const fn t6_pair_accounting(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const fn c6_pair_accounting(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_qualified_reference_timing(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'cost_invalid("timing_environment_not_qualified")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'cost_invalid("timing_status_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'cost_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn timing_harness_refuses_without_a_qualified_environment' tdi-ai/tests/tdi25_reference_cost.rs
grep -Fq 'tdi25-reference-cost-v1' docs/TDI-25-REFERENCE-COST-V1.md
grep -Fq 'Timing non qualifié' docs/TDI-25-REFERENCE-COST-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-REFERENCE-COST-V1.md
! git ls-files | grep -Eq 'tdi25-qualified-timing-environment'
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/tests/tdi25_reference_cost.rs
grep -Eq '\| 46 \| Numerical precision study \| \*\*landed\*\* in #[0-9]+, exact head `[0-9a-f]{40}`, merge `[0-9a-f]{40}`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 47 \| Reference cost study \| \*\*(current candidate|landed)[^|]*timing non qualifié' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 46
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
