#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-11.2 pre-arm ERROR: $*" >&2
    exit 1
}

PREARM="docs/TDI-11.2-PROSPECTIVE-INSTRUMENTATION-PREARM.md"
STATE="docs/tdi11.2-prearm.yaml"
GATE="docs/TDI-11.2-IMPLEMENTATION-GATE.md"
STATUS="docs/TDI-11.2-STATUS.md"

for file in "$PREARM" "$STATE" "$GATE" "$STATUS"; do
    test -s "$file" || fail "missing required TDI-11.2 pre-arm surface: $file"
done

test -x scripts/check-tdi11-bootstrap.sh || fail "TDI-11.0 bootstrap checker missing or not executable"
bash scripts/check-tdi11-bootstrap.sh >/tmp/tdi11.2-tdi11-bootstrap.log
rm -f /tmp/tdi11.2-tdi11-bootstrap.log

grep -Fq 'MODEL_EXECUTION_AUTHORIZED: NO' "$PREARM" \
    || fail "pre-arm no-model-execution marker missing"
grep -Fq 'Status: **ACTIVE PRE-ARM — MODEL EXECUTION NOT AUTHORIZED**' "$STATUS" \
    || fail "status no-model-execution marker missing"
grep -Fq 'model_execution_authorized: false' "$STATE" \
    || fail "machine-readable model execution guard is not false"
grep -Fq 'final_execution_authorized: false' "$STATE" \
    || fail "machine-readable final execution guard is not false"

if grep -Fq 'model_execution_authorized: true' "$STATE"; then
    fail "model execution was armed in the pre-arm state"
fi
if grep -Fq 'final_execution_authorized: true' "$STATE"; then
    fail "final execution was armed in the pre-arm state"
fi

grep -Fq '  - Development' "$STATE" || fail "Development domain missing"
grep -Fq '  - Validation' "$STATE" || fail "Validation domain missing"
if grep -Fq '  - Final' "$STATE"; then
    fail "Final domain must not exist in TDI-11.2 pre-arm"
fi

blocking_count="$(grep -c ': unresolved_blocking$' "$STATE")"
[[ "$blocking_count" == "12" ]] \
    || fail "expected 12 unresolved model-execution blockers, found $blocking_count"

for marker in \
    'complete_world_hidden_truth' \
    'evaluator_labels' \
    'hidden_difficulty' \
    'future_trajectory_state' \
    'alternative_arm_outcomes' \
    'final_seed_material'; do
    grep -Fq "  - $marker" "$STATE" || fail "missing forbidden input: $marker"
done

for marker in \
    'decoder_statistics' \
    'hidden_state_summary' \
    'visible_evidence_summary' \
    'verifier_result' \
    'retrieval_tool_result' \
    'action_history' \
    'runtime_resource_summary' \
    'resample_summary'; do
    grep -Fq "  - $marker" "$STATE" || fail "missing inherited visible source class: $marker"
done

# Pre-arm must not introduce a runnable concrete-model or final TDI-11.2 surface.
mapfile -t forbidden < <(
    find tdi-ai tdi-bench scripts .github/workflows -type f \
        \( -iname '*tdi11.2*runner*' -o -iname '*tdi11_2*runner*' \
           -o -iname '*tdi11.2*model*run*' -o -iname '*tdi11_2*model*run*' \
           -o -iname '*tdi11.2*final*' -o -iname '*tdi11_2*final*' \) \
        ! -path 'scripts/check-tdi11.2-prearm.sh' \
        -print
)
if ((${#forbidden[@]} != 0)); then
    printf 'Unexpected TDI-11.2 executable/final surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    fail "pre-arm cannot contain a concrete-model or final runner"
fi

printf 'TDI-11.0 inherited bootstrap: PASS\n'
printf 'TDI-11.2 model execution: BLOCKED\n'
printf 'TDI-11.2 final execution: BLOCKED\n'
printf 'TDI-11.2 unresolved freeze fields: %s/12\n' "$blocking_count"
printf 'TDI-11.2 Development/Validation-only domain guard: PASS\n'
printf 'TDI-11.2 forbidden-input guard: PASS\n'
printf 'TDI-11.2 concrete-model/final runner surface: ABSENT\n'
printf 'TDI-11.2 pre-arm gate: PASS\n'
