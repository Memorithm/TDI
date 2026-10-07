#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi24_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_d_attribution_audit
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_sequence_length_scaling
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_c_preflight
grep -Fq 'tdi24-stage-d-attribution-audit-v1' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub const ATTRIBUTION_AUDITED_SLICES: [(u8, &str); 10]' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn run_stage_d_attribution_audit(' tdi-ai/src/tdi24_eval.rs
grep -Fq 'pub fn validate_stage_d_attribution_audit_report' tdi-ai/src/tdi24_eval.rs
grep -Fq 'attribution_invalid("admissibility_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'attribution_invalid("withheld_entry")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'attribution_invalid("scientific_attribution_admissible")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'attribution_invalid("case_evidence_drift")' tdi-ai/src/tdi24_eval.rs
grep -Fq 'fn every_slice_supports_software_semantics_only' tdi-ai/tests/tdi24_stage_d_attribution_audit.rs
grep -Fq 'tdi24-stage-d-attribution-audit-v1' docs/TDI-24-STAGE-D-ATTRIBUTION-AUDIT-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-24-STAGE-D-ATTRIBUTION-AUDIT-V1.md
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi24_eval.rs tdi-ai/tests/tdi24_stage_d_attribution_audit.rs
grep -Fq '| 39 | sequence-length scaling | **landed** in #730, exact head `36ff40db13d42c2a13e932194dacb94721eced48`, merge `c9d935cd77e3b0b49bb27b5e3528cbc60514b764`;' docs/TDI-24-CAMPAIGN-50.md
grep -Eq '\| 40 \| Stage-D attribution audit \| \*\*(current stacked slice|landed)' docs/TDI-24-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-24-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 39
