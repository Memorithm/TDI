#!/usr/bin/env bash
# TDI-24 issue #690: authoritative Stage-C provenance (envelope v2).
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_provenance_authority
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
test -f tdi-ai/build.rs
grep -Fq 'TDI_AI_BUILD_RUSTC_VERSION' tdi-ai/build.rs
grep -Fq 'tdi24-provenance-envelope-v2' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const LEGACY_PROVENANCE_ENVELOPE_CONTRACT' tdi-ai/src/tdi24_eval.rs
grep -Fq 'tdi24-evaluator-config-identity-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'tdi24-source-digest-fnv1a64-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'tdi24-admitted-population-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'tdi24-case-seed-binding-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub struct AdmittedCase' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_provenance_binding' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn stage_c_config_identity_bundle(config: &EvaluatorConfig)' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn stage_c_preflight_admitted_cases' tdi-ai/src/tdi24_eval.rs
for reason in provenance_required case_not_admitted case_population_mismatch \
  case_already_evaluated case_seed_mismatch duplicate_admitted_case \
  config_identity_mismatch code_identity_drift data_identity_drift \
  seed_identity_drift toolchain_drift compiler_identity_unavailable \
  population_exceeds_budget; do
  grep -Fq "reason: \"${reason}\"" tdi-ai/src/tdi24_eval.rs
  grep -Fq "\"${reason}\"" tdi-ai/tests/tdi24_provenance_authority.rs
done
grep -Fq 'provenance_envelope: tdi24-provenance-envelope-v2' docs/tdi24-stage-c-preflight.yaml
grep -Fq 'Provenance authority (issue #690, envelope v2)' docs/TDI-24-STAGE-C-PREFLIGHT.md
# The declared CI gate compiler must be the compiler that ran these tests.
rustc +1.97.1 --version | grep -Eq '^rustc 1\.97\.1 '
grep -Fq 'reason: "record_identity_mismatch"' tdi-ai/src/tdi24_eval.rs
grep -Fq '"record_identity_mismatch"' tdi-ai/tests/tdi24_provenance_authority.rs
