#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-failure-taxonomy-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub enum FailureClass' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct FailureRecord' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct FailureLedger' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn classify_eval_error' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn retain_failure' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn retain_eval_error' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_failure_record' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_eval_record_contracts' tdi-ai/src/tdi24_eval.rs
grep -Fq '| 27 | Paired uncertainty engine | **landed** in #671, exact head `7c640354626de37400bf0f86655e6c0e21284d39`, merge `c547fa5f7f12e001f4437c3bb054d75b239099a7`;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 28 | Failure taxonomy | **landed** in #677, exact head `c8bfb4aa82e6bbf9b0767ad0a3e73f93a98b6b85`, merge `b0f5c76c04fd2e67d22cad8a20aecbe825122d9d`;' docs/TDI-24-CAMPAIGN-50.md
# Do not pin the global merged count here: later qualified slices must advance it.
