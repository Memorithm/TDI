#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_tasks
grep -Fq "tdi25-protected-label-api-v1" tdi-ai/src/tdi25_tasks.rs
grep -Fq "pub fn run_inference_callback" tdi-ai/src/tdi25_tasks.rs
grep -Fq "pub struct ProtectedLabel" tdi-ai/src/tdi25_tasks.rs
grep -Fq '| 17 | Development/Validation split manifest | **landed** in #654;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 18 | Protected-label inference API | **landed** in #656;' docs/TDI-25-CAMPAIGN-50.md
