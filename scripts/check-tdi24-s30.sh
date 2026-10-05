#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-stage-c-preflight-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_stage_c_preflight' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_stage_c_preflight_for_label' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_stage_c_preflight_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct StageCPreflightReport' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct StageCPreflightBudget' tdi-ai/src/tdi24_eval.rs
grep -Fq 'MAX_PREFLIGHT_PAIRS_PER_FAMILY' tdi-ai/src/tdi24_eval.rs
grep -Fq 'StageCPreflightInvalid' tdi-ai/src/tdi24_eval.rs
grep -Fq 'reason: "protected_or_final_access_forbidden"' tdi-ai/src/tdi24_eval.rs
grep -Fq 'reason: "summary_hides_failures"' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn stage_c_preflight_has_zero_protected_or_final_access' tdi-ai/tests/tdi24_stage_c_preflight.rs
grep -Fq 'status: candidate_pending_exact_head_qualification_and_merge' docs/tdi24-stage-c-preflight.yaml
grep -Fq 'protected_or_final_access: false' docs/tdi24-stage-c-preflight.yaml
grep -Fq 'confirmatory_execution: false' docs/tdi24-stage-c-preflight.yaml
grep -Fq 'scientific_claim: false' docs/tdi24-stage-c-preflight.yaml
grep -Fq '| 29 | Provenance envelope | **landed** in #687, exact head `cd098d81ccbedce5f0a75a9216b4d72350729f14`, merge `11245ad9e4ba93cd0580a82e1a78aa2d11760a34`;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 30 | Stage-C preflight | **current stacked slice**;' docs/TDI-24-CAMPAIGN-50.md
# Do not pin the global merged count here: later qualified slices must advance it.
