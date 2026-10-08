#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_input_noise_robustness
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_multi_seed_replication
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_sequence_length_scaling
grep -Fq 'tdi24-input-noise-robustness-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const NOISE_AMPLITUDES: [f64; 3] = [1e-3, 1e-2, 1e-1];' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn perturb_operand(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_input_noise_robustness(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_input_noise_robustness_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'noise_invalid("seed_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'noise_invalid("clean_reference_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'noise_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn perturbations_are_bounded_masked_and_deterministic' tdi-ai/tests/tdi24_input_noise_robustness.rs
grep -Fq 'tdi24-input-noise-robustness-v1' docs/TDI-24-INPUT-NOISE-ROBUSTNESS-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-INPUT-NOISE-ROBUSTNESS-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_input_noise_robustness.rs
grep -Fq '| 41 | Multi-seed replication | **landed** in #737, exact head `05344dba224e16cf6c7dd205d07090163083e2cc`, merge `cee0db74ca17f675c52ab66b8188bfc2760c2ede`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 42 \| Input-noise robustness \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 41
