#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_matched_matrix
grep -Fq 'tdi25-matched-evaluator-matrix-v2' tdi-ai/src/tdi25_matched_matrix.rs
grep -Fq 'pub fn require_complete_primary_matrix' tdi-ai/src/tdi25_matched_matrix.rs
grep -Fq 'SecondaryControlCannotSatisfyPrimary' tdi-ai/src/tdi25_matched_matrix.rs
grep -Fq 'G6 has a sealed Neutral path, but remains a secondary attribution control' docs/TDI-25-MATCHED-EVALUATOR-GATE.md
grep -Fq 'pub mod tdi25_matched_matrix;' tdi-ai/src/experimental.rs
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
