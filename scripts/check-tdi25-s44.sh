#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_mirror_parity_stress
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_translation_origin_stress
grep -Fq 'tdi25-mirror-parity-stress-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const MIRROR_STRESS_TRANSFORMS: [MirrorStressTransform; 3] = [' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn mirror_stress_matched_input(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn evaluate_mirror_parity_stress(' tdi-ai/src/tdi25_matched_reference.rs
grep -Fq 'pub fn run_mirror_parity_stress(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn validate_mirror_parity_stress_report' tdi-ai/src/tdi25_eval.rs
grep -Fq 'mirror_stress_invalid("c6_complex_structure_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'mirror_stress_invalid("c6_chiral_target_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'mirror_stress_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn transformations_are_the_upstream_involutions_and_keep_positions' tdi-ai/tests/tdi25_mirror_parity_stress.rs
grep -Fq 'tdi25-mirror-parity-stress-v1' docs/TDI-25-MIRROR-PARITY-STRESS-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-MIRROR-PARITY-STRESS-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/src/tdi25_matched_reference.rs tdi-ai/tests/tdi25_mirror_parity_stress.rs
grep -Eq '\| 43 \| Translation/origin stress suite \| \*\*landed\*\* in #[0-9]+, exact head `[0-9a-f]{40}`, merge `[0-9a-f]{40}`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 44 \| Mirror/parity stress suite \| \*\*(current candidate|landed)' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 43
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
