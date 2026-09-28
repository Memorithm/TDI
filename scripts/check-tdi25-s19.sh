#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_tasks
grep -Fq "tdi25-seed-case-canonicalization-v1" tdi-ai/src/tdi25_tasks.rs
grep -Fq "pub fn register_seed" tdi-ai/src/tdi25_tasks.rs
grep -Fq "pub fn canonicalize_torsor_transport_input" tdi-ai/src/tdi25_tasks.rs
grep -Fq '| 18 | Protected-label inference API | **landed** in #656;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 19 | Seed/case canonicalization + hash | **current stacked slice**;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '**18/50 merged**' docs/TDI-25-CAMPAIGN-50.md
