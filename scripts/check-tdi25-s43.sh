#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_translation_origin_stress
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_input_noise_robustness
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
grep -Fq 'tdi25-translation-origin-stress-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const ORIGIN_STRESS_OFFSETS: [f64; 3] = [1.0, 1e3, 1e6];' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn stress_matched_input(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn origin_stress_direction(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_translation_origin_stress(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_translation_origin_stress(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_translation_origin_stress_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reason: "clean_reference_drift"' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'origin_stress_invalid("seed_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'origin_stress_invalid("c6_origin_shift_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'origin_stress_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn transformations_preserve_the_physical_pairing_and_touch_declared_fields' tdi-ai/tests/tdi25_translation_origin_stress.rs
grep -Fq 'tdi25-translation-origin-stress-v1' docs/TDI-25-TRANSLATION-ORIGIN-STRESS-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-TRANSLATION-ORIGIN-STRESS-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_translation_origin_stress.rs
grep -Fq '| 42 | Input/noise robustness | **landed** in #742, exact head `85c83e47cd48ef1be40dfe52c066e03fa784c24a`, merge `c491c891c162c97afc8ae5a760f7609f712ab5a9`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 43 \| Translation/origin stress suite \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 42
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
