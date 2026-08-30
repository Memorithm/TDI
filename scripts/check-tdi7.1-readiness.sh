#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "ERROR: $*" >&2
    exit 1
}

printf '\n===== TDI-7.0 PREREGISTRATION INTEGRITY =====\n'
sha256sum -c docs/TDI-7.0-ATTENTION-RECOVERY-PREREGISTRATION.sha256

printf '\n===== HISTORICAL TDI-6.8 MANIFEST =====\n'
sha256sum -c docs/TDI-6.8-SCIENTIFIC-CODE.sha256

printf '\n===== TDI-7.1 SPECIFICATION SURFACES =====\n'
test -f docs/TDI-7.1-EVALUATOR-SPEC.md || fail "missing TDI-7.1 evaluator specification"
test -f docs/TDI-7.1-COMPLETION-CHECKLIST.md || fail "missing TDI-7.1 completion checklist"
grep -Fq 'deterministic_local_row_stochastic_v1' docs/TDI-7.1-EVALUATOR-SPEC.md \
    || fail "semantic identifier missing from evaluator specification"
grep -Fq 'deficit = d / (1 + d)' docs/TDI-7.1-EVALUATOR-SPEC.md \
    || fail "frozen deficit formula missing from evaluator specification"
grep -Fq 'Frozen early observation depths for this evaluator: `1` and `2`.' docs/TDI-7.1-EVALUATOR-SPEC.md \
    || fail "frozen early depths missing from evaluator specification"
grep -Fq 'Target depth: `5`' docs/TDI-7.1-EVALUATOR-SPEC.md \
    || fail "target depth missing from evaluator specification"

printf '\n===== TDI-7.2 SURFACE EXCLUSION =====\n'
BOUNDED_FILES=(
    tdi-ai/examples/tdi7_features.rs
    tdi-ai/examples/tdi7_end_to_end.rs
    tdi-bench/src/bin/tdi-attention-v71-tasks.rs
    tdi-bench/src/bin/tdi-attention-v71-interventions.rs
    tdi-bench/src/bin/tdi-attention-v71-model.rs
    tdi-bench/src/bin/tdi-attention-v71-bootstrap.rs
)
for path in "${BOUNDED_FILES[@]}"; do
    test -f "$path" || fail "missing bounded evaluator file: $path"
    if grep -Fq 'I_ACCEPT_THE_TDI7_HOLDOUT_FREEZE' "$path"; then
        fail "final-holdout token leaked into bounded file: $path"
    fi
    if grep -Fq 'TDI7_CONFIRM_FINAL_HOLDOUT' "$path"; then
        fail "final-holdout authorization variable leaked into bounded file: $path"
    fi
    if grep -Fq '7_100_030_000' "$path"; then
        fail "final-holdout seed start leaked into bounded file: $path"
    fi
done

printf '\n===== TDI-7.1 PRELIGHT HOLDOUT REFUSAL =====\n'
if TDI7_CONFIRM_FINAL_HOLDOUT=sentinel bash scripts/reproduce-tdi7.1-preflight.sh >/tmp/tdi71-refusal.out 2>&1; then
    cat /tmp/tdi71-refusal.out >&2
    fail "preflight accepted a final-holdout authorization environment"
fi
grep -Fq 'refuses any final-holdout authorization variable' /tmp/tdi71-refusal.out \
    || fail "preflight refusal reason was not explicit"
rm -f /tmp/tdi71-refusal.out

printf '\n===== TDI-7.1 COMPLETE BOUNDED PREFLIGHT =====\n'
bash scripts/reproduce-tdi7.1-preflight.sh

printf '\nTDI-7.1 readiness gate: PASS\n'
printf 'TDI-7.2 final holdout: BLOCKED / NOT ACCESSED\n'
