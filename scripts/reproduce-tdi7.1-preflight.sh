#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -n "${TDI7_CONFIRM_FINAL_HOLDOUT:-}" ]]; then
    echo "ERROR: TDI-7.1 preflight refuses any final-holdout authorization variable" >&2
    exit 2
fi

TDI_COMMIT_SHA="$(git rev-parse HEAD)"

printf '\n===== TDI-7.1 PROTOCOL GUARDS =====\n'
cargo run --quiet -p tdi-bench --bin tdi-attention-v71

printf '\n===== TDI-7.1 TASK GENERATORS =====\n'
cargo run --quiet -p tdi-bench --bin tdi-attention-v71-tasks

printf '\n===== TDI-7.1 INTERVENTION FIXTURES =====\n'
cargo run --quiet -p tdi-bench --bin tdi-attention-v71-interventions

printf '\n===== TDI-7.1 TDI-AI FEATURE EXTRACTION =====\n'
cargo run --quiet -p tdi-ai --example tdi7_features

printf '\n===== TDI-7.1 NESTED B0/B1 MODEL =====\n'
cargo run --quiet -p tdi-bench --bin tdi-attention-v71-model

printf '\n===== TDI-7.1 PAIRED BOOTSTRAP =====\n'
cargo run --quiet -p tdi-bench --bin tdi-attention-v71-bootstrap

printf '\n===== TDI-7.1 PROVENANCE =====\n'
cargo run --quiet -p tdi-bench --bin tdi-attention-v71-provenance -- \
    --tdi-commit "$TDI_COMMIT_SHA"

printf '\n===== TDI-7.1 TARGETED TESTS =====\n'
cargo test --quiet -p tdi-bench --bin tdi-attention-v71
cargo test --quiet -p tdi-bench --bin tdi-attention-v71-tasks
cargo test --quiet -p tdi-bench --bin tdi-attention-v71-interventions
cargo test --quiet -p tdi-bench --bin tdi-attention-v71-model
cargo test --quiet -p tdi-bench --bin tdi-attention-v71-bootstrap
cargo test --quiet -p tdi-bench --bin tdi-attention-v71-provenance
cargo test --quiet -p tdi-ai --example tdi7_features

printf '\nTDI-7.1 bounded preflight: PASS\n'
printf 'TDI-7.2 final holdout: NOT ACCESSED\n'
