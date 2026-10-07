#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_multi_seed_replication
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_d_attribution_audit
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-multi-seed-replication-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const MULTI_SEED_BLOCKS: [u64; 4] = [0, 1, 2, 3];' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const MULTI_SEED_BLOCK_STRIDE: u64 = MAX_PREFLIGHT_PAIRS_PER_FAMILY;' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_multi_seed_replication(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_multi_seed_replication_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'replication_invalid("block_overlap")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'replication_invalid("frozen_offset_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'replication_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn first_block_reproduces_the_stage_c_preflight' tdi-ai/tests/tdi24_multi_seed_replication.rs
grep -Fq 'tdi24-multi-seed-replication-v1' docs/TDI-24-MULTI-SEED-REPLICATION-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-MULTI-SEED-REPLICATION-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_multi_seed_replication.rs
grep -Fq '| 40 | Stage-D attribution audit | **landed** in #734, exact head `d86d6e37bb2fac723e73432ced0e20c343cb0047`, merge `dadd65a554ddf0304df104460a97f09f440d4893`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 41 \| Multi-seed replication \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 40
