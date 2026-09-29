#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_stage_b_audit
grep -Fq 'status: qualified_merged_non_authorizing' docs/tdi25-stage-b-freeze.yaml
grep -Fq 'qualified_head_sha: 787a138ea76ee1d375a1db67007bf7c6690fcce4' docs/tdi25-stage-b-freeze.yaml
grep -Fq 'merge_commit_sha: 2cc7c45d5cae6b9bfcd47bc679ea815199321fe9' docs/tdi25-stage-b-freeze.yaml
grep -Fq 'protected_or_final_access: false' docs/tdi25-stage-b-freeze.yaml
grep -Fq 'scientific_claim: false' docs/tdi25-stage-b-freeze.yaml
grep -Fq '| 19 | Seed/case canonicalization + hash | **landed** in #658;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 20 | Stage-B leakage/balance audit | **landed** in #660;' docs/TDI-25-CAMPAIGN-50.md
