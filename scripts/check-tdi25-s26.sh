#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
grep -Fq 'tdi25-metric-registry-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn freeze_metric_registry' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_metric_registry' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct MetricRegistry' tdi-ai/src/tdi25_eval.rs
grep -Fq 'PrimaryFamilyMetricId' tdi-ai/src/tdi25_eval.rs
grep -Fq 'CrossFamilySummaryMetricId' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub mod tdi25_eval;' tdi-ai/src/experimental.rs
grep -Fq '| 25 | Optimizer/update-budget matcher | **landed** in #673, exact head `d171ee9815813704b7214f5c812b2e6e864b6116`, merge `efb03b8c3e37bcb537952bdca2ce545765da29c8`;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 26 | Metric registry | **current candidate** in #' docs/TDI-25-CAMPAIGN-50.md
