#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_reflection_adversarial_set
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_input_noise_robustness
grep -Fq 'tdi24-reflection-adversarial-set-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const ADVERSARIAL_DOMINANCE_RATIOS: [f64; 5] = [0.5, 0.9, 0.99, 1.01, 2.0];' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const ADVERSARIAL_CHIRALITY_SCALES: [f64; 2] = [1.0, 1e-3];' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn reflection_adversarial_pairs(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_reflection_adversarial_set(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_reflection_adversarial_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'adversarial_invalid("direct_only_discrimination")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'adversarial_invalid("pair_digest_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'adversarial_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn pairs_are_exact_mirrors_with_declared_channels' tdi-ai/tests/tdi24_reflection_adversarial_set.rs
grep -Fq 'tdi24-reflection-adversarial-set-v1' docs/TDI-24-REFLECTION-ADVERSARIAL-SET-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-REFLECTION-ADVERSARIAL-SET-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_reflection_adversarial_set.rs
grep -Fq '| 42 | Input-noise robustness | **landed** in #740, exact head `9de1c9a5a8299d27da28165a07d7beb6d5efbf70`, merge `ef73feb0e3140f6e7acc9931bbd4b682c43cd2c9`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 43 \| Reflection adversarial set \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 42
