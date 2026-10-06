#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_c_preflight
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_failure_resource_accounting
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
grep -Fq 'tdi25-stage-c-preflight-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct StageCPreflightBudget' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct StageCPreflightReport' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_stage_c_preflight(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_stage_c_preflight_for_label' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_stage_c_preflight_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'require_complete_primary_matrix()' tdi-ai/src/tdi25_eval.rs
grep -Fq 'require_complete_primary_accounting(&accounting)?' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reason: "protected_or_final_access"' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reason: "scientific_claim"' tdi-ai/src/tdi25_eval.rs
# The preflight is bounded and never exposes a protected/final surface.
grep -Fq 'pub const MAX_PREFLIGHT_SEED_BLOCKS: u64 = 2;' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const MAX_PREFLIGHT_CASES_PER_BLOCK: u64 = 16;' tdi-ai/src/tdi25_eval.rs
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/tests/tdi25_stage_c_preflight.rs
grep -Fq '| 29 | Typed failure/resource accounting | **landed** in #699, exact head `' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 30 | Stage-C bounded preflight | **landed** in #701, exact head `d32d6a3985ae76ad5db272078de69893817efd88`, merge `4a58ff8c5aaf244d75b4c228decc9e5003359e07`;' docs/TDI-25-CAMPAIGN-50.md
# Monotone: later qualified slices must be able to advance the merged count.
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 30
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
