#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-metric-registry-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn freeze_metric_registry' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_metric_registry' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct MetricRegistry' tdi-ai/src/tdi24_eval.rs
grep -Fq '| 25 | Optimizer/update-budget contract | **landed** in #659, exact head `922ffe5c1dabed26f8662b0be319466ef020e8a9`, merge `5f4bd6c51e81adb4ef831b3caf3aeb0db62ff498`;' docs/TDI-24-CAMPAIGN-50.md
