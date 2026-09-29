#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
grep -Fq 'tdi24-provenance-envelope-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct ProvenanceEnvelope' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_provenance_envelope' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn stage_c_config_identity_bundle' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn stage_c_code_identity_bundle' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn stage_c_data_identity_bundle' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn stage_c_seed_identity' tdi-ai/src/tdi24_eval.rs
grep -Fq 'PROVENANCE_TOOLCHAIN_ID' tdi-ai/src/tdi24_eval.rs
grep -Fq 'PROVENANCE_TOOLCHAIN_CHANNEL' tdi-ai/src/tdi24_eval.rs
grep -Fq 'ProvenanceEnvelopeInvalid' tdi-ai/src/tdi24_eval.rs
grep -Fq 'open_with_provenance' tdi-ai/src/tdi24_eval.rs
grep -Fq 'for_pinned_stage_c_run' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn provenance_envelope_accepts_pinned_non_final_run' tdi-ai/src/tdi24_eval.rs
grep -Fq '| 28 | Failure taxonomy | **landed** in #677, exact head `c8bfb4aa82e6bbf9b0767ad0a3e73f93a98b6b85`, merge `b0f5c76c04fd2e67d22cad8a20aecbe825122d9d`;' docs/TDI-24-CAMPAIGN-50.md
grep -Fq '| 29 | Provenance envelope | **current stacked slice**;' docs/TDI-24-CAMPAIGN-50.md
# Do not pin the global merged count here: later qualified slices must advance it.
