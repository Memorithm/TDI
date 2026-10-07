#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_attention
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_sequence_length_scaling
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_width_scaling
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-sequence-length-scaling-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const SEQUENCE_LENGTHS: [usize; SEQUENCE_LENGTH_COUNT] = [2, 4, 8];' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn sequence_window_rows' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_sequence_length_scaling(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_sequence_length_scaling_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'sequence_invalid("cost_accounting_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'sequence_invalid("length_set_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'sequence_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn smoke_study_has_exact_bounded_cost_accounting' tdi-ai/tests/tdi24_sequence_length_scaling.rs
grep -Fq 'tdi24-sequence-length-scaling-v1' docs/TDI-24-SEQUENCE-LENGTH-SCALING-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-SEQUENCE-LENGTH-SCALING-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_sequence_length_scaling.rs
grep -Fq '| 38 | width scaling | **landed** in #727, exact head `59655ad0f54d01205e4e7ef90c9d5337ff4426df`, merge `0eb49ea22d750c8a542aaad7fc2aa4f56087c84a`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 39 \| sequence-length scaling \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 38
