#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_failure_resource_accounting
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_production_pairing
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
grep -Fq 'tdi25-failure-resource-accounting-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub enum FailureClass' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn classify_eval_error' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct ArmFailureRecord' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct ArmResourceAccount' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct FailureResourceLedger' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn account_matched_primary_block' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn require_complete_primary_accounting' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_failure_resource_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reason: "failure_not_retained"' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reason: "primary_failures_retained"' tdi-ai/src/tdi25_eval.rs
# Scores may only be counted through evaluator-backed paths.
! grep -Fq 'pub fn record_scored' tdi-ai/src/tdi25_eval.rs
grep -Fq '| 28 | Family-stratified synthesis | **landed** in #689, exact head `e34411b6b6e2e23bf36374bdd07368c2b9bf3121`, merge `14102d8745b79d41aca821a37c81c90077ea77d2`;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 29 | Typed failure/resource accounting | **current candidate**' docs/TDI-25-CAMPAIGN-50.md
# Monotone: later qualified slices must be able to advance the merged count.
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 28
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
