#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-paired-uncertainty-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn summarize_paired_uncertainty' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct PairedEffectSummary' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct RevealedMatchOutcome' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct ConfidenceInterval' tdi-ai/src/tdi24_eval.rs
grep -Fq 'BoundedHoeffdingPairedDifference' tdi-ai/src/tdi24_eval.rs
grep -Fq 'ClusterHoeffdingPairedDifference' tdi-ai/src/tdi24_eval.rs
grep -Fq 'reason: "duplicate_pair_identity"' tdi-ai/src/tdi24_eval.rs
grep -Fq '| 26 | Metric registry | **landed** in #667, exact head `9d98deba309da78fb5f57930c980ea14041af434`, merge `277896b34d7a0f76af0d176ec4c9356a5b4e68cc`;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 27 | Paired uncertainty engine | **landed** in #671, exact head `7c640354626de37400bf0f86655e6c0e21284d39`, merge `c547fa5f7f12e001f4437c3bb054d75b239099a7`;' docs/TDI-24-CAMPAIGN-50.md
# Do not pin the global merged count here: later qualified slices must advance it.
