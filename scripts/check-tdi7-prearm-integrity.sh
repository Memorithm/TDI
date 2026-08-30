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
require_line "final_holdout_generator_count=UNFROZEN"
require_line "intervention_aggregation=two_sites_per_generator_equal_record_weighting"
require_line "intervention_amplitude=0.25"
require_line "early_observation_depths=1,2"
require_line "target_depth=5"
require_line "target_definition=bounded_retrieval_deficit:d/(1+d)"
require_line "bootstrap_seed=0x5444493745324501"
require_line "bootstrap_replicates=2000"
require_line "relevance_margin=0.02"
require_line "classifier_policy=beneficial_then_harmful_then_equivalent_then_inconclusive"
require_line "final_holdout_status=NOT_ACCESSED"

printf '\n===== TDI-7 EVIDENCE HANDOFF =====\n'
cargo test --quiet -p tdi-ai --example tdi7_evidence_schema
cargo run --quiet -p tdi-ai --example tdi7_evidence_schema

printf '\n===== TDI-7.2 FINAL POPULATION DECISION =====\n'
cargo test --quiet -p tdi-ai --example tdi7_arming_decision
DECISION_OUTPUT="$(cargo run --quiet -p tdi-ai --example tdi7_arming_decision)"
printf '%s\n' "$DECISION_OUTPUT"
grep -Fxq 'TDI-7.2 final-population decision: VALID' <<<"$DECISION_OUTPUT" \
    || fail "final population decision record did not validate"
grep -Fxq 'authorization_state=NOT_AUTHORIZED' <<<"$DECISION_OUTPUT" \
    || fail "final population decision record is not explicitly unauthorized"
grep -Fxq 'final_holdout_accessed=false' <<<"$DECISION_OUTPUT" \
    || fail "population decision validator did not preserve no-access status"
grep -Fxq 'arming_allowed=false' <<<"$DECISION_OUTPUT" \
    || fail "population decision validator unexpectedly allows arming"

printf '\n===== TDI-7.2 FINAL SEED-SELECTION DECISION =====\n'
cargo test --quiet -p tdi-ai --example tdi7_seed_selection_decision
SELECTION_OUTPUT="$(cargo run --quiet -p tdi-ai --example tdi7_seed_selection_decision)"
printf '%s\n' "$SELECTION_OUTPUT"
grep -Fxq 'TDI-7.2 seed-selection decision: VALID' <<<"$SELECTION_OUTPUT" \
    || fail "final seed-selection decision record did not validate"
grep -Fxq 'selection_status=UNRESOLVED' <<<"$SELECTION_OUTPUT" \
    || fail "seed-selection decision is not explicitly unresolved"
grep -Fxq 'authorization_state=NOT_AUTHORIZED' <<<"$SELECTION_OUTPUT" \
    || fail "seed-selection decision record is not explicitly unauthorized"
grep -Fxq 'final_holdout_accessed=false' <<<"$SELECTION_OUTPUT" \
    || fail "seed-selection validator did not preserve no-access status"
grep -Fxq 'arming_allowed=false' <<<"$SELECTION_OUTPUT" \
    || fail "seed-selection validator unexpectedly allows arming"

printf '\n===== TDI-7.2 FINAL REJECTION-POLICY DECISION =====\n'
cargo test --quiet -p tdi-ai --example tdi7_rejection_policy_decision
POLICY_OUTPUT="$(cargo run --quiet -p tdi-ai --example tdi7_rejection_policy_decision)"
printf '%s\n' "$POLICY_OUTPUT"
grep -Fxq 'TDI-7.2 rejection-policy decision: VALID' <<<"$POLICY_OUTPUT" \
    || fail "final rejection-policy decision record did not validate"
grep -Fxq 'policy_status=UNRESOLVED' <<<"$POLICY_OUTPUT" \
    || fail "rejection policy is not explicitly unresolved"
grep -Fxq 'authorization_state=NOT_AUTHORIZED' <<<"$POLICY_OUTPUT" \
    || fail "rejection-policy decision record is not explicitly unauthorized"
grep -Fxq 'final_holdout_accessed=false' <<<"$POLICY_OUTPUT" \
    || fail "rejection-policy validator did not preserve no-access status"
grep -Fxq 'arming_allowed=false' <<<"$POLICY_OUTPUT" \
    || fail "rejection-policy validator unexpectedly allows arming"

set +e
cargo run --quiet -p tdi-ai --example tdi7_arming_decision -- --require-frozen \
    >/tmp/tdi7-final-population-decision.log 2>&1
population_status=$?
cargo run --quiet -p tdi-ai --example tdi7_seed_selection_decision -- --require-frozen \
    >/tmp/tdi7-final-seed-selection.log 2>&1
selection_status=$?
cargo run --quiet -p tdi-ai --example tdi7_rejection_policy_decision -- --require-frozen \
    >/tmp/tdi7-final-rejection-policy.log 2>&1
policy_status=$?
set -e
cat /tmp/tdi7-final-population-decision.log
cat /tmp/tdi7-final-seed-selection.log
cat /tmp/tdi7-final-rejection-policy.log

blocked=0
if [[ "$population_status" -eq 3 ]]; then
    grep -Fq 'BLOCKED: final holdout generator count decision is UNRESOLVED' \
        /tmp/tdi7-final-population-decision.log \
        || fail "unresolved population decision did not fail closed with the frozen diagnostic"
    blocked=1
elif [[ "$population_status" -ne 0 ]]; then
    fail "final population decision validator failed unexpectedly with status $population_status"
fi

if [[ "$selection_status" -eq 3 ]]; then
    grep -Fq 'BLOCKED: final holdout seed-selection rule is UNRESOLVED' \
        /tmp/tdi7-final-seed-selection.log \
        || fail "unresolved seed-selection decision did not fail closed with the frozen diagnostic"
    blocked=1
elif [[ "$selection_status" -ne 0 ]]; then
    fail "final seed-selection decision validator failed unexpectedly with status $selection_status"
fi

if [[ "$policy_status" -eq 3 ]]; then
    grep -Fq 'BLOCKED: final holdout rejection policy is UNRESOLVED' \
        /tmp/tdi7-final-rejection-policy.log \
        || fail "unresolved rejection policy did not fail closed with the frozen diagnostic"
    blocked=1
elif [[ "$policy_status" -ne 0 ]]; then
    fail "final rejection-policy validator failed unexpectedly with status $policy_status"
fi

if [[ "$blocked" -eq 1 ]]; then
    echo "TDI-7.2 must remain unarmed; population size, seed selection, and rejection policy are separate reviewed decisions." >&2
    exit 3
fi

fail "all pre-holdout decisions are frozen, but the separately reviewed arming transition has not been implemented"
