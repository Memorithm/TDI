#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi24_stage_a_audit
grep -Fq 'status: candidate_pending_exact_head_qualification_and_merge' docs/tdi24-stage-a-freeze.yaml
grep -Fq 'protected_or_final_access: false' docs/tdi24-stage-a-freeze.yaml
grep -Fq 'scientific_claim: false' docs/tdi24-stage-a-freeze.yaml
grep -Fq '**14/50 merged**' docs/TDI-24-CAMPAIGN-50.md
