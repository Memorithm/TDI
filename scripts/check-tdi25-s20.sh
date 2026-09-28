#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_b_audit
grep -Fq 'status: candidate_pending_exact_head_qualification_and_merge' docs/tdi25-stage-b-freeze.yaml
grep -Fq 'protected_or_final_access: false' docs/tdi25-stage-b-freeze.yaml
grep -Fq 'scientific_claim: false' docs/tdi25-stage-b-freeze.yaml
grep -Fq '| 19 | Seed/case canonicalization + hash | **landed** in #658;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 20 | Stage-B leakage/balance audit | **current stacked slice**;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '**19/50 merged**' docs/TDI-25-CAMPAIGN-50.md
