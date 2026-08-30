#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "ERROR: $*" >&2
    exit 1
}

if [[ -n "${TDI7_CONFIRM_FINAL_HOLDOUT:-}" ]]; then
    fail "pre-arm integrity audit refuses any final-holdout authorization variable"
fi

if [[ -n "$(git status --porcelain)" ]]; then
    fail "pre-arm integrity audit requires a clean worktree"
fi

TDI_COMMIT_SHA="$(git rev-parse HEAD)"

printf '\n===== TDI-7.1 READINESS =====\n'
bash scripts/check-tdi7.1-readiness.sh

printf '\n===== TDI-7.2 UNARMED SURFACE =====\n'
bash scripts/check-tdi7.2-unarmed.sh

printf '\n===== TDI-7.1 CANONICAL PROVENANCE =====\n'
PROVENANCE="$(
    cargo run --quiet -p tdi-bench --bin tdi-attention-v71-provenance -- \
        --tdi-commit "$TDI_COMMIT_SHA"
)"
printf '%s\n' "$PROVENANCE"

require_line() {
    local expected="$1"
    grep -Fxq "$expected" <<<"$PROVENANCE" \
        || fail "canonical provenance mismatch: expected '$expected'"
}

require_line "semantic_id=deterministic_local_row_stochastic_v1"
require_line "training_generator_count=96"
require_line "development_generator_count=48"
require_line "validation_generator_count=48"
require_line "intervention_amplitude=0.25"
require_line "early_observation_depths=1,2"
require_line "target_depth=5"
require_line "target_definition=bounded_retrieval_deficit:d/(1+d)"
require_line "bootstrap_seed=0x5444493745324501"
require_line "bootstrap_replicates=2000"
require_line "relevance_margin=0.02"
require_line "final_holdout_status=NOT_ACCESSED"

printf '\n===== TDI-7.2 PRE-ARM BLOCKERS =====\n'
if grep -Fxq 'final_holdout_generator_count=UNFROZEN' <<<"$PROVENANCE"; then
    echo "BLOCKED: final holdout generator count was not frozen by TDI-7.0/TDI-7.1" >&2
    echo "TDI-7.2 must remain unarmed; do not infer a count from train/dev/validation." >&2
    exit 3
fi

fail "final holdout generator-count state is neither explicitly UNFROZEN nor a reviewed frozen value"
