#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi25_tasks
grep -Fq "tdi25-split-manifest-v1" tdi-ai/src/tdi25_tasks.rs
grep -Fq '| 16 | Difficulty strata | **landed** in #612;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 17 | Development/Validation split manifest | **current stacked slice**;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '**16/50 merged**' docs/TDI-25-CAMPAIGN-50.md
