#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_chiral

grep -Fq 'pub const CHANNEL_DECOMPOSITION_CONTRACT: &str = "tdi24-channel-decomposition-v1";' tdi-ai/src/tdi24_chiral.rs
grep -Fq 'pub fn decomposed_observables(' tdi-ai/src/tdi24_chiral.rs
grep -Fq '`s(q,k) = q^T k`' docs/TDI-24-CHANNEL-DECOMPOSITION.md
grep -Fq '`m(q,k) = q^T M k`' docs/TDI-24-CHANNEL-DECOMPOSITION.md
grep -Fq '`chi(q,k) = q^T J k`' docs/TDI-24-CHANNEL-DECOMPOSITION.md
grep -Fq '| 03 | Matched V6 vector reference | **landed** in #487;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 04 | Channel decomposition contract | **current slice**;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '**3/50 merged** (#406, #471, #487).' docs/TDI-24-CAMPAIGN-50.md
