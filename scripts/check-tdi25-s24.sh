#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
grep -Fq 'tdi25-parameter-readout-matcher-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn match_parameter_readouts' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct ParameterReadoutCapacity' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct MatchedParameterReadout' tdi-ai/src/tdi25_eval.rs
grep -Fq 'ParameterReadoutMismatch' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-readout-budget-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'tdi25-g6-evaluator-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub mod tdi25_eval;' tdi-ai/src/experimental.rs
grep -Fq '| 23 | G6 evaluator | **landed** in #666, exact head `3403b8768d24ac1543b75120d15a09dd5b58c50e`, merge `d6c8226025de1dbe4848f83d8092c316066f97b1`;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 24 | Parameter/readout matcher | **current candidate** in #669;' docs/TDI-25-CAMPAIGN-50.md
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
