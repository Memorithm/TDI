#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
grep -Fq 'tdi25-paired-uncertainty-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn summarize_paired_uncertainty_by_seed_block' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct PairedEffectSummary' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct RevealedMatchOutcome' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct ConfidenceInterval' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct G6AttributionContrastSummary' tdi-ai/src/tdi25_eval.rs
grep -Fq 'BoundedHoeffdingPairedDifference' tdi-ai/src/tdi25_eval.rs
grep -Fq 'ClusterHoeffdingPairedDifference' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reason: "duplicate_pair_identity"' tdi-ai/src/tdi25_eval.rs
grep -Fq '| 26 | Metric registry | **landed** in #679, exact head `c21c4463bb23eb8f078d1ec51612cd04def112d2`, merge `62b015cf3752835bda2f4f476f118118ef4208b6`;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 27 | Paired uncertainty engine | **current candidate** in #683;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '**26/50 merged**' docs/TDI-25-CAMPAIGN-50.md
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
