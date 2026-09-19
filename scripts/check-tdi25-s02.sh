#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi25

grep -Fq 'pub const SOURCE_CONTRACT_PIN_VERSION: &str = "tdi25-source-contract-pin-v1";' tdi-ai/src/tdi25_torsor_chiral.rs
grep -Fq 'torsor: "tdi22-torsor-dual-pairing-v1"' tdi-ai/src/tdi25_torsor_chiral.rs
grep -Fq 'chiral: "tdi24-mirror-coupled-chiral-v1"' tdi-ai/src/tdi25_torsor_chiral.rs
grep -Fq 'TDI-22 torsor source: `tdi22-torsor-dual-pairing-v1`' docs/TDI-25-SOURCE-CONTRACTS.md
grep -Fq 'TDI-24 chiral source: `tdi24-mirror-coupled-chiral-v1`' docs/TDI-25-SOURCE-CONTRACTS.md
grep -Fq '| 01 | TDI-25 programme + comparison scaffold | **landed** in #408;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 02 | Contract/version provenance pin | **landed** in #489;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '**2/50 merged** (#408, #489).' docs/TDI-25-CAMPAIGN-50.md
